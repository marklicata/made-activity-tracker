# MADE Activity Tracker - Test Suite

This directory contains all tests for the MADE Activity Tracker application.

## Structure

```
tests/
├── rust/                    # Backend (Tauri/Rust) tests
│   ├── unit/               # Unit tests for individual modules
│   │   ├── metrics/        # Metrics calculation tests
│   │   ├── db/             # Database operation tests
│   │   ├── github/         # GitHub API client tests
│   │   ├── embeddings/     # Embedding generation tests
│   │   └── search/         # Search functionality tests
│   ├── integration/        # Integration tests
│   │   ├── sync_pipeline_test.rs
│   │   ├── metrics_pipeline_test.rs
│   │   ├── search_pipeline_test.rs
│   │   └── auth_flow_test.rs
│   └── fixtures/           # Test data and mocks
│       ├── github_responses/
│       └── seed_data/
├── frontend/               # React frontend tests
│   ├── unit/              # Component and hook tests
│   │   ├── components/
│   │   ├── hooks/
│   │   └── stores/
│   ├── integration/       # Frontend integration tests
│   └── mocks/             # Tauri API mocks
├── e2e/                   # End-to-end tests (Playwright)
│   └── specs/
│       ├── auth.spec.ts
│       ├── sync.spec.ts
│       ├── dashboard.spec.ts
│       ├── search.spec.ts
│       ├── duplicates.spec.ts
│       ├── roadmap.spec.ts
│       ├── settings.spec.ts
│       ├── filters.spec.ts
│       └── export.spec.ts
└── api/                   # API tests (Phase 5)
    ├── rest/
    └── mcp/
```

## Setup

Before running tests, ensure dependencies are installed:

```bash
# From project root
npm install

# Install Playwright browsers for E2E tests
npx playwright install

# Install Rust toolchain if not already installed
# Visit https://rustup.rs/
```

## Running Tests

### Frontend Unit Tests
```bash
# From project root
npm test          # Run all unit tests with Vitest
npm run test:ui   # Run with interactive UI
```

### E2E Tests
```bash
# From project root
npm run test:e2e  # E2E tests with Playwright
```

### Rust Tests
```bash
# From project root
cd src-tauri
cargo test

# Or from tests/ directory
cd ../src-tauri
cargo test
```

### Coverage

#### Frontend Coverage
```bash
# From project root
npm run test:coverage
```

#### Rust Coverage
```bash
# From src-tauri directory
# First install tarpaulin: cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Coverage Targets

| Area | Target |
|------|--------|
| Metrics calculations | 100% |
| Business days logic | 100% |
| Database operations | 90% |
| UI components | 80% |
| E2E critical paths | 100% |
