"""Integration tests for full workflow."""

import pytest
from unittest.mock import Mock, AsyncMock
from amplifier_module_hooks_activity_tracker.hooks import ActivityTrackerHook


class MockIssueManager:
    """Mock IssueManager that simulates issue-manager behavior."""
    
    def __init__(self):
        self.issues = {}
        self.dependencies = []
        self.next_id = 1

    def create_issue(self, **kwargs):
        """Create a mock issue."""
        issue_id = f"test-{self.next_id}"
        self.next_id += 1
        
        issue = Mock()
        issue.id = issue_id
        issue.title = kwargs.get('title', 'Test Issue')
        issue.description = kwargs.get('description', '')
        issue.status = kwargs.get('status', 'open')
        issue.priority = kwargs.get('priority', 2)
        issue.issue_type = kwargs.get('issue_type', 'task')
        issue.metadata = kwargs.get('metadata', {})
        issue.discovered_from = kwargs.get('discovered_from')
        
        self.issues[issue_id] = issue
        return issue

    def update_issue(self, issue_id, **kwargs):
        """Update a mock issue."""
        if issue_id in self.issues:
            issue = self.issues[issue_id]
            for key, value in kwargs.items():
                setattr(issue, key, value)
            return issue
        return None

    def close_issue(self, issue_id, reason=None):
        """Close a mock issue."""
        if issue_id in self.issues:
            issue = self.issues[issue_id]
            issue.status = "closed"
            issue.closed_reason = reason
            return issue
        return None

    def list_issues(self, status=None):
        """List mock issues."""
        if status:
            return [i for i in self.issues.values() if i.status == status]
        return list(self.issues.values())

    def add_dependency(self, from_id, to_id, dep_type="blocks"):
        """Add a dependency."""
        self.dependencies.append({
            "from": from_id,
            "to": to_id,
            "type": dep_type
        })


class TestFullWorkflow:
    """Test complete session lifecycle."""

    @pytest.fixture
    def mock_issue_manager_integration(self):
        """Create mock issue manager with state tracking."""
        return MockIssueManager()

    @pytest.fixture
    def coordinator_with_issue_manager(self, mock_issue_manager_integration):
        """Create coordinator with mock issue manager."""
        coordinator = Mock()
        coordinator.get = Mock(return_value=mock_issue_manager_integration)
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
        assert "Session:" in session_issue.title
        assert session_issue.status != "closed"

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
        assert session_issue.status == "closed"

        # Verify new idea filed
        assert len(mock_issue_manager_integration.issues) == 2
        new_idea = [i for i in mock_issue_manager_integration.issues.values() if i != session_issue][0]
        assert "rate limiting" in new_idea.title.lower()

        # Verify dependency created
        assert len(mock_issue_manager_integration.dependencies) >= 1
        dep = mock_issue_manager_integration.dependencies[0]
        assert dep["type"] == "discovered-from"

    @pytest.mark.asyncio
    async def test_duplicate_detection_workflow(
        self, mock_config, coordinator_with_issue_manager, mock_issue_manager_integration, mock_issue
    ):
        """Test workflow with duplicate detection."""
        # Create existing issue
        existing = mock_issue_manager_integration.create_issue(
            title="Implement OAuth authentication",
            description="Add OAuth 2.0 support"
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
        assert session_issue.status != "closed"

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
