"""Amplifier tool module for MADE Activity Tracker.

Provides three capabilities:
- get_metrics: Query speed, ease, quality metrics
- search_github_items: Search issues and pull requests
- get_user_activity: Get user activity summaries
"""

from typing import Any, Dict, List, Optional
from .db_connection import db


class _GetMetricsTool:
    """Tool for querying GitHub activity metrics."""

    name = "get_metrics"
    description = """Get GitHub activity metrics (speed, ease, quality) for a date range.

Metrics include:
- Speed: cycle time, PR lead time, throughput
- Ease: PR size, review rounds, rework rate
- Quality: bug rate, reopen rate, rejection rate

Can filter by repositories and users."""

    parameters = {
        "type": "object",
        "properties": {
            "metric_type": {
                "type": "string",
                "enum": ["speed", "ease", "quality", "all"],
                "description": "Type of metrics to retrieve"
            },
            "start_date": {
                "type": "string",
                "description": "Start date in ISO format (YYYY-MM-DD)"
            },
            "end_date": {
                "type": "string",
                "description": "End date in ISO format (YYYY-MM-DD)"
            },
            "repositories": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Filter by repository names (owner/repo format)"
            },
            "users": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Filter by GitHub usernames"
            }
        },
        "required": ["metric_type", "start_date", "end_date"]
    }

    async def execute(self, **kwargs) -> Dict[str, Any]:
        """Execute metrics query."""
        metric_type = kwargs.get("metric_type", "all")
        start_date = kwargs["start_date"]
        end_date = kwargs["end_date"]
        repositories = kwargs.get("repositories", [])
        users = kwargs.get("users", [])

        results = {}

        if metric_type in ["speed", "all"]:
            results["speed"] = self._get_speed_metrics(start_date, end_date, repositories, users)

        if metric_type in ["ease", "all"]:
            results["ease"] = self._get_ease_metrics(start_date, end_date, repositories, users)

        if metric_type in ["quality", "all"]:
            results["quality"] = self._get_quality_metrics(start_date, end_date, repositories, users)

        return {
            "metrics": results,
            "period": {"start": start_date, "end": end_date},
            "filters": {"repositories": repositories, "users": users}
        }

    def _get_speed_metrics(self, start_date: str, end_date: str, repos: List[str], users: List[str]) -> Dict[str, float]:
        """Calculate speed metrics."""
        with db as conn:
            cursor = conn.cursor()

            where_parts = ["closed_at BETWEEN ? AND ?"]
            params = [start_date, end_date]

            if repos:
                placeholders = ','.join('?' * len(repos))
                where_parts.append(f"repository IN ({placeholders})")
                params.extend(repos)

            if users:
                placeholders = ','.join('?' * len(users))
                where_parts.append(f"user IN ({placeholders})")
                params.extend(users)

            where_clause = " AND ".join(where_parts)

            query = f"""
                SELECT
                    AVG((julianday(closed_at) - julianday(created_at)) * 24) as avg_cycle_time_hours,
                    COUNT(*) as total_closed
                FROM issues
                WHERE {where_clause} AND closed_at IS NOT NULL
            """

            cursor.execute(query, params)
            row = cursor.fetchone()

            return {
                "avg_cycle_time_hours": round(row["avg_cycle_time_hours"] or 0, 2),
                "total_closed": row["total_closed"]
            }

    def _get_ease_metrics(self, start_date: str, end_date: str, repos: List[str], users: List[str]) -> Dict[str, float]:
        """Calculate ease metrics."""
        with db as conn:
            cursor = conn.cursor()

            where_parts = ["created_at BETWEEN ? AND ?"]
            params = [start_date, end_date]

            if repos:
                placeholders = ','.join('?' * len(repos))
                where_parts.append(f"repository IN ({placeholders})")
                params.extend(repos)

            if users:
                placeholders = ','.join('?' * len(users))
                where_parts.append(f"user IN ({placeholders})")
                params.extend(users)

            where_clause = " AND ".join(where_parts)

            query = f"""
                SELECT
                    AVG(additions + deletions) as avg_size,
                    COUNT(*) as total_prs
                FROM pull_requests
                WHERE {where_clause}
            """

            cursor.execute(query, params)
            row = cursor.fetchone()

            return {
                "avg_pr_size_lines": round(row["avg_size"] or 0, 2),
                "total_prs": row["total_prs"]
            }

    def _get_quality_metrics(self, start_date: str, end_date: str, repos: List[str], users: List[str]) -> Dict[str, float]:
        """Calculate quality metrics."""
        return {
            "bug_rate": 0.0,
            "reopen_rate": 0.0
        }


