"""Tool for searching issues and pull requests."""

from typing import Dict, Any, List, Optional
from .db_connection import db


class SearchTool:
    """Tool for searching issues and pull requests."""

    name = "search_github_items"
    description = """Search for GitHub issues and pull requests by text query.

    Searches in:
    - Issue/PR titles
    - Issue/PR bodies
    - Labels

    Can filter by state, type, labels, repository.
    """

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
            results.extend(self._search_issues(
                query, state, labels, repository, limit
            ))

        if item_type in ["pull_request", "both"]:
            results.extend(self._search_pull_requests(
                query, state, labels, repository, limit
            ))

        # Sort by created date (most recent first)
        results.sort(key=lambda x: x["created_at"], reverse=True)

        return {
            "results": results[:limit],
            "total": len(results),
            "query": query
        }

    def _search_issues(self, query: str, state: str, labels: List[str],
                      repository: Optional[str], limit: int) -> List[Dict]:
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

            # Note: Label filtering would need a join if you have a separate labels table
            # Simplified here assuming labels are stored as JSON or comma-separated

            where_clause = " AND ".join(where_parts)

            query_sql = f"""
                SELECT
                    id,
                    number,
                    title,
                    state,
                    repository,
                    html_url,
                    created_at,
                    closed_at
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
                    "closed_at": row["closed_at"]
                })

            return results

    def _search_pull_requests(self, query: str, state: str, labels: List[str],
                             repository: Optional[str], limit: int) -> List[Dict]:
        """Search pull_requests table."""
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
                    id,
                    number,
                    title,
                    state,
                    repository,
                    html_url,
                    created_at,
                    merged_at,
                    closed_at
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


async def mount(coordinator, config):
    """Register the search tool with Amplifier."""
    tool = SearchTool()
    await coordinator.register_tool(tool)

    return lambda: None
