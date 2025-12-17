# MADE Activity Tracker - Project Plan

> **M**etrics for **A**ctivity, **D**elivery & **E**fficiency

## Project Overview

A desktop application that tracks team productivity across GitHub repositories, providing metrics organized by **Speed**, **Ease**, and **Quality**.

---

## Decisions Log

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Desktop Framework | Tauri | Small bundle (~10MB), Rust backend |
| Frontend | React + TypeScript | Team familiarity, ecosystem |
| Styling | Tailwind CSS | Rapid UI development |
| State | Zustand | Simple, performant |
| Relational DB | SQLite | Offline, complex queries |
| Vector DB | LanceDB | Native hybrid search |
| Embeddings | FastEmbed-rs (local) | Offline, free, ~80MB model |
| GitHub API | GraphQL | Efficient data fetching |
| Auth | Device Flow | Best for desktop apps |

---

## Configuration Model

```yaml
# User provides:
repositories:
  - owner: "your-org"
    name: "repo-1"
    enabled: true
  - owner: "your-org"
    name: "repo-2"
    enabled: true

squads:
  - name: "Platform Team"
    members: ["alice", "bob"]
    color: "#3b82f6"
  - name: "Product Team"  
    members: ["charlie", "diana"]
    color: "#22c55e"

settings:
  history_days: 90          # Configurable, default 90
  excluded_bots:
    - "dependabot[bot]"
    - "renovate[bot]"
    - "github-actions[bot]"
  bug_labels: ["bug", "defect"]
  feature_labels: ["feature", "enhancement"]
```

### Terminology

- **Squads** = Team groupings (via labels + manual config)
- **Cycles** = Milestones (used for roadmap view)

---

## Metrics Framework

### Speed (How fast work completes)

| Metric | Calculation | Good Direction |
|--------|-------------|----------------|
| **Cycle Time** | Issue created → closed (business days) | ↓ Lower |
| **PR Lead Time** | PR opened → merged (hours) | ↓ Lower |
| **Throughput** | Items completed per week | ↑ Higher |

### Ease (How smooth the process is)

| Metric | Calculation | Good Direction |
|--------|-------------|----------------|
| **PR Size** | Additions + Deletions | ↓ Smaller |
| **Review Rounds** | Distinct review submissions per PR | ↓ Fewer |
| **Time to First Review** | PR opened → first review | ↓ Lower |
| **Rework Rate** | Changes after first approval | ↓ Lower |

### Quality (How good the output is)

| Metric | Calculation | Good Direction |
|--------|-------------|----------------|
| **Bug Rate** | Issues with bug labels / total issues | ↓ Lower |
| **Reopen Rate** | Issues reopened / issues closed | ↓ Lower |
| **PR Rejection Rate** | PRs closed without merge / total PRs | ↓ Lower |

---

## Data Architecture

### Dual Database Design

```
┌─────────────────────────────────────────────────────┐
│                   MADE Tracker                      │
├──────────────────────┬──────────────────────────────┤
│       SQLite         │         LanceDB              │
│  (Relational Data)   │    (Search & Vectors)        │
├──────────────────────┼──────────────────────────────┤
│ • Issues, PRs        │ • Issue/PR embeddings        │
│ • Users, Repos       │ • Duplicate detection        │
│ • Metrics history    │ • Semantic search            │
│ • Config, state      │ • AI context retrieval       │
│                      │                              │
│ Complex JOINs ✅      │ Hybrid search ✅              │
│ Aggregations ✅       │ Vector similarity ✅          │
│ Date range queries ✅ │ BM25 + cosine ranking ✅      │
└──────────────────────┴──────────────────────────────┘
```

### Local Embeddings (FastEmbed-rs)

```
┌─────────────────────────────────────────┐
│         Local Embedding Pipeline        │
├─────────────────────────────────────────┤
│  Model: all-MiniLM-L6-v2                │
│  Dimensions: 384                        │
│  Size: ~80MB (downloaded on first run)  │
│  Speed: ~50ms per document (CPU)        │
│  Quality: Good for semantic similarity  │
├─────────────────────────────────────────┤
│  First Run:                             │
│  1. Detect missing model                │
│  2. Download from Hugging Face          │
│  3. Cache in app data folder            │
│  4. Works offline forever               │
└─────────────────────────────────────────┘
```

---

## Development Phases

