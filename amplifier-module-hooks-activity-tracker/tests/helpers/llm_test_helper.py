"""LLM-powered test helper for generating realistic OpenAI API mock responses.

This module uses an actual LLM to generate context-aware, realistic responses that match
the exact format expected by the ActivityAnalyzer module. This approach is superior to
traditional mocking because:

1. Responses are contextually appropriate
2. Variations are realistic and diverse
3. Edge cases can be naturally generated
4. Maintains the complexity of real LLM outputs

Basic Usage:
    >>> helper = LLMTestHelper()
    >>> response = await helper.generate_find_related_work_response(
    ...     prompt="Fix login bug",
    ...     issues=[{"id": "123", "title": "Login fails on Safari"}]
    ... )
    >>> assert "related" in response
    >>> assert isinstance(response["related"], list)

Advanced Usage:
    >>> # Generate response with specific characteristics
    >>> response = await helper.generate_with_instruction(
    ...     instruction="Return 2 duplicate issues with high confidence",
    ...     context={"prompt": "Add user auth"},
    ...     format_spec="OpenAI find_related_work format"
    ... )
"""

import json
import os
from typing import Any, Literal
from openai import AsyncOpenAI


class LLMTestHelper:
    """Generates realistic OpenAI-style responses for testing ActivityAnalyzer.

    This helper uses an actual LLM to create context-aware mock responses that
    match the exact structure expected by analyzer.py methods.

    Attributes:
        client: AsyncOpenAI client for generating responses
        model: Model to use for generation (default: gpt-4o-mini for cost efficiency)

    Example:
        >>> helper = LLMTestHelper()
        >>> # Generate response for find_related_work
        >>> response = await helper.generate_find_related_work_response(
        ...     prompt="Fix authentication bug in login flow",
        ...     issues=[
        ...         {"id": "TASK-123", "title": "Login fails", "description": "Users can't login"},
        ...         {"id": "TASK-456", "title": "Add OAuth", "description": "Support Google login"}
        ...     ],
        ...     num_related=1,
        ...     relationship_types=["duplicate"]
        ... )
    """

    def __init__(self, model: str = "gpt-4o-mini", api_key: str | None = None):
        """Initialize LLM test helper.

        Args:
            model: OpenAI model to use (default: gpt-4o-mini for fast/cheap testing)
            api_key: OpenAI API key (default: from OPENAI_API_KEY env var)
        """
        self.model = model
        self.client = AsyncOpenAI(api_key=api_key or os.getenv("OPENAI_API_KEY"))

    async def generate_find_related_work_response(
        self,
        prompt: str,
        issues: list[dict[str, str]],
        num_related: int = 1,
        relationship_types: list[Literal["duplicate", "blocker", "collaboration"]] | None = None,
        confidence_range: tuple[float, float] = (0.7, 0.95),
    ) -> dict[str, Any]:
        """Generate realistic response for find_related_work() method.

        This creates a response matching the exact format expected by ActivityAnalyzer's
        _llm_reasoning method, including the "related" array with issue_id, confidence,
        reasoning, and relationship_type fields.

        Args:
            prompt: User's session prompt to find related work for
            issues: List of issue dicts with id, title, description
            num_related: Number of related items to return (0 for none)
            relationship_types: Types to include (duplicate/blocker/collaboration)
            confidence_range: Min/max confidence values to use

        Returns:
            Dict matching OpenAI response format:
            {
                "related": [
                    {
                        "issue_id": "TASK-123",
                        "confidence": 0.85,
                        "reasoning": "Both involve login authentication",
                        "relationship_type": "duplicate"
                    }
                ]
            }

        Example:
            >>> helper = LLMTestHelper()
            >>> response = await helper.generate_find_related_work_response(
            ...     prompt="Fix login page CSS styling",
            ...     issues=[
            ...         {"id": "BUG-1", "title": "Login button broken", "description": "CSS issues"},
            ...         {"id": "FEAT-2", "title": "Add OAuth", "description": "New feature"}
            ...     ],
            ...     num_related=1,
            ...     relationship_types=["collaboration"]
            ... )
            >>> assert len(response["related"]) == 1
            >>> assert response["related"][0]["relationship_type"] == "collaboration"
        """
        if relationship_types is None:
            relationship_types = ["duplicate", "blocker", "collaboration"]

        instruction = f"""You are generating TEST DATA for a coding assistant's work tracker.

Generate a realistic JSON response as if you were analyzing whether existing issues are related to a new task.

User's new task: "{prompt}"

Existing issues to consider:
{self._format_issues(issues)}

Generate a response with EXACTLY {num_related} related item(s).
Use relationship types from: {relationship_types}
Confidence should be between {confidence_range[0]} and {confidence_range[1]}.

Make the reasoning realistic and specific - explain WHY items are related based on:
- Similar functionality/components
- Duplicate work
- Dependencies/blockers
- Collaboration opportunities

Return ONLY valid JSON matching this exact structure:
{{
    "related": [
        {{
            "issue_id": "TASK-XXX",
            "confidence": 0.85,
            "reasoning": "Specific reason why this is related",
            "relationship_type": "duplicate" | "blocker" | "collaboration"
        }}
    ]
}}

If num_related is 0, return: {{"related": []}}"""

        return await self._generate_json_response(instruction)

    async def generate_session_work_response(
        self,
        messages: list[dict[str, str]],
        completed: bool = True,
        num_new_ideas: int = 0,
        priority_range: tuple[int, int] = (1, 3),
    ) -> dict[str, Any]:
        """Generate realistic response for analyze_session_work() method.

        Creates a response matching the format expected by ActivityAnalyzer's
        analyze_session_work method, analyzing a session transcript to extract
        completed status, summary, and new ideas.

        Args:
            messages: Session messages with 'role' and 'content' fields
            completed: Whether main task should be marked complete
            num_new_ideas: Number of new ideas/tasks discovered
            priority_range: Min/max priority values (0=highest, 4=lowest)

        Returns:
            Dict matching OpenAI response format:
            {
                "completed": true,
                "summary": "Implemented user authentication with OAuth",
                "new_ideas": [
                    {
                        "title": "Add rate limiting",
                        "description": "Prevent brute force attacks",
                        "suggested_priority": 2
                    }
                ]
            }

        Example:
            >>> helper = LLMTestHelper()
            >>> messages = [
            ...     {"role": "user", "content": "Help me add login"},
            ...     {"role": "assistant", "content": "I'll create auth module"},
            ...     {"role": "user", "content": "Also need password reset"}
            ... ]
            >>> response = await helper.generate_session_work_response(
            ...     messages=messages,
            ...     completed=True,
            ...     num_new_ideas=1
            ... )
            >>> assert response["completed"] is True
            >>> assert len(response["new_ideas"]) == 1
        """
        instruction = f"""You are generating TEST DATA for analyzing a coding session transcript.

Session messages:
{self._format_messages(messages)}

Generate a realistic analysis with:
- completed: {completed}
- summary: Brief summary of work accomplished (realistic based on messages)
- new_ideas: EXACTLY {num_new_ideas} new idea(s) discovered during session

For each new idea:
- title: Short, clear task title
- description: Detailed context about what needs to be done
- suggested_priority: {priority_range[0]} to {priority_range[1]} (0=highest urgency, 4=lowest)

Make ideas realistic - things that naturally come up during development:
- Edge cases discovered
- Bugs found
- Refactoring opportunities
- Technical debt
- Testing needs
- Documentation gaps

Return ONLY valid JSON matching this exact structure:
{{
    "completed": true | false,
    "summary": "Brief summary of work done",
    "new_ideas": [
        {{
            "title": "Short task title",
            "description": "Detailed description with context",
            "suggested_priority": 2
        }}
    ]
}}

If num_new_ideas is 0, return empty array: "new_ideas": []"""

        return await self._generate_json_response(instruction)

    async def generate_with_instruction(
        self,
        instruction: str,
        context: dict[str, Any],
        format_spec: Literal["find_related_work", "session_work"] = "find_related_work",
    ) -> dict[str, Any]:
        """Generate response with custom instruction and context.

        This is the most flexible method - you provide exactly what you want and get
        a realistic response. Useful for edge cases and specific test scenarios.

        Args:
            instruction: Specific instruction for what to generate
            context: Context dict with relevant information
            format_spec: Which response format to use

        Returns:
            Dict matching the specified format

        Example:
            >>> helper = LLMTestHelper()
            >>> response = await helper.generate_with_instruction(
            ...     instruction="Generate 3 high-confidence duplicate issues",
            ...     context={
            ...         "prompt": "Fix login bug",
            ...         "issues": [
            ...             {"id": "1", "title": "Login broken"},
            ...             {"id": "2", "title": "Auth fails"},
            ...             {"id": "3", "title": "Can't sign in"}
            ...         ]
            ...     },
            ...     format_spec="find_related_work"
            ... )
        """
        if format_spec == "find_related_work":
            format_example = """{
    "related": [
        {
            "issue_id": "...",
            "confidence": 0.9,
            "reasoning": "...",
            "relationship_type": "duplicate"
        }
    ]
}"""
        else:  # session_work
            format_example = """{
    "completed": true,
    "summary": "...",
    "new_ideas": [
        {
            "title": "...",
            "description": "...",
            "suggested_priority": 2
        }
    ]
}"""

        full_instruction = f"""You are generating TEST DATA for a coding assistant.

{instruction}

Context:
{json.dumps(context, indent=2)}

Return ONLY valid JSON matching this exact structure:
{format_example}

Make the response realistic and contextually appropriate."""

        return await self._generate_json_response(full_instruction)

    async def generate_error_response(
        self,
        error_type: Literal["invalid_json", "missing_fields", "wrong_types", "empty"],
    ) -> dict[str, Any] | str:
        """Generate intentionally malformed responses for error testing.

        Args:
            error_type: Type of error to generate

        Returns:
            Malformed response for testing error handling

        Example:
            >>> helper = LLMTestHelper()
            >>> bad_response = await helper.generate_error_response("invalid_json")
            >>> # Use this to test error handling in your analyzer
        """
        if error_type == "invalid_json":
            return '{"related": [{"issue_id": "123", "confidence": 0.9}]'  # Missing closing brace

        if error_type == "missing_fields":
            return {"related": [{"issue_id": "123"}]}  # Missing confidence, reasoning

        if error_type == "wrong_types":
            return {
                "related": [
                    {
                        "issue_id": 123,  # Should be string
                        "confidence": "high",  # Should be float
                        "reasoning": None,
                        "relationship_type": "invalid_type",
                    }
                ]
            }

        if error_type == "empty":
            return {}

        return {"error": "Unknown error type"}

    async def _generate_json_response(self, instruction: str) -> dict[str, Any]:
        """Internal: Generate JSON response using LLM.

        Args:
            instruction: Full instruction for the LLM

        Returns:
            Parsed JSON response
        """
        response = await self.client.chat.completions.create(
            model=self.model,
            messages=[
                {
                    "role": "system",
                    "content": "You generate realistic test data for a coding assistant. Always return valid JSON only.",
                },
                {"role": "user", "content": instruction},
            ],
            response_format={"type": "json_object"},
            temperature=0.8,  # Higher temperature for more varied test data
            timeout=15,
        )

        result_text = response.choices[0].message.content
        return json.loads(result_text)

    def _format_issues(self, issues: list[dict[str, str]]) -> str:
        """Format issues for prompt."""
        lines = []
        for issue in issues:
            lines.append(
                f"- {issue['id']}: {issue.get('title', 'No title')} - {issue.get('description', 'No description')[:100]}"
            )
        return "\n".join(lines)

    def _format_messages(self, messages: list[dict[str, str]]) -> str:
        """Format messages for prompt."""
        lines = []
        for msg in messages:
            role = msg.get("role", "unknown").upper()
            content = msg.get("content", "")[:300]
            lines.append(f"{role}: {content}")
        return "\n".join(lines)


