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
        "repository": "owner/repo",
        "notify_threshold": 0.85,
        "embedding_model": "text-embedding-3-small",
        "similarity_threshold": 0.7,
        "auto_track_sessions": True,
        "auto_file_ideas": True,
        "silent_mode": False,
    }


@pytest.fixture
def mock_issue():
    """Create a mock issue dict (GitHub API format)."""
    def _create_issue(
        issue_id=None,
        number=None,
        title="Test Issue",
        body="Test description",
        state="open",
        labels=None,
        assignees=None,
    ):
        # Support both issue_id and number for backwards compatibility
        issue_number = number or issue_id or 123
        return {
            "number": issue_number,
            "title": title,
            "body": body,
            "state": state,
            "author": "testuser",
            "labels": labels or [],
            "assignees": assignees or [],
            "created_at": datetime.now().isoformat(),
            "updated_at": datetime.now().isoformat(),
            "closed_at": None,
            "comments": 0,
            "url": f"https://github.com/owner/repo/issues/{issue_number}",
        }
    
    return _create_issue


@pytest.fixture
def mock_github_tools():
    """Create mock responses for GitHub tools."""
    def _create_tool_response(success=True, output=None, error=None):
        return {
            "success": success,
            "output": output or {},
            "error": error,
        }
    
    return _create_tool_response


@pytest.fixture
def mock_coordinator(mock_github_tools):
    """Create a mock coordinator with GitHub tools."""
    coordinator = Mock()
    coordinator.on = Mock()
    coordinator.emit = AsyncMock()
    
    # Mock call_tool to return GitHub-style responses
    async def mock_call_tool(tool_name, params):
        if tool_name == "github_list_issues":
            return mock_github_tools(success=True, output={"issues": [], "count": 0})
        elif tool_name == "github_create_issue":
            return mock_github_tools(success=True, output={"issue": {"number": 123, "title": params.get("title")}})
        elif tool_name == "github_update_issue":
            return mock_github_tools(success=True, output={"issue": {"number": params.get("issue_number")}})
        else:
            return mock_github_tools(success=False, error={"message": "Unknown tool"})
    
    coordinator.call_tool = AsyncMock(side_effect=mock_call_tool)
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
