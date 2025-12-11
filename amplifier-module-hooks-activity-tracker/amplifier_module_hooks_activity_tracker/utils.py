"""Utility functions for activity tracker."""

import hashlib
import os
import subprocess
from datetime import datetime, timedelta
from pathlib import Path
from typing import Any


def compute_content_hash(text: str) -> str:
    """Generate SHA-256 hash of text.

    Args:
        text: Text to hash

    Returns:
        Hex digest of SHA-256 hash
    """
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def format_notification(related_items: list[dict[str, Any]]) -> str:
    """Format notification message for display.

    Args:
        related_items: List of related work items with issue, confidence, reasoning

    Returns:
        Formatted notification message
    """
    if not related_items:
        return ""

    lines = ["[Activity Tracker] Found related work:"]
    for item in related_items:
        issue = item.get("issue")
        confidence = item.get("confidence", 0.0)
        reasoning = item.get("reasoning", "")
        rel_type = item.get("relationship_type", "related")

        if issue:
            # GitHub API returns 'number' and 'title' (not 'id')
            issue_number = issue.get("number")
            issue_title = issue.get("title", "Untitled")
            lines.append(
                f"  • #{issue_number}: \"{issue_title}\" "
                f"(confidence: {confidence:.0%}, {rel_type})"
            )
            if reasoning:
                lines.append(f"    Reason: {reasoning}")

    return "\n".join(lines)


def parse_git_status(git_output: str) -> dict[str, list[str]]:
    """Parse git status output into structured data.

    Args:
        git_output: Output from 'git status --short'

    Returns:
        Dict with 'modified', 'added', 'deleted', 'untracked' file lists
    """
    result: dict[str, list[str]] = {
        "modified": [],
        "added": [],
        "deleted": [],
        "untracked": [],
    }

    for line in git_output.strip().split("\n"):
        if not line:
            continue

        status = line[:2]
        filename = line[3:].strip()

        if status[0] == "M" or status[1] == "M":
            result["modified"].append(filename)
        elif status[0] == "A":
            result["added"].append(filename)
        elif status[0] == "D":
            result["deleted"].append(filename)
        elif status == "??":
            result["untracked"].append(filename)

    return result


def find_recently_modified_files(directory: Path, hours: int = 24) -> list[str]:
    """Find files modified in last N hours.

    Args:
        directory: Directory to search
        hours: Number of hours to look back

    Returns:
        List of relative file paths
    """
    cutoff_time = datetime.now().timestamp() - (hours * 3600)
    recent_files = []

    try:
        for root, _, files in os.walk(directory):
            for file in files:
                filepath = Path(root) / file

                # Skip hidden files and directories
                if any(part.startswith(".") for part in filepath.parts):
                    continue

                try:
                    mtime = filepath.stat().st_mtime
                    if mtime >= cutoff_time:
                        rel_path = filepath.relative_to(directory)
                        recent_files.append(str(rel_path))
                except (OSError, ValueError):
                    continue

    except Exception:
        pass  # Fail silently

    return recent_files[:50]  # Limit to 50 files


def get_git_status(timeout: int = 5) -> str | None:
    """Get git status if in git repo.

    Args:
        timeout: Command timeout in seconds

    Returns:
        Git status output or None if not a git repo or error
    """
    try:
        result = subprocess.run(
            ["git", "status", "--short"],
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
        )
        if result.returncode == 0:
            return result.stdout
        return None
    except (FileNotFoundError, subprocess.TimeoutExpired, Exception):
        return None


def sanitize_llm_response(response: str) -> str:
    """Clean up LLM response for parsing.

    Args:
        response: Raw LLM response

    Returns:
        Cleaned response
    """
    # Remove markdown code blocks if present
    response = response.strip()
    if response.startswith("```json"):
        response = response[7:]
    elif response.startswith("```"):
        response = response[3:]

    if response.endswith("```"):
        response = response[:-3]

    return response.strip()


def validate_config(config: dict[str, Any], required_keys: list[str]) -> bool:
    """Validate configuration against required keys.

    Args:
        config: Configuration dictionary
        required_keys: List of required keys

    Returns:
        True if valid, False otherwise
    """
    return all(key in config for key in required_keys)
