"""Simple tests for module mount function - matching actual API."""

import pytest
from unittest.mock import Mock, AsyncMock, patch
from amplifier_module_hooks_activity_tracker import mount


class TestModuleMountSimple:
    """Simple tests for mount function."""
    
    @pytest.mark.asyncio
    async def test_mount_basic(self):
        """Test basic mounting."""
        mock_coordinator = Mock()
        mock_coordinator.on = Mock()
        
        with patch('amplifier_module_hooks_activity_tracker.hooks.ActivityTrackerHook') as mock_hook_class:
            mock_hook = Mock()
            mock_hook.on_session_start = Mock()
            mock_hook.on_session_end = Mock()
            mock_hook_class.return_value = mock_hook
            
            await mount(mock_coordinator, {})
            
            # Should create hook
            assert mock_hook_class.called
            
            # Should register events
            assert mock_coordinator.on.call_count >= 2
    
    @pytest.mark.asyncio
    async def test_mount_with_none_config(self):
        """Test mounting with None config."""
        mock_coordinator = Mock()
        mock_coordinator.on = Mock()
        
        with patch('amplifier_module_hooks_activity_tracker.hooks.ActivityTrackerHook') as mock_hook_class:
            mock_hook = Mock()
            mock_hook.on_session_start = Mock()
            mock_hook.on_session_end = Mock()
            mock_hook_class.return_value = mock_hook
            
            await mount(mock_coordinator, None)
            
            # Should work with None config (uses defaults)
            assert mock_hook_class.called
    
    @pytest.mark.asyncio
    async def test_mount_applies_defaults(self):
        """Test that defaults are applied."""
        mock_coordinator = Mock()
        mock_coordinator.on = Mock()
        
        with patch('amplifier_module_hooks_activity_tracker.hooks.ActivityTrackerHook') as mock_hook_class:
            mock_hook = Mock()
            mock_hook.on_session_start = Mock()
            mock_hook.on_session_end = Mock()
            mock_hook_class.return_value = mock_hook
            
            await mount(mock_coordinator, {})
            
            # Check config passed to hook has defaults
            call_args = mock_hook_class.call_args[0][0]
            assert "notify_threshold" in call_args
            assert "embedding_model" in call_args
            assert call_args["notify_threshold"] == 0.85
    
    @pytest.mark.asyncio
    async def test_mount_merges_config(self):
        """Test config merging."""
        mock_coordinator = Mock()
        mock_coordinator.on = Mock()
        
        custom_config = {"notify_threshold": 0.95}
        
        with patch('amplifier_module_hooks_activity_tracker.hooks.ActivityTrackerHook') as mock_hook_class:
            mock_hook = Mock()
            mock_hook.on_session_start = Mock()
            mock_hook.on_session_end = Mock()
            mock_hook_class.return_value = mock_hook
            
            await mount(mock_coordinator, custom_config)
            
            call_args = mock_hook_class.call_args[0][0]
            assert call_args["notify_threshold"] == 0.95  # Custom
            assert call_args["embedding_model"] == "text-embedding-3-small"  # Default
    
    @pytest.mark.asyncio
    async def test_mount_registers_session_start(self):
        """Test session:start is registered."""
        mock_coordinator = Mock()
        mock_coordinator.on = Mock()
        
        with patch('amplifier_module_hooks_activity_tracker.hooks.ActivityTrackerHook') as mock_hook_class:
            mock_hook = Mock()
            mock_hook.on_session_start = Mock()
            mock_hook.on_session_end = Mock()
            mock_hook_class.return_value = mock_hook
            
            await mount(mock_coordinator, {})
            
            # Find session:start registration
            calls = mock_coordinator.on.call_args_list
            session_start_calls = [c for c in calls if c[0][0] == "session:start"]
            
            assert len(session_start_calls) > 0
    
    @pytest.mark.asyncio
    async def test_mount_registers_session_end(self):
        """Test session:end is registered."""
        mock_coordinator = Mock()
        mock_coordinator.on = Mock()
        
        with patch('amplifier_module_hooks_activity_tracker.hooks.ActivityTrackerHook') as mock_hook_class:
            mock_hook = Mock()
            mock_hook.on_session_start = Mock()
            mock_hook.on_session_end = Mock()
            mock_hook_class.return_value = mock_hook
            
            await mount(mock_coordinator, {})
            
            # Find session:end registration
            calls = mock_coordinator.on.call_args_list
            session_end_calls = [c for c in calls if c[0][0] == "session:end"]
            
            assert len(session_end_calls) > 0
