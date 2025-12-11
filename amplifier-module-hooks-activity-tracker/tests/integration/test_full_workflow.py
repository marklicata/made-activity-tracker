"""Integration tests for full workflow."""

import pytest
from unittest.mock import Mock, AsyncMock
from amplifier_module_hooks_activity_tracker.hooks import ActivityTrackerHook


class MockGitHubTools:
    """Mock GitHub tools that simulates GitHub API behavior."""
    
    def __init__(self):
        self.issues = {}
        self.next_id = 1

    def create_issue(self, title=None, body=None, **kwargs):
        """Create a mock issue."""
        issue_number = self.next_id
        self.next_id += 1
        
        issue = {
            "number": issue_number,
            "title": title or "Untitled",
            "body": body or "",
            "state": "open",
            "author": "testuser",
            "labels": [],
            "assignees": [],
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "closed_at": None,
            "url": f"https://github.com/owner/repo/issues/{issue_number}",
        }
        
        self.issues[issue_number] = issue
        return issue

    def update_issue(self, issue_number=None, **kwargs):
        """Update a mock issue."""
        if issue_number and issue_number in self.issues:
            issue = self.issues[issue_number]
            if "title" in kwargs:
                issue["title"] = kwargs["title"]
            if "body" in kwargs:
                issue["body"] = kwargs["body"]
            if "state" in kwargs:
                issue["state"] = kwargs["state"]
            return issue
        return None

    def list_issues(self, state=None, **kwargs):
        """List mock issues."""
        if state:
            return [i for i in self.issues.values() if i["state"] == state]
        return list(self.issues.values())


