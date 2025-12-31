# MADE Activity Tracker

**Metrics for Activity, Delivery & Efficiency**

A desktop application for tracking GitHub team activity across multiple repositories with semantic search, productivity insights, and team analytics.

---

## 🚀 Features

### ✅ Core Platform
- **GitHub Integration**: OAuth authentication with automatic token refresh
- **Multi-Repository Tracking**: Monitor unlimited GitHub repositories
- **SQLite Database**: Fast local storage with full-text search
- **FastEmbed Embeddings**: Local semantic embeddings (all-MiniLM-L6-v2)
- **GitHub CLI Fallback**: Automatic fallback for SAML-protected repositories

### ✅ Dashboard & Metrics
- **Speed Metrics**: Cycle time, PR lead time, throughput trends
- **Ease Metrics**: PR size, review rounds, time to first review, rework rate
- **Quality Metrics**: Bug rate, reopen rate, PR rejection rate
- **Interactive Charts**: Recharts visualizations with real-time updates
- **Advanced Filtering**: Date ranges, repositories, squads, users
- **Filter Persistence**: Saved preferences across sessions

### ✅ Project Deep Dive
- **Timeline View**: Chronological activity across issues, PRs, and reviews
- **Contributor Analysis**: Top contributors with activity breakdown
- **Activity Heatmap**: Visual representation of project activity patterns
- **Lifecycle Metrics**: Project-specific speed, ease, and quality metrics
- **Project Summary**: Overview cards with key statistics

### ✅ Amplifier Metrics Dashboard (NEW!)
- **PR-Based Metrics**: Industry-standard productivity metrics with benchmarks
- **Productivity Multiplier**: Overall team performance score vs industry average
- **Speed Metrics**: PRs per day, turnaround time, cycle time distribution
- **Ease Metrics**: Concurrent projects, context switching, work patterns
- **Quality Metrics**: PR merge rate, bug ratio, feature work percentage
- **Benchmark Comparisons**: Industry and elite performer comparisons
- **Interactive Visualizations**: Distribution charts and trend analysis

### ✅ AI Chat Panel
- **Natural Language Queries**: Ask questions about your GitHub activity
- **Context-Aware**: Understands current page and filter state
- **Three Custom Tools**:
  - `get_metrics`: Query speed, ease, and quality metrics
  - `search_github_items`: Search issues and PRs by text
  - `get_user_activity`: Get user activity summaries
- **Amplifier Foundation**: Powered by Microsoft Amplifier with bundle composition
- **Entry Point Architecture**: Tools registered via Python package entry points
- **Persistent Chat**: Conversation history saved across sessions
- **Provider Support**: Works with Anthropic Claude or OpenAI GPT models
- **Smart Responses**: Provides data-driven insights and recommendations

### ✅ User-Centric View
- **User Tracking**: Monitor specific team members across all repositories
- **Activity Dashboard**: Individual user cards with status indicators
- **Repository Distribution**: See where each user focuses their work
- **Collaboration Matrix**: Visualize PR review relationships between team members
- **Activity Trends**: Line charts showing velocity changes over time
- **Focus Analysis**: Repository concentration with HHI score
- **Team Summary**: Aggregate metrics across all tracked users
- **CSV Export**: Export team and individual user reports
- **Date Range Filtering**: Flexible time period selection with presets

### ✅ Search & Discovery
- **Hybrid Search**: Keyword and semantic search across issues and PRs
- **Duplicate Detection**: Find similar issues using cosine similarity
- **Smart Ranking**: Results sorted by relevance score

### ✅ Roadmap Planning
- **Milestone Tracking**: Visualize upcoming cycles and deliverables
- **Progress Monitoring**: Track completion rates and trends

---

## 📋 Prerequisites

