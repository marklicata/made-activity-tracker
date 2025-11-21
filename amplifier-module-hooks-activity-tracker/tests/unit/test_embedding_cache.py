"""Unit tests for embedding cache."""

import pytest
import numpy as np
from pathlib import Path
from amplifier_module_hooks_activity_tracker.embedding_cache import EmbeddingCache


class TestEmbeddingCache:
    """Test EmbeddingCache class."""

    @pytest.fixture
    def cache(self, temp_cache_dir):
        """Create cache instance."""
        cache_path = temp_cache_dir / "test_cache.db"
        return EmbeddingCache(cache_path)

    @pytest.fixture
    def sample_embedding(self):
        """Create sample embedding vector."""
        return np.array([0.1, 0.2, 0.3, 0.4, 0.5], dtype=np.float32)

    @pytest.mark.asyncio
    async def test_cache_miss(self, cache):
        """Test cache miss returns None."""
        result = await cache.get("nonexistent-id", "some-hash")
        assert result is None

    @pytest.mark.asyncio
    async def test_cache_set_and_get(self, cache, sample_embedding):
        """Test storing and retrieving embedding."""
        issue_id = "test-issue-123"
        content_hash = "abc123"
        model = "test-model"

        # Store embedding
        await cache.set(issue_id, sample_embedding, model, content_hash)

        # Retrieve embedding
        result = await cache.get(issue_id, content_hash)

        assert result is not None
        assert isinstance(result, np.ndarray)
        assert np.allclose(result, sample_embedding)

    @pytest.mark.asyncio
    async def test_cache_invalidation_on_content_change(self, cache, sample_embedding):
        """Test cache miss when content hash changes."""
        issue_id = "test-issue-123"
        old_hash = "old-hash"
        new_hash = "new-hash"

        # Store with old hash
        await cache.set(issue_id, sample_embedding, "model", old_hash)

        # Try to retrieve with new hash
        result = await cache.get(issue_id, new_hash)
        assert result is None

    @pytest.mark.asyncio
    async def test_cache_update_replaces(self, cache, sample_embedding):
        """Test updating an existing cache entry."""
        issue_id = "test-issue-123"
        content_hash = "same-hash"

        # Store first embedding
        embedding1 = sample_embedding
        await cache.set(issue_id, embedding1, "model1", content_hash)

        # Store second embedding (should replace)
        embedding2 = sample_embedding * 2
        await cache.set(issue_id, embedding2, "model2", content_hash)

        # Should get second embedding
        result = await cache.get(issue_id, content_hash)
        assert np.allclose(result, embedding2)

    def test_invalidate(self, cache, sample_embedding):
        """Test cache invalidation."""
        import asyncio

        issue_id = "test-issue-123"
        content_hash = "abc123"

        # Store embedding
        asyncio.run(cache.set(issue_id, sample_embedding, "model", content_hash))

        # Invalidate
        cache.invalidate(issue_id)

        # Should be gone
        result = asyncio.run(cache.get(issue_id, content_hash))
        assert result is None

    def test_clear(self, cache, sample_embedding):
        """Test clearing entire cache."""
        import asyncio

        # Store multiple embeddings
        asyncio.run(cache.set("issue-1", sample_embedding, "model", "hash1"))
        asyncio.run(cache.set("issue-2", sample_embedding * 2, "model", "hash2"))

        # Clear cache
        cache.clear()

        # All should be gone
        result1 = asyncio.run(cache.get("issue-1", "hash1"))
        result2 = asyncio.run(cache.get("issue-2", "hash2"))
        assert result1 is None
        assert result2 is None

    def test_get_stats_empty(self, cache):
        """Test stats on empty cache."""
        stats = cache.get_stats()

        assert stats["total_entries"] == 0
        assert stats["model_count"] == 0

    def test_get_stats_with_entries(self, cache, sample_embedding):
        """Test stats with entries."""
        import asyncio

        # Add entries
        asyncio.run(cache.set("issue-1", sample_embedding, "model-a", "hash1"))
        asyncio.run(cache.set("issue-2", sample_embedding, "model-b", "hash2"))

        stats = cache.get_stats()

        assert stats["total_entries"] == 2
        assert stats["model_count"] == 2

    @pytest.mark.asyncio
    async def test_database_persistence(self, temp_cache_dir, sample_embedding):
        """Test cache persists across instances."""
        cache_path = temp_cache_dir / "persist_test.db"

        # Create first cache and store data
        cache1 = EmbeddingCache(cache_path)
        await cache1.set("test-id", sample_embedding, "model", "hash")

        # Create second cache instance (should load existing DB)
        cache2 = EmbeddingCache(cache_path)
        result = await cache2.get("test-id", "hash")

        assert result is not None
        assert np.allclose(result, sample_embedding)

    @pytest.mark.asyncio
    async def test_multiple_issues(self, cache, sample_embedding):
        """Test caching multiple issues."""
        issues = [
            ("issue-1", "hash-1", sample_embedding),
            ("issue-2", "hash-2", sample_embedding * 2),
            ("issue-3", "hash-3", sample_embedding * 3),
        ]

        # Store all
        for issue_id, content_hash, embedding in issues:
            await cache.set(issue_id, embedding, "model", content_hash)

        # Retrieve all
        for issue_id, content_hash, expected_embedding in issues:
            result = await cache.get(issue_id, content_hash)
            assert result is not None
            assert np.allclose(result, expected_embedding)

    def test_error_handling_corrupt_data(self, cache, sample_embedding):
        """Test handling of corrupt cache data."""
        import asyncio
        import sqlite3

        issue_id = "test-issue"
        content_hash = "hash"

        # Store valid data
        asyncio.run(cache.set(issue_id, sample_embedding, "model", content_hash))

        # Corrupt the data directly in DB
        conn = sqlite3.connect(str(cache.cache_path))
        conn.execute(
            "UPDATE embeddings SET embedding = ? WHERE issue_id = ?",
            (b"corrupt-data", issue_id),
        )
        conn.commit()
        conn.close()

        # Try to retrieve - should return None gracefully
        result = asyncio.run(cache.get(issue_id, content_hash))
        assert result is None
