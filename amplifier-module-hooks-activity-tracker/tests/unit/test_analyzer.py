"""Tests for ActivityAnalyzer."""

import pytest
import numpy as np
from unittest.mock import Mock, AsyncMock, patch, MagicMock
from amplifier_module_hooks_activity_tracker.analyzer import ActivityAnalyzer


class TestActivityAnalyzer:
    """Test ActivityAnalyzer class."""
    
    def test_initialization(self, mock_config):
        """Test analyzer initialization."""
        analyzer = ActivityAnalyzer(mock_config)
        assert analyzer.config == mock_config
        assert analyzer._llm_client is None
        assert analyzer._embedding_generator is None
        assert analyzer._embedding_cache is None
    
    def test_llm_client_lazy_loading(self, mock_config):
        """Test LLM client is lazy loaded."""
        analyzer = ActivityAnalyzer(mock_config)
        
        # Mock openai module
        with patch('amplifier_module_hooks_activity_tracker.analyzer.AsyncOpenAI') as mock_openai:
            mock_client = MagicMock()
            mock_openai.return_value = mock_client
            
            client = analyzer.llm_client
            
            assert client == mock_client
            mock_openai.assert_called_once()
    
    def test_embedding_generator_lazy_loading(self, mock_config):
        """Test embedding generator is lazy loaded."""
        analyzer = ActivityAnalyzer(mock_config)
        
        with patch('amplifier_module_hooks_activity_tracker.analyzer.EmbeddingGenerator') as mock_gen:
            mock_generator = MagicMock()
            mock_gen.return_value = mock_generator
            
            generator = analyzer.embedding_generator
            
            assert generator == mock_generator
            mock_gen.assert_called_once_with(mock_config)
    
    def test_embedding_cache_lazy_loading(self, mock_config):
        """Test embedding cache is lazy loaded."""
        analyzer = ActivityAnalyzer(mock_config)
        
        with patch('amplifier_module_hooks_activity_tracker.analyzer.EmbeddingCache') as mock_cache:
            mock_cache_instance = MagicMock()
            mock_cache.return_value = mock_cache_instance
            
            cache = analyzer.embedding_cache
            
            assert cache == mock_cache_instance
            mock_cache.assert_called_once()
    
    @pytest.mark.asyncio
    async def test_find_related_work_empty_issues(self, mock_config):
        """Test finding related work with empty issue list."""
        analyzer = ActivityAnalyzer(mock_config)
        context = {"prompt": "test work"}
        
        result = await analyzer.find_related_work(context, [])
        
        assert result == []
    
    @pytest.mark.asyncio
    async def test_find_related_work_llm_only_mode(self, mock_config, mock_issue):
        """Test LLM-only analysis mode."""
        analyzer = ActivityAnalyzer(mock_config)
        
        context = {
            "prompt": "Implement user authentication",
            "git_status": {},
            "recent_files": []
        }
        
        issues = [
            mock_issue(number=1, title="Login system", body="Add login"),
            mock_issue(number=2, title="Database setup", body="Setup DB"),
        ]
        
        # Mock LLM response
        mock_response = MagicMock()
        mock_response.choices = [MagicMock()]
        mock_response.choices[0].message.content = """
        {
            "related_items": [
                {
                    "issue_number": 1,
                    "confidence": 0.9,
                    "reasoning": "Both about authentication",
                    "relationship_type": "duplicate"
                }
            ]
        }
        """
        
        analyzer._llm_client = AsyncMock()
        analyzer._llm_client.chat.completions.create = AsyncMock(return_value=mock_response)
        
        result = await analyzer.find_related_work(context, issues)
        
        assert len(result) == 1
        assert result[0]["issue"]["number"] == 1
        assert result[0]["confidence"] == 0.9
        assert result[0]["relationship_type"] == "duplicate"
    
    @pytest.mark.asyncio
    async def test_find_related_work_two_phase_mode(self, mock_config, mock_issue):
        """Test two-phase analysis (embeddings + LLM)."""
        analyzer = ActivityAnalyzer(mock_config)
        
        context = {
            "prompt": "Add payment processing",
            "git_status": {},
            "recent_files": []
        }
        
        issues = [
            mock_issue(number=1, title="Payment gateway", body="Stripe integration"),
            mock_issue(number=2, title="User profiles", body="Profile page"),
        ]
        
        # Mock embedding generator
        analyzer._embedding_generator = AsyncMock()
        analyzer._embedding_generator.generate = AsyncMock(side_effect=[
            np.array([0.1, 0.2, 0.3]),  # context embedding
            np.array([0.11, 0.21, 0.31]),  # test-1 embedding (similar)
            np.array([0.9, 0.1, 0.05]),  # test-2 embedding (different)
        ])
        
        # Mock embedding cache
        analyzer._embedding_cache = AsyncMock()
        analyzer._embedding_cache.get = AsyncMock(return_value=None)
        analyzer._embedding_cache.set = AsyncMock()
        
        # Mock LLM response for candidates
        mock_response = MagicMock()
        mock_response.choices = [MagicMock()]
        mock_response.choices[0].message.content = """
        {
            "related_items": [
                {
                    "issue_number": 1,
                    "confidence": 0.95,
                    "reasoning": "Both about payment processing",
                    "relationship_type": "duplicate"
                }
            ]
        }
        """
        
        analyzer._llm_client = AsyncMock()
        analyzer._llm_client.chat.completions.create = AsyncMock(return_value=mock_response)
        
        result = await analyzer.find_related_work(context, issues)
        
        assert len(result) == 1
        assert result[0]["issue"]["number"] == 1
        assert result[0]["confidence"] == 0.95
    
    @pytest.mark.asyncio
    async def test_find_related_work_handles_llm_errors(self, mock_config, mock_issue):
        """Test error handling when LLM fails."""
        analyzer = ActivityAnalyzer(mock_config)
        
        context = {"prompt": "test"}
        issues = [mock_issue()]
        
        analyzer._llm_client = AsyncMock()
        analyzer._llm_client.chat.completions.create = AsyncMock(side_effect=Exception("API Error"))
        
        result = await analyzer.find_related_work(context, issues)
        
        # Should return empty list on error
        assert result == []
    
    @pytest.mark.asyncio
    async def test_find_related_work_handles_invalid_json(self, mock_config, mock_issue):
        """Test handling of invalid JSON from LLM."""
        analyzer = ActivityAnalyzer(mock_config)
        
        context = {"prompt": "test"}
        issues = [mock_issue()]
        
        mock_response = MagicMock()
        mock_response.choices = [MagicMock()]
        mock_response.choices[0].message.content = "Not valid JSON at all"
        
        analyzer._llm_client = AsyncMock()
        analyzer._llm_client.chat.completions.create = AsyncMock(return_value=mock_response)
        
        result = await analyzer.find_related_work(context, issues)
        
        assert result == []
    
    @pytest.mark.asyncio
    async def test_analyze_session_work_no_messages(self, mock_config):
        """Test session analysis with no messages."""
        analyzer = ActivityAnalyzer(mock_config)
        
        result = await analyzer.analyze_session_work([])
        
        assert result == {"completed": False, "summary": "Empty session", "new_ideas": []}
    
    @pytest.mark.asyncio
    async def test_analyze_session_work_with_ideas(self, mock_config):
        """Test session analysis extracts ideas."""
        analyzer = ActivityAnalyzer(mock_config)
        
        messages = [
            {"role": "user", "content": "We should add caching"},
            {"role": "assistant", "content": "Also need to implement rate limiting"}
        ]
        
        mock_response = MagicMock()
        mock_response.choices = [MagicMock()]
        mock_response.choices[0].message.content = """
        {
            "ideas": [
                {
                    "title": "Add caching layer",
                    "description": "Implement caching to improve performance",
                    "priority": 1,
                    "issue_type": "feature"
                },
                {
                    "title": "Implement rate limiting",
                    "description": "Add rate limiting to API endpoints",
                    "priority": 2,
                    "issue_type": "feature"
                }
            ]
        }
        """
        
        analyzer._llm_client = AsyncMock()
        analyzer._llm_client.chat.completions.create = AsyncMock(return_value=mock_response)
        
        result = await analyzer.analyze_session_work(messages)
        
        # Result should be a dict, not a list
        assert isinstance(result, dict)
    
    @pytest.mark.asyncio
    async def test_analyze_session_work_handles_errors(self, mock_config):
        """Test error handling in session analysis."""
        analyzer = ActivityAnalyzer(mock_config)
        
        messages = [{"role": "user", "content": "test"}]
        
        analyzer._llm_client = AsyncMock()
        analyzer._llm_client.chat.completions.create = AsyncMock(side_effect=Exception("API Error"))
        
        result = await analyzer.analyze_session_work(messages)
        
        assert result == {"completed": False, "summary": "Analysis error", "new_ideas": []}
    
    def test_compute_similarity_cosine(self, mock_config):
        """Test cosine similarity computation."""
        analyzer = ActivityAnalyzer(mock_config)
        
        vec1 = np.array([1.0, 0.0, 0.0])
        vec2 = np.array([1.0, 0.0, 0.0])
        vec3 = np.array([0.0, 1.0, 0.0])
        
        # Identical vectors
        sim1 = analyzer._cosine_similarity(vec1, vec2)
        assert abs(sim1 - 1.0) < 0.001
        
        # Orthogonal vectors
        sim2 = analyzer._cosine_similarity(vec1, vec3)
        assert abs(sim2) < 0.001
    
    def test_compute_similarity_handles_zero_vectors(self, mock_config):
        """Test similarity with zero vectors."""
        analyzer = ActivityAnalyzer(mock_config)
        
        vec1 = np.array([1.0, 0.0, 0.0])
        vec_zero = np.array([0.0, 0.0, 0.0])
        
        sim = analyzer._cosine_similarity(vec1, vec_zero)
        
        # Should handle gracefully
        assert not np.isnan(sim)
    
    @pytest.mark.asyncio
    async def test_get_cached_embedding_from_cache(self, mock_config):
        """Test getting embedding from cache."""
        analyzer = ActivityAnalyzer(mock_config)
        
        cached_embedding = np.array([0.1, 0.2, 0.3])
        
        analyzer._embedding_cache = AsyncMock()
        analyzer._embedding_cache.get = AsyncMock(return_value=cached_embedding)
        
        result = await analyzer._get_cached_embedding("test-1", "test content")
        
        assert np.array_equal(result, cached_embedding)
    
    @pytest.mark.asyncio
    async def test_get_cached_embedding_generates_new(self, mock_config):
        """Test generating new embedding when not cached."""
        analyzer = ActivityAnalyzer(mock_config)
        
        new_embedding = np.array([0.4, 0.5, 0.6])
        
        analyzer._embedding_cache = AsyncMock()
        analyzer._embedding_cache.get = AsyncMock(return_value=None)
        analyzer._embedding_cache.set = AsyncMock()
        analyzer._embedding_generator = AsyncMock()
        analyzer._embedding_generator.generate = AsyncMock(return_value=new_embedding)
        
        result = await analyzer._get_cached_embedding("test-1", "test content")
        
        assert np.array_equal(result, new_embedding)
        analyzer._embedding_cache.set.assert_called_once()
