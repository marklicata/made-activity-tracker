"""Embedding generator using OpenAI API."""

import logging
import os
from typing import Any

import numpy as np

logger = logging.getLogger(__name__)


class EmbeddingGenerator:
    """Generates embeddings using configured model."""

    def __init__(self, config: dict[str, Any]):
        """Initialize embedding generator.

        Args:
            config: Configuration dictionary with embedding_model key
        """
        self.config = config
        self.model = config.get("embedding_model", "text-embedding-3-small")
        self._client = None

    @property
    def client(self):
        """Lazy-load OpenAI client."""
        if self._client is None:
            try:
                from openai import AsyncOpenAI

                api_key = os.getenv("OPENAI_API_KEY")
                if not api_key:
                    logger.warning("OPENAI_API_KEY not set")
                self._client = AsyncOpenAI(api_key=api_key)
            except Exception as e:
                logger.error(f"Failed to initialize OpenAI client: {e}")
        return self._client

    async def generate(self, text: str) -> np.ndarray | None:
        """Generate embedding for text.

        Args:
            text: Text to embed

        Returns:
            Embedding vector as numpy array or None if failed
        """
        if not text or not text.strip():
            logger.warning("Empty text provided for embedding")
            return None

        if not self.client:
            logger.error("OpenAI client not available")
            return None

        try:
            # Call OpenAI API
            response = await self.client.embeddings.create(
                model=self.model, input=text, encoding_format="float"
            )

            # Extract embedding
            embedding = response.data[0].embedding

            # Convert to numpy array
            embedding_array = np.array(embedding, dtype=np.float32)

            # Validate
            if not self._validate_embedding(embedding_array):
                logger.error("Generated embedding failed validation")
                return None

            return embedding_array

        except Exception as e:
            logger.error(f"Embedding generation failed: {e}")
            return None

    async def generate_batch(self, texts: list[str]) -> list[np.ndarray | None]:
        """Generate embeddings for multiple texts.

        Args:
            texts: List of texts to embed

        Returns:
            List of embedding vectors (or None for failures)
        """
        if not texts:
            return []

        # Filter out empty texts
        valid_texts = [t for t in texts if t and t.strip()]
        if not valid_texts:
            return [None] * len(texts)

        if not self.client:
            logger.error("OpenAI client not available")
            return [None] * len(texts)

        try:
            # Call OpenAI API with batch
            response = await self.client.embeddings.create(
                model=self.model, input=valid_texts, encoding_format="float"
            )

            # Extract embeddings
            embeddings = []
            for item in response.data:
                embedding = np.array(item.embedding, dtype=np.float32)
                if self._validate_embedding(embedding):
                    embeddings.append(embedding)
                else:
                    embeddings.append(None)

            return embeddings

        except Exception as e:
            logger.error(f"Batch embedding generation failed: {e}")
            return [None] * len(texts)

    def _validate_embedding(self, embedding: np.ndarray) -> bool:
        """Validate embedding vector.

        Args:
            embedding: Embedding vector

        Returns:
            True if valid
        """
        if embedding is None:
            return False

        if not isinstance(embedding, np.ndarray):
            return False

        if embedding.size == 0:
            return False

        if not np.isfinite(embedding).all():
            return False

        return True
