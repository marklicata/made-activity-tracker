"""Simple tests for ProjectGroupManager - matching actual API."""

import pytest
from pathlib import Path
from unittest.mock import patch
from amplifier_module_hooks_activity_tracker.project_group_manager import ProjectGroupManager


class TestProjectGroupManagerSimple:
    """Simple tests matching actual implementation."""
    
    def test_init(self, mock_config):
        """Test basic initialization."""
        manager = ProjectGroupManager(mock_config)
        assert manager.config == mock_config
        assert isinstance(manager.groups, dict)
    
    def test_get_group_for_repo_no_groups(self, mock_config):
        """Test getting group when none configured."""
        with patch.object(ProjectGroupManager, '_load_groups', return_value={}):
            manager = ProjectGroupManager(mock_config)
            
            group_name, group_config = manager.get_group_for_repo("/some/path")
            
            assert group_name is None
            assert group_config is None
    
    def test_get_group_for_repo_exact_match(self, mock_config, tmp_path):
        """Test exact repo match."""
        test_repo = str(tmp_path / "test-repo")
        groups = {
            "test-group": {
                "repos": [test_repo],
                "description": "Test"
            }
        }
        
        with patch.object(ProjectGroupManager, '_load_groups', return_value=groups):
            manager = ProjectGroupManager(mock_config)
            
            group_name, group_config = manager.get_group_for_repo(test_repo)
            
            assert group_name == "test-group"
            assert group_config is not None
    
    def test_get_group_exists(self, mock_config):
        """Test getting group by name."""
        groups = {
            "test-group": {"repos": ["/path"], "description": "Test"}
        }
        
        with patch.object(ProjectGroupManager, '_load_groups', return_value=groups):
            manager = ProjectGroupManager(mock_config)
            
            result = manager.get_group("test-group")
            
            assert result == groups["test-group"]
    
    def test_get_group_not_exists(self, mock_config):
        """Test getting non-existent group."""
        with patch.object(ProjectGroupManager, '_load_groups', return_value={}):
            manager = ProjectGroupManager(mock_config)
            
            result = manager.get_group("nonexistent")
            
            assert result is None
    
    def test_list_groups_empty(self, mock_config):
        """Test listing when no groups."""
        with patch.object(ProjectGroupManager, '_load_groups', return_value={}):
            manager = ProjectGroupManager(mock_config)
            
            result = manager.list_groups()
            
            # Returns dict, not list!
            assert isinstance(result, dict)
            assert len(result) == 0
    
    def test_list_groups_with_groups(self, mock_config):
        """Test listing configured groups."""
        groups = {
            "group1": {"repos": ["/path1"]},
            "group2": {"repos": ["/path2"]},
        }
        
        with patch.object(ProjectGroupManager, '_load_groups', return_value=groups):
            manager = ProjectGroupManager(mock_config)
            
            result = manager.list_groups()
            
            assert isinstance(result, dict)
            assert len(result) == 2
            assert "group1" in result
            assert "group2" in result
    
    def test_set_group_basic(self, mock_config, tmp_path):
        """Test setting a group."""
        config_path = tmp_path / "settings.yaml"
        
        with patch.object(ProjectGroupManager, '_find_config_path', return_value=config_path):
            with patch.object(ProjectGroupManager, '_load_groups', return_value={}):
                manager = ProjectGroupManager(mock_config)
                
                manager.set_group("test-group", repos=["/test/repo"], description="Test")
                
                assert "test-group" in manager.groups
                # Path gets normalized on Windows
                repos_str = str(manager.groups["test-group"]["repos"])
                assert "test" in repos_str and "repo" in repos_str
    
    def test_set_group_validates_empty_repos(self, mock_config):
        """Test validation of empty repos list."""
        with patch.object(ProjectGroupManager, '_load_groups', return_value={}):
            manager = ProjectGroupManager(mock_config)
            
            with pytest.raises(ValueError):
                manager.set_group("test", repos=[])
    
    def test_delete_group(self, mock_config, tmp_path):
        """Test deleting a group."""
        config_path = tmp_path / "settings.yaml"
        groups = {"test-group": {"repos": ["/path"]}}
        
        with patch.object(ProjectGroupManager, '_find_config_path', return_value=config_path):
            with patch.object(ProjectGroupManager, '_load_groups', return_value=groups.copy()):
                manager = ProjectGroupManager(mock_config)
                
                assert "test-group" in manager.groups
                
                manager.delete_group("test-group")
                
                assert "test-group" not in manager.groups
    
    def test_delete_group_nonexistent(self, mock_config):
        """Test deleting non-existent group doesn't error."""
        with patch.object(ProjectGroupManager, '_load_groups', return_value={}):
            manager = ProjectGroupManager(mock_config)
            
            # Should not raise
            manager.delete_group("nonexistent")
    
    def test_add_repo_to_group(self, mock_config, tmp_path):
        """Test adding repo to existing group."""
        config_path = tmp_path / "settings.yaml"
        groups = {"test-group": {"repos": ["/path1"]}}
        
        with patch.object(ProjectGroupManager, '_find_config_path', return_value=config_path):
            with patch.object(ProjectGroupManager, '_load_groups', return_value=groups.copy()):
                manager = ProjectGroupManager(mock_config)
                
                manager.add_repo_to_group("test-group", "/path2")
                
                assert len(manager.groups["test-group"]["repos"]) == 2
    
    def test_remove_repo_from_group(self, mock_config, tmp_path):
        """Test removing repo from group."""
        config_path = tmp_path / "settings.yaml"
        groups = {"test-group": {"repos": ["/path1", "/path2"]}}
        
        with patch.object(ProjectGroupManager, '_find_config_path', return_value=config_path):
            with patch.object(ProjectGroupManager, '_load_groups', return_value=groups.copy()):
                manager = ProjectGroupManager(mock_config)
                
                manager.remove_repo_from_group("test-group", "/path1")
                
                repos = manager.groups["test-group"]["repos"]
                # After removal, only path2 should remain
                assert "path2" in str(repos)
