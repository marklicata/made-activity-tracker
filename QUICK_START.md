# Quick Start Guide

Get the MADE Activity Tracker up and running in minutes.

---

## Prerequisites

Before you start, ensure you have:

1. **Node.js 18+** and npm
2. **Rust 1.75+** ([install via rustup](https://rustup.rs/))
3. **Python 3.11+** (for AI Chat features)
4. **GitHub OAuth App** with Device Flow enabled
   - Create at: https://github.com/settings/developers
   - Copy your Client ID
5. **API Key** (for AI Chat):
   - Anthropic: https://console.anthropic.com/
   - OR OpenAI: https://platform.openai.com/api-keys

---

## Installation Steps

### 1. Clone and Install Dependencies

```bash
# Clone the repository (if you haven't already)
git clone https://github.com/marklicata/made-activity-tracker
cd made-activity-tracker

# Install npm packages
npm install
```

### 2. Setup Python Environment (for AI Chat)

```bash
cd src-tauri/amplifier-tools

# Create virtual environment
python -m venv .venv

# Activate virtual environment
# Windows:
.venv\Scripts\activate
# Mac/Linux:
source .venv/bin/activate

# Install Python dependencies
pip install -e .

# Verify entry points are registered
python -c "import pkg_resources; [print(ep) for ep in pkg_resources.iter_entry_points('amplifier.modules')]"
# Should show: made-activity-tools = made_activity_tools.activity_tracking_tools:mount

cd ../..
```

**Note**: The Python setup also runs automatically via the `setup:python` npm script, but manual setup ensures everything is configured correctly.

### 3. Configure GitHub OAuth

Edit `src-tauri/src/github/commands.rs`:

```rust
// Replace with your GitHub OAuth App Client ID
const GITHUB_CLIENT_ID: &str = "YOUR_CLIENT_ID_HERE";
```

### 4. Set Environment Variables

```bash
# Windows (PowerShell):
$env:ANTHROPIC_API_KEY = "your-key-here"
# OR
$env:OPENAI_API_KEY = "your-key-here"

# Mac/Linux:
export ANTHROPIC_API_KEY="your-key-here"
# OR
export OPENAI_API_KEY="your-key-here"
```

**Tip**: Add these to your shell profile (`.bashrc`, `.zshrc`, or PowerShell profile) to persist across sessions.

---

## Running the Application

### Development Mode

```bash
npm run tauri dev
```

**What happens:**
1. Frontend builds with Vite (~30 seconds)
2. Rust backend compiles (10-20 minutes on first build)
3. Python sidecar starts automatically
4. Application window opens

**First Build Notes:**
- Rust compilation includes 600+ crates - this is normal
- Subsequent builds are much faster (30-60 seconds)
- FastEmbed downloads ~80MB model on first semantic search use

### Alternative: Run Components Separately

```bash
# Terminal 1: Frontend only
npm run dev:frontend

# Terminal 2: Backend only
npm run dev:backend

# Terminal 3: Full Tauri app
npm run tauri dev
```

---

## First Time Setup (In App)

### 1. Login with GitHub

1. Click "Sign in with GitHub"
2. Browser opens with device code prompt
3. Enter the code shown in the app
4. Approve OAuth permissions
5. Return to app - you're logged in!

### 2. Add Repositories

1. Navigate to **Settings** → **Repositories**
2. Add repositories in `owner/repo` format:
   - Example: `facebook/react`
   - Example: `microsoft/vscode`
3. Enable/disable repos as needed

### 3. (Optional) Configure Squads

1. Go to **Settings** → **Squads**
2. Create team groups
3. Add GitHub usernames to each squad
4. Assign colors for visual identification

### 4. First Sync

1. Click "Sync Now" in the dashboard
2. Initial sync takes 2-5 minutes for ~25 repos
3. Watch the progress bar
4. Data is now available for analysis!

---

## Using Key Features

### Dashboard & Metrics

- **Amplifier View**: Toggle to see industry benchmark comparisons
- **DORA View**: Traditional DevOps metrics
- **Filter Panel**: Filter by date range, repositories, squads, users
- **Filters persist** across page navigation and app restarts

### AI Chat Assistant

1. Click the **chat icon** in the top navigation
2. Chat panel slides in from the right
3. Ask natural language questions:
   - "What's our average cycle time this month?"
   - "Show me bugs related to authentication"
   - "What has Alice been working on?"
4. AI uses three specialized tools:
   - `get_metrics`: Speed, ease, quality metrics
   - `search_github_items`: Search issues/PRs
   - `get_user_activity`: User activity summaries

**AI Chat Architecture:**
- Powered by Microsoft Amplifier Foundation
- Tools registered via Python package entry points
- Runs as Python sidecar process (Flask server)
- Supports Anthropic Claude and OpenAI GPT models

### Team Tracking

1. Navigate to **Team View**
2. Enter GitHub usernames to track
3. View aggregate metrics and user cards
4. Click any user for detailed analysis
5. Export reports to CSV

### Project Deep Dive

1. Go to **Projects** list
2. Click any repository
3. Explore:
   - Timeline of all activity
   - Contributor rankings
   - Activity heatmap
   - Lifecycle metrics

---

## Testing

```bash
# Run frontend tests
npm test

# Run tests with UI
npm run test:ui

# Run tests with coverage
npm run test:coverage

# Run E2E tests
npm run test:e2e

# Run Rust tests
cd src-tauri
cargo test
```

**Current Test Status:**
- 9 test files passed
- 235 tests passed
- 41 tests marked as todo (future work)
- Coverage includes: hooks, components, utilities, metrics

---

## Build for Production

```bash
# Build release version
npm run build:release

# Output location:
# Windows: src-tauri/target/release/made-activity-tracker.exe
# Mac: src-tauri/target/release/made-activity-tracker
# Linux: src-tauri/target/release/made-activity-tracker
```

**Release build takes longer** (30-60 minutes) but produces optimized binaries.

---

## Troubleshooting Quick Fixes

### Build Fails

**Check Rust version:**
```bash
rustc --version  # Should be 1.75+
rustup update stable
```

**Clean build artifacts:**
```bash
cd src-tauri
cargo clean
cd ..
npm run tauri dev
```

### AI Chat Not Working

**Verify Python setup:**
```bash
cd src-tauri/amplifier-tools
python --version  # Should be 3.11+
.venv\Scripts\activate  # Windows
source .venv/bin/activate  # Mac/Linux
pip list | grep amplifier  # Should show 4 amplifier packages
```

**Check API key:**
```bash
# Windows:
echo %ANTHROPIC_API_KEY%
# Mac/Linux:
echo $ANTHROPIC_API_KEY
```

**View sidecar logs:**
Open app → Press F12 → Check console for Python sidecar output

### GitHub Sync Issues

**Rate limit exceeded:**
- Install GitHub CLI: https://cli.github.com
- Run: `gh auth login`
- App will automatically use CLI for higher limits

**SAML authentication required:**
- Install and authenticate with GitHub CLI
- App falls back to CLI automatically

### Port Conflicts

If AI Chat shows port errors:
```bash
# Windows:
netstat -ano | findstr :5000
# Mac/Linux:
lsof -i :5000
```

Kill the process using port 5000 or change `AMPLIFIER_PORT` in environment variables.

---

## Performance Tips

### Speed Up Development

1. **Use debug builds** during development (default):
   ```bash
   npm run tauri dev  # Debug mode, faster compile
   ```

2. **Hot reload**: TypeScript changes reload instantly without Rust rebuild

3. **Incremental compilation**: Rust only recompiles changed code after first build

4. **Parallel jobs** (8+ CPU cores):
   ```bash
   cd src-tauri
   cargo build -j 8
   ```

### Reduce Build Time

**Disable embeddings temporarily** if you don't need semantic search:

1. Comment out in `src-tauri/Cargo.toml`:
   ```toml
   # fastembed = { version = "4", default-features = false }
   ```

2. Build time drops from 15-20 minutes to ~5 minutes

---

## Next Steps

After successfully running the app:

1. **Read the full README**: Detailed feature documentation
2. **Check `specs/` folder**: Architecture and design docs
3. **Review `TROUBLESHOOTING.md`**: Platform-specific issues
4. **Explore the codebase**: See `src/` for frontend, `src-tauri/src/` for backend
5. **Run tests**: Ensure everything works in your environment

---

## Getting Help

- **Build issues**: See `TROUBLESHOOTING.md`
- **Architecture questions**: Check `specs/` folder
- **Test failures**: Run with verbose logging: `RUST_LOG=debug npm run tauri dev`
- **Feature requests**: Check GitHub issues or create a new one

---

## Summary: From Zero to Running

```bash
# 1. Install dependencies
npm install

# 2. Setup Python for AI Chat
cd src-tauri/amplifier-tools
python -m venv .venv
.venv\Scripts\activate  # or: source .venv/bin/activate
pip install -e .
cd ../..

# 3. Configure OAuth (edit src-tauri/src/github/commands.rs)

# 4. Set API key
export ANTHROPIC_API_KEY="your-key"

# 5. Run!
npm run tauri dev
```

First build: 10-20 minutes
Subsequent builds: 30-60 seconds
Ready to track GitHub activity!
