# AI Chat Panel - Revised Implementation Specification
## (Leveraging Amplifier's Built-in Capabilities)

## Overview
This document provides a streamlined implementation plan that uses Amplifier's built-in session management, conversation history, and provider abstraction. We only implement what's unique to our application: custom database query tools and frontend UI.

## What Amplifier Provides (We Don't Duplicate)
✅ Session management and persistence
✅ Conversation history across messages
✅ Provider abstraction (Anthropic/OpenAI/Azure/Ollama)
✅ Context management
✅ Specialized agents
✅ Module/tool extensibility framework

## What We Build
🔨 Custom tools for GitHub metrics database queries
🔨 Frontend UI (React components)
🔨 Tauri integration layer
🔨 App context injection

---

## Prerequisites

### Required Tools
- Python 3.9+ with UV package manager
- Rust toolchain (already installed)
- Node.js/npm (already installed)
- API Key: `ANTHROPIC_API_KEY` or `OPENAI_API_KEY`

### Install UV Package Manager
```bash
# Windows PowerShell
powershell -c "irm https://astral.sh/uv/install.ps1 | iex"

# macOS/Linux/WSL
curl -LsSf https://astral.sh/uv/install.sh | sh
```

### Install Amplifier
**Note**: We'll add Amplifier as a dependency in `pyproject.toml` instead of using `uv tool install`, since we're importing it as a library (not using CLI):

```toml
dependencies = [
    "amplifier-core @ git+https://github.com/microsoft/amplifier-core",
    "amplifier-profiles @ git+https://github.com/microsoft/amplifier-profiles",
    ...
]
```

---

## Phase 1: Python Custom Tools for Database Queries

### Step 1.1: Create Project Structure
**Goal**: Set up Python project for custom Amplifier tools

**Actions**:
1. Create directory: `src-tauri/amplifier-tools/`
2. Navigate: `cd src-tauri/amplifier-tools`
3. Initialize Python project: `uv init`

**File structure**:
```
src-tauri/amplifier-tools/
├── pyproject.toml
├── README.md
├── .gitignore
└── src/
    └── made_activity_tools/
        ├── __init__.py
        ├── metrics_tool.py
        ├── search_tool.py
        ├── user_activity_tool.py
        └── db_connection.py
```

**Verification**: Directory structure exists

---

### Step 1.2: Configure pyproject.toml
**Goal**: Define package with Amplifier tool entry points

**File: `src-tauri/amplifier-tools/pyproject.toml`**
```toml
[project]
name = "made-activity-tools"
version = "0.1.0"
description = "Custom Amplifier tools for MADE Activity Tracker"
requires-python = ">=3.9"
dependencies = [
    "amplifier-core",
]

[project.entry-points."amplifier.tools"]
metrics = "made_activity_tools.metrics_tool:mount"
search = "made_activity_tools.search_tool:mount"
user_activity = "made_activity_tools.user_activity_tool:mount"

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"
```

**Actions**:
1. Create file with above content
2. Run: `uv pip install -e .`

**Verification**: Package installs in editable mode

---

### Step 1.3: Create Database Connection Utility
**Goal**: Provide SQLite connection for tools

**File: `src-tauri/amplifier-tools/src/made_activity_tools/db_connection.py`**
```python
import sqlite3
import os
from pathlib import Path
from typing import Optional

class DatabaseConnection:
    """Manages SQLite database connection for MADE Activity Tracker."""

    def __init__(self, db_path: Optional[str] = None):
        if db_path is None:
            # Default to user's data directory
            # This path should match where your Tauri app stores the database
            if os.name == 'nt':  # Windows
                base = os.environ.get('APPDATA')
            else:  # macOS/Linux
                base = os.path.expanduser('~/.local/share')

            db_path = os.path.join(base, 'com.made.activity-tracker', 'activity.db')

        self.db_path = db_path
        self._conn = None

    def connect(self) -> sqlite3.Connection:
        """Get or create database connection."""
        if self._conn is None:
            if not os.path.exists(self.db_path):
                raise FileNotFoundError(f"Database not found at {self.db_path}")

            self._conn = sqlite3.connect(self.db_path)
            self._conn.row_factory = sqlite3.Row  # Access columns by name

        return self._conn

    def close(self):
        """Close database connection."""
        if self._conn:
            self._conn.close()
            self._conn = None

    def __enter__(self):
        return self.connect()

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()

# Global instance that can be configured
db = DatabaseConnection()

def set_db_path(path: str):
    """Configure database path (called by server on startup)."""
    global db
    db = DatabaseConnection(path)
```

