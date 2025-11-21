"""Tests for module initialization and mount function."""

import pytest
from unittest.mock import Mock, AsyncMock, patch, MagicMock
from amplifier_module_hooks_activity_tracker import mount


class TestModuleMount:
    """Test module mount function."""
    
    @pytest.mark.asyncio
    async def test_mount_basic(self):
        """Test basic module mounting."""
        mock_coordinator = Mock()
        mock_coordinator.on = Mock()
        
        config = {
            "notify_threshold": 0.85,
            "auto_track_sessions": True,
        }
        
        with patch('amplifier_module_hooks_activity_tracker.hooks.ActivityTrackerHook') as mock_hook_class:
            mock_hook = Mock()
            mock_hook_class.return_value = mock_hook
            
            await mount(mock_coordinator, config)
            
            # Should create hook with config
            mock_hook_class.assert_called_once_with(config)
            
            # Should register lifecycle hooks
            assert mock_coordinator.on.call_count == 2
            calls = mock_coordinator.on.call_args_list
            
            # Check session:start registered
            assert any(call[0][0] == "session:start" for call in calls)
            # Check session:end registered
            assert any(call[0][0] == "session:end" for call in calls)
    
    @pytest.mark.asyncio
    async def test_mount_with_defaults(self):
        """Test mounting with default configuration."""
        mock_coordinator = Mock()
        mock_coordinator.on = Mock()
        
        with patch('amplifier_module_hooks_activity_tracker.hooks.ActivityTrackerHook') as mock_hook_class:
            mock_hook = Mock()
            mock_hook_class.return_value = mock_hook
            
            await mount(mock_coordinator, None)
            
            # Should use default config
            call_config = mock_hook_class.call_args[0][0]
            assert call_config["notify_threshold"] == 0.85
            assert call_config["embedding_model"] == "text-embedding-3-small"
            assert call_config["similarity_threshold"] == 0.7
            assert call_config["auto_track_sessions"] is True
            assert call_config["auto_file_ideas"] is True
            assert call_config["silent_mode"] is False
    
    @pytest.mark.asyncio
    async def test_mount_merges_config_with_defaults(self):
        """Test config merging with defaults."""
        mock_coordinator = Mock()
        mock_coordinator.on = Mock()
        
        partial_config = {
            "notify_threshold": 0.95,  # Override
            # Other values should use defaults
        }
        
        with patch('amplifier_module_hooks_activity_tracker.hooks.ActivityTrackerHook') as mock_hook_class:
            mock_hook = Mock()
            mock_hook_class.return_value = mock_hook
            
            await mount(mock_coordinator, partial_config)
            
            call_config = mock_hook_class.call_args[0][0]
            assert call_config["notify_threshold"] == 0.95  # Overridden
            assert call_config["embedding_model"] == "text-embedding-3-small"  # Default
    
    @pytest.mark.asyncio
    async def test_mount_registers_correct_handlers(self):
        """Test that correct handlers are registered."""
        mock_coordinator = Mock()
        mock_coordinator.on = Mock()
        
        with patch('amplifier_module_hooks_activity_tracker.hooks.ActivityTrackerHook') as mock_hook_class:
            mock_hook = Mock()
            mock_hook.on_session_start = Mock()
            mock_hook.on_session_end = Mock()
            mock_hook_class.return_value = mock_hook
            
            await mount(mock_coordinator, {})
            
            # Find the registered handlers
            calls = mock_coordinator.on.call_args_list
            
            session_start_call = [c for c in calls if c[0][0] == "session:start"][0]
            session_end_call = [c for c in calls if c[0][0] == "session:end"][0]
            
            # Should register the hook's methods
            assert session_start_call[0][1] == mock_hook.on_session_start
            assert session_end_call[0][1] == mock_hook.on_session_end
    
    @pytest.mark.asyncio
    async def test_mount_handles_errors(self):
        """Test error handling during mount."""
        mock_coordinator = Mock()
        mock_coordinator.on = Mock(side_effect=Exception("Registration failed"))
        
        with patch('amplifier_module_hooks_activity_tracker.ActivityTrackerHook'):
            # Should log error but not crash
            await mount(mock_coordinator, {})
    
    @pytest.mark.asyncio
    async def test_mount_logs_success(self):
        """Test that successful mount is logged."""
        mock_coordinator = Mock()
        mock_coordinator.on = Mock()
        
        with patch('amplifier_module_hooks_activity_tracker.ActivityTrackerHook'):
            with patch('amplifier_module_hooks_activity_tracker.logging.getLogger') as mock_logger:
                mock_log = Mock()
                mock_logger.return_value = mock_log
                
                await mount(mock_coordinator, {})
                
                # Should log success
                info_calls = [str(call) for call in mock_log.info.call_args_list]
                assert any("mounted successfully" in str(call).lower() for call in info_calls)
    
    @pytest.mark.asyncio
    async def test_mount_validates_config_types(self):
        """Test that config values have correct types."""
        mock_coordinator = Mock()
        mock_coordinator.on = Mock()
        
        config = {
            "notify_threshold": "0.85",  # String instead of float
            "auto_track_sessions": "true",  # String instead of bool
        }
        
        with patch('amplifier_module_hooks_activity_tracker.hooks.ActivityTrackerHook') as mock_hook_class:
            mock_hook = Mock()
            mock_hook_class.return_value = mock_hook
            
            await mount(mock_coordinator, config)
            
            # Should still work (no validation currently, but test exists for future)
            mock_hook_class.assert_called_once()
    
    @pytest.mark.asyncio
    async def test_mount_with_all_config_options(self):
        """Test mounting with all configuration options."""
        mock_coordinator = Mock()
        mock_coordinator.on = Mock()
        
        config = {
            "notify_threshold": 0.90,
            "embedding_model": "custom-model",
            "similarity_threshold": 0.75,
            "auto_track_sessions": False,
            "auto_file_ideas": False,
            "silent_mode": True,
        }
        
        with patch('amplifier_module_hooks_activity_tracker.hooks.ActivityTrackerHook') as mock_hook_class:
            mock_hook = Mock()
            mock_hook_class.return_value = mock_hook
            
            await mount(mock_coordinator, config)
            
            call_config = mock_hook_class.call_args[0][0]
            assert call_config == config
    
    @pytest.mark.asyncio
    async def test_mount_idempotent(self):
        """Test that mounting multiple times doesn't break."""
        mock_coordinator = Mock()
        mock_coordinator.on = Mock()
        
        with patch('amplifier_module_hooks_activity_tracker.ActivityTrackerHook'):
            # Mount twice
            await mount(mock_coordinator, {})
            await mount(mock_coordinator, {})
            
            # Should have registered twice (no deduplication currently)
            assert mock_coordinator.on.call_count >= 4
    
    @pytest.mark.asyncio
    async def test_mount_preserves_config_object(self):
        """Test that original config is not modified."""
        mock_coordinator = Mock()
        mock_coordinator.on = Mock()
        
        original_config = {"notify_threshold": 0.85}
        config_copy = original_config.copy()
        
        with patch('amplifier_module_hooks_activity_tracker.ActivityTrackerHook'):
            await mount(mock_coordinator, original_config)
            
            # Original should not be modified
            assert original_config == config_copy
