"""SQLite-based caching for embeddings."""

import logging
import pickle
import sqlite3
from datetime import datetime
from pathlib import Path
from typing import Any

import numpy as np

logger = logging.getLogger(__name__)


class EmbeddingCache:
    """Caches embeddings to avoid regeneration."""

    def __init__(self, cache_path: Path | None = None):
        """Initialize embedding cache.

        Args:
            cache_path: Path to SQLite database (default: .amplifier/embeddings_cache.db)
        """
        if cache_path is None:
            cache_path = Path.cwd() / ".amplifier" / "embeddings_cache.db"

        self.cache_path = cache_path
        self._init_db()

    def _init_db(self) -> None:
        """Initialize database with schema."""
        # Ensure directory exists
        self.cache_path.parent.mkdir(parents=True, exist_ok=True)

        conn = sqlite3.connect(str(self.cache_path))
        try:
            conn.execute("""
                CREATE TABLE IF NOT EXISTS embeddings (
                    issue_id TEXT PRIMARY KEY,
                    embedding BLOB NOT NULL,
                    content_hash TEXT NOT NULL,
                    model TEXT NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    accessed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            """)

            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_content_hash ON embeddings(content_hash)"
            )
            conn.execute("CREATE INDEX IF NOT EXISTS idx_model ON embeddings(model)")

            conn.commit()
        finally:
            conn.close()

    async def get(self, issue_id: str, content_hash: str) -> np.ndarray | None:
        """Get cached embedding if exists and content unchanged.

        Args:
            issue_id: Issue ID
            content_hash: Hash of issue content

        Returns:
            Embedding vector or None if cache miss
        """
        conn = sqlite3.connect(str(self.cache_path))
        try:
            cursor = conn.execute(
                "SELECT embedding FROM embeddings WHERE issue_id = ? AND content_hash = ?",
                (issue_id, content_hash),
            )
            row = cursor.fetchone()

            if row:
                # Update accessed_at
                conn.execute(
                    "UPDATE embeddings SET accessed_at = ? WHERE issue_id = ?",
                    (datetime.now(), issue_id),
                )
                conn.commit()

                # Deserialize embedding
                try:
                    embedding = pickle.loads(row[0])
                    return embedding
                except Exception as e:
                    logger.error(f"Failed to deserialize embedding for {issue_id}: {e}")
                    return None

            return None

        except Exception as e:
            logger.error(f"Cache get failed for {issue_id}: {e}")
            return None
        finally:
            conn.close()

    async def set(
        self, issue_id: str, embedding: np.ndarray, model: str, content_hash: str
    ) -> None:
        """Store embedding in cache.

        Args:
            issue_id: Issue ID
            embedding: Embedding vector
            model: Model name used
            content_hash: Hash of issue content
        """
        conn = sqlite3.connect(str(self.cache_path))
        try:
            # Serialize embedding
            embedding_blob = pickle.dumps(embedding)

            conn.execute(
                """
                INSERT OR REPLACE INTO embeddings 
                (issue_id, embedding, content_hash, model, created_at, accessed_at)
                VALUES (?, ?, ?, ?, ?, ?)
                """,
                (issue_id, embedding_blob, content_hash, model, datetime.now(), datetime.now()),
            )
            conn.commit()

        except Exception as e:
            logger.error(f"Cache set failed for {issue_id}: {e}")
        finally:
            conn.close()

    def invalidate(self, issue_id: str) -> None:
        """Invalidate cached embedding for issue.

        Args:
            issue_id: Issue ID to invalidate
        """
        conn = sqlite3.connect(str(self.cache_path))
        try:
            conn.execute("DELETE FROM embeddings WHERE issue_id = ?", (issue_id,))
            conn.commit()
        except Exception as e:
            logger.error(f"Cache invalidate failed for {issue_id}: {e}")
        finally:
            conn.close()

    def clear(self) -> None:
        """Clear entire cache (for testing)."""
        conn = sqlite3.connect(str(self.cache_path))
        try:
            conn.execute("DELETE FROM embeddings")
            conn.commit()
            logger.info("Cache cleared")
        except Exception as e:
            logger.error(f"Cache clear failed: {e}")
        finally:
            conn.close()

    def get_stats(self) -> dict[str, Any]:
        """Get cache statistics.

        Returns:
            Dict with total_entries, models, oldest_entry, newest_entry
        """
        conn = sqlite3.connect(str(self.cache_path))
        try:
            cursor = conn.execute(
                """
                SELECT 
                    COUNT(*) as total,
                    COUNT(DISTINCT model) as model_count,
                    MIN(created_at) as oldest,
                    MAX(created_at) as newest
                FROM embeddings
                """
            )
            row = cursor.fetchone()

            if row:
                return {
                    "total_entries": row[0],
                    "model_count": row[1],
                    "oldest_entry": row[2],
                    "newest_entry": row[3],
                }

            return {"total_entries": 0, "model_count": 0, "oldest_entry": None, "newest_entry": None}

        except Exception as e:
            logger.error(f"Failed to get cache stats: {e}")
            return {}
        finally:
            conn.close()