**Verification**: Module imports without errors

---

### Step 1.4: Implement Metrics Tool
**Goal**: Tool for querying speed/ease/quality metrics

**File: `src-tauri/amplifier-tools/src/made_activity_tools/metrics_tool.py`**
```python
from typing import Dict, Any, List, Optional
from datetime import datetime
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

# Amplifier mount function (required)
async def mount(coordinator, config):
    """Register the metrics tool with Amplifier."""
    tool = MetricsTool()
    await coordinator.register_tool(tool)

    # Cleanup function
    def cleanup():
        pass

    return cleanup
```

**Verification**: Tool implements required interface

---

### Step 1.5: Implement Search Tool
**Goal**: Tool for searching issues and PRs

**File: `src-tauri/amplifier-tools/src/made_activity_tools/search_tool.py`**
```python
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
```

**Verification**: Tool implements required interface

---

### Step 1.6: Implement User Activity Tool
**Goal**: Tool for querying individual user activity

**File: `src-tauri/amplifier-tools/src/made_activity_tools/user_activity_tool.py`**
```python
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
```

**Verification**: Tool implements required interface

---

### Step 1.7: Package Init
**Goal**: Export all tools from package

**File: `src-tauri/amplifier-tools/src/made_activity_tools/__init__.py`**
```python
"""Custom Amplifier tools for MADE Activity Tracker."""

from .metrics_tool import MetricsTool, mount as mount_metrics
from .search_tool import SearchTool, mount as mount_search
from .user_activity_tool import UserActivityTool, mount as mount_user_activity
from .db_connection import set_db_path

__all__ = [
    'MetricsTool',
    'SearchTool',
    'UserActivityTool',
    'set_db_path',
    'mount_metrics',
    'mount_search',
    'mount_user_activity'
]

__version__ = "0.1.0"
```

**Verification**: Package imports cleanly

---

## Phase 2: Python HTTP Server (Amplifier Wrapper)

### Step 2.1: Create HTTP Server
**Goal**: Lightweight HTTP server that uses AmplifierSession

**File: `src-tauri/amplifier-tools/src/made_activity_tools/server.py`**
```python
#!/usr/bin/env python3
"""HTTP server that wraps Amplifier for Tauri integration."""

import asyncio
import os
import sys
from flask import Flask, request, jsonify, Response
from flask_cors import CORS
from amplifier_core import AmplifierSession
from made_activity_tools import set_db_path

app = Flask(__name__)
CORS(app)

# Configuration
AUTH_TOKEN = os.environ.get('AMPLIFIER_AUTH_TOKEN', 'dev-token')
DB_PATH = os.environ.get('DATABASE_PATH')
API_KEY = os.environ.get('ANTHROPIC_API_KEY') or os.environ.get('OPENAI_API_KEY')
PROVIDER = 'anthropic' if os.environ.get('ANTHROPIC_API_KEY') else 'openai'

# Configure database path
if DB_PATH:
    set_db_path(DB_PATH)

# Amplifier configuration
AMPLIFIER_CONFIG = {
    "session": {
        "orchestrator": "loop-basic",
        "context": "context-simple"
    },
    "providers": [
        {
            "module": f"provider-{PROVIDER}",
            "config": {
                "api_key": API_KEY
            }
        }
    ],
    "tools": [
        {"module": "made_activity_tools.metrics_tool"},
        {"module": "made_activity_tools.search_tool"},
        {"module": "made_activity_tools.user_activity_tool"}
    ]
}

def check_auth():
    """Verify request authentication."""
    token = request.headers.get('X-Auth-Token')
    return token == AUTH_TOKEN

@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint."""
    return jsonify({
        'status': 'ok',
        'provider': PROVIDER,
        'has_api_key': bool(API_KEY)
    })

@app.route('/chat', methods=['POST'])
async def chat():
    """Process chat message through Amplifier."""
    if not check_auth():
        return jsonify({'error': 'Unauthorized'}), 401

    if not API_KEY:
        return jsonify({'error': 'No API key configured'}), 500

    try:
        data = request.json
        user_message = data.get('message', '')
        app_context = data.get('context', {})

        # Build system prompt with app context
        system_prompt = build_system_prompt(app_context)

        # Create Amplifier session and execute
        async with AmplifierSession(config=AMPLIFIER_CONFIG) as session:
            # Add system context as first message if needed
            full_prompt = f"{system_prompt}\n\nUser: {user_message}"

            response = await session.execute(full_prompt)

            return jsonify({
                'response': response,
                'context': app_context
            })

    except Exception as e:
        print(f"Error in chat: {e}", file=sys.stderr)
        return jsonify({'error': str(e)}), 500

def build_system_prompt(context: dict) -> str:
    """Build system prompt from app context."""
    parts = ["You are an AI assistant helping users analyze GitHub activity data."]

    if context.get('current_page'):
        parts.append(f"The user is currently viewing: {context['current_page']}")

    filters = context.get('filters', {})

    if date_range := filters.get('date_range'):
        parts.append(f"Date range: {date_range['start']} to {date_range['end']}")

    if repos := filters.get('repositories'):
        parts.append(f"Filtered repositories: {', '.join(repos)}")

    if squads := filters.get('squads'):
        parts.append(f"Filtered squads: {', '.join(squads)}")

    if users := filters.get('users'):
        parts.append(f"Filtered users: {', '.join(users)}")

    parts.append("\nUse the available tools to query the database and provide accurate answers.")

    return "\n".join(parts)

@app.route('/shutdown', methods=['POST'])
def shutdown():
    """Shutdown server."""
    if not check_auth():
        return jsonify({'error': 'Unauthorized'}), 401

    func = request.environ.get('werkzeug.server.shutdown')
    if func:
        func()

    return jsonify({'status': 'shutting down'})

if __name__ == '__main__':
    port = int(os.environ.get('AMPLIFIER_PORT', 5000))

    print(f"Starting Amplifier server on port {port}", file=sys.stderr)
    print(f"Provider: {PROVIDER}", file=sys.stderr)
    print(f"Database: {DB_PATH or 'default location'}", file=sys.stderr)

    app.run(
        host='127.0.0.1',
        port=port,
        debug=False,
        use_reloader=False
    )
```

