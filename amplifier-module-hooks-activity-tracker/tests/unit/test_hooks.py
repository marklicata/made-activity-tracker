"""Unit tests for activity tracker hooks."""

import pytest
from unittest.mock import Mock, AsyncMock, patch, MagicMock
from amplifier_module_hooks_activity_tracker.hooks import ActivityTrackerHook


class TestActivityTrackerHook:
    """Test ActivityTrackerHook class."""

    @pytest.fixture
    def hook(self, mock_config):
        """Create hook instance with config."""
        return ActivityTrackerHook(mock_config.copy())

    @pytest.fixture
    def hook_with_mocks(self, mock_config):
        """Create hook with all dependencies pre-mocked."""
        hook = ActivityTrackerHook(mock_config.copy())
        
        # Pre-mock analyzer to avoid lazy loading issues
        hook._analyzer = Mock()
        hook._analyzer.find_related_work = AsyncMock(return_value=[])
        hook._analyzer.analyze_session_work = AsyncMock(return_value={
            "completed": False,
            "summary": "",
            "new_ideas": []
        })
        
        # Pre-mock group_manager to avoid lazy loading issues
        hook._group_manager = Mock()
        hook._group_manager.get_group_for_repo = Mock(return_value=(None, None))
        
        return hook

    def test_initialization(self, hook, mock_config):
        """Test hook initializes correctly."""
        assert hook.config == mock_config
        assert hook.session_issues == {}
        assert hook._analyzer is None
        assert hook._group_manager is None

    @pytest.mark.asyncio
    async def test_session_start_basic(self, hook_with_mocks, sample_event_data, mock_coordinator):
        """Test basic session start handling."""
        # Add coordinator to event data
        event_data = {**sample_event_data, "coordinator": mock_coordinator}
        
        await hook_with_mocks.on_session_start(event_data)
        
        # Verify GitHub tools were called
        assert mock_coordinator.call_tool.called
        session_id = event_data["session_id"]
        assert session_id in hook_with_mocks.session_issues

    @pytest.mark.asyncio
    async def test_session_start_no_session_id(self, hook_with_mocks):
        """Test session start with missing session_id."""
        event_data = {"coordinator": Mock()}
        
        # Should not raise exception
        await hook_with_mocks.on_session_start(event_data)
        
        # Should return early, no issues created
        assert len(hook_with_mocks.session_issues) == 0

    @pytest.mark.asyncio
    async def test_session_start_no_coordinator(self, hook_with_mocks):
        """Test session start with missing coordinator."""
        event_data = {"session_id": "test-123", "initial_prompt": "Test"}
        
        # Should not raise exception
        await hook_with_mocks.on_session_start(event_data)

    @pytest.mark.asyncio
    async def test_session_start_no_repository(self, hook_with_mocks):
        """Test session start when repository not configured."""
        # Remove repository from config
        hook_with_mocks.config.pop("repository", None)
        
        coordinator = Mock()
        coordinator.call_tool = AsyncMock()
        
        event_data = {
            "session_id": "test-123",
            "initial_prompt": "Test",
            "coordinator": coordinator,
        }
        
        # Should not raise exception
        await hook_with_mocks.on_session_start(event_data)

    @pytest.mark.asyncio
    async def test_session_end_basic(self, hook_with_mocks, sample_event_data, mock_coordinator):
        """Test basic session end handling."""
        # Set up tracking issue
        session_id = sample_event_data["session_id"]
        hook_with_mocks.session_issues[session_id] = 123  # Issue number, not ID
        
        # Configure analyzer to return completed work
        hook_with_mocks._analyzer.analyze_session_work = AsyncMock(
            return_value={
                "completed": True,
                "summary": "Test summary",
                "new_ideas": [],
            }
        )
        
        # Add coordinator to event data
        event_data = {**sample_event_data, "coordinator": mock_coordinator}
        
        await hook_with_mocks.on_session_end(event_data)
        
        # Verify cleanup
        assert session_id not in hook_with_mocks.session_issues

    @pytest.mark.asyncio
    async def test_session_end_with_new_ideas(self, hook_with_mocks, sample_event_data, mock_coordinator):
        """Test session end with new ideas filed."""
        session_id = sample_event_data["session_id"]
        hook_with_mocks.session_issues[session_id] = "test-issue-123"
        
        # Mock analyzer with new ideas
        hook_with_mocks._analyzer.analyze_session_work = AsyncMock(
            return_value={
                "completed": True,
                "summary": "Test summary",
                "new_ideas": [
                    {
                        "title": "New idea 1",
                        "description": "Description 1",
                        "suggested_priority": 1,
                    },
                    {
                        "title": "New idea 2",
                        "description": "Description 2",
                        "suggested_priority": 2,
                    },
                ],
            }
        )
        
        # Add coordinator
        event_data = {**sample_event_data, "coordinator": mock_coordinator}
        
        await hook_with_mocks.on_session_end(event_data)
        
        # Verify coordinator.call_tool was called (for update and creating new ideas)
        assert mock_coordinator.call_tool.called

    @pytest.mark.asyncio
    async def test_session_end_no_tracking_issue(self, hook_with_mocks, sample_event_data, mock_coordinator):
        """Test session end when no tracking issue exists."""
        # No tracking issue set up
        event_data = {**sample_event_data, "coordinator": mock_coordinator}
        
        # Should not raise exception
        await hook_with_mocks.on_session_end(event_data)

    def test_capture_context_basic(self, hook_with_mocks, sample_event_data):
        """Test context capture."""
        context = hook_with_mocks._capture_context(sample_event_data)
        
        assert context["session_id"] == sample_event_data["session_id"]
        assert context["prompt"] == sample_event_data["initial_prompt"]
        assert "working_dir" in context
        assert "timestamp" in context

    @patch("amplifier_module_hooks_activity_tracker.hooks.get_git_status")
    def test_capture_context_with_git(self, mock_get_git, hook_with_mocks, sample_event_data):
        """Test context capture with git status."""
        mock_get_git.return_value = "M file.py"
        
        context = hook_with_mocks._capture_context(sample_event_data)
        
        assert context["git_status"] == "M file.py"

    @patch("amplifier_module_hooks_activity_tracker.hooks.get_git_status")
    def test_capture_context_no_git(self, mock_get_git, hook_with_mocks, sample_event_data):
        """Test context capture without git."""
        mock_get_git.return_value = None
        
        context = hook_with_mocks._capture_context(sample_event_data)
        
        assert context["git_status"] is None

    @pytest.mark.asyncio
    async def test_query_group_work_single_repo(self, hook_with_mocks, mock_coordinator):
        """Test querying work in single repo."""
        # Mock coordinator to return issues
        async def mock_call_tool(tool_name, params):
            return {"success": True, "output": {"issues": [{"number": 1}, {"number": 2}]}}
        
        mock_coordinator.call_tool = AsyncMock(side_effect=mock_call_tool)
        
        result = await hook_with_mocks._query_group_work(None, "owner/repo", mock_coordinator)
        
        assert len(result) == 2
        assert mock_coordinator.call_tool.called

    @pytest.mark.asyncio
    async def test_query_group_work_multi_repo(self, hook_with_mocks, mock_coordinator, tmp_path):
        """Test querying work across multiple repos."""
        # Create mock group config
        group_config = {
            "repositories": ["owner/repo1", "owner/repo2"]
        }
        
        # Mock coordinator to return issues
        async def mock_call_tool(tool_name, params):
            return {"success": True, "output": {"issues": [{"number": 1}]}}
        
        mock_coordinator.call_tool = AsyncMock(side_effect=mock_call_tool)
        
        result = await hook_with_mocks._query_group_work(group_config, "owner/repo", mock_coordinator)
        
        # Should have called call_tool for each repo
        assert mock_coordinator.call_tool.call_count == 2
        # Should have 2 issues total (one from each repo)
        assert len(result) == 2

    def test_format_session_description(self, hook_with_mocks, sample_context):
        """Test session description formatting."""
        description = hook_with_mocks._format_session_description(sample_context)
        
        assert sample_context["prompt"] in description
        assert sample_context["working_dir"] in description
        assert sample_context["timestamp"] in description

    @pytest.mark.asyncio
    async def test_error_handling_session_start(self, hook_with_mocks):
        """Test error handling in session start."""
        # Event data that causes coordinator.get to fail
        coordinator = Mock()
        coordinator.get = Mock(side_effect=Exception("Test error"))
        
        event_data = {
            "session_id": "test-123",
            "initial_prompt": "Test",
            "coordinator": coordinator,
        }
        
        # Should not raise exception (caught internally)
        await hook_with_mocks.on_session_start(event_data)

    @pytest.mark.asyncio
    async def test_error_handling_session_end(self, hook_with_mocks):
        """Test error handling in session end."""
        # Event data that causes coordinator.get to fail
        coordinator = Mock()
        coordinator.get = Mock(side_effect=Exception("Test error"))
        
        event_data = {
            "session_id": "test-123",
            "coordinator": coordinator,
            "messages": [],
        }
        
        # Should not raise exception (caught internally)
        await hook_with_mocks.on_session_end(event_data)
