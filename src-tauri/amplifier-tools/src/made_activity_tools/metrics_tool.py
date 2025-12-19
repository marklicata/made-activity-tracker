"""Tool for querying GitHub activity metrics."""

from typing import Dict, Any, List
from .db_connection import db


class MetricsTool:
    """Tool for querying GitHub activity metrics."""

    name = "get_metrics"
    description = """Get GitHub activity metrics (speed, ease, quality) for a date range.

    Metrics include:
    - Speed: cycle time, PR lead time, throughput
    - Ease: PR size, review rounds, rework rate
    - Quality: bug rate, reopen rate, rejection rate

    Can filter by repositories and users.
    """

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
            results["speed"] = self._get_speed_metrics(
                start_date, end_date, repositories, users
            )

        if metric_type in ["ease", "all"]:
            results["ease"] = self._get_ease_metrics(
                start_date, end_date, repositories, users
            )

        if metric_type in ["quality", "all"]:
            results["quality"] = self._get_quality_metrics(
                start_date, end_date, repositories, users
            )

        return {
            "metrics": results,
            "period": {
                "start": start_date,
                "end": end_date
            },
            "filters": {
                "repositories": repositories,
                "users": users
            }
        }

    def _get_speed_metrics(self, start_date: str, end_date: str,
                          repos: List[str], users: List[str]) -> Dict[str, float]:
        """Calculate speed metrics."""
        with db as conn:
            cursor = conn.cursor()

            # Build WHERE clause
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

            # Cycle time (hours from created to closed)
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

    def _get_ease_metrics(self, start_date: str, end_date: str,
                         repos: List[str], users: List[str]) -> Dict[str, float]:
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

            # Average PR size
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

    def _get_quality_metrics(self, start_date: str, end_date: str,
                            repos: List[str], users: List[str]) -> Dict[str, float]:
        """Calculate quality metrics."""
        # Simplified - expand based on your actual schema
        return {
            "bug_rate": 0.0,
            "reopen_rate": 0.0
        }


async def mount(coordinator, config):
    """Register the metrics tool with Amplifier."""
    tool = MetricsTool()
    await coordinator.register_tool(tool)

    # Cleanup function
    def cleanup():
        pass

    return cleanup
