# MADE Activity Tools

Custom Amplifier tools for querying GitHub activity metrics from the MADE Activity Tracker database.

## Overview

This package integrates with **Microsoft Amplifier Foundation** to provide AI-powered chat capabilities. It includes:
- Custom tools for querying metrics, searching, and user activity
- Amplifier Foundation integration with bundle composition
- Support for Anthropic Claude and OpenAI GPT providers

## Architecture

The implementation follows amplifier-foundation's bundle composition pattern:

```
amplifier-tools/
├── bundle.md                    # Main bundle with tool configuration
├── providers/
│   ├── anthropic.yaml          # Anthropic Claude provider
│   └── openai.yaml             # OpenAI GPT provider
└── src/
    └── made_activity_tools/
        ├── tool_module.py      # Unified tool module (kernel protocol)
        ├── db_connection.py    # Database connection utility
        └── server.py           # Flask HTTP server
```

### How It Works

1. **Bundle Composition**: `bundle.md` includes the foundation bundle and adds the custom tool module
2. **Tool Module**: Single module exports three capabilities (get_metrics, search_github_items, get_user_activity)
3. **Local Mounting**: Tool module is mounted via `file://` URI from local source
4. **Provider Selection**: Compose with either `providers/anthropic.yaml` or `providers/openai.yaml`

## Installation

```bash
# Create Python virtual environment
python -m venv .venv

# Activate venv (Windows)
.venv\Scripts\activate

# Or activate venv (Mac/Linux)
source .venv/bin/activate

# Install dependencies
pip install -e .
```

## Tools

The unified tool module provides three capabilities:

### 1. get_metrics
Query speed, ease, and quality metrics:
- **Speed**: cycle time, PR lead time, throughput
- **Ease**: PR size, review rounds, rework rate
- **Quality**: bug rate, reopen rate, rejection rate

Can filter by repositories and users.

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

```bash
# Set required environment variables
export ANTHROPIC_API_KEY="your-key-here"  # Or OPENAI_API_KEY
export DATABASE_PATH="/path/to/activity.db"  # Optional, auto-detects if not set
export AMPLIFIER_PORT=5000  # Optional, defaults to 5000
export AMPLIFIER_AUTH_TOKEN="your-token"  # Optional, defaults to 'dev-token'

# Start the server
python src/made_activity_tools/server.py
```

### Environment Variables

- `ANTHROPIC_API_KEY` or `OPENAI_API_KEY`: API key for LLM provider (required)
- `DATABASE_PATH`: Path to SQLite database (optional, auto-detects default location)
- `AMPLIFIER_PORT`: Server port (optional, defaults to 5000)
- `AMPLIFIER_AUTH_TOKEN`: Authentication token (optional, defaults to 'dev-token')

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

## Development

### Testing Tools Directly

```python
# Test the tool module directly
import asyncio
from made_activity_tools import set_db_path
from made_activity_tools.tool_module import get_metrics

# Set database path
set_db_path("/path/to/activity.db")

# Test metrics
result = asyncio.run(get_metrics(
    metric_type="speed",
    start_date="2024-01-01",
    end_date="2024-12-31"
))
print(result)
```

### Testing via Amplifier Session

```python
import asyncio
from pathlib import Path
from amplifier_foundation import load_bundle

async def test_session():
    # Load bundle
    bundle_path = Path(__file__).parent / "bundle.md"
    bundle = await load_bundle(str(bundle_path))
    
    # Add provider
    provider = await load_bundle("./providers/anthropic.yaml")
    composed = bundle.compose(provider)
    
    # Create session
    prepared = await composed.prepare()
    async with prepared.create_session() as session:
        response = await session.execute("What metrics are available?")
        print(response)

asyncio.run(test_session())
```

## Bundle Customization

### Adding More Tools

Edit `bundle.md` to add additional tool modules:

```yaml
tools:
  - module: made-activity-tools
    source: file://./src/made_activity_tools/tool_module.py
  - module: custom-tool
    source: file://./src/custom_tool/module.py
```

### Changing Providers

Compose with a different provider:

```python
# Use OpenAI instead of Anthropic
provider = await load_bundle("./providers/openai.yaml")
composed = base_bundle.compose(provider)
```

### Overriding Configuration

Create a custom overlay bundle:

```yaml
# custom.yaml
bundle:
  name: custom-config
  version: 1.0.0

providers:
  - module: provider-anthropic
    config:
      temperature: 0.3  # Override temperature
      max_tokens: 4000
```

Then compose: `bundle.compose(custom_bundle)`

## Troubleshooting

### "Module not found" errors
Ensure you're running from the correct directory and the tool module path in `bundle.md` is correct:
```yaml
source: file://./src/made_activity_tools/tool_module.py
```

### "Database not found" errors
Set the `DATABASE_PATH` environment variable or ensure the default location is correct.

### Provider API errors
Verify your API key is set correctly and has sufficient credits.

### Import errors
Make sure all dependencies are installed: `pip install -e .`

## Architecture Notes

This implementation follows amplifier-foundation's design philosophy:

- **Mechanism, not policy**: Bundle defines WHAT to load, foundation handles HOW
- **Ruthless simplicity**: Single tool module, clear bundle structure
- **Text-first**: Bundle is human-readable YAML/Markdown
- **Composable**: Base bundle + provider bundle composition

The old approach (entry points, separate tool classes) has been replaced with:
- Unified tool module following kernel protocol
- Local file-based mounting via `file://` URIs
- Standard bundle composition patterns
