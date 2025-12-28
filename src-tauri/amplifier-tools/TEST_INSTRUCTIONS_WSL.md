# Testing Instructions for WSL

Since you're using WSL, here are the Linux-specific commands.

## Step 1: Open WSL Terminal

Open your WSL terminal (Ubuntu, etc.) and navigate to the project:

```bash
cd /mnt/c/Users/malicata/source/made-activity-tracker/src-tauri/amplifier-tools
```

## Step 2: Activate Virtual Environment

The `.venv` directory exists and is already Linux-compatible:

```bash
source .venv/bin/activate
```

You should see `(.venv)` appear in your prompt.

## Step 3: Install/Update Dependencies

```bash
# Make sure dependencies are installed
pip install -e .
```

This will install:
- amplifier-core (from GitHub)
- amplifier-foundation (from GitHub)
- anthropic
- flask, flask-cors

**Note**: First time will download from GitHub and may take a minute.

## Step 4: Set Environment Variables

```bash
# Set your Anthropic API key
export ANTHROPIC_API_KEY="sk-ant-your-actual-key-here"

# Optional: Set database path if not in default location
export DATABASE_PATH="/path/to/your/activity.db"
```

To make these persist across sessions, add to `~/.bashrc` or `~/.zshrc`:

```bash
echo 'export ANTHROPIC_API_KEY="your-key"' >> ~/.bashrc
source ~/.bashrc
```

## Step 5: Run Comprehensive Test Suite

```bash
python run_tests.py
```

This will test:
1. ✓ Python version (3.11+)
2. ✓ Python imports (tool_module, amplifier-foundation, amplifier-core)
3. ✓ File structure (bundle.md, providers/, tool_module.py)
4. ✓ Environment variables (API key, database path)
5. ✓ Bundle loading (can parse and load bundle.md)

### Expected Output

If everything is working, you should see:

```
======================================================================
  MADE Activity Tools - Test Suite
======================================================================

======================================================================
  Test 1: Python Version
======================================================================

✓ PASS: Python version check
       Version: 3.11.x
       Meets requirement

... (more tests) ...

======================================================================
  Test Summary
======================================================================

✓✓✓ ALL TESTS PASSED ✓✓✓

You can now start the server:
  python src/made_activity_tools/server.py
```

## Step 6: Start the Server

If all tests pass, start the server:

```bash
python src/made_activity_tools/server.py
```

Expected output:
```
Starting Amplifier server on port 5000
Provider: anthropic
Database: /path/to/activity.db
API Key configured: True
 * Serving Flask app 'server'
 * Debug mode: off
WARNING: This is a development server. Do not use it in production.
 * Running on http://127.0.0.1:5000
```

**Note**: The first time you run this, it will download modules from GitHub. This can take 30-60 seconds. You'll see messages like:
```
Downloading git+https://github.com/microsoft/amplifier-foundation@main
Downloading git+https://github.com/microsoft/amplifier-module-provider-anthropic@main
...
```

## Step 7: Test Health Endpoint

Open a **new WSL terminal** (keep the server running) and test:

```bash
curl http://127.0.0.1:5000/health
```

Expected response:
```json
{
  "status": "ok",
  "provider": "anthropic",
  "has_api_key": true,
  "db_path": "/path/to/activity.db"
}
```

## Step 8: Test Chat Endpoint

```bash
curl -X POST http://127.0.0.1:5000/chat \
  -H "Content-Type: application/json" \
  -H "X-Auth-Token: dev-token" \
  -d '{"message": "What metrics are available?"}'
```

Expected: You should get a JSON response with an AI-generated answer about the available metrics tools.

## Database Path Notes for WSL

If your database is in Windows filesystem, you need to use WSL path format:

```bash
# Windows path: C:\Users\malicata\AppData\Roaming\com.made.activity-tracker\activity.db
# WSL path:
export DATABASE_PATH="/mnt/c/Users/malicata/AppData/Roaming/com.made.activity-tracker/activity.db"
```

Or if the database is in WSL filesystem:
```bash
export DATABASE_PATH="$HOME/.local/share/com.made.activity-tracker/activity.db"
```

## Quick Start (All Commands)

Here's the complete sequence:

```bash
# Navigate to project
cd /mnt/c/Users/malicata/source/made-activity-tracker/src-tauri/amplifier-tools

# Activate venv
source .venv/bin/activate

# Install dependencies (first time only)
pip install -e .

# Set API key
export ANTHROPIC_API_KEY="your-key-here"

# Optional: Set database path
export DATABASE_PATH="/path/to/activity.db"

# Run tests
python run_tests.py

# If tests pass, start server
python src/made_activity_tools/server.py
```

## Troubleshooting

### "python: command not found"

```bash
# Check if python3 is available
python3 --version

# If yes, use python3 instead
python3 run_tests.py
python3 src/made_activity_tools/server.py
```

### "pip: command not found"

```bash
# Install pip
sudo apt update
sudo apt install python3-pip

# Or use python -m pip
python -m pip install -e .
```

### Permission errors on .venv

```bash
# Fix permissions
chmod -R u+w .venv
```

### "ModuleNotFoundError" after pip install

```bash
# Make sure venv is activated
source .venv/bin/activate

# Reinstall
pip install --force-reinstall -e .
```

### Port 5000 already in use

```bash
# Use a different port
export AMPLIFIER_PORT=5001
python src/made_activity_tools/server.py
```

### Can't access server from Windows

If you want to access the server from Windows browser/apps:

```bash
# Start server on all interfaces
# In server.py, the app.run() call already uses 127.0.0.1
# Access from Windows using: http://localhost:5000
```

WSL2 automatically forwards localhost ports to Windows.

## What to Report Back

After running the tests, please share:

1. **Test Results**: Copy/paste output from `python run_tests.py`
2. **Server Startup**: Copy/paste first 10-20 lines when starting server
3. **Health Check**: Copy/paste response from health endpoint
4. **Any Errors**: Full error messages if anything fails

This will help me identify any remaining issues!