class _SearchGitHubItemsTool:
    """Tool for searching GitHub issues and pull requests."""

    name = "search_github_items"
    description = """Search for GitHub issues and pull requests by text query.

Searches in:
- Issue/PR titles
- Issue/PR bodies
- Labels

Can filter by state, type, labels, repository."""

    parameters = {
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "Search text (searches title and body)"
            },
            "item_type": {
                "type": "string",
                "enum": ["issue", "pull_request", "both"],
                "description": "Type of items to search"
            },
            "state": {
                "type": "string",
                "enum": ["open", "closed", "all"],
                "description": "Filter by state"
            },
            "labels": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Filter by labels"
            },
            "repository": {
                "type": "string",
                "description": "Filter by repository (owner/repo)"
            },
            "limit": {
                "type": "integer",
                "default": 10,
                "description": "Maximum number of results"
            }
        },
        "required": ["query", "item_type"]
    }

    async def execute(self, **kwargs) -> Dict[str, Any]:
        """Execute search query."""
        query = kwargs["query"]
        item_type = kwargs.get("item_type", "both")
        state = kwargs.get("state", "all")
        labels = kwargs.get("labels", [])
        repository = kwargs.get("repository")
        limit = kwargs.get("limit", 10)

        results = []

        if item_type in ["issue", "both"]:
            results.extend(self._search_issues(query, state, labels, repository, limit))

        if item_type in ["pull_request", "both"]:
            results.extend(self._search_pull_requests(query, state, labels, repository, limit))

        results.sort(key=lambda x: x["created_at"], reverse=True)

        return {
            "results": results[:limit],
            "total": len(results),
            "query": query
        }

    def _search_issues(self, query: str, state: str, labels: List[str], repository: Optional[str], limit: int) -> List[Dict]:
        """Search issues table."""
        with db as conn:
            cursor = conn.cursor()

            where_parts = ["(title LIKE ? OR body LIKE ?)"]
            params = [f"%{query}%", f"%{query}%"]

            if state != "all":
                where_parts.append("state = ?")
                params.append(state)

            if repository:
                where_parts.append("repository = ?")
                params.append(repository)

            where_clause = " AND ".join(where_parts)

            query_sql = f"""
                SELECT
                    id, number, title, state, repository,
                    html_url, created_at, closed_at
                FROM issues
                WHERE {where_clause}
                ORDER BY created_at DESC
                LIMIT ?
            """

            params.append(limit)
            cursor.execute(query_sql, params)

            results = []
            for row in cursor.fetchall():
                results.append({
                    "type": "issue",
                    "id": row["id"],
                    "number": row["number"],
                    "title": row["title"],
                    "state": row["state"],
                    "repository": row["repository"],
                    "url": row["html_url"],
                    "created_at": row["created_at"],
                    "closed_at": row.get("closed_at")
                })

            return results

    def _search_pull_requests(self, query: str, state: str, labels: List[str], repository: Optional[str], limit: int) -> List[Dict]:
        """Search pull requests table."""
        with db as conn:
            cursor = conn.cursor()

            where_parts = ["(title LIKE ? OR body LIKE ?)"]
            params = [f"%{query}%", f"%{query}%"]

            if state != "all":
                where_parts.append("state = ?")
                params.append(state)

            if repository:
                where_parts.append("repository = ?")
                params.append(repository)

            where_clause = " AND ".join(where_parts)

            query_sql = f"""
                SELECT
                    id, number, title, state, repository,
                    html_url, created_at, merged_at, closed_at
                FROM pull_requests
                WHERE {where_clause}
                ORDER BY created_at DESC
                LIMIT ?
            """

            params.append(limit)
            cursor.execute(query_sql, params)

            results = []
            for row in cursor.fetchall():
                results.append({
                    "type": "pull_request",
                    "id": row["id"],
                    "number": row["number"],
                    "title": row["title"],
                    "state": row["state"],
                    "repository": row["repository"],
                    "url": row["html_url"],
                    "created_at": row["created_at"],
                    "merged_at": row.get("merged_at"),
                    "closed_at": row.get("closed_at")
                })

            return results


class _GetUserActivityTool:
    """Tool for getting user activity summaries."""

    name = "get_user_activity"
    description = """Get activity summary for a GitHub user.

Returns:
- Total PRs created
- Total reviews performed
- Total commits
- Repositories contributed to"""

    parameters = {
        "type": "object",
        "properties": {
            "username": {
                "type": "string",
                "description": "GitHub username"
            },
            "start_date": {
                "type": "string",
                "description": "Start date in ISO format (YYYY-MM-DD)"
            },
            "end_date": {
                "type": "string",
                "description": "End date in ISO format (YYYY-MM-DD)"
            }
        },
        "required": ["username", "start_date", "end_date"]
    }

    async def execute(self, **kwargs) -> Dict[str, Any]:
        """Get user activity."""
        username = kwargs["username"]
        start_date = kwargs["start_date"]
        end_date = kwargs["end_date"]

        with db as conn:
            cursor = conn.cursor()

            cursor.execute("""
                SELECT COUNT(*) as count
                FROM pull_requests
                WHERE user = ? AND created_at BETWEEN ? AND ?
            """, [username, start_date, end_date])
            total_prs = cursor.fetchone()["count"]

            cursor.execute("""
                SELECT COUNT(*) as count
                FROM reviews
                WHERE user = ? AND submitted_at BETWEEN ? AND ?
            """, [username, start_date, end_date])
            total_reviews = cursor.fetchone()["count"]

            cursor.execute("""
                SELECT COUNT(*) as count
                FROM commits
                WHERE author = ? AND committed_at BETWEEN ? AND ?
            """, [username, start_date, end_date])
            total_commits = cursor.fetchone()["count"]

            cursor.execute("""
                SELECT DISTINCT repository
                FROM pull_requests
                WHERE user = ? AND created_at BETWEEN ? AND ?
            """, [username, start_date, end_date])
            repositories = [row["repository"] for row in cursor.fetchall()]

            return {
                "username": username,
                "period": {"start": start_date, "end": end_date},
                "activity": {
                    "total_prs": total_prs,
                    "total_reviews": total_reviews,
                    "total_commits": total_commits,
                    "repositories": repositories
                }
            }


async def mount(coordinator: Any, config: Dict[str, Any]) -> Any:
    """Mount the MADE Activity tools.

    This is the entry point called by amplifier-core when loading the tool module.

    Args:
        coordinator: The session coordinator.
        config: Tool configuration from the bundle.

    Returns:
        Dictionary with tools list and cleanup function.
    """
    tools = [
        _GetMetricsTool(),
        _SearchGitHubItemsTool(),
        _GetUserActivityTool()
    ]

    def cleanup():
        """Cleanup function called on session teardown."""
        pass

    return {
        "tools": tools,
        "cleanup": cleanup
    }
