# MADE Activity Tools

Custom Amplifier tools for querying GitHub activity metrics from the MADE Activity Tracker database.

## Overview

This package integrates with **Microsoft Amplifier Foundation** to provide AI-powered chat capabilities in the MADE Activity Tracker desktop application. It includes:
- **Three custom tools** for querying metrics, searching, and user activity
- **Entry point architecture** for automatic tool registration
- **Amplifier Foundation integration** with bundle composition pattern
- **Multi-provider support** for Anthropic Claude and OpenAI GPT models
- **Flask HTTP server** for Tauri sidecar communication

## Architecture

The implementation follows Amplifier Foundation's bundle composition pattern with entry point-based tool registration:

```
amplifier-tools/
├── bundle.md                           # Main bundle configuration
├── providers/
│   ├── anthropic.yaml                  # Anthropic Claude provider config
│   └── openai.yaml                     # OpenAI GPT provider config
├── pyproject.toml                      # Python package with entry points
└── src/
    └── made_activity_tools/
        ├── __init__.py                 # Package initialization
        ├── activity_tracking_tools.py  # Tool implementations with mount()
        ├── db_connection.py            # Database connection utility
        └── server.py                   # Flask HTTP server for Tauri

```

### How It Works

1. **Entry Point Registration**: Tools are registered in `pyproject.toml` under `[project.entry-points."amplifier.modules"]`
2. **Tool Module**: `activity_tracking_tools.py` exports three tool classes and a `mount()` function
3. **Bundle Composition**: `bundle.md` includes the foundation bundle from GitHub
4. **Automatic Discovery**: Amplifier Core discovers and loads tools via entry points
5. **Provider Selection**: Server composes with either `providers/anthropic.yaml` or `providers/openai.yaml`
6. **Sidecar Communication**: Flask server provides HTTP API for Tauri frontend

## Installation

### Prerequisites

- Python 3.11 or higher
- pip (Python package manager)
- Access to GitHub (for downloading Amplifier packages)

### Setup Steps

```bash
# Navigate to amplifier-tools directory
cd src-tauri/amplifier-tools

# Create Python virtual environment
python -m venv .venv

# Activate venv (Windows)
.venv\Scripts\activate

# Or activate venv (Mac/Linux)
source .venv/bin/activate

# Install package in editable mode (includes entry points)
pip install -e .
```

### Verify Installation

```bash
# Check that entry points are registered
python -c "import pkg_resources; [print(ep) for ep in pkg_resources.iter_entry_points('amplifier.modules')]"
```

Expected output:
```
made-activity-tools = made_activity_tools.activity_tracking_tools:mount
```

### Verify Amplifier Packages

```bash
# List installed amplifier packages
pip list | grep amplifier
```

Expected packages:
- `amplifier-core`
- `amplifier-foundation`
- `amplifier-module-provider-anthropic`
- `amplifier-module-provider-openai`

## Tools

The tool module (`activity_tracking_tools.py`) provides three specialized tools for querying GitHub activity data:

### 1. GetMetricsTool (`get_metrics`)

Query speed, ease, and quality metrics for date ranges:

**Parameters:**
- `metric_type`: "speed", "ease", "quality", or "all"
- `start_date`: ISO date format (YYYY-MM-DD)
- `end_date`: ISO date format (YYYY-MM-DD)
- `repositories`: Optional array of repo names (owner/repo format)
- `users`: Optional array of GitHub usernames

**Returns:**
- **Speed metrics**: avg_cycle_time_hours, total_closed
- **Ease metrics**: avg_pr_size_lines, total_prs
- **Quality metrics**: bug_rate, reopen_rate

**Example:**
```json
{
  "metric_type": "speed",
  "start_date": "2024-01-01",
  "end_date": "2024-12-31",
  "repositories": ["facebook/react"],
  "users": ["gaearon"]
}
```

### 2. SearchGitHubItemsTool (`search_github_items`)

Search issues and pull requests by text query:

**Parameters:**
- `query`: Search text (searches titles and bodies)
- `item_type`: "issue", "pull_request", or "both"
- `state`: "open", "closed", or "all"
- `labels`: Optional array of label names
- `repository`: Optional repo name (owner/repo)
- `limit`: Maximum results (default: 10)

**Returns:**
- Array of matching issues/PRs with metadata
- Total count
- Query used

**Example:**
```json
{
  "query": "authentication bug",
  "item_type": "both",
  "state": "all",
  "repository": "myorg/myrepo",
  "limit": 20
}
```

### 3. GetUserActivityTool (`get_user_activity`)

Get activity summary for a specific GitHub user:

**Parameters:**
- `username`: GitHub username
- `start_date`: ISO date format (YYYY-MM-DD)
- `end_date`: ISO date format (YYYY-MM-DD)

**Returns:**
- `total_prs`: PRs created by user
- `total_reviews`: Reviews performed
- `total_commits`: Commits authored
- `repositories`: Array of repos contributed to