class MockOpenAIResponse:
    """Mock OpenAI response structure for testing.

    Simulates the structure of an actual OpenAI API response so tests can work
    with responses as if they came from the real API.

    Example:
        >>> mock_response = MockOpenAIResponse.from_dict({
        ...     "related": [{"issue_id": "123", "confidence": 0.9, "reasoning": "...", "relationship_type": "duplicate"}]
        ... })
        >>> # Use in place of real OpenAI response
        >>> result_text = mock_response.choices[0].message.content
        >>> data = json.loads(result_text)
    """

    class Choice:
        class Message:
            def __init__(self, content: str):
                self.content = content

        def __init__(self, content: str):
            self.message = self.Message(content)

    def __init__(self, content: str):
        """Initialize mock response.

        Args:
            content: JSON string content
        """
        self.choices = [self.Choice(content)]

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "MockOpenAIResponse":
        """Create mock response from dict.

        Args:
            data: Dict to convert to JSON response

        Returns:
            MockOpenAIResponse with JSON content

        Example:
            >>> response = MockOpenAIResponse.from_dict({"related": []})
            >>> assert '"related"' in response.choices[0].message.content
        """
        return cls(json.dumps(data))


# Convenience functions for common test scenarios


async def create_find_related_response(
    prompt: str,
    issues: list[dict[str, str]],
    num_related: int = 1,
    **kwargs,
) -> MockOpenAIResponse:
    """Convenience function to create find_related_work mock response.

    Args:
        prompt: Session prompt
        issues: Available issues
        num_related: Number to mark as related
        **kwargs: Additional args for generate_find_related_work_response

    Returns:
        MockOpenAIResponse ready to use in tests

    Example:
        >>> # In your pytest test
        >>> mock_response = await create_find_related_response(
        ...     prompt="Fix login",
        ...     issues=[{"id": "1", "title": "Login bug", "description": "Broken"}],
        ...     num_related=1
        ... )
        >>> # Use with unittest.mock
        >>> with patch("openai.AsyncOpenAI.chat.completions.create", return_value=mock_response):
        ...     result = await analyzer.find_related_work(context, issues)
    """
    helper = LLMTestHelper()
    data = await helper.generate_find_related_work_response(
        prompt=prompt, issues=issues, num_related=num_related, **kwargs
    )
    return MockOpenAIResponse.from_dict(data)


async def create_session_work_response(
    messages: list[dict[str, str]],
    completed: bool = True,
    num_new_ideas: int = 0,
    **kwargs,
) -> MockOpenAIResponse:
    """Convenience function to create analyze_session_work mock response.

    Args:
        messages: Session messages
        completed: Task completion status
        num_new_ideas: Number of new ideas
        **kwargs: Additional args for generate_session_work_response

    Returns:
        MockOpenAIResponse ready to use in tests

    Example:
        >>> # In your pytest test
        >>> messages = [
        ...     {"role": "user", "content": "Add auth"},
        ...     {"role": "assistant", "content": "Creating module"}
        ... ]
        >>> mock_response = await create_session_work_response(
        ...     messages=messages,
        ...     completed=True,
        ...     num_new_ideas=2
        ... )
        >>> with patch("openai.AsyncOpenAI.chat.completions.create", return_value=mock_response):
        ...     result = await analyzer.analyze_session_work(messages)
    """
    helper = LLMTestHelper()
    data = await helper.generate_session_work_response(
        messages=messages, completed=completed, num_new_ideas=num_new_ideas, **kwargs
    )
    return MockOpenAIResponse.from_dict(data)
