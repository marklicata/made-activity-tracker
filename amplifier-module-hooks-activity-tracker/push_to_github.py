"""Push the activity tracker to GitHub repository."""

import subprocess
import sys

def run_git_command(cmd, description):
    """Run a git command and show results."""
    print(f"\n{'='*60}")
    print(f"{description}")
    print(f"{'='*60}")
    print(f"Running: {' '.join(cmd)}")
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    if result.stdout:
        print(result.stdout)
    if result.stderr:
        print("STDERR:", result.stderr)
    
    if result.returncode != 0:
        print(f"ERROR: Command failed with code {result.returncode}")
        return False
    
    return True

def main():
    """Push to GitHub."""
    print("Pushing Activity Tracker to GitHub")
    print("Repository: ramparte/amplifier-activity-tracker")
    print("="*60)
    
    # Initialize git
    if not run_git_command(
        ["git", "init"],
        "Step 1: Initialize Git Repository"
    ):
        return 1
    
    # Check status
    result = subprocess.run(["git", "status", "--short"], capture_output=True, text=True)
    file_count = len([l for l in result.stdout.split('\n') if l.strip()])
    print(f"\nFiles to add: {file_count}")
    
    # Add all files
    if not run_git_command(
        ["git", "add", "."],
        "Step 2: Stage All Files"
    ):
        return 1
    
    # Create commit
    commit_message = """Initial commit: Activity Tracker for Amplifier with 80% test coverage

Comprehensive activity tracking system that integrates with Amplifier to help
engineering teams coordinate work and prevent duplicate effort.

Features:
- Automatic duplicate detection using LLM + embeddings
- Session lifecycle tracking (start/end hooks)
- Multi-repo project group support
- SQLite-based embedding cache for performance
- LLM-powered idea extraction from sessions
- 80% test coverage with 128 passing tests

Architecture:
- Built on Paul Payne's issue-manager module
- Hook module for session lifecycle integration
- ActivityAnalyzer with two-phase matching (embeddings -> LLM)
- ProjectGroupManager for multi-repo coordination
- EmbeddingGenerator with OpenAI integration
- Comprehensive test suite with LLM-powered mocks

Test Coverage (80.03%):
- utils.py: 91%
- __init__.py: 88%
- project_group_manager.py: 85%
- embedding_cache.py: 82%
- hooks.py: 79%
- embedding_generator.py: 77%
- analyzer.py: 73%

Implementation:
- ~1,450 lines of production code
- ~1,100 lines of test code
- ~500 lines of documentation
- Novel LLM-powered test mock generator for realistic API testing

Documentation:
- Complete installation and usage guides
- API reference from code inspection
- Testing strategy and results
- Example configurations

🤖 Generated with [Amplifier](https://github.com/microsoft/amplifier)

Co-Authored-By: Amplifier <240397093+microsoft-amplifier@users.noreply.github.com>
"""
    
    if not run_git_command(
        ["git", "commit", "-m", commit_message],
        "Step 3: Create Commit"
    ):
        return 1
    
    # Add remote
    if not run_git_command(
        ["git", "remote", "add", "origin", "https://github.com/ramparte/amplifier-activity-tracker.git"],
        "Step 4: Add Remote Repository"
    ):
        # Remote might already exist, check
        result = subprocess.run(["git", "remote", "-v"], capture_output=True, text=True)
        if "ramparte/amplifier-activity-tracker" not in result.stdout:
            print("ERROR: Failed to add remote")
            return 1
        print("Remote already exists, continuing...")
    
    # Set branch to main
    if not run_git_command(
        ["git", "branch", "-M", "main"],
        "Step 5: Rename Branch to Main"
    ):
        return 1
    
    # Push to GitHub
    print("\n" + "="*60)
    print("Step 6: Pushing to GitHub")
    print("="*60)
    print("This will push to: https://github.com/ramparte/amplifier-activity-tracker")
    print("\nNote: You may need to authenticate with GitHub")
    
    if not run_git_command(
        ["git", "push", "-u", "origin", "main"],
        "Pushing to main branch"
    ):
        print("\nIf authentication failed, you can push manually:")
        print("  cd amplifier-module-hooks-activity-tracker")
        print("  git push -u origin main")
        return 1
    
    print("\n" + "="*60)
    print("SUCCESS! Repository pushed to GitHub")
    print("="*60)
    print("\nView at: https://github.com/ramparte/amplifier-activity-tracker")
    
    return 0

if __name__ == "__main__":
    sys.exit(main())
