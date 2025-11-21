"""Shared test fixtures and configuration."""

import sys
import pytest
from pathlib import Path
from unittest.mock import Mock, AsyncMock, MagicMock
from datetime import datetime
import uuid


# Mock OpenAI module before any imports
@pytest.fixture(scope="session", autouse=True)
def mock_openai_module():
    """Mock the entire openai module to avoid import errors."""
    mock_openai = MagicMock()
    mock_openai.AsyncOpenAI = MagicMock
    sys.modules['openai'] = mock_openai
    return mock_openai


@pytest.fixture(autouse=True)
def mock_environment(monkeypatch):
    """Set up test environment variables."""
    monkeypatch.setenv('OPENAI_API_KEY', 'test-key-12345')


@pytest.fixture
def mock_config():
    """Standard configuration for testing."""
    return {
        "notify_threshold": 0.85,
        "embedding_model": "text-embedding-3-small",
        "similarity_threshold": 0.7,
        "auto_track_sessions": True,
        "auto_file_ideas": True,
        "silent_mode": False,
    }


@pytest.fixture
def mock_issue():
    """Create a mock issue object."""
    def _create_issue(
        issue_id=None,
        title="Test Issue",
        description="Test description",
        status="open",
        priority=2,
        issue_type="task",
    ):
        issue = Mock()
        issue.id = issue_id or str(uuid.uuid4())
        issue.title = title
        issue.description = description
        issue.status = status
        issue.priority = priority
        issue.issue_type = issue_type
        issue.assignee = None
        issue.created_at = datetime.now()
        issue.updated_at = datetime.now()
        issue.closed_at = None
        issue.metadata = {}
        return issue
    
    return _create_issue


@pytest.fixture
def mock_issue_manager():
    """Create a mock issue manager with predictable behavior."""
    manager = Mock()
    
    # Create a fresh Mock each time to avoid state bleeding
    created_issue = Mock()
    created_issue.id = "test-issue-123"
    created_issue.title = "Test Issue"
    created_issue.status = "open"
    
    manager.create_issue = Mock(return_value=created_issue)
    manager.update_issue = Mock(return_value=created_issue)
    manager.close_issue = Mock(return_value=created_issue)
    manager.list_issues = Mock(return_value=[])
    manager.add_dependency = Mock()
    
    return manager


@pytest.fixture
def mock_coordinator(mock_issue_manager):
    """Create a mock coordinator."""
    coordinator = Mock()
    coordinator.get = Mock(return_value=mock_issue_manager)
    coordinator.on = Mock()
    coordinator.emit = AsyncMock()
    return coordinator


@pytest.fixture
def sample_context():
    """Sample session context."""
    return {
        "session_id": "test-session-123",
        "prompt": "Implement user authentication",
        "working_dir": "/test/path",
        "git_status": "M src/auth.py\n?? new_file.py",
        "recent_files": ["src/auth.py", "tests/test_auth.py"],
        "timestamp": datetime.now().isoformat(),
    }


@pytest.fixture
def sample_event_data(sample_context):
    """Sample event data for hooks.
    
    Note: coordinator is NOT included here to avoid fixture dependency issues.
    Tests should add coordinator manually.
    """
    return {
        "session_id": sample_context["session_id"],
        "initial_prompt": sample_context["prompt"],
        "messages": [
            {"role": "user", "content": "Implement authentication"},
            {"role": "assistant", "content": "I'll implement OAuth 2.0"},
        ],
    }


@pytest.fixture
def temp_cache_dir(tmp_path):
    """Temporary directory for cache testing."""
    cache_dir = tmp_path / ".amplifier"
    cache_dir.mkdir()
    return cache_dir


@pytest.fixture
def mock_llm_response():
    """Mock LLM response for analysis."""
    return {
        "related": [
            {
                "issue_id": "test-123",
                "confidence": 0.9,
                "reasoning": "Both tasks involve implementing authentication",
                "relationship_type": "duplicate",
            }
        ]
    }


@pytest.fixture
def mock_session_analysis():
    """Mock session analysis result."""
    return {
        "completed": True,
        "summary": "Implemented OAuth 2.0 authentication",
        "new_ideas": [
            {
                "title": "Add rate limiting",
                "description": "Prevent brute force attacks",
                "suggested_priority": 1,
            }
        ],
    }