1. **Node.js** 18+ and npm
2. **Rust** 1.75+ ([install via rustup](https://rustup.rs/))
3. **GitHub OAuth App**:
   - Go to: https://github.com/settings/developers
   - Click "New OAuth App"
   - Enable "Device Flow"
   - Copy the **Client ID**
4. **GitHub CLI** (optional): For SAML-protected repositories
   - Install from: https://cli.github.com
   - Run: `gh auth login`

5. **Python 3.11+** (for AI Chat Panel):
   - Install Python from: https://www.python.org/downloads/
   - Verify installation: `python --version`
   - Required for Amplifier Foundation integration

6. **Anthropic or OpenAI API Key** (for AI Chat Panel):
   - Get Anthropic key: https://console.anthropic.com/
   - Or OpenAI key: https://platform.openai.com/api-keys
   - Set environment variable: `ANTHROPIC_API_KEY` or `OPENAI_API_KEY`
   - Required for AI-powered natural language queries

---

## 🛠️ Setup Instructions

### 1. Install Dependencies

```bash
# Install npm packages
npm install

# Rust dependencies installed automatically on first build

# Python dependencies for AI Chat (optional but recommended)
cd src-tauri/amplifier-tools
python -m venv .venv
# Windows
.venv\Scripts\activate
# Mac/Linux
source .venv/bin/activate
pip install -e .
cd ../..
```

**Note**: Python setup runs automatically with `npm run tauri dev`, but manual setup ensures faster startup.

### 2. Configure GitHub OAuth

Edit `src-tauri/src/github/commands.rs`:

```rust
// Replace with your GitHub OAuth App Client ID
const GITHUB_CLIENT_ID: &str = "YOUR_CLIENT_ID_HERE";
```

### 3. Run the App

```bash
npm run tauri dev
```

**First Run:**
- FastEmbed downloads ~80MB model (one-time, cached locally)
- Takes ~30 seconds to initialize

---

## 📖 Usage Guide

### Initial Setup

1. **Login with GitHub**
   - Click "Sign in with GitHub"
   - Browser opens with device code
   - Enter the code shown in the app
   - Approve access

2. **Configure Repositories**
   - Go to Settings → Repositories
   - Add repos: `owner/repo-name` format
   - Example: `facebook/react`, `microsoft/vscode`
   - Enable/disable repos as needed

3. **Configure Squads** (optional)
   - Go to Settings → Squads
   - Create squad groups for your teams
   - Add GitHub usernames to each squad

4. **First Sync**
   - Click "Sync Now" in the app
   - Initial sync takes ~2-5 minutes for 25 repos
   - Progress bar shows status

### Dashboard Views

1. **Amplifier Metrics** (recommended)
   - Toggle to "Amplifier" view in the dashboard
   - See industry benchmark comparisons
   - Track productivity multiplier
   - Monitor Speed, Ease, and Quality metrics

2. **DORA Metrics**
   - Toggle to "DORA" view for traditional DevOps metrics
   - Cycle time, lead time, throughput

### AI Chat Assistant

1. **Open Chat Panel**
   - Click the chat icon in the top navigation
   - Panel slides in from the right side

2. **Ask Natural Language Questions**
   - "What's our average cycle time this month?"
   - "Show me metrics for the backend team"
   - "Find bugs about authentication"
   - "What has Alice been working on?"

3. **Context-Aware Queries**
   - Chat understands your current filters
   - References the page you're viewing
   - Provides data-driven insights

### Team Tracking

1. **Add Team Members**
   - Navigate to Team View
   - Enter GitHub usernames to track
   - View aggregate team metrics

2. **Analyze Individual Contributors**
   - Click on any user card for detailed view
   - See activity trends, focus analysis, and repository distribution
   - Export reports to CSV

3. **Monitor Collaboration**
   - View collaboration matrix to see PR review patterns
   - Identify bottlenecks and collaboration opportunities

### Project Analysis

1. **Select a Project**
   - Go to Projects list
   - Click on any repository

2. **Explore Project Insights**
   - View timeline of all activity
   - See contributor rankings
   - Analyze activity heatmap
   - Review lifecycle metrics

---

## 📁 Project Structure

```
made-activity-tracker/
├── src/                          # React frontend
│   ├── pages/
│   │   ├── Dashboard.tsx         # Metrics overview
│   │   ├── TeamView.tsx          # Team tracking dashboard
│   │   ├── UserDetail.tsx        # Individual user analysis
│   │   ├── ProjectsList.tsx     # Repository list
│   │   ├── ProjectDetail.tsx    # Project deep dive
│   │   ├── Search.tsx            # Hybrid search
│   │   ├── Roadmap.tsx           # Milestones view
│   │   └── Settings.tsx          # Configuration
│   ├── components/
│   │   ├── team/                 # User tracking components
│   │   ├── project/              # Project analysis components
│   │   └── common/               # Shared components
│   └── utils/                    # Utilities and helpers
│
├── src-tauri/                    # Rust backend
│   ├── amplifier-tools/          # Python tools for AI chat
│   │   ├── src/made_activity_tools/
│   │   │   ├── activity_tracking_tools.py  # Tool implementations
│   │   │   ├── db_connection.py            # Database utilities
│   │   │   └── server.py                   # Flask HTTP server
│   │   ├── bundle.md             # Amplifier bundle configuration
│   │   ├── providers/            # LLM provider configs
│   │   └── pyproject.toml        # Python package with entry points
│   └── src/
│       ├── ai/                   # AI chat panel integration (Rust)
│       │   ├── sidecar.rs        # Python sidecar process manager
│       │   └── commands.rs       # Tauri commands for chat
│       ├── github/               # Auth, sync, CLI fallback
│       ├── db/                   # SQLite queries and models
│       ├── metrics/              # Calculations
│       ├── embeddings/           # FastEmbed integration
│       ├── search/               # Hybrid search engine
│       ├── project/              # Project analytics
│       ├── team/                 # User tracking
│       └── config/               # Configuration
│
├── specs/                        # Planning & reference docs
│   ├── PLAN.md                   # Project plan and phases
│   ├── TROUBLESHOOTING.md        # Issue resolution guide
│   ├── COMPREHENSIVE_TESTING_PLAN.md
│   └── dashboard_metrics_data_analysis.md
│
└── tests/                        # Test suites
```

---

## 📊 Metrics Explained

### Speed (How fast work completes)

| Metric | Description | Target |
|--------|-------------|--------|
| Avg Cycle Time | Business days from issue open → close | < 5 days |
| Avg PR Lead Time | Business hours from PR open → merge | < 24 hours |
| Throughput | Issues/PRs completed per week | Increasing |

### Ease (How smooth the process is)

| Metric | Description | Target |
|--------|-------------|--------|
| Avg PR Size | Lines changed per PR | < 300 lines |
| Avg Review Rounds | Review iterations per PR | < 2 rounds |
| Time to First Review | Hours until first review | < 4 hours |
| Rework Rate | PRs with extensive changes | < 20% |

### Quality (How good the output is)

| Metric | Description | Target |
|--------|-------------|--------|
| Bug Rate | % of issues that are bugs | < 15% |
| Reopen Rate | % of issues reopened | < 5% |
| PR Rejection Rate | % of PRs closed without merge | < 10% |

---

## 🧪 Testing

```bash
# Run frontend tests (Vitest)
npm test

# Run tests with UI
npm run test:ui

# Run tests with coverage
npm run test:coverage

# Run E2E tests (Playwright)
npm run test:e2e

# Run Rust tests
cd src-tauri
cargo test
```

**Current Test Coverage** (as of latest build):
- **Test Files**: 9 passed, 3 skipped
- **Tests**: 235 passed, 41 todo
- Coverage includes: hooks, components, utilities, metrics calculations

---

## 🔧 Configuration

Config file location: `%APPDATA%\made-activity-tracker\config.json`

```json
{
  "repositories": [
    {
      "owner": "facebook",
      "name": "react",
      "enabled": true
    }
  ],
  "squads": [
    {
      "id": "frontend",
      "name": "Frontend Squad",
      "members": ["johndoe", "janedoe"],
      "color": "#3b82f6"
    }
  ],
  "excluded_bots": [
    "dependabot[bot]",
    "renovate[bot]"
  ],
  "excluded_labels": [
    "duplicate",
    "invalid"
  ],
  "bug_labels": [
    "bug",
    "defect"
  ],
  "history_days": 90
}
```

---

## 🗺️ Roadmap

### Completed Features ✅
- Core platform with GitHub integration
- Dashboard with advanced filtering and Amplifier-style metrics
- AI-powered chat panel for natural language queries
- Project Deep Dive analytics
- User-Centric View with team tracking
- Hybrid search with semantic similarity
- GitHub CLI fallback for SAML repos

### Upcoming Features
- Webhooks for real-time updates
- Advanced trend analysis with forecasting
- Custom metric definitions
- Export to PDF reports
- API integration for external tools
- Team performance insights and recommendations

---

## 🆘 Troubleshooting

### "Failed to initialize database"
- Check file permissions in `%APPDATA%\made-activity-tracker`
- Delete `tracker.db` to reset

### "GitHub API rate limit exceeded"
- Wait 1 hour for reset
- Install GitHub CLI (`gh`) for higher limits
- Check rate limit: https://github.com/settings/applications

### "SAML authentication required"
- Install GitHub CLI: https://cli.github.com
- Run: `gh auth login`
- App will automatically use CLI for SAML repos

### "FastEmbed model not found"
- Ensure internet connection for first download
- Model cached at: `%LOCALAPPDATA%\fastembed`

### "Sync hangs or fails"
- Check GitHub token is valid: Settings → Login Status
- Verify repo names: `owner/repo` format
- Check logs: `%APPDATA%\made-activity-tracker\logs`

### "AI Chat not working"
- **Ensure Python 3.11+ is installed**: `python --version`
- **Check API key is set**:
  - Windows: `echo %ANTHROPIC_API_KEY%` or `echo %OPENAI_API_KEY%`
  - Mac/Linux: `echo $ANTHROPIC_API_KEY` or `echo $OPENAI_API_KEY`
- **Install Python dependencies**:
  ```bash
  cd src-tauri/amplifier-tools
  python -m venv .venv
  .venv\Scripts\activate  # Windows
  source .venv/bin/activate  # Mac/Linux
  pip install -e .
  ```
- **Verify entry points are registered**:
  ```bash
  python -c "import pkg_resources; [print(ep) for ep in pkg_resources.iter_entry_points('amplifier.modules')]"
  ```
  Should show: `made-activity-tools = made_activity_tools.activity_tracking_tools:mount`
- **Check Python sidecar logs** in the app developer console (F12)
- **Note**: First chat request may take 10-15 seconds as Amplifier Foundation loads bundles from GitHub
- **Common issues**:
  - Database path not found: Ensure app has synced at least once
  - Module import errors: Reinstall with `pip install -e .`
  - Port conflicts: Check if port 5000 is available

---

## 📚 Documentation

- [Quick Start Guide](./QUICK_START.md) - Get up and running quickly
- [Build Environment](./BUILD_ENVIRONMENT.md) - Development setup
- [Dev Workflow](./DEV_WORKFLOW.md) - Contributing guidelines
- [Feature Specs](./specs/) - Detailed feature documentation

---

## 📄 License

MIT

---

## 🤝 Contributing

Contributions are welcome! See feature specs in `./specs/` for planned features and architecture details.
