"""Tests for ActivityAnalyzer using LLM-generated mocks."""

import pytest
from unittest.mock import AsyncMock
from amplifier_module_hooks_activity_tracker.analyzer import ActivityAnalyzer
from tests.helpers import LLMMockGenerator


class TestActivityAnalyzerWithLLMMocks:
    """Test analyzer with realistic LLM-generated mock responses."""
    
    @pytest.mark.asyncio
    async def test_find_related_work_with_llm_mock(self, mock_config, mock_issue):
        """Test finding related work with LLM-generated response."""
        analyzer = ActivityAnalyzer(mock_config)
        
        context = {
            "prompt": "Implement user authentication system",
            "git_status": {},
            "recent_files": []
        }
        
        issues = [
            mock_issue(issue_id="issue-1", title="Login page", body="Create login UI"),
            mock_issue(issue_id="issue-2", title="Database schema", body="User tables"),
        ]
        
        # Generate realistic mock response using LLM
        mock_gen = LLMMockGenerator()
        mock_response = mock_gen.generate_find_related_work_response(
            context_prompt=context["prompt"],
            issue_titles=["Login page", "Database schema"],
            num_related=1
        )
        
        # Create mock client
        mock_client = mock_gen.create_mock_llm_client([mock_response])
        analyzer._llm_client = mock_client
        
        # Test
        result = await analyzer.find_related_work(context, issues)
        
        # Verify structure (content will be LLM-generated but realistic)
        assert isinstance(result, list)
        if len(result) > 0:  # LLM might find matches
            assert "issue" in result[0] or "confidence" in result[0]
    
    @pytest.mark.asyncio
    async def test_analyze_session_work_with_llm_mock(self, mock_config):
        """Test session analysis with LLM-generated response."""
        analyzer = ActivityAnalyzer(mock_config)
        
        messages = [
            {"role": "user", "content": "I need to add caching to the API"},
            {"role": "assistant", "content": "I'll help you implement Redis caching"},
            {"role": "user", "content": "Great, let's also add rate limiting"},
        ]
        
        # Generate realistic mock
        mock_gen = LLMMockGenerator()
        mock_response = mock_gen.generate_session_analysis_response(
            messages=messages,
            expect_completion=True
        )
        
        # Create mock client
        mock_client = mock_gen.create_mock_llm_client([mock_response])
        analyzer._llm_client = mock_client
        
        # Test
        result = await analyzer.analyze_session_work(messages)
        
        # Verify structure
        assert isinstance(result, dict)
        assert "completed" in result
        assert "summary" in result
        assert "new_ideas" in result
        assert isinstance(result["new_ideas"], list)
    
    @pytest.mark.asyncio
    async def test_find_related_work_llm_fallback(self, mock_config, mock_issue):
        """Test that static fallback works when LLM unavailable."""
        analyzer = ActivityAnalyzer(mock_config)
        
        context = {"prompt": "test work"}
        issues = [mock_issue()]
        
        # Use static fallback (no OpenAI key needed)
        mock_gen = LLMMockGenerator()
        mock_gen._has_openai = False  # Force fallback
        
        mock_response = mock_gen.generate_find_related_work_response(
            context_prompt="test",
            issue_titles=["test"],
            num_related=1
        )
        
        mock_client = mock_gen.create_mock_llm_client([mock_response])
        analyzer._llm_client = mock_client
        
        # Should work with static mock
        result = await analyzer.find_related_work(context, issues)
        assert isinstance(result, list)
    
    @pytest.mark.asyncio  
    async def test_session_analysis_llm_fallback(self, mock_config):
        """Test session analysis with static fallback."""
        analyzer = ActivityAnalyzer(mock_config)
        
        messages = [{"role": "user", "content": "test"}]
        
        # Force static fallback
        mock_gen = LLMMockGenerator()
        mock_gen._has_openai = False
        
        mock_response = mock_gen.generate_session_analysis_response(
            messages=messages,
            expect_completion=False
        )
        
        mock_client = mock_gen.create_mock_llm_client([mock_response])
        analyzer._llm_client = mock_client
        
        result = await analyzer.analyze_session_work(messages)
        
        assert isinstance(result, dict)
        assert "completed" in result
        assert result["completed"] == False  # As specified
