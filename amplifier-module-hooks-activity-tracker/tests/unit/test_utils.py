"""Unit tests for utility functions."""

import pytest
from pathlib import Path
from amplifier_module_hooks_activity_tracker.utils import (
    compute_content_hash,
    format_notification,
    parse_git_status,
    sanitize_llm_response,
    get_git_status,
)


class TestComputeContentHash:
    def test_basic_hash(self):
        """Test basic hash generation."""
        text = "Hello World"
        hash1 = compute_content_hash(text)
        hash2 = compute_content_hash(text)
        
        assert hash1 == hash2
        assert len(hash1) == 64  # SHA-256 hex digest length
    
    def test_different_content_different_hash(self):
        """Test different content produces different hash."""
        hash1 = compute_content_hash("Hello")
        hash2 = compute_content_hash("World")
        
        assert hash1 != hash2
    
    def test_empty_string(self):
        """Test empty string hashing."""
        hash_val = compute_content_hash("")
        assert len(hash_val) == 64


class TestFormatNotification:
    def test_empty_list(self):
        """Test formatting empty list."""
        result = format_notification([])
        assert result == ""
    
    def test_single_item(self, mock_issue):
        """Test formatting single item."""
        issue = mock_issue(issue_id="test-123", title="Test Issue")
        items = [
            {
                "issue": issue,
                "confidence": 0.9,
                "reasoning": "Similar work",
                "relationship_type": "duplicate",
            }
        ]
        
        result = format_notification(items)
        
        assert "[Activity Tracker]" in result
        assert "test-123" in result
        assert "Test Issue" in result
        assert "90%" in result
        assert "duplicate" in result
    
    def test_multiple_items(self, mock_issue):
        """Test formatting multiple items."""
        items = [
            {
                "issue": mock_issue(issue_id="test-1", title="Issue 1"),
                "confidence": 0.9,
                "reasoning": "Reason 1",
                "relationship_type": "duplicate",
            },
            {
                "issue": mock_issue(issue_id="test-2", title="Issue 2"),
                "confidence": 0.7,
                "reasoning": "Reason 2",
                "relationship_type": "related",
            },
        ]
        
        result = format_notification(items)
        
        assert "test-1" in result
        assert "test-2" in result


class TestParseGitStatus:
    def test_modified_files(self):
        """Test parsing modified files."""
        git_output = "M  src/file1.py\n M src/file2.py"
        result = parse_git_status(git_output)
        
        assert "src/file1.py" in result["modified"]
        assert "src/file2.py" in result["modified"]
    
    def test_untracked_files(self):
        """Test parsing untracked files."""
        git_output = "?? new_file.py\n?? another.py"
        result = parse_git_status(git_output)
        
        assert "new_file.py" in result["untracked"]
        assert "another.py" in result["untracked"]
    
    def test_mixed_status(self):
        """Test parsing mixed status."""
        git_output = "M  modified.py\nA  added.py\n?? untracked.py"
        result = parse_git_status(git_output)
        
        assert "modified.py" in result["modified"]
        assert "added.py" in result["added"]
        assert "untracked.py" in result["untracked"]
    
    def test_empty_output(self):
        """Test parsing empty output."""
        result = parse_git_status("")
        
        assert len(result["modified"]) == 0
        assert len(result["added"]) == 0
        assert len(result["untracked"]) == 0


class TestSanitizeLLMResponse:
    def test_plain_json(self):
        """Test sanitizing plain JSON."""
        response = '{"key": "value"}'
        result = sanitize_llm_response(response)
        
        assert result == '{"key": "value"}'
    
    def test_json_with_markdown_blocks(self):
        """Test sanitizing JSON with markdown code blocks."""
        response = '```json\n{"key": "value"}\n```'
        result = sanitize_llm_response(response)
        
        assert result == '{"key": "value"}'
    
    def test_generic_code_block(self):
        """Test sanitizing generic code block."""
        response = '```\n{"key": "value"}\n```'
        result = sanitize_llm_response(response)
        
        assert result == '{"key": "value"}'
    
    def test_whitespace_handling(self):
        """Test whitespace is trimmed."""
        response = '  \n  {"key": "value"}  \n  '
        result = sanitize_llm_response(response)
        
        assert result == '{"key": "value"}'


class TestGetGitStatus:
    def test_git_not_available(self, monkeypatch):
        """Test when git command is not available."""
        def mock_run(*args, **kwargs):
            raise FileNotFoundError()
        
        monkeypatch.setattr("subprocess.run", mock_run)
        result = get_git_status()
        
        assert result is None
    
    def test_not_git_repo(self, monkeypatch):
        """Test when not in git repo."""
        from unittest.mock import Mock
        
        mock_result = Mock()
        mock_result.returncode = 128  # Git error
        mock_result.stdout = ""
        
        def mock_run(*args, **kwargs):
            return mock_result
        
        monkeypatch.setattr("subprocess.run", mock_run)
        result = get_git_status()
        
        assert result is None
