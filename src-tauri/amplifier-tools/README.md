# MADE Activity Tools

Custom Amplifier tools for querying GitHub activity metrics from the MADE Activity Tracker database.

## Installation

```bash
# Install in development mode
uv pip install -e .
```

## Tools

- **metrics**: Query speed, ease, and quality metrics
- **search**: Search issues and pull requests
- **user_activity**: Get user activity summaries

## Usage

These tools are automatically registered with Amplifier via entry points and can be used through the Amplifier chat interface.

## Development

Run tests:
```bash
python test_tools.py
```
