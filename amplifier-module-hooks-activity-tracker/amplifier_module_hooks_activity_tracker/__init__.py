"""Activity tracking hook module for Amplifier.

Provides automatic work tracking with LLM-powered duplicate detection.
"""

from typing import Any

__version__ = "1.0.0"
__all__ = ["mount"]


async def mount(coordinator, config: dict[str, Any] | None = None) -> None:
    """Mount activity tracking hook module.

    Args:
        coordinator: Module coordinator from Amplifier
        config: Configuration dict with optional keys:
            - notify_threshold: Confidence threshold for notifications (default: 0.85)
            - embedding_model: Embedding model name (default: text-embedding-3-small)
            - similarity_threshold: Embedding similarity threshold (default: 0.7)
            - auto_track_sessions: Auto-create tracking issues (default: True)
            - auto_file_ideas: Auto-file discovered ideas (default: True)
            - silent_mode: Suppress notifications (default: False)

    Returns:
        None (hook registered with coordinator)
    """
    import logging

    from .hooks import ActivityTrackerHook

    logger = logging.getLogger(__name__)

    config = config or {}

    # Validate configuration
    config.setdefault("notify_threshold", 0.85)
    config.setdefault("embedding_model", "text-embedding-3-small")
    config.setdefault("similarity_threshold", 0.7)
    config.setdefault("auto_track_sessions", True)
    config.setdefault("auto_file_ideas", True)
    config.setdefault("silent_mode", False)

    logger.info(f"Mounting activity-tracker hook with config: {config}")

    try:
        # Create hook instance
        hook = ActivityTrackerHook(config)

        # Register lifecycle hooks
        coordinator.on("session:start", hook.on_session_start)
        coordinator.on("session:end", hook.on_session_end)

        logger.info("Activity tracking hook mounted successfully")

    except Exception as e:
        logger.error(f"Failed to mount activity-tracker hook: {e}")
        raise

    return None