**Verification**: Server starts and responds to /health

---

### Step 2.2: Add Dependencies
**Goal**: Install Flask and required packages

**File: `src-tauri/amplifier-tools/pyproject.toml`** - Dependencies already defined in Step 1.2:
```toml
dependencies = [
    "amplifier-core @ git+https://github.com/microsoft/amplifier-core",
    "amplifier-profiles @ git+https://github.com/microsoft/amplifier-profiles",
    "flask>=3.0.0",
    "flask-cors>=4.0.0",
]
```

**Note**: We need both `amplifier-core` (provides `AmplifierSession` class) and `amplifier-profiles` (for loading profile configurations). The "foundation" refers to a profile configuration, not a separate package.

**Actions**:
```bash
cd src-tauri/amplifier-tools
uv pip install -e .
```

**Verification**: All packages install

---

### Step 2.3: Test Tools Locally
**Goal**: Verify tools work before integration

**File: `src-tauri/amplifier-tools/test_tools.py`**
```python
"""Manual test script for tools."""
import asyncio
from made_activity_tools import MetricsTool, SearchTool, UserActivityTool
from made_activity_tools import set_db_path

# Set your database path
set_db_path("path/to/your/activity.db")

async def test_metrics():
    tool = MetricsTool()
    result = await tool.execute(
        metric_type="speed",
        start_date="2024-01-01",
        end_date="2024-01-31"
    )
    print("Metrics:", result)

async def test_search():
    tool = SearchTool()
    result = await tool.execute(
        query="bug",
        item_type="issue",
        limit=5
    )
    print("Search:", result)

async def test_user_activity():
    tool = UserActivityTool()
    result = await tool.execute(
        username="test-user",
        start_date="2024-01-01",
        end_date="2024-01-31"
    )
    print("User Activity:", result)

if __name__ == '__main__':
    asyncio.run(test_metrics())
    asyncio.run(test_search())
    asyncio.run(test_user_activity())
```

**Actions**:
1. Update database path
2. Run: `python test_tools.py`

**Verification**: Tools return valid data

---

## Phase 3: Rust Tauri Integration

### Step 3.1: Create Sidecar Launcher Module
**Goal**: Rust code to start/stop Python server

**File: `src-tauri/src/ai/mod.rs`**
```rust
pub mod sidecar;
pub mod types;
pub mod commands;

pub use sidecar::AmplifierSidecar;
pub use types::*;
pub use commands::*;
```

