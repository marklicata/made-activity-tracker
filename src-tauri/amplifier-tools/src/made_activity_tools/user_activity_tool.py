"""Tool for querying user activity and contributions."""

from typing import Dict, Any, List
from .db_connection import db


class UserActivityTool:
    """Tool for querying user activity and contributions."""

    name = "get_user_activity"
    description = """Get activity summary for a GitHub user.

    Returns:
    - Total PRs created
    - Total reviews performed
    - Total commits
    - Repositories contributed to
    - Collaboration patterns
    """

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

            # Count PRs created
            cursor.execute("""
                SELECT COUNT(*) as count
                FROM pull_requests
                WHERE user = ? AND created_at BETWEEN ? AND ?
            """, [username, start_date, end_date])
            total_prs = cursor.fetchone()["count"]

            # Count reviews performed
            cursor.execute("""
                SELECT COUNT(*) as count
                FROM reviews
                WHERE user = ? AND submitted_at BETWEEN ? AND ?
            """, [username, start_date, end_date])
            total_reviews = cursor.fetchone()["count"]

            # Count commits
            cursor.execute("""
                SELECT COUNT(*) as count
                FROM commits
                WHERE author = ? AND committed_at BETWEEN ? AND ?
            """, [username, start_date, end_date])
            total_commits = cursor.fetchone()["count"]

            # Get repositories contributed to
            cursor.execute("""
                SELECT DISTINCT repository
                FROM (
                    SELECT repository FROM pull_requests
                    WHERE user = ? AND created_at BETWEEN ? AND ?
                    UNION
                    SELECT repository FROM commits
                    WHERE author = ? AND committed_at BETWEEN ? AND ?
                )
            """, [username, start_date, end_date, username, start_date, end_date])
            repositories = [row["repository"] for row in cursor.fetchall()]

            return {
                "username": username,
                "period": {
                    "start": start_date,
                    "end": end_date
                },
                "activity": {
                    "total_prs": total_prs,
                    "total_reviews": total_reviews,
                    "total_commits": total_commits
                },
                "repositories": repositories
            }


async def mount(coordinator, config):
    """Register the user activity tool with Amplifier."""
    tool = UserActivityTool()
    await coordinator.register_tool(tool)

    return lambda: None
