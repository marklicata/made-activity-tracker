"""LLM-powered mock response generator for testing.

Uses a real LLM to generate realistic OpenAI API responses for testing,
avoiding the complexity of hand-crafting mock responses.
"""

import json
import os
from typing import Any
from unittest.mock import MagicMock


class LLMMockGenerator:
    """Generates realistic OpenAI API mock responses using a real LLM."""
    
    def __init__(self):
        """Initialize the mock generator."""
        self._client = None
        self._has_openai = self._check_openai_available()
    
    def _check_openai_available(self) -> bool:
        """Check if OpenAI is available and configured."""
        try:
            from openai import OpenAI
            return bool(os.getenv("OPENAI_API_KEY"))
        except ImportError:
            return False
    
    @property
    def client(self):
        """Lazy-load OpenAI client."""
        if self._client is None and self._has_openai:
            from openai import OpenAI
            self._client = OpenAI(api_key=os.getenv("OPENAI_API_KEY"))
        return self._client
    
    def generate_find_related_work_response(
        self,
        context_prompt: str,
        issue_titles: list[str],
        num_related: int = 2
    ) -> MagicMock:
        """Generate mock response for find_related_work API call.
        
        Args:
            context_prompt: The user's work description
            issue_titles: List of existing issue titles to match against
            num_related: Number of related items to return
            
        Returns:
            Mock OpenAI response object with realistic related items
        """
        if not self.client:
            # Fallback to static mock if no OpenAI
            return self._static_related_work_response(num_related)
        
        # Use LLM to generate realistic matches
        prompt = f"""Given this work description: "{context_prompt}"
        
And these existing issues: {json.dumps(issue_titles)}

Generate a JSON response with {num_related} most related issues. For each, provide:
- issue_id: Pick from the issue titles (use index as ID)
- confidence: Float 0-1 (how related it is)
- reasoning: Why it's related
- relationship_type: "duplicate", "blocker", or "related"

Return ONLY valid JSON matching this structure:
{{
  "related_items": [
    {{"issue_id": "0", "confidence": 0.95, "reasoning": "...", "relationship_type": "duplicate"}}
  ]
}}"""
        
        try:
            response = self.client.chat.completions.create(
                model="gpt-3.5-turbo",
                messages=[{"role": "user", "content": prompt}],
                response_format={"type": "json_object"},
                temperature=0.7,
                max_tokens=500
            )
            
            json_text = response.choices[0].message.content
            parsed = json.loads(json_text)
            
            # Create mock response
            mock_response = MagicMock()
            mock_response.choices = [MagicMock()]
            mock_response.choices[0].message.content = json.dumps(parsed)
            
            return mock_response
            
        except Exception as e:
            print(f"LLM generation failed: {e}, using static mock")
            return self._static_related_work_response(num_related)
    
    def generate_session_analysis_response(
        self,
        messages: list[dict[str, str]],
        expect_completion: bool = True
    ) -> MagicMock:
        """Generate mock response for analyze_session_work API call.
        
        Args:
            messages: Session messages to analyze
            expect_completion: Whether work was completed
            
        Returns:
            Mock OpenAI response with session analysis
        """
        if not self.client:
            return self._static_session_analysis_response(expect_completion)
        
        # Extract conversation
        conversation = "\n".join([
            f"{m.get('role', 'unknown')}: {m.get('content', '')}" 
            for m in messages[-5:]  # Last 5 messages
        ])
        
        prompt = f"""Analyze this coding session conversation:

{conversation}

Generate a JSON response with:
- completed: boolean (was the work finished?)
- summary: string (2-3 sentence summary of what was done)
- new_ideas: array of objects with title, description, priority (1-4), issue_type

Return ONLY valid JSON:
{{
  "completed": {"true" if expect_completion else "false"},
  "summary": "Brief summary...",
  "new_ideas": [
    {{"title": "Idea title", "description": "Details", "priority": 2, "issue_type": "feature"}}
  ]
}}"""
        
        try:
            response = self.client.chat.completions.create(
                model="gpt-3.5-turbo",
                messages=[{"role": "user", "content": prompt}],
                response_format={"type": "json_object"},
                temperature=0.7,
                max_tokens=300
            )
            
            json_text = response.choices[0].message.content
            
            mock_response = MagicMock()
            mock_response.choices = [MagicMock()]
            mock_response.choices[0].message.content = json_text
            
            return mock_response
            
        except Exception as e:
            print(f"LLM generation failed: {e}, using static mock")
            return self._static_session_analysis_response(expect_completion)
    
    def _static_related_work_response(self, num_items: int = 2) -> MagicMock:
        """Fallback static mock for related work."""
        result = {
            "related_items": [
                {
                    "issue_id": f"test-{i}",
                    "confidence": 0.85 - (i * 0.1),
                    "reasoning": f"Similar work in area {i}",
                    "relationship_type": "related"
                }
                for i in range(num_items)
            ]
        }
        
        mock_response = MagicMock()
        mock_response.choices = [MagicMock()]
        mock_response.choices[0].message.content = json.dumps(result)
        return mock_response
    
    def _static_session_analysis_response(self, completed: bool = True) -> MagicMock:
        """Fallback static mock for session analysis."""
        result = {
            "completed": completed,
            "summary": "Implemented test feature with proper error handling",
            "new_ideas": [
                {
                    "title": "Add caching layer",
                    "description": "Implement Redis caching for performance",
                    "priority": 2,
                    "issue_type": "feature"
                }
            ]
        }
        
        mock_response = MagicMock()
        mock_response.choices = [MagicMock()]
        mock_response.choices[0].message.content = json.dumps(result)
        return mock_response
    
    def create_mock_llm_client(self, responses: list[MagicMock]) -> MagicMock:
        """Create a mock LLM client that returns pre-generated responses.
        
        Args:
            responses: List of mock responses to return in order
            
        Returns:
            Mock AsyncOpenAI client
        """
        from unittest.mock import AsyncMock
        
        mock_client = AsyncMock()
        mock_client.chat.completions.create = AsyncMock(side_effect=responses)
        return mock_client


# Example usage in tests:
"""
from tests.helpers import LLMMockGenerator

@pytest.mark.asyncio
async def test_find_related_work_with_llm_mock():
    analyzer = ActivityAnalyzer(config)
    
    # Generate realistic mock response
    mock_gen = LLMMockGenerator()
    mock_response = mock_gen.generate_find_related_work_response(
        context_prompt="Implement user authentication",
        issue_titles=["Login system", "Password reset", "Database schema"],
        num_related=2
    )
    
    # Use the mock
    mock_client = mock_gen.create_mock_llm_client([mock_response])
    analyzer._llm_client = mock_client
    
    # Test
    result = await analyzer.find_related_work(context, issues)
    assert len(result) > 0
"""
