# Testing Instructions

Since bash commands aren't working from the assistant, please run these tests manually in your terminal.

## Step 1: Open Terminal

Open PowerShell or Command Prompt and navigate to the project:

```powershell
cd C:\Users\malicata\source\made-activity-tracker\src-tauri\amplifier-tools
```

## Step 2: Activate Virtual Environment

```powershell
# PowerShell
.venv\Scripts\Activate.ps1

# Or CMD
.venv\Scripts\activate.bat
```

## Step 3: Set Environment Variables

```powershell
# PowerShell - set your actual API key
$env:ANTHROPIC_API_KEY = "sk-ant-your-actual-key-here"

# Optional: Set database path if not in default location
$env:DATABASE_PATH = "C:\path\to\your\activity.db"
```

Or for CMD:
```cmd
set ANTHROPIC_API_KEY=sk-ant-your-actual-key-here
set DATABASE_PATH=C:\path\to\your\activity.db
```

## Step 4: Run Comprehensive Test Suite

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

## Step 5: Start the Server

If all tests pass, start the server:

```bash
python src/made_activity_tools/server.py
```

Expected output:
```
Starting Amplifier server on port 5000
Provider: anthropic
Database: C:\path\to\activity.db
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

## Step 6: Test Health Endpoint

Open a **new terminal** (keep the server running) and test:

```powershell
# PowerShell
curl http://127.0.0.1:5000/health

# Or using Invoke-WebRequest
Invoke-WebRequest -Uri http://127.0.0.1:5000/health | Select-Object -Expand Content
```

Expected response:
```json
{
  "status": "ok",
  "provider": "anthropic",
  "has_api_key": true,
  "db_path": "C:\\path\\to\\activity.db"
}
```

## Step 7: Test Chat Endpoint

```powershell
# PowerShell
$body = @{
    message = "What metrics are available?"
} | ConvertTo-Json

Invoke-WebRequest `
  -Uri http://127.0.0.1:5000/chat `
  -Method POST `
  -Headers @{"X-Auth-Token"="dev-token"; "Content-Type"="application/json"} `
  -Body $body | Select-Object -Expand Content
```

Or using curl:
```bash
curl -X POST http://127.0.0.1:5000/chat ^
  -H "Content-Type: application/json" ^
  -H "X-Auth-Token: dev-token" ^
  -d "{\"message\": \"What metrics are available?\"}"
```

Expected: You should get a JSON response with an AI-generated answer about the available metrics tools.

## Troubleshooting

### Test Failures

**Import errors:**
```bash
pip install -e .
```

**Bundle loading fails:**
- Check that `bundle.md` exists
- Verify YAML frontmatter is valid
- Check that `src/made_activity_tools/tool_module.py` exists

**API key not set:**
```powershell
$env:ANTHROPIC_API_KEY = "your-key-here"
```

**Database not found:**
- Set `DATABASE_PATH` environment variable
- Or ensure database is at default location

### Server Issues

**Port already in use:**
```powershell
$env:AMPLIFIER_PORT = 5001  # Use different port
python src/made_activity_tools/server.py
```

**Module download hangs:**
- Check internet connection
- Try deleting cache: `rm -r ~/.amplifier/cache` and restart

**Import errors at runtime:**
- Ensure venv is activated
- Run `pip install -e .` again

## What to Report Back

After running the tests, please share:

1. **Test Results**: Copy/paste output from `python run_tests.py`
2. **Server Startup**: Copy/paste first 10-20 lines when starting server
3. **Health Check**: Copy/paste response from health endpoint
4. **Any Errors**: Full error messages if anything fails

This will help me identify any remaining issues!
