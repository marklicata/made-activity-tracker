"""Project group manager for multi-repo coordination."""

import logging
from pathlib import Path
from typing import Any

import yaml

logger = logging.getLogger(__name__)


class ProjectGroupManager:
    """Manages project groups for multi-repo tracking."""

    def __init__(self, config: dict[str, Any]):
        """Initialize project group manager.

        Args:
            config: Configuration dictionary
        """
        self.config = config
        self.config_path = self._find_config_path()
        self.groups = self._load_groups()

    def _find_config_path(self) -> Path:
        """Find configuration file path.

        Checks in order:
        1. .amplifier/settings.yaml (project-level)
        2. ~/.amplifier/settings.yaml (user-level)

        Returns:
            Path to config file (may not exist yet)
        """
        # Check project-level config
        project_config = Path.cwd() / ".amplifier" / "settings.yaml"
        if project_config.exists():
            return project_config

        # Check user-level config
        user_config = Path.home() / ".amplifier" / "settings.yaml"
        if user_config.exists():
            return user_config

        # Default to project-level (will be created if needed)
        return project_config

    def _load_groups(self) -> dict[str, dict[str, Any]]:
        """Load project groups from configuration.

        Returns:
            Dict of group configurations
        """
        if not self.config_path.exists():
            logger.debug(f"Config file not found: {self.config_path}")
            return {}

        try:
            with open(self.config_path, "r", encoding="utf-8") as f:
                config = yaml.safe_load(f) or {}

            activity_config = config.get("activity", {})
            groups = activity_config.get("project_groups", {})

            if not isinstance(groups, dict):
                logger.warning("Invalid project_groups format in config")
                return {}

            logger.info(f"Loaded {len(groups)} project groups")
            return groups

        except Exception as e:
            logger.error(f"Failed to load project groups: {e}")
            return {}

    def _save_groups(self) -> None:
        """Save project groups to configuration."""
        try:
            # Load existing config or create new
            if self.config_path.exists():
                with open(self.config_path, "r", encoding="utf-8") as f:
                    config = yaml.safe_load(f) or {}
            else:
                config = {}

            # Update activity section
            if "activity" not in config:
                config["activity"] = {}

            config["activity"]["project_groups"] = self.groups

            # Ensure directory exists
            self.config_path.parent.mkdir(parents=True, exist_ok=True)

            # Write config
            with open(self.config_path, "w", encoding="utf-8") as f:
                yaml.dump(config, f, default_flow_style=False, sort_keys=False)

            logger.info(f"Saved {len(self.groups)} project groups to {self.config_path}")

        except Exception as e:
            logger.error(f"Failed to save project groups: {e}")

    def get_group_for_repo(self, repo_path: str) -> tuple[str | None, dict[str, Any] | None]:
        """Determine which group (if any) this repo belongs to.

        Args:
            repo_path: Path to repository

        Returns:
            Tuple of (group_name, group_config) or (None, None)
        """
        repo_path_resolved = Path(repo_path).resolve()

        for group_name, group_data in self.groups.items():
            repos = group_data.get("repos", [])
            for group_repo in repos:
                try:
                    if Path(group_repo).resolve() == repo_path_resolved:
                        logger.debug(f"Repo {repo_path} belongs to group {group_name}")
                        return group_name, group_data
                except Exception as e:
                    logger.debug(f"Failed to resolve repo path {group_repo}: {e}")
                    continue

        logger.debug(f"Repo {repo_path} does not belong to any group")
        return None, None

    def get_group(self, group_name: str) -> dict[str, Any] | None:
        """Get group configuration by name.

        Args:
            group_name: Group name

        Returns:
            Group configuration or None
        """
        return self.groups.get(group_name)

    def set_group(
        self, group_name: str, repos: list[str], description: str | None = None
    ) -> None:
        """Create or update project group.

        Args:
            group_name: Group name
            repos: List of repository paths
            description: Optional group description
        """
        # Validate repos
        if not repos or not isinstance(repos, list):
            raise ValueError("repos must be a non-empty list")

        # Resolve all repo paths
        resolved_repos = []
        for repo in repos:
            try:
                resolved = str(Path(repo).resolve())
                resolved_repos.append(resolved)
            except Exception as e:
                logger.warning(f"Failed to resolve repo path {repo}: {e}")
                resolved_repos.append(repo)

        self.groups[group_name] = {
            "repos": resolved_repos,
            "description": description or f"Project group: {group_name}",
        }

        self._save_groups()
        logger.info(f"Updated group {group_name} with {len(resolved_repos)} repos")

    def list_groups(self) -> dict[str, dict[str, Any]]:
        """List all configured project groups.

        Returns:
            Dict of all groups
        """
        return self.groups.copy()

    def delete_group(self, group_name: str) -> None:
        """Delete a project group.

        Args:
            group_name: Group name to delete
        """
        if group_name in self.groups:
            del self.groups[group_name]
            self._save_groups()
            logger.info(f"Deleted group {group_name}")
        else:
            logger.warning(f"Group {group_name} not found")

    def add_repo_to_group(self, group_name: str, repo_path: str) -> None:
        """Add a repository to existing group.

        Args:
            group_name: Group name
            repo_path: Repository path to add
        """
        if group_name not in self.groups:
            raise ValueError(f"Group {group_name} does not exist")

        resolved_path = str(Path(repo_path).resolve())
        repos = self.groups[group_name].get("repos", [])

        if resolved_path not in repos:
            repos.append(resolved_path)
            self.groups[group_name]["repos"] = repos
            self._save_groups()
            logger.info(f"Added repo {repo_path} to group {group_name}")
        else:
            logger.info(f"Repo {repo_path} already in group {group_name}")

    def remove_repo_from_group(self, group_name: str, repo_path: str) -> None:
        """Remove a repository from group.

        Args:
            group_name: Group name
            repo_path: Repository path to remove
        """
        if group_name not in self.groups:
            raise ValueError(f"Group {group_name} does not exist")

        resolved_path = str(Path(repo_path).resolve())
        repos = self.groups[group_name].get("repos", [])

        if resolved_path in repos:
            repos.remove(resolved_path)
            self.groups[group_name]["repos"] = repos
            self._save_groups()
            logger.info(f"Removed repo {repo_path} from group {group_name}")
        else:
            logger.info(f"Repo {repo_path} not in group {group_name}")
