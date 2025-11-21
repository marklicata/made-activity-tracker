"""Example usage of LLMTestHelper for testing ActivityAnalyzer.

This file demonstrates how to use the LLM test helper in your pytest tests
to generate realistic mock responses for the ActivityAnalyzer module.
"""

import pytest
from unittest.mock import AsyncMock, patch
from amplifier_module_hooks_activity_tracker.analyzer import ActivityAnalyzer
from .llm_test_helper import (
    LLMTestHelper,
    create_find_related_response,
    create_session_work_response,
)


# Example 1: Testing find_related_work with realistic responses
@pytest.mark.asyncio
async def test_find_related_work_with_duplicates():
    """Test that find_related_work correctly identifies duplicate issues."""
    
    # Setup
    config = {"similarity_threshold": 0.7}
    analyzer = ActivityAnalyzer(config)
    
    context = {
        "prompt": "Fix login authentication bug",
        "working_dir": "/app",
        "git_status": "modified: src/auth.py",
        "recent_files": ["src/auth.py", "src/login.py"],
    }
    
    # Mock Issue objects
    class MockIssue:
        def __init__(self, id, title, description):
            self.id = id
            self.title = title
            self.description = description
    
    issues = [
        MockIssue("BUG-123", "Login fails on Safari", "Users cannot login using Safari browser"),
        MockIssue("BUG-456", "Auth token expired", "Authentication tokens expire too quickly"),
        MockIssue("FEAT-789", "Add OAuth support", "Implement Google OAuth login"),
    ]
    
    # Generate realistic mock response using LLM
    mock_response = await create_find_related_response(
        prompt=context["prompt"],
        issues=[
            {"id": issue.id, "title": issue.title, "description": issue.description}
            for issue in issues
        ],
        num_related=2,  # Expect 2 related items
        relationship_types=["duplicate", "collaboration"],
        confidence_range=(0.75, 0.95),
    )
    
    # Patch the OpenAI client
    with patch.object(analyzer, "_llm_client", AsyncMock()):
        analyzer.llm_client.chat.completions.create = AsyncMock(return_value=mock_response)
        
        # Execute
        result = await analyzer.find_related_work(context, issues)
        
        # Verify
        assert isinstance(result, list)
        assert len(result) <= 2
        for item in result:
            assert "issue" in item
            assert "confidence" in item
            assert "reasoning" in item
            assert "relationship_type" in item
            assert 0.0 <= item["confidence"] <= 1.0
            assert item["relationship_type"] in ["duplicate", "blocker", "collaboration"]


# Example 2: Testing analyze_session_work with new ideas
@pytest.mark.asyncio
async def test_analyze_session_work_discovers_new_ideas():
    """Test that analyze_session_work correctly extracts new ideas from session."""
    
    config = {}
    analyzer = ActivityAnalyzer(config)
    
    messages = [
        {"role": "user", "content": "Help me implement user authentication"},
        {"role": "assistant", "content": "I'll create an auth module with login/logout"},
        {"role": "user", "content": "We should also handle password reset"},
        {"role": "assistant", "content": "Good point, adding password reset flow"},
        {"role": "user", "content": "What about rate limiting to prevent brute force?"},
        {"role": "assistant", "content": "That's important, let me add that to the list"},
    ]
    
    # Generate realistic mock response
    mock_response = await create_session_work_response(
        messages=messages,
        completed=True,
        num_new_ideas=2,  # Expect password reset and rate limiting ideas
        priority_range=(1, 3),
    )
    
    # Patch the OpenAI client
    with patch.object(analyzer, "_llm_client", AsyncMock()):
        analyzer.llm_client.chat.completions.create = AsyncMock(return_value=mock_response)
        
        # Execute
        result = await analyzer.analyze_session_work(messages)
        
        # Verify
        assert result["completed"] is True
        assert isinstance(result["summary"], str)
        assert len(result["summary"]) > 0
        assert isinstance(result["new_ideas"], list)
        assert len(result["new_ideas"]) == 2
        
        for idea in result["new_ideas"]:
            assert "title" in idea
            assert "description" in idea
            assert "suggested_priority" in idea
            assert 0 <= idea["suggested_priority"] <= 4


# Example 3: Using LLMTestHelper directly for custom scenarios
@pytest.mark.asyncio
async def test_find_related_work_no_matches():
    """Test that find_related_work returns empty list when nothing matches."""
    
    config = {}
    analyzer = ActivityAnalyzer(config)
    helper = LLMTestHelper()
    
    context = {"prompt": "Refactor CSS styles", "working_dir": "/app"}
    
    issues = [
        {"id": "BUG-1", "title": "Database crash", "description": "PostgreSQL crashes"},
        {"id": "BUG-2", "title": "API timeout", "description": "Backend API slow"},
    ]
    
    # Generate response with NO related items
    data = await helper.generate_find_related_work_response(
        prompt=context["prompt"],
        issues=issues,
        num_related=0,  # No matches
    )
    
    # Verify structure
    assert "related" in data
    assert isinstance(data["related"], list)
    assert len(data["related"]) == 0