**File: `src-tauri/src/ai/types.rs`**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppContext {
    pub current_page: String,
    pub filters: FilterState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterState {
    pub date_range: Option<DateRange>,
    pub repositories: Vec<String>,
    pub squads: Vec<String>,
    pub users: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub context: AppContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub response: String,
    pub context: AppContext,
}
```

**Verification**: Module compiles

---

### Step 3.2: Implement Sidecar Launcher
**Goal**: Start Python server as child process

**File: `src-tauri/src/ai/sidecar.rs`**
```rust
use std::process::{Child, Command, Stdio};
use std::net::TcpListener;
use anyhow::{Result, anyhow};
use std::env;
use std::path::PathBuf;

pub struct AmplifierSidecar {
    process: Option<Child>,
    pub port: u16,
    pub auth_token: String,
}

impl AmplifierSidecar {
    pub fn new() -> Self {
        let auth_token = uuid::Uuid::new_v4().to_string();

        Self {
            process: None,
            port: 0,
            auth_token,
        }
    }

    pub fn start(&mut self, db_path: PathBuf) -> Result<()> {
        // Find available port
        self.port = self.find_available_port()?;

        // Get path to Python server
        let server_path = self.get_server_path()?;

        println!("Starting Amplifier sidecar on port {}", self.port);
        println!("Server path: {}", server_path);
        println!("Database path: {:?}", db_path);

        // Start Python server
        let mut cmd = Command::new("python");
        cmd.arg(&server_path)
            .env("AMPLIFIER_PORT", self.port.to_string())
            .env("AMPLIFIER_AUTH_TOKEN", &self.auth_token)
            .env("DATABASE_PATH", db_path.to_string_lossy().to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Pass through API keys
        if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
            cmd.env("ANTHROPIC_API_KEY", key);
        }
        if let Ok(key) = env::var("OPENAI_API_KEY") {
            cmd.env("OPENAI_API_KEY", key);
        }

        let child = cmd.spawn()?;
        self.process = Some(child);

        // Wait for server to be ready
        std::thread::sleep(std::time::Duration::from_secs(2));

        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            process.kill()?;
        }
        Ok(())
    }

    fn find_available_port(&self) -> Result<u16> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        Ok(port)
    }

    fn get_server_path(&self) -> Result<String> {
        // In development
        if cfg!(debug_assertions) {
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            let server_path = PathBuf::from(manifest_dir)
                .join("amplifier-tools")
                .join("src")
                .join("made_activity_tools")
                .join("server.py");

            if !server_path.exists() {
                return Err(anyhow!("Server script not found at {:?}", server_path));
            }

            Ok(server_path.to_string_lossy().to_string())
        } else {
            // In production, bundle with app
            let exe_dir = std::env::current_exe()?
                .parent()
                .ok_or_else(|| anyhow!("Failed to get exe directory"))?
                .to_path_buf();

            Ok(exe_dir
                .join("amplifier-tools")
                .join("server.py")
                .to_string_lossy()
                .to_string())
        }
    }
}

impl Drop for AmplifierSidecar {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
```

**Add to `Cargo.toml`**:
```toml
uuid = { version = "1.0", features = ["v4"] }
anyhow = "1.0"
```

**Verification**: Compiles successfully

---

### Step 3.3: Create HTTP Client
**Goal**: Rust client to communicate with Python server

**File: `src-tauri/src/ai/client.rs`** (add to mod.rs)
```rust
use crate::ai::types::*;
use anyhow::Result;
use reqwest;
use std::time::Duration;

pub struct AmplifierClient {
    base_url: String,
    auth_token: String,
    client: reqwest::Client,
}

impl AmplifierClient {
    pub fn new(port: u16, auth_token: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url: format!("http://127.0.0.1:{}", port),
            auth_token,
            client,
        }
    }

    pub async fn health_check(&self) -> Result<bool> {
        let response = self.client
            .get(&format!("{}/health", self.base_url))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let response = self.client
            .post(&format!("{}/chat", self.base_url))
            .header("X-Auth-Token", &self.auth_token)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Chat failed: {}", error));
        }

