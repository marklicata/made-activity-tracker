# MADE Activity Tracker

**Metrics for Activity, Delivery & Efficiency**

A desktop app for tracking GitHub team activity across multiple repositories with semantic search and productivity insights.

---

## 🚀 Project Status: Phase 1 Complete

✅ **What's Working:**
- ✅ GitHub OAuth Device Flow authentication
- ✅ SQLite database with full schema
- ✅ GitHub GraphQL sync (issues, PRs, milestones, reviews)
- ✅ FastEmbed local embeddings (all-MiniLM-L6-v2)
- ✅ Business days calculations
- ✅ Metrics engine (Speed, Ease, Quality)
- ✅ Settings UI for repo/squad configuration
- ✅ React frontend scaffold with routing

⏳ **Not Yet Implemented:**
- LanceDB integration for vector search
- Duplicate detection
- Full dashboard visualizations
- Squad-specific metrics filtering

---

## 📋 Prerequisites

1. **Node.js** 18+ and npm
2. **Rust** 1.75+ ([install via rustup](https://rustup.rs/))
3. **GitHub OAuth App**:
   - Go to: https://github.com/settings/developers
   - Click "New OAuth App"
   - Enable "Device Flow"
   - Copy the **Client ID**

---

## 🛠️ Setup Instructions

### 1. Install Dependencies

```bash
cd C:\Users\malicata\source\made-activity-tracker

# Install npm packages
npm install

# Rust dependencies will be installed on first build
```

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
- FastEmbed will download ~80MB model (one-time, cached locally)
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

### Daily Workflow

- **Dashboard**: View Speed, Ease, Quality metrics
- **Search**: Find issues/PRs (keyword search for now)
- **Roadmap**: See upcoming cycles and milestones
- **Refresh**: Click sync icon to update data

---

## 📁 Project Structure

```
made-activity-tracker/
├── src/                       # React frontend
│   ├── pages/                 # Main app pages
│   │   ├── Dashboard.tsx      # Metrics overview
│   │   ├── Search.tsx         # Issue/PR search
│   │   ├── Roadmap.tsx        # Cycles view
│   │   ├── Settings.tsx       # Config management
│   │   └── Login.tsx          # Auth flow
│   ├── components/            # Reusable components
│   ├── stores/                # Zustand state management
│   └── lib/                   # Utilities
│
├── src-tauri/                 # Rust backend
│   └── src/
│       ├── github/            # Auth + sync
│       ├── db/                # SQLite queries
│       ├── metrics/           # Calculations
│       ├── embeddings/        # FastEmbed integration
│       ├── search/            # Search (Phase 3)
│       └── config/            # App configuration
│
├── tests/                     # Test scaffolds
└── PLAN.md                    # Full project plan
```

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
# Run Rust tests
cd src-tauri
cargo test

# Run frontend tests
npm test

# Run E2E tests (Playwright)
npm run test:e2e

# Run with coverage
npm run test:coverage
```

---

## 🚧 Known Issues & Limitations

### Phase 1 Limitations

1. **Search**: Only basic keyword search (no semantic/vector search yet)
2. **Dashboard**: Placeholder charts, need real visualizations
3. **Embeddings**: Generated but not stored in vector DB yet
4. **Duplicate Detection**: Not implemented yet
5. **User Filtering**: Squad/user-specific metrics not working yet

### Workarounds

- **Sync Takes Long**: First sync caches everything, subsequent syncs are incremental
- **Model Download**: Happens automatically, but requires internet once
- **Rate Limits**: GitHub allows 5,000 API calls/hour — should be enough for 25 repos

---

## 🗺️ Roadmap

### Phase 2 (Next)
- LanceDB integration for vector storage
- Hybrid search (keyword + semantic)
- Duplicate detection with cosine similarity
- Enhanced dashboard charts (Recharts)
- User/squad filtering

### Phase 3
- Historical trends and snapshots
- Export functionality (CSV, JSON)
- Advanced roadmap visualizations
- Customizable metrics definitions

### Phase 4
- Local REST API for AI tool integration
- MCP (Model Context Protocol) server
- Webhooks for real-time updates

---

## 📝 Development Notes

### Adding a New Metric

1. Add calculation logic to `src-tauri/src/metrics/calculator.rs`
2. Add field to `DashboardMetrics` struct
3. Update dashboard UI in `src/pages/Dashboard.tsx`
4. Add test in `tests/rust/unit/metrics_test.rs`

### Adding a New Label Type

Update config in Settings UI or edit `config.json`:

```json
{
  "custom_labels": {
    "priority_high": ["urgent", "p1", "critical"],
    "tech_debt": ["debt", "refactor", "cleanup"]
  }
}
```

---

## 🤝 Contributing

See `PLAN.md` for the full technical specification and architecture.

---

## 📄 License

MIT

---

## 🆘 Troubleshooting

### "Failed to initialize database"
- Check file permissions in `%APPDATA%\made-activity-tracker`
- Delete `tracker.db` to reset

### "GitHub API rate limit exceeded"
- Wait 1 hour for reset, or configure repos to sync less frequently
- Check rate limit: https://github.com/settings/applications

### "FastEmbed model not found"
- Ensure internet connection for first download
- Model cached at: `%LOCALAPPDATA%\fastembed`
- Delete cache to re-download

### "Sync hangs or fails"
- Check GitHub token is valid: Settings → Login Status
- Verify repo names are correct: `owner/repo`
- Check app logs: `%APPDATA%\made-activity-tracker\logs`

---

## 📚 Resources

- [Tauri Docs](https://tauri.app/)
- [GitHub GraphQL API](https://docs.github.com/en/graphql)
- [FastEmbed](https://github.com/Anush008/fastembed-rs)
- [Project Plan](./PLAN.md)