class TestFullWorkflow:
    """Test complete session lifecycle."""

    @pytest.fixture
    def mock_issue_manager_integration(self):
        """Create mock GitHub tools with state tracking."""
        return MockGitHubTools()

    @pytest.fixture
    def coordinator_with_issue_manager(self, mock_issue_manager_integration):
        """Create coordinator with mock GitHub tools."""
        coordinator = Mock()
        
        # Mock call_tool to interact with GitHub tools
        async def mock_call_tool(tool_name, params):
            if tool_name == "github_create_issue":
                issue = mock_issue_manager_integration.create_issue(
                    title=params.get("title"), body=params.get("body")
                )
                return {"success": True, "output": {"issue": issue}}
            elif tool_name == "github_update_issue":
                # Extract issue_number and pass other params
                issue_number = params.get("issue_number")
                update_params = {k: v for k, v in params.items() if k != "issue_number"}
                issue = mock_issue_manager_integration.update_issue(
                    issue_number=issue_number, **update_params
                )
                return {"success": True, "output": {"issue": issue}}
            elif tool_name == "github_list_issues":
                issues = mock_issue_manager_integration.list_issues(
                    state=params.get("state")
                )
                return {"success": True, "output": {"issues": issues}}
            return {"success": False, "error": {"message": "Unknown tool"}}
        
        coordinator.call_tool = AsyncMock(side_effect=mock_call_tool)
        return coordinator

    @pytest.mark.asyncio
    async def test_complete_session_lifecycle(
        self, mock_config, coordinator_with_issue_manager, mock_issue_manager_integration
    ):
        """Test complete session from start to end."""
        # Create hook
        hook = ActivityTrackerHook(mock_config)

        # Mock analyzer to avoid real LLM calls
        mock_analyzer = Mock()
        mock_analyzer.find_related_work = AsyncMock(return_value=[])
        mock_analyzer.analyze_session_work = AsyncMock(
            return_value={
                "completed": True,
                "summary": "Implemented authentication feature",
                "new_ideas": [
                    {
                        "title": "Add rate limiting",
                        "description": "Prevent brute force",
                        "suggested_priority": 1,
                    }
                ],
            }
        )
        hook._analyzer = mock_analyzer

        # Mock group manager
        mock_group_manager = Mock()
        mock_group_manager.get_group_for_repo = Mock(return_value=(None, None))
        hook._group_manager = mock_group_manager

        # Session start
        start_event = {
            "session_id": "test-session-123",
            "initial_prompt": "Implement user authentication",
            "coordinator": coordinator_with_issue_manager,
        }

        await hook.on_session_start(start_event)

        # Verify session tracking issue created
        assert len(mock_issue_manager_integration.issues) == 1
        session_issue = list(mock_issue_manager_integration.issues.values())[0]
        assert "Session:" in session_issue["title"]
        assert session_issue["state"] != "closed"

        # Session end
        end_event = {
            "session_id": "test-session-123",
            "coordinator": coordinator_with_issue_manager,
            "messages": [
                {"role": "user", "content": "Implement auth"},
                {"role": "assistant", "content": "Done"},
            ],
        }

        await hook.on_session_end(end_event)

        # Verify session issue closed
        assert session_issue["state"] == "closed"

        # Verify new idea filed
        assert len(mock_issue_manager_integration.issues) == 2
        new_idea = [i for i in mock_issue_manager_integration.issues.values() if i != session_issue][0]
        assert "rate limiting" in new_idea["title"].lower()

    @pytest.mark.asyncio
    async def test_duplicate_detection_workflow(
        self, mock_config, coordinator_with_issue_manager, mock_issue_manager_integration, mock_issue
    ):
        """Test workflow with duplicate detection."""
        # Create existing issue
        existing = mock_issue_manager_integration.create_issue(
            title="Implement OAuth authentication",
            body="Add OAuth 2.0 support"
        )

        # Create hook
        hook = ActivityTrackerHook(mock_config)

        # Mock analyzer to return duplicate
        mock_analyzer = Mock()
        mock_analyzer.find_related_work = AsyncMock(
            return_value=[
                {
                    "issue": existing,
                    "confidence": 0.95,
                    "reasoning": "Both implement OAuth authentication",
                    "relationship_type": "duplicate",
                }
            ]
        )
        hook._analyzer = mock_analyzer

        # Mock group manager
        mock_group_manager = Mock()
        mock_group_manager.get_group_for_repo = Mock(return_value=(None, None))
        hook._group_manager = mock_group_manager

        # Session start with similar prompt
        start_event = {
            "session_id": "test-session-456",
            "initial_prompt": "Add OAuth authentication to the app",
            "coordinator": coordinator_with_issue_manager,
        }

        # Should not raise exception
        await hook.on_session_start(start_event)

        # Verify analyzer was called with existing issues
        assert mock_analyzer.find_related_work.called

    @pytest.mark.asyncio
    async def test_session_without_completion(
        self, mock_config, coordinator_with_issue_manager, mock_issue_manager_integration
    ):
        """Test session that doesn't complete the task."""
        hook = ActivityTrackerHook(mock_config)

        # Mock analyzer to indicate incomplete work
        mock_analyzer = Mock()
        mock_analyzer.find_related_work = AsyncMock(return_value=[])
        mock_analyzer.analyze_session_work = AsyncMock(
            return_value={
                "completed": False,
                "summary": "Made progress but not finished",
                "new_ideas": [],
            }
        )
        hook._analyzer = mock_analyzer

        mock_group_manager = Mock()
        mock_group_manager.get_group_for_repo = Mock(return_value=(None, None))
        hook._group_manager = mock_group_manager

        # Session lifecycle
        start_event = {
            "session_id": "incomplete-session",
            "initial_prompt": "Long task",
            "coordinator": coordinator_with_issue_manager,
        }
        await hook.on_session_start(start_event)

        end_event = {
            "session_id": "incomplete-session",
            "coordinator": coordinator_with_issue_manager,
            "messages": [{"role": "user", "content": "Started but not done"}],
        }
        await hook.on_session_end(end_event)

        # Verify issue is updated but not closed
        session_issue = list(mock_issue_manager_integration.issues.values())[0]
        assert session_issue["state"] != "closed"

    @pytest.mark.asyncio
    async def test_error_recovery_workflow(
        self, mock_config, coordinator_with_issue_manager, mock_issue_manager_integration
    ):
        """Test workflow with errors during processing."""
        hook = ActivityTrackerHook(mock_config)

        # Mock analyzer to fail
        mock_analyzer = Mock()
        mock_analyzer.find_related_work = AsyncMock(
            side_effect=Exception("LLM API error")
        )
        hook._analyzer = mock_analyzer

        # Mock group manager
        mock_group_manager = Mock()
        mock_group_manager.get_group_for_repo = Mock(return_value=(None, None))
        hook._group_manager = mock_group_manager

        # Session start - should not crash despite analyzer failure
        start_event = {
            "session_id": "test-session-error",
            "initial_prompt": "Test error handling",
            "coordinator": coordinator_with_issue_manager,
        }

        await hook.on_session_start(start_event)

        # Verify tracking issue still created despite analysis failure
        assert len(mock_issue_manager_integration.issues) == 1