        let result: ChatResponse = response.json().await?;
        Ok(result)
    }
}
```

Add to `Cargo.toml`:
```toml
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["full"] }
```

**Verification**: Compiles

---

### Step 3.4: Create Tauri Commands
**Goal**: Expose chat to frontend

**File: `src-tauri/src/ai/commands.rs`**
```rust
use crate::ai::{AmplifierClient, ChatRequest, ChatResponse};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn send_chat_message(
    state: State<'_, AppState>,
    request: ChatRequest,
) -> Result<ChatResponse, String> {
    let client = state.amplifier_client.lock().await;

    client
        .chat(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_amplifier_health(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let client = state.amplifier_client.lock().await;

    client
        .health_check()
        .await
        .map_err(|e| e.to_string())
}
```

**Verification**: Commands compile

---

### Step 3.5: Update AppState and main.rs
**Goal**: Initialize sidecar on app startup

**File: `src-tauri/src/main.rs`**

Update imports:
```rust
mod ai;
use ai::{AmplifierSidecar, AmplifierClient, send_chat_message, check_amplifier_health};
use tokio::sync::Mutex as TokioMutex;
use std::sync::Arc;
```

Update AppState:
```rust
pub struct AppState {
    pub db: Mutex<Connection>,
    pub amplifier_client: Arc<TokioMutex<AmplifierClient>>,
}
```

Update main function:
```rust
#[tokio::main]
async fn main() {
    // ... existing database setup ...

    // Get database path
    let db_path = /* your database path */;

    // Start Amplifier sidecar
    let mut sidecar = AmplifierSidecar::new();
    sidecar.start(db_path.clone())
        .expect("Failed to start Amplifier sidecar");

    println!("Amplifier sidecar started on port {}", sidecar.port);

    // Create client
    let client = AmplifierClient::new(sidecar.port, sidecar.auth_token.clone());

    // Health check
    match client.health_check().await {
        Ok(true) => println!("Amplifier health check passed"),
        Ok(false) => eprintln!("Amplifier health check failed"),
        Err(e) => eprintln!("Amplifier health check error: {}", e),
    }

    let app_state = AppState {
        db: Mutex::new(conn),
        amplifier_client: Arc::new(TokioMutex::new(client)),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // ... existing commands ...
            send_chat_message,
            check_amplifier_health,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    // Cleanup on exit
    drop(sidecar);
}
```

**Verification**: App compiles and starts

---

## Phase 4: Frontend Components

### Step 4.1: Create TypeScript Types
**Goal**: Frontend types matching Rust types

**File: `src/types/ai.ts`**
```typescript
export interface AppContext {
  current_page: string;
  filters: FilterState;
}

export interface FilterState {
  date_range?: {
    start: string;
    end: string;
  };
  repositories: string[];
  squads: string[];
  users: string[];
}

export interface ChatRequest {
  message: string;
  context: AppContext;
}

export interface ChatResponse {
  response: string;
  context: AppContext;
}

export interface ChatMessage {
  role: 'user' | 'assistant';
  content: string;
  timestamp: number;
}
```

---

### Step 4.2: Create Chat Store
**Goal**: Simple Zustand store for UI state only

**File: `src/stores/chatStore.ts`**
```typescript
import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { ChatMessage, AppContext } from '../types/ai';
import { invoke } from '@tauri-apps/api/tauri';

interface ChatStore {
  isOpen: boolean;
  messages: ChatMessage[];
  isLoading: boolean;
  error: string | null;

  togglePanel: () => void;
  setOpen: (open: boolean) => void;
  sendMessage: (message: string, context: AppContext) => Promise<void>;
  clearMessages: () => void;
}

export const useChatStore = create<ChatStore>()(
  persist(
    (set, get) => ({
      isOpen: false,
      messages: [],
      isLoading: false,
      error: null,

      togglePanel: () => set((state) => ({ isOpen: !state.isOpen })),

      setOpen: (open: boolean) => set({ isOpen: open }),

      sendMessage: async (message: string, context: AppContext) => {
        const { messages } = get();

        // Add user message immediately
        const userMessage: ChatMessage = {
          role: 'user',
          content: message,
          timestamp: Date.now(),
        };

        set({
          messages: [...messages, userMessage],
          isLoading: true,
          error: null,
        });

        try {
          const response = await invoke<{ response: string }>('send_chat_message', {
            request: { message, context },
          });

          // Add assistant response
          const assistantMessage: ChatMessage = {
            role: 'assistant',
            content: response.response,
            timestamp: Date.now(),
          };

          set((state) => ({
            messages: [...state.messages, assistantMessage],
            isLoading: false,
          }));
        } catch (error) {
          console.error('Chat error:', error);
          set({
            error: error instanceof Error ? error.message : 'Unknown error',
            isLoading: false,
          });
        }
      },

      clearMessages: () => set({ messages: [] }),
    }),
    {
      name: 'chat-storage',
      partialize: (state) => ({ isOpen: state.isOpen }),
    }
  )
);
```

**Verification**: Store compiles

---

### Step 4.3-4.7: Create UI Components
**Goal**: Chat interface components

Use the same ChatPanel, ChatMessage, ChatInput, and ChatSuggestions components from the original spec (Phase 9 steps 3-6), but **remove** any references to:
- Message IDs
- Feedback buttons
- Session management
- Chat history loading

The components should be simpler since Amplifier handles history.

**Key simplifications**:
- No `loadHistory()` function
- No `sendFeedback()` function
- Messages are only stored in component state (ephemeral)
- Simpler ChatMessage component without feedback UI

---

## Phase 5: Testing & Polish

### Step 5.1: Integration Testing
**Goal**: Verify end-to-end flow

**Test steps**:
1. Start app: `npm run dev:tauri`
2. Verify sidecar starts (check console)
3. Open chat panel
4. Send test message: "What metrics are available?"
5. Verify response from Amplifier
6. Test tool calls: "Show me metrics for January 2024"
7. Verify database query executes
8. Test search: "Find bugs about authentication"
9. Test user activity: "What has alice been working on?"

**Verification**: All queries work end-to-end

---

### Step 5.2: Error Handling
**Goal**: Graceful failures

**Add error handling for**:
- API key missing
- Database not found
- Sidecar fails to start
- Network errors
- Query timeouts

**Verification**: Clear error messages displayed

---

### Step 5.3: Performance
**Goal**: Responsive UI

**Optimizations**:
- Show loading state immediately
- Stream responses if Amplifier supports it
- Cache frequently used queries
- Debounce input

**Verification**: UI feels responsive

---

## Completion Checklist

### Backend
- [ ] Python custom tools package created
- [ ] Metrics tool implemented
- [ ] Search tool implemented
- [ ] User activity tool implemented
- [ ] HTTP server wraps AmplifierSession
- [ ] Tools registered as Amplifier modules
- [ ] Server starts and responds to health checks

### Rust Integration
- [ ] Sidecar launcher implemented
- [ ] HTTP client implemented
- [ ] Tauri commands exposed
- [ ] AppState includes Amplifier client
- [ ] Sidecar starts on app launch
- [ ] Health check passes

### Frontend
- [ ] TypeScript types defined
- [ ] Chat store implemented (simple)
- [ ] ChatPanel component
- [ ] ChatMessage component
- [ ] ChatInput component
- [ ] ChatSuggestions component
- [ ] Toggle button in navigation
- [ ] Panel state persists

### Testing
- [ ] Tools tested locally
- [ ] End-to-end chat flow works
- [ ] Database queries execute
- [ ] Context passed correctly
- [ ] Error handling robust
- [ ] UI responsive

---

## Key Differences from Original Spec

**Removed** (Amplifier handles it):
- ❌ Chat history database table
- ❌ Session management code
- ❌ Chat CRUD queries
- ❌ Provider switching logic
- ❌ Conversation persistence
- ❌ Message feedback system

**Simplified**:
- ✅ Python sidecar is just HTTP wrapper around AmplifierSession
- ✅ No custom Amplifier client logic (use library)
- ✅ Frontend store is UI-only (no persistence)
- ✅ Fewer Tauri commands needed

**New/Changed**:
- ✅ Custom tools as Amplifier modules
- ✅ Tool registration via entry points
- ✅ Context passed via system prompt
- ✅ Amplifier config defines capabilities

---

## Estimated Timeline

- **Phase 1**: 3 days (Python tools - most complex part)
- **Phase 2**: 1 day (HTTP server wrapper)
- **Phase 3**: 2 days (Rust integration)
- **Phase 4**: 2 days (Frontend UI)
- **Phase 5**: 1 day (Testing & polish)

**Total: ~9 days** (vs 15 days in original spec)

---

## Next Steps

1. Start with Phase 1: Build and test Python tools locally
2. Once tools work, wrap in HTTP server (Phase 2)
3. Integrate with Tauri (Phase 3)
4. Build UI (Phase 4)
5. Test and polish (Phase 5)

Let me know when you're ready to start implementing!