### Phase 1: Foundation ✅ **COMPLETE**
- [x] Project scaffold (Tauri + React)
- [x] SQLite schema & migrations
- [x] GitHub OAuth Device Flow
- [x] GraphQL sync engine
- [x] FastEmbed-rs integration
- [x] Config management
- [x] Basic React shell
- [x] Settings UI
- [x] Metrics calculation engine

### Phase 2: Dashboard ✅ **COMPLETE**
- [x] Metrics calculation engine
- [x] Speed/Ease/Quality cards
- [x] Date range filtering (7/30/90/180/365 days)
- [x] Repository multi-select filtering
- [x] Squad filtering
- [x] User filtering
- [x] Filter persistence
- [x] Recharts visualizations
- [x] Real-time chart updates
- [x] Comprehensive test coverage

### Phase 3: Intelligence (Next)
- [ ] LanceDB integration for vector storage
- [ ] Hybrid search (keyword + semantic)
- [ ] Duplicate detection
- [ ] Roadmap view (cycles/milestones)
- [ ] Duplicate warnings on new issues
- [ ] Historical trend calculations
- [ ] Export functionality (CSV, JSON)

### Phase 4: Team Features
- [ ] Squad management
- [ ] Per-person metrics
- [ ] Squad comparisons
- [ ] CSV/JSON export
- [ ] Historical snapshots

### Phase 5: AI Integration
- [ ] Local REST API server
- [ ] MCP server for AI tools
- [ ] Context retrieval endpoint
- [ ] Structured query interface

---

## File Structure

```
made-activity-tracker/
├── src/                          # React frontend
│   ├── components/
│   │   └── Layout.tsx
│   ├── pages/
│   │   ├── Dashboard.tsx
│   │   ├── Login.tsx
│   │   ├── Roadmap.tsx
│   │   ├── Search.tsx
│   │   └── Settings.tsx
│   ├── stores/
│   │   ├── authStore.ts
│   │   ├── configStore.ts
│   │   └── syncStore.ts
│   ├── App.tsx
│   ├── main.tsx
│   └── index.css
│
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── config/
│   │   │   ├── mod.rs
│   │   │   └── commands.rs
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── migrations.rs
│   │   │   ├── models.rs
│   │   │   └── queries.rs
│   │   ├── embeddings/
│   │   │   ├── mod.rs
│   │   │   └── generator.rs
│   │   ├── github/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs
│   │   │   ├── commands.rs
│   │   │   ├── graphql.rs
│   │   │   └── sync.rs
│   │   ├── metrics/
│   │   │   ├── mod.rs
│   │   │   ├── business_days.rs
│   │   │   ├── calculator.rs
│   │   │   └── commands.rs
│   │   ├── search/
│   │   │   ├── mod.rs
│   │   │   ├── commands.rs
│   │   │   ├── duplicates.rs
│   │   │   └── hybrid.rs
│   │   └── main.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── tests/
│   ├── rust/
│   │   ├── unit/
│   │   ├── integration/
│   │   └── fixtures/
│   ├── frontend/
│   │   ├── unit/
│   │   ├── integration/
│   │   └── mocks/
│   ├── e2e/
│   │   └── specs/
│   └── api/
│
├── package.json
├── tsconfig.json
├── tailwind.config.js
├── vite.config.ts
├── vitest.config.ts
├── PLAN.md
└── README.md
```

---

## Testing Strategy

### Coverage Targets

| Area | Target |
|------|--------|
| Metrics calculations | 100% |
| Business days logic | 100% |
| Database operations | 90% |
| UI components | 80% |
| E2E critical paths | 100% |

### Test Categories

- **Rust Unit Tests**: Core logic (metrics, business days, embeddings)
- **Rust Integration**: Full pipelines (sync, search)
- **Frontend Unit**: Components, hooks, stores
- **Frontend Integration**: User flows with mocked Tauri
- **E2E**: Critical paths with Playwright

---

## Current Status

**Phase 1**: ✅ Complete
**Phase 2**: ✅ Complete
**Phase 3**: 🚧 In Planning

## Next Steps

1. **LanceDB Integration** - Store embeddings in vector database
2. **Hybrid Search** - Combine keyword (SQLite FTS5) + semantic (LanceDB)
3. **Duplicate Detection** - Cosine similarity on embeddings
4. **Historical Trends** - Fetch 2x data period for period-over-period comparison
4. **Build out dashboard** - Connect metrics to UI