# Example 4: Testing error handling with malformed responses
@pytest.mark.asyncio
async def test_analyze_session_work_handles_invalid_json():
    """Test that analyzer handles invalid JSON gracefully."""
    
    config = {}
    analyzer = ActivityAnalyzer(config)
    helper = LLMTestHelper()
    
    # Generate intentionally malformed response
    bad_response = await helper.generate_error_response("missing_fields")
    
    # This would be used in a test where you verify error handling
    assert isinstance(bad_response, dict)
    # The analyzer should handle this gracefully and return default values


# Example 5: Custom instruction for edge cases
@pytest.mark.asyncio
async def test_find_related_work_high_confidence_duplicates():
    """Test detection of high-confidence duplicate issues."""
    
    helper = LLMTestHelper()
    
    # Generate response with specific characteristics
    response = await helper.generate_with_instruction(
        instruction="""Generate 1 related issue with:
        - confidence: 0.95 or higher
        - relationship_type: duplicate
        - reasoning that explains it's essentially the same work""",
        context={
            "prompt": "Fix login page not loading",
            "issues": [
                {"id": "BUG-1", "title": "Login screen broken", "description": "Login won't load"},
                {"id": "BUG-2", "title": "Homepage slow", "description": "Home page performance"},
            ],
        },
        format_spec="find_related_work",
    )
    
    # Verify high confidence duplicate
    assert len(response["related"]) == 1
    assert response["related"][0]["confidence"] >= 0.95
    assert response["related"][0]["relationship_type"] == "duplicate"


# Example 6: Parameterized testing with multiple scenarios
@pytest.mark.asyncio
@pytest.mark.parametrize(
    "scenario",
    [
        {"num_related": 0, "expected_len": 0},
        {"num_related": 1, "expected_len": 1},
        {"num_related": 3, "expected_len": 3},
    ],
)
async def test_find_related_work_various_counts(scenario):
    """Test find_related_work with different numbers of related items."""
    
    helper = LLMTestHelper()
    
    response = await helper.generate_find_related_work_response(
        prompt="Fix authentication",
        issues=[
            {"id": f"BUG-{i}", "title": f"Issue {i}", "description": f"Description {i}"}
            for i in range(5)
        ],
        num_related=scenario["num_related"],
    )
    
    assert len(response["related"]) == scenario["expected_len"]


# Example 7: Testing with realistic session transcripts
@pytest.mark.asyncio
async def test_session_analysis_incomplete_work():
    """Test analysis of incomplete session with pending work."""
    
    helper = LLMTestHelper()
    
    messages = [
        {"role": "user", "content": "Start implementing payment processing"},
        {"role": "assistant", "content": "Setting up payment module structure"},
        {"role": "user", "content": "Need to add Stripe integration"},
        {"role": "assistant", "content": "Working on Stripe API connection"},
        # Session ends without completion
    ]
    
    response = await helper.generate_session_work_response(
        messages=messages,
        completed=False,  # Work not finished
        num_new_ideas=1,  # Maybe discovered webhook handling needs
    )
    
    assert response["completed"] is False
    assert "payment" in response["summary"].lower() or "stripe" in response["summary"].lower()
    assert len(response["new_ideas"]) == 1


# Example 8: Integration test with full analyzer workflow
@pytest.mark.asyncio
async def test_full_analyzer_workflow():
    """Test complete workflow from context to related work identification."""
    
    config = {"similarity_threshold": 0.7, "embedding_model": "text-embedding-3-small"}
    analyzer = ActivityAnalyzer(config)
    
    context = {
        "prompt": "Implement caching layer for API responses",
        "working_dir": "/app/backend",
        "git_status": "modified: src/api/handlers.py",
        "recent_files": ["src/api/handlers.py", "src/cache.py"],
    }
    
    class MockIssue:
        def __init__(self, id, title, description):
            self.id = id
            self.title = title
            self.description = description
    
    issues = [
        MockIssue("PERF-1", "API response slow", "Need to optimize API performance"),
        MockIssue("FEAT-2", "Add Redis cache", "Implement Redis caching layer"),
        MockIssue("BUG-3", "Database timeout", "DB queries timing out"),
    ]
    
    # Generate realistic response
    mock_response = await create_find_related_response(
        prompt=context["prompt"],
        issues=[{"id": i.id, "title": i.title, "description": i.description} for i in issues],
        num_related=2,
        relationship_types=["duplicate", "collaboration"],
    )
    
    # Mock LLM client and embedding generator to avoid real API calls
    with patch.object(analyzer, "_llm_client", AsyncMock()), \
         patch.object(analyzer, "_embedding_generator", None):
        
        analyzer.llm_client.chat.completions.create = AsyncMock(return_value=mock_response)
        
        # This will trigger LLM-only analysis path
        result = await analyzer.find_related_work(context, issues)
        
        # Verify we got structured results
        assert isinstance(result, list)
        for item in result:
            assert hasattr(item["issue"], "id")
            assert isinstance(item["confidence"], (int, float))
            assert isinstance(item["reasoning"], str)


if __name__ == "__main__":
    # Run examples
    import asyncio
    
    print("Running example tests...")
    print("\nExample 1: Testing with duplicates")
    asyncio.run(test_find_related_work_with_duplicates())
    print("✓ Passed")
    
    print("\nExample 2: Testing session analysis")
    asyncio.run(test_analyze_session_work_discovers_new_ideas())
    print("✓ Passed")
    
    print("\nExample 3: Testing no matches")
    asyncio.run(test_find_related_work_no_matches())
    print("✓ Passed")
    
    print("\nAll examples passed!")
