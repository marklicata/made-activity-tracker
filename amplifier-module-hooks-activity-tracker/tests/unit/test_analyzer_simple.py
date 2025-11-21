"""Simple tests for ActivityAnalyzer - matching actual API."""

import pytest
import numpy as np
from unittest.mock import Mock, AsyncMock, patch
from amplifier_module_hooks_activity_tracker.analyzer import ActivityAnalyzer


class TestActivityAnalyzerSimple:
    """Simple tests matching actual implementation."""
    
    def test_init(self, mock_config):
        """Test basic initialization."""
        analyzer = ActivityAnalyzer(mock_config)
        assert analyzer.config == mock_config
        assert analyzer._llm_client is None
        assert analyzer._embedding_generator is None
        assert analyzer._embedding_cache is None
    
    def test_cosine_similarity_identical(self, mock_config):
        """Test cosine similarity with identical vectors."""
        analyzer = ActivityAnalyzer(mock_config)
        vec1 = np.array([1.0, 0.0, 0.0])
        vec2 = np.array([1.0, 0.0, 0.0])
        
        sim = analyzer._cosine_similarity(vec1, vec2)
        assert abs(sim - 1.0) < 0.001
    
    def test_cosine_similarity_orthogonal(self, mock_config):
        """Test cosine similarity with orthogonal vectors."""
        analyzer = ActivityAnalyzer(mock_config)
        vec1 = np.array([1.0, 0.0, 0.0])
        vec2 = np.array([0.0, 1.0, 0.0])
        
        sim = analyzer._cosine_similarity(vec1, vec2)
        assert abs(sim) < 0.001
    
    @pytest.mark.asyncio
    async def test_find_related_work_empty_issues(self, mock_config):
        """Test with no issues."""
        analyzer = ActivityAnalyzer(mock_config)
        context = {"prompt": "test"}
        
        result = await analyzer.find_related_work(context, [])
        assert result == []
    
    @pytest.mark.asyncio
    async def test_analyze_session_work_empty_messages(self, mock_config):
        """Test session analysis with no messages."""
        analyzer = ActivityAnalyzer(mock_config)
        
        result = await analyzer.analyze_session_work([])
        
        assert isinstance(result, dict)
        assert "completed" in result
        assert "summary" in result
        assert "new_ideas" in result
        assert result["completed"] == False
    
    @pytest.mark.asyncio
    async def test_analyze_session_work_no_client(self, mock_config):
        """Test when LLM client unavailable."""
        analyzer = ActivityAnalyzer(mock_config)
        analyzer._llm_client = None
        
        messages = [{"role": "user", "content": "test"}]
        result = await analyzer.analyze_session_work(messages)
        
        assert isinstance(result, dict)
        assert result["completed"] == False
    
    def test_format_context_for_embedding(self, mock_config):
        """Test context formatting."""
        analyzer = ActivityAnalyzer(mock_config)
        
        context = {
            "prompt": "Test prompt",
            "git_status": {"modified": ["file.py"]},
            "recent_files": ["test.py"]
        }
        
        formatted = analyzer._format_context_for_embedding(context)
        
        assert isinstance(formatted, str)
        assert "Test prompt" in formatted
    
    @pytest.mark.asyncio
    async def test_get_cached_embedding_returns_none_when_no_cache(self, mock_config):
        """Test cached embedding when cache unavailable."""
        analyzer = ActivityAnalyzer(mock_config)
        
        # Mock cache to return None
        mock_cache = AsyncMock()
        mock_cache.get = AsyncMock(return_value=None)
        analyzer._embedding_cache = mock_cache
        
        # Mock generator to return None  
        mock_gen = AsyncMock()
        mock_gen.generate = AsyncMock(return_value=None)
        analyzer._embedding_generator = mock_gen
        
        result = await analyzer._get_cached_embedding("test-id", "test content")
        
        # Should return None when both cache and generator fail
        assert result is None
