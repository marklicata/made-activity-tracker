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

## Running Tests

### Rust Tests
```bash
cd frontend/src-tauri
cargo test
```

### Frontend Tests
```bash
cd frontend
npm test          # Unit tests with Vitest
npm run test:e2e  # E2E tests with Playwright
```

### Coverage
```bash
# Rust coverage
cargo tarpaulin --out Html

# Frontend coverage
npm run test:coverage
```

## Coverage Targets

| Area | Target |
|------|--------|
| Metrics calculations | 100% |
| Business days logic | 100% |
| Database operations | 90% |
| UI components | 80% |
| E2E critical paths | 100% |
