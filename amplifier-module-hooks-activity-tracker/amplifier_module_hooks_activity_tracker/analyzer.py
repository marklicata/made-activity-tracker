"""Activity analyzer with LLM and embedding-based duplicate detection."""

import json
import logging
import os
from typing import Any

import numpy as np

from .utils import sanitize_llm_response

logger = logging.getLogger(__name__)


class ActivityAnalyzer:
    """Analyzes session context and finds related work using LLM and embeddings."""

    def __init__(self, config: dict[str, Any]):
        """Initialize activity analyzer.

        Args:
            config: Configuration dictionary
        """
        self.config = config
        self._llm_client = None
        self._embedding_generator = None
        self._embedding_cache = None

    @property
    def llm_client(self):
        """Lazy-load LLM client."""
        if self._llm_client is None:
            try:
                from openai import AsyncOpenAI

                api_key = os.getenv("OPENAI_API_KEY")
                if not api_key:
                    logger.warning("OPENAI_API_KEY not set, LLM analysis will fail")
                self._llm_client = AsyncOpenAI(api_key=api_key)
            except Exception as e:
                logger.error(f"Failed to initialize LLM client: {e}")
        return self._llm_client

    @property
    def embedding_generator(self):
        """Lazy-load embedding generator."""
        if self._embedding_generator is None:
            from .embedding_generator import EmbeddingGenerator

            self._embedding_generator = EmbeddingGenerator(self.config)
        return self._embedding_generator

    @property
    def embedding_cache(self):
        """Lazy-load embedding cache."""
        if self._embedding_cache is None:
            from .embedding_cache import EmbeddingCache

            self._embedding_cache = EmbeddingCache()
        return self._embedding_cache

    async def find_related_work(
        self, context: dict[str, Any], open_issues: list[Any]
    ) -> list[dict[str, Any]]:
        """Find work items related to session context.

        Uses two-phase approach:
        1. Embedding similarity pre-filter (fast)
        2. LLM reasoning (accurate)

        Args:
            context: Session context with prompt, working_dir, git_status, recent_files
            open_issues: List of Issue objects from issue-manager

        Returns:
            List of dicts with issue, confidence, reasoning, relationship_type
        """
        if not open_issues:
            return []

        # Try two-phase analysis (embeddings + LLM)
        try:
            return await self._two_phase_analysis(context, open_issues)
        except Exception as e:
            logger.warning(f"Two-phase analysis failed, falling back to LLM-only: {e}")
            # Fallback to LLM-only analysis
            return await self._llm_only_analysis(context, open_issues[:20])

    async def _two_phase_analysis(
        self, context: dict[str, Any], open_issues: list[Any]
    ) -> list[dict[str, Any]]:
        """Two-phase analysis with embeddings pre-filter.

        Args:
            context: Session context
            open_issues: List of Issue objects

        Returns:
            List of related work items
        """
        # Phase 1: Embedding similarity pre-filter
        context_text = self._format_context_for_embedding(context)
        context_embedding = await self.embedding_generator.generate(context_text)

        if context_embedding is None:
            # Embedding generation failed, fall back to LLM-only
            return await self._llm_only_analysis(context, open_issues[:20])

        candidates = []
        for issue in open_issues:
            try:
                issue_text = f"{issue.title} {issue.description}"
                issue_embedding = await self._get_cached_embedding(issue.id, issue_text)

                if issue_embedding is not None:
                    similarity = self._cosine_similarity(context_embedding, issue_embedding)
                    threshold = self.config.get("similarity_threshold", 0.7)

                    if similarity > threshold:
                        candidates.append({"issue": issue, "similarity": similarity})
            except Exception as e:
                logger.debug(f"Failed to process issue {issue.id}: {e}")

        if not candidates:
            logger.info("No candidates found via embeddings, trying LLM-only")
            return await self._llm_only_analysis(context, open_issues[:20])

        # Sort by similarity and take top 10
        candidates.sort(key=lambda x: x["similarity"], reverse=True)
        top_candidates = candidates[:10]

        logger.info(f"Filtered to {len(top_candidates)} candidates via embeddings")

        # Phase 2: LLM reasoning on candidates
        return await self._llm_reasoning(context, top_candidates)

    async def _llm_only_analysis(
        self, context: dict[str, Any], open_issues: list[Any]
    ) -> list[dict[str, Any]]:
        """LLM-only analysis without embedding pre-filter.

        Args:
            context: Session context
            open_issues: List of Issue objects (should be limited to ~20)

        Returns:
            List of related work items
        """
        if not open_issues:
            return []

        candidates = [{"issue": issue, "similarity": 1.0} for issue in open_issues]
        return await self._llm_reasoning(context, candidates)

    async def _llm_reasoning(
        self, context: dict[str, Any], candidates: list[dict[str, Any]]
    ) -> list[dict[str, Any]]:
        """Use LLM to determine actual relationships.

        Args:
            context: Session context
            candidates: List of candidate issues with similarity scores

        Returns:
            List of related work items with confidence and reasoning
        """
        if not self.llm_client:
            logger.error("LLM client not available")
            return []

        prompt = self._build_analysis_prompt(context, candidates)

        try:
            response = await self.llm_client.chat.completions.create(
                model="gpt-4",  # Use GPT-4 for better reasoning
                messages=[
                    {
                        "role": "system",
                        "content": "You are an expert at analyzing work tasks and identifying duplicates or related work.",
                    },
                    {"role": "user", "content": prompt},
                ],
                response_format={"type": "json_object"},
                timeout=30,
            )

            result_text = response.choices[0].message.content
            result = json.loads(sanitize_llm_response(result_text))

            return self._parse_llm_result(result, candidates)

        except Exception as e:
            logger.error(f"LLM analysis failed: {e}", exc_info=True)
            return []

    async def analyze_session_work(self, messages: list[dict[str, Any]]) -> dict[str, Any]:
        """Analyze session transcript to extract work done and new ideas.

        Args:
            messages: List of message dicts from session

        Returns:
            Dict with completed, summary, new_ideas
        """
        if not messages:
            return {"completed": False, "summary": "Empty session", "new_ideas": []}

        if not self.llm_client:
            logger.error("LLM client not available")
            return {"completed": False, "summary": "Analysis unavailable", "new_ideas": []}

        # Take last 30 messages
        recent = messages[-30:]
        prompt = self._build_session_analysis_prompt(recent)

        try:
            response = await self.llm_client.chat.completions.create(
                model="gpt-4",
                messages=[
                    {
                        "role": "system",
                        "content": "You are an expert at analyzing coding sessions and extracting insights.",
                    },
                    {"role": "user", "content": prompt},
                ],
                response_format={"type": "json_object"},
                timeout=30,
            )

            result_text = response.choices[0].message.content
            result = json.loads(sanitize_llm_response(result_text))

            # Validate result structure
            if not isinstance(result, dict):
                logger.error("Invalid result structure from LLM")
                return {"completed": False, "summary": "Analysis failed", "new_ideas": []}

            return {
                "completed": result.get("completed", False),
                "summary": result.get("summary", "No summary available"),
                "new_ideas": result.get("new_ideas", []),
            }

        except Exception as e:
            logger.error(f"Session analysis failed: {e}", exc_info=True)
            return {"completed": False, "summary": "Analysis error", "new_ideas": []}

    async def _get_cached_embedding(self, issue_id: str, content: str) -> np.ndarray | None:
        """Get cached embedding or generate new one.

        Args:
            issue_id: Issue ID
            content: Issue content (title + description)

        Returns:
            Embedding vector or None
        """
        from .utils import compute_content_hash

        content_hash = compute_content_hash(content)

        # Try cache
        cached = await self.embedding_cache.get(issue_id, content_hash)
        if cached is not None:
            return cached

        # Generate new embedding
        embedding = await self.embedding_generator.generate(content)
        if embedding is not None:
            # Cache it
            model = self.config.get("embedding_model", "text-embedding-3-small")
            await self.embedding_cache.set(issue_id, embedding, model, content_hash)

        return embedding

    def _cosine_similarity(self, vec1: np.ndarray, vec2: np.ndarray) -> float:
        """Compute cosine similarity between two vectors.

        Args:
            vec1: First vector
            vec2: Second vector

        Returns:
            Similarity score (0-1)
        """
        try:
            dot_product = np.dot(vec1, vec2)
            norm1 = np.linalg.norm(vec1)
            norm2 = np.linalg.norm(vec2)

            if norm1 == 0 or norm2 == 0:
                return 0.0

            similarity = dot_product / (norm1 * norm2)
            return float(max(0.0, min(1.0, similarity)))  # Clamp to [0, 1]

        except Exception as e:
            logger.error(f"Cosine similarity calculation failed: {e}")
            return 0.0

    def _format_context_for_embedding(self, context: dict[str, Any]) -> str:
        """Format context for embedding generation.

        Args:
            context: Session context

        Returns:
            Formatted text
        """
        parts = [context.get("prompt", "")]

        if context.get("git_status"):
            parts.append(f"Git changes: {context['git_status'][:200]}")

        if context.get("recent_files"):
            files = context["recent_files"][:5]
            parts.append(f"Recent files: {', '.join(files)}")

        return " ".join(parts)

    def _build_analysis_prompt(
        self, context: dict[str, Any], candidates: list[dict[str, Any]]
    ) -> str:
        """Build prompt for LLM analysis.

        Args:
            context: Session context
            candidates: Candidate issues

        Returns:
            Formatted prompt
        """
        prompt = f"""Programmer starting session:
Prompt: {context.get('prompt')}
Working directory: {context.get('working_dir')}
Git status: {context.get('git_status', 'None')[:200]}
Recent files: {', '.join(context.get('recent_files', [])[:10])}

Potentially related open work (pre-filtered by embeddings):

"""

        for idx, candidate in enumerate(candidates, 1):
            issue = candidate["issue"]
            sim = candidate.get("similarity", 0)
            prompt += f"""{idx}. ID: {issue.id}
   Title: {issue.title}
   Description: {issue.description[:200]}
   Similarity: {sim:.2f}

"""

        prompt += """
Determine which items are ACTUALLY related. Be conservative - only flag:
1. Duplicates (same work, different words)
2. Strong blockers (can't proceed without this)
3. Close collaboration opportunities

For each related item, provide:
- issue_id
- confidence (0.0-1.0, where 1.0 = definitely duplicate)
- reasoning (why it's related)
- relationship_type (duplicate | blocker | collaboration)

Return JSON:
{
    "related": [
        {
            "issue_id": "...",
            "confidence": 0.9,
            "reasoning": "...",
            "relationship_type": "duplicate"
        }
    ]
}

If nothing is truly related, return: {"related": []}
"""
        return prompt

    def _build_session_analysis_prompt(self, messages: list[dict[str, Any]]) -> str:
        """Build prompt for session analysis.

        Args:
            messages: Recent messages

        Returns:
            Formatted prompt
        """
        formatted_messages = self._format_messages(messages)

        prompt = f"""Analyze this Amplifier coding session and extract:

1. Was the main task completed? (yes/no)
2. Brief summary of work accomplished
3. New ideas, tasks, or bugs discovered during the session

For each new idea, provide:
- title (short, clear)
- description (detailed context)
- suggested_priority (0-4, based on urgency/importance, 0=highest)

Session transcript:
{formatted_messages}

Return JSON:
{{
    "completed": true/false,
    "summary": "...",
    "new_ideas": [
        {{
            "title": "...",
            "description": "...",
            "suggested_priority": 2
        }}
    ]
}}
"""
        return prompt

    def _format_messages(self, messages: list[dict[str, Any]]) -> str:
        """Format messages for prompt.

        Args:
            messages: List of message dicts

        Returns:
            Formatted string
        """
        lines = []
        for msg in messages:
            role = msg.get("role", "unknown")
            content = msg.get("content", "")
            lines.append(f"{role.upper()}: {content[:500]}")

        return "\n".join(lines)

    def _parse_llm_result(
        self, result: dict[str, Any], candidates: list[dict[str, Any]]
    ) -> list[dict[str, Any]]:
        """Parse LLM result into structured format.

        Args:
            result: LLM response dict
            candidates: Original candidate list

        Returns:
            List of related work items
        """
        related_items = []
        related_list = result.get("related", [])

        if not isinstance(related_list, list):
            logger.error("Invalid related list from LLM")
            return []

        # Build issue lookup
        issue_lookup = {c["issue"].id: c["issue"] for c in candidates}

        for item in related_list:
            if not isinstance(item, dict):
                continue

            issue_id = item.get("issue_id")
            if not issue_id or issue_id not in issue_lookup:
                continue

            related_items.append(
                {
                    "issue": issue_lookup[issue_id],
                    "confidence": item.get("confidence", 0.0),
                    "reasoning": item.get("reasoning", ""),
                    "relationship_type": item.get("relationship_type", "related"),
                }
            )

        return related_items