**Example:**
```json
{
  "username": "octocat",
  "start_date": "2024-01-01",
  "end_date": "2024-12-31"
}
```

## Running the Server

The Flask server provides an HTTP API for the Tauri frontend to communicate with the AI chat system.

### Environment Variables

Set these environment variables before starting the server:

**Required:**
- `ANTHROPIC_API_KEY` or `OPENAI_API_KEY`: API key for LLM provider

**Optional:**
- `DATABASE_PATH`: Path to SQLite database (auto-detects if not set)
  - Windows: `%APPDATA%\made-activity-tracker\tracker.db`
  - Linux: `~/.local/share/com.made.activity-tracker/made.db`
  - Mac: `~/Library/Application Support/made-activity-tracker/tracker.db`
- `AMPLIFIER_PORT`: Server port (default: 5000)
- `AMPLIFIER_AUTH_TOKEN`: Authentication token (default: 'dev-token')

### Manual Server Start (for testing)

```bash
# Windows (PowerShell)
$env:ANTHROPIC_API_KEY = "your-key-here"
$env:DATABASE_PATH = "$env:APPDATA\made-activity-tracker\tracker.db"
python src/made_activity_tools/server.py

# Mac/Linux
export ANTHROPIC_API_KEY="your-key-here"
export DATABASE_PATH="$HOME/.local/share/com.made.activity-tracker/made.db"
python src/made_activity_tools/server.py
```

### Automatic Start (via Tauri)

The server starts automatically when you run the main application:

```bash
# From repository root
npm run tauri dev
```

The Tauri backend manages the Python sidecar process automatically.

### API Endpoints

**GET /health**
Health check with configuration status.

**POST /chat**
Process chat message through Amplifier.
```json
{
  "message": "Show me metrics for last week",
  "context": {
    "current_page": "dashboard",
    "filters": {
      "date_range": {"start": "2024-01-01", "end": "2024-01-07"},
      "repositories": ["owner/repo"]
    }
  }
}
```

**POST /shutdown**
Gracefully shutdown server.

### 2. search_github_items
Search issues and pull requests by text query. Searches titles, bodies, and labels.
Can filter by state, type, repository, and labels.

### 3. get_user_activity
Get user activity summaries including:
- Total PRs created
- Total reviews performed
- Total commits
- Repositories contributed to

## Running the Server

### Testing Tools Directly

You can test individual tools without running the full server:

```python
import asyncio
import os
from made_activity_tools.activity_tracking_tools import GetMetricsTool
from made_activity_tools.db_connection import set_db_path

# Set database path
set_db_path(os.path.expandvars("%APPDATA%/made-activity-tracker/tracker.db"))

# Test metrics tool
async def test_metrics():
    tool = GetMetricsTool()
    result = await tool.execute(
        metric_type="speed",
        start_date="2024-01-01",
        end_date="2024-12-31"
    )
    print(result)

asyncio.run(test_metrics())
```

### Testing via HTTP API

Test the server endpoints using curl or any HTTP client:

```bash
# Start the server
python src/made_activity_tools/server.py

# In another terminal, test the health endpoint
curl http://localhost:5000/health

# Test a chat request
curl -X POST http://localhost:5000/chat \
  -H "Content-Type: application/json" \
  -d '{
    "message": "What metrics are available?",
    "context": {
      "current_page": "dashboard"
    }
  }'
```

### Testing via Amplifier Session (Advanced)

For testing the Amplifier integration directly:

```python
import asyncio
import os
from pathlib import Path
from amplifier_core import load_bundle

async def test_session():
    # Set API key
    os.environ["ANTHROPIC_API_KEY"] = "your-key-here"

    # Load bundle
    bundle_path = Path(__file__).parent / "bundle.md"
    bundle = await load_bundle(str(bundle_path))

    # Add provider
    provider_path = Path(__file__).parent / "providers" / "anthropic.yaml"
    provider = await load_bundle(str(provider_path))
    composed = bundle.compose(provider)

    # Create session
    prepared = await composed.prepare()
    async with prepared.create_session() as session:
        response = await session.send("What metrics are available?")
        print(response)

asyncio.run(test_session())
```

## Customization

### Adding More Tools

To add additional tools to the module:

1. **Create tool class** in `activity_tracking_tools.py`:
   ```python
   class MyCustomTool:
       name = "my_custom_tool"
       description = "Description of what this tool does"
       parameters = {
           "type": "object",
           "properties": { ... }
       }

       async def execute(self, **kwargs):
           # Tool implementation
           return {"result": "data"}
   ```

2. **Mount in the `mount()` function**:
   ```python
   async def mount(coordinator: ModuleCoordinator, config: dict):
       # ... existing tools ...

       custom_tool = MyCustomTool()
       await coordinator.mount("tools", custom_tool, name=custom_tool.name)

       return None
   ```

3. **Reinstall package** to register changes:
   ```bash
   pip install -e .
   ```

### Changing Providers

