"""Simple tests for EmbeddingGenerator - matching actual API."""

import pytest
import numpy as np
from unittest.mock import AsyncMock, MagicMock
from amplifier_module_hooks_activity_tracker.embedding_generator import EmbeddingGenerator


class TestEmbeddingGeneratorSimple:
    """Simple tests matching actual implementation."""
    
    def test_init_default_model(self, mock_config):
        """Test initialization with default model."""
        generator = EmbeddingGenerator(mock_config)
        assert generator.model == "text-embedding-3-small"
        assert generator._client is None
    
    def test_init_custom_model(self):
        """Test initialization with custom model."""
        config = {"embedding_model": "custom-model"}
        generator = EmbeddingGenerator(config)
        assert generator.model == "custom-model"
    
    @pytest.mark.asyncio
    async def test_generate_empty_text(self, mock_config):
        """Test generation with empty text."""
        generator = EmbeddingGenerator(mock_config)
        
        result = await generator.generate("")
        assert result is None
        
        result = await generator.generate("   ")
        assert result is None
    
    @pytest.mark.asyncio
    async def test_generate_no_client(self, mock_config):
        """Test generation when client is None."""
        generator = EmbeddingGenerator(mock_config)
        generator._client = None
        
        result = await generator.generate("test")
        assert result is None
    
    @pytest.mark.asyncio
    async def test_generate_success(self, mock_config):
        """Test successful embedding generation."""
        generator = EmbeddingGenerator(mock_config)
        
        # Mock client and response
        mock_response = MagicMock()
        mock_response.data = [MagicMock()]
        mock_response.data[0].embedding = [0.1, 0.2, 0.3]
        
        mock_client = AsyncMock()
        mock_client.embeddings.create = AsyncMock(return_value=mock_response)
        generator._client = mock_client
        
        result = await generator.generate("test text")
        
        assert isinstance(result, np.ndarray)
        assert result.shape == (3,)
        assert np.allclose(result, [0.1, 0.2, 0.3])
    
    @pytest.mark.asyncio
    async def test_generate_api_error(self, mock_config):
        """Test handling of API errors."""
        generator = EmbeddingGenerator(mock_config)
        
        mock_client = AsyncMock()
        mock_client.embeddings.create = AsyncMock(side_effect=Exception("API Error"))
        generator._client = mock_client
        
        result = await generator.generate("test")
        assert result is None
    
    @pytest.mark.asyncio
    async def test_generate_batch_empty(self, mock_config):
        """Test batch generation with empty list."""
        generator = EmbeddingGenerator(mock_config)
        
        result = await generator.generate_batch([])
        assert result == []
    
    @pytest.mark.asyncio
    async def test_generate_batch_single_item(self, mock_config):
        """Test batch generation with single item."""
        generator = EmbeddingGenerator(mock_config)
        
        mock_response = MagicMock()
        mock_response.data = [MagicMock()]
        mock_response.data[0].embedding = [0.1, 0.2]
        
        mock_client = AsyncMock()
        mock_client.embeddings.create = AsyncMock(return_value=mock_response)
        generator._client = mock_client
        
        result = await generator.generate_batch(["text1"])
        
        assert len(result) == 1
        assert isinstance(result[0], np.ndarray)
    
    def test_validate_embedding_valid(self, mock_config):
        """Test embedding validation with valid embedding."""
        generator = EmbeddingGenerator(mock_config)
        
        valid_embedding = np.array([0.1, 0.2, 0.3])
        assert generator._validate_embedding(valid_embedding) == True
    
    def test_validate_embedding_empty(self, mock_config):
        """Test embedding validation with empty array."""
        generator = EmbeddingGenerator(mock_config)
        
        empty_embedding = np.array([])
        assert generator._validate_embedding(empty_embedding) == False
    
    def test_validate_embedding_nan(self, mock_config):
        """Test embedding validation with NaN values."""
        generator = EmbeddingGenerator(mock_config)
        
        nan_embedding = np.array([0.1, np.nan, 0.3])
        assert generator._validate_embedding(nan_embedding) == False
