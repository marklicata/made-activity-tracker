"""Activity tracking hook for Amplifier sessions."""

import logging
import os
from datetime import datetime
from pathlib import Path
from typing import Any

from .utils import format_notification, get_git_status, find_recently_modified_files

logger = logging.getLogger(__name__)


class ActivityTrackerHook:
    """Main hook class that integrates with Amplifier session lifecycle."""

    def __init__(self, config: dict[str, Any]):
        """Initialize activity tracker hook.

        Args:
            config: Configuration dictionary
        """
        self.config = config
        self.session_issues: dict[str, str] = {}  # session_id -> issue_id

        # Lazy-load dependencies to avoid circular imports
        self._analyzer = None
        self._group_manager = None

    @property
    def analyzer(self):
        """Lazy-load activity analyzer."""
        if self._analyzer is None:
            from .analyzer import ActivityAnalyzer

            self._analyzer = ActivityAnalyzer(self.config)
        return self._analyzer

    @property
    def group_manager(self):
        """Lazy-load project group manager."""
        if self._group_manager is None:
            from .project_group_manager import ProjectGroupManager

            self._group_manager = ProjectGroupManager(self.config)
        return self._group_manager

    async def on_session_start(self, event_data: dict[str, Any]) -> None:
        """Handle session start event.

        Args:
            event_data: Event data with session_id, initial_prompt, coordinator
        """
        try:
            session_id = event_data.get("session_id")
            if not session_id:
                logger.warning("Session start event missing session_id")
                return

            logger.info(f"Activity tracker: session start {session_id}")

            # Capture context
            context = self._capture_context(event_data)

            # Get issue-manager from coordinator
            coordinator = event_data.get("coordinator")
            if not coordinator:
                logger.warning("No coordinator in event data")
                return

            issue_manager = coordinator.get("issue-manager")
            if not issue_manager:
                logger.info("issue-manager not available, skipping tracking")
                return

            # Determine project group
            group_name, group_config = self.group_manager.get_group_for_repo(
                context["working_dir"]
            )

            # Query open work across group or single repo
            open_work = await self._query_group_work(group_config, issue_manager)

            logger.info(f"Found {len(open_work)} open issues to check")

            # LLM analysis to find related work
            if open_work and not self.config.get("silent_mode"):
                try:
                    related = await self.analyzer.find_related_work(context, open_work)

                    # Notify if high-confidence matches
                    notify_threshold = self.config.get("notify_threshold", 0.85)
                    high_conf = [r for r in related if r.get("confidence", 0) > notify_threshold]

                    if high_conf:
                        self._notify_related_work(event_data, high_conf)
                except Exception as e:
                    logger.error(f"Analysis failed: {e}", exc_info=True)

            # Create session tracking issue if enabled
            if self.config.get("auto_track_sessions", True):
                try:
                    session_issue = issue_manager.create_issue(
                        title=f"Session: {context['prompt'][:50]}",
                        description=self._format_session_description(context),
                        issue_type="task",
                        metadata={
                            "session_id": session_id,
                            "auto_tracked": True,
                            "working_dir": context["working_dir"],
                        },
                    )
                    self.session_issues[session_id] = session_issue.id
                    logger.info(f"Created tracking issue: {session_issue.id}")
                except Exception as e:
                    logger.error(f"Failed to create tracking issue: {e}")

        except Exception as e:
            logger.error(f"Session start handler failed: {e}", exc_info=True)

    async def on_session_end(self, event_data: dict[str, Any]) -> None:
        """Handle session end event.

        Args:
            event_data: Event data with session_id, messages, coordinator
        """
        try:
            session_id = event_data.get("session_id")
            if not session_id:
                logger.warning("Session end event missing session_id")
                return

            logger.info(f"Activity tracker: session end {session_id}")

            # Get issue-manager
            coordinator = event_data.get("coordinator")
            if not coordinator:
                return

            issue_manager = coordinator.get("issue-manager")
            if not issue_manager:
                return

            # Get session issue
            session_issue_id = self.session_issues.get(session_id)
            if not session_issue_id:
                logger.info("No tracking issue for this session")
                return

            # Analyze session work if enabled
            if self.config.get("auto_file_ideas", True):
                try:
                    messages = event_data.get("messages", [])
                    analysis = await self.analyzer.analyze_session_work(messages)

                    # Update session issue
                    if analysis.get("completed"):
                        issue_manager.close_issue(
                            session_issue_id, reason=analysis.get("summary", "Completed")
                        )
                        logger.info(f"Closed session issue: {session_issue_id}")
                    else:
                        issue_manager.update_issue(
                            session_issue_id, description=analysis.get("summary", "Work in progress")
                        )
                        logger.info(f"Updated session issue: {session_issue_id}")

                    # File new ideas
                    new_ideas = analysis.get("new_ideas", [])
                    for idea in new_ideas:
                        try:
                            new_issue = issue_manager.create_issue(
                                title=idea.get("title", "New idea"),
                                description=idea.get("description", ""),
                                priority=idea.get("suggested_priority", 2),
                                issue_type="task",
                                discovered_from=session_issue_id,
                            )

                            # Add discovered-from dependency
                            issue_manager.add_dependency(
                                new_issue.id, session_issue_id, dep_type="discovered-from"
                            )

                            logger.info(f"Filed new idea: {new_issue.id} - {idea.get('title')}")
                        except Exception as e:
                            logger.error(f"Failed to file idea: {e}")

                except Exception as e:
                    logger.error(f"Session analysis failed: {e}", exc_info=True)

            # Cleanup
            if session_id in self.session_issues:
                del self.session_issues[session_id]

        except Exception as e:
            logger.error(f"Session end handler failed: {e}", exc_info=True)

    def _capture_context(self, event_data: dict[str, Any]) -> dict[str, Any]:
        """Capture session context.

        Args:
            event_data: Event data

        Returns:
            Context dictionary
        """
        prompt = event_data.get("initial_prompt", "")
        working_dir = os.getcwd()

        context = {
            "session_id": event_data.get("session_id"),
            "prompt": prompt,
            "working_dir": working_dir,
            "git_status": None,
            "recent_files": [],
            "timestamp": datetime.now().isoformat(),
        }

        # Get git status
        git_status = get_git_status()
        if git_status:
            context["git_status"] = git_status

        # Get recently modified files
        try:
            recent_files = find_recently_modified_files(Path(working_dir), hours=24)
            context["recent_files"] = recent_files[:20]  # Limit to 20
        except Exception as e:
            logger.debug(f"Failed to get recent files: {e}")

        return context

    async def _query_group_work(
        self, group_config: dict[str, Any] | None, issue_manager: Any
    ) -> list[Any]:
        """Query open work across project group.

        Args:
            group_config: Project group configuration
            issue_manager: IssueManager instance

        Returns:
            List of Issue objects
        """
        open_work = []

        if group_config:
            # Multi-repo: query each repo in group
            for repo_path in group_config.get("repos", []):
                try:
                    repo_issues = await self._get_issues_for_repo(repo_path)
                    open_work.extend(repo_issues)
                except Exception as e:
                    logger.error(f"Failed to query repo {repo_path}: {e}")
        else:
            # Single repo: query current issue-manager
            try:
                open_work = issue_manager.list_issues(status="open")
            except Exception as e:
                logger.error(f"Failed to query issues: {e}")

        return open_work

    async def _get_issues_for_repo(self, repo_path: str) -> list[Any]:
        """Get issues from a specific repo.

        Args:
            repo_path: Path to repository

        Returns:
            List of Issue objects
        """
        from pathlib import Path

        data_dir = Path(repo_path) / ".amplifier" / "issues"
        if not data_dir.exists():
            return []

        try:
            # Import here to avoid circular dependency
            from amplifier_module_issue_manager import IssueManager

            temp_manager = IssueManager(data_dir)
            return temp_manager.list_issues(status="open")
        except Exception as e:
            logger.error(f"Failed to load issues from {repo_path}: {e}")
            return []

    def _notify_related_work(self, event_data: dict[str, Any], related: list[dict[str, Any]]) -> None:
        """Notify user of related work.

        Args:
            event_data: Event data (for potential coordinator access)
            related: List of related work items
        """
        notification = format_notification(related)
        if notification:
            # TODO: Use coordinator notification system if available
            # For now, just log
            print(notification)
            logger.info(notification)

    def _format_session_description(self, context: dict[str, Any]) -> str:
        """Format session description.

        Args:
            context: Session context

        Returns:
            Formatted description
        """
        lines = [
            f"**Session started**: {context.get('timestamp')}",
            f"**Working directory**: {context.get('working_dir')}",
            f"**Prompt**: {context.get('prompt')}",
        ]

        if context.get("git_status"):
            lines.append(f"\n**Git status**:\n```\n{context['git_status']}\n```")

        if context.get("recent_files"):
            files = context["recent_files"][:10]
            lines.append(f"\n**Recent files**: {', '.join(files)}")

        return "\n".join(lines)