The server automatically selects the provider based on which API key is set:

- If `ANTHROPIC_API_KEY` is set → Uses Anthropic Claude
- If `OPENAI_API_KEY` is set → Uses OpenAI GPT

To explicitly use a specific provider, modify `server.py`:

```python
# Force Anthropic
provider_path = Path(__file__).parent.parent / "providers" / "anthropic.yaml"

# Force OpenAI
provider_path = Path(__file__).parent.parent / "providers" / "openai.yaml"
```

### Customizing Provider Settings

Edit `providers/anthropic.yaml` or `providers/openai.yaml`:

```yaml
bundle:
  name: custom-anthropic
  version: 1.0.0

providers:
  - module: provider-anthropic
    config:
      model: "claude-3-5-sonnet-20241022"  # Change model
      temperature: 0.3                      # Adjust temperature
      max_tokens: 4000                      # Set max tokens
```

## Troubleshooting

### "Module not found" or "Entry point not found" errors

**Symptoms:** Server fails to start, or tools are not available in chat

**Solution:**
1. Verify installation: `pip install -e .`
2. Check entry points:
   ```bash
   python -c "import pkg_resources; [print(ep) for ep in pkg_resources.iter_entry_points('amplifier.modules')]"
   ```
3. Reinstall if needed:
   ```bash
   pip uninstall made-activity-tools
   pip install -e .
   ```

### "Database not found" errors

**Symptoms:** Tools return empty results or database errors

**Solution:**
1. Run the main app at least once to create the database
2. Check database location:
   - Windows: `%APPDATA%\made-activity-tracker\tracker.db`
   - Linux: `~/.local/share/com.made.activity-tracker/made.db`
3. Set `DATABASE_PATH` environment variable explicitly if needed

### Provider API errors

**Symptoms:** Chat returns authentication or rate limit errors

**Solution:**
1. Verify API key is set:
   - Windows: `echo %ANTHROPIC_API_KEY%`
   - Mac/Linux: `echo $ANTHROPIC_API_KEY`
2. Check API key has sufficient credits
3. Verify network connectivity to provider API

### Port conflicts (Address already in use)

**Symptoms:** Server fails to start with "port 5000 already in use"

**Solution:**
1. Find process using port 5000:
   - Windows: `netstat -ano | findstr :5000`
   - Mac/Linux: `lsof -i :5000`
2. Kill the process or set different port:
   ```bash
   export AMPLIFIER_PORT=5001
   ```

### Import errors / Package dependency issues

**Symptoms:** `ModuleNotFoundError` for amplifier packages

**Solution:**
1. Ensure you have internet access (packages installed from GitHub)
2. Reinstall all dependencies:
   ```bash
   pip uninstall -y amplifier-core amplifier-foundation amplifier-module-provider-anthropic amplifier-module-provider-openai
   pip install -e .
   ```
3. Check Python version: `python --version` (must be 3.11+)

### "First request takes too long" (10-15 seconds)

**Symptoms:** Initial chat message has long delay

**This is normal behavior:**
- Amplifier Foundation downloads bundles from GitHub on first use
- Bundles are cached locally after first download
- Subsequent requests are fast (< 1 second)

## Architecture Notes

This implementation follows Amplifier Foundation's design philosophy:

### Core Principles

- **Entry Point Discovery**: Tools are automatically discovered via Python package entry points
- **Bundle Composition**: Bundles define WHAT to load, Amplifier Core handles HOW
- **Tool Mounting**: Tools register themselves with the coordinator via the `mount()` function
- **Stateless Tools**: Each tool is a simple class with `name`, `description`, `parameters`, and `execute()`
- **Provider Agnostic**: Same tools work with any LLM provider (Anthropic, OpenAI, etc.)

### Tool Architecture

Each tool follows a simple pattern:

```python
class ToolName:
    name = "tool_name"              # Tool identifier
    description = "..."             # What the tool does
    parameters = {...}              # JSON schema for parameters

    async def execute(self, **kwargs) -> dict:
        # Implementation
        return {"result": "data"}
```

### Mount Function

The `mount()` function is the entry point called by Amplifier Core:

```python
async def mount(coordinator: ModuleCoordinator, config: dict):
    tool = MyTool()
    await coordinator.mount("tools", tool, name=tool.name)
    return None  # Optional cleanup function
```

### Database Connection

Tools access the SQLite database via a context manager:

```python
from .db_connection import db

with db as conn:
    cursor = conn.cursor()
    cursor.execute("SELECT ...")
    results = cursor.fetchall()
```

### HTTP Server Flow

1. **Tauri frontend** sends POST to `/chat` with message and context
2. **Flask server** receives request
3. **Amplifier session** processes message with tools available
4. **Tools execute** queries against SQLite database
5. **LLM generates** response using tool results
6. **Server returns** formatted response to frontend
7. **Chat UI** displays response to user

This architecture keeps the AI chat system decoupled from the Rust backend while providing seamless integration with the application's data.
