# Test Coverage Improvement Plan

## Executive Summary

This document outlines a comprehensive plan to improve test coverage across the MADE Activity Tracker project. Current coverage is approximately **1.7% for frontend** and **0% for backend**, far below project targets (80-100% depending on area).

## Current State Analysis

### Coverage Status
- **Frontend Unit Tests:** 1/59 files (~1.7%)
- **Backend Unit Tests:** 0/38 files (~0%)
- **Integration Tests:** 0% implemented
- **E2E Tests:** 0% implemented (all skipped)

### Testing Infrastructure
- ✅ Vitest configured for frontend unit tests
- ✅ Playwright configured for E2E tests
- ✅ Tauri API mocks in place
- ✅ Test fixtures for GitHub responses
- ✅ Coverage reporting enabled

## Testing Strategy

### Phase 1: Critical Business Logic (Priority: HIGHEST)
Focus on core functionality that impacts data integrity and business metrics.

### Phase 2: Integration & Data Flow
Test end-to-end data pipelines and integrations.

### Phase 3: User Interface
Expand component and page-level testing.

### Phase 4: E2E User Workflows
Test complete user journeys through the application.

---

## Phase 1: Critical Business Logic Testing

### 1.1 Metrics Calculations (Target: 100%)
**Priority:** P0 - CRITICAL

**Files to Test:**
- `src-tauri/src/metrics/commands.rs`
- `src-tauri/src/metrics/mod.rs`
- `src-tauri/src/db/metrics_queries.rs`

**Test Coverage:**
```rust
// Test file: tests/rust/unit/metrics_test.rs

#[test_suite]
mod metrics_calculations {
    // Core metrics
    - test_calculate_pr_cycle_time()
    - test_calculate_pr_cycle_time_with_weekend()
    - test_calculate_pr_cycle_time_with_holidays()
    - test_calculate_coding_time()
    - test_calculate_pickup_time()
    - test_calculate_review_time()
    - test_calculate_merge_time()

    // Edge cases
    - test_metrics_with_missing_timestamps()
    - test_metrics_with_future_dates()
    - test_metrics_with_invalid_state_transitions()
    - test_metrics_with_draft_pr()
    - test_metrics_with_force_push()

    // Aggregations
    - test_user_daily_metrics()
    - test_user_weekly_metrics()
    - test_user_monthly_metrics()
    - test_squad_aggregated_metrics()
    - test_project_aggregated_metrics()

    // Benchmarks
    - test_amplifier_benchmark_calculations()
    - test_industry_benchmark_comparisons()
    - test_percentile_calculations()
}
```

**Success Criteria:**
- 100% line coverage for metrics calculation functions
- All edge cases handled
- Performance benchmarks included
- Documentation for calculation formulas

---

### 1.2 Business Days Logic (Target: 100%)
**Priority:** P0 - CRITICAL

**Files to Test:**
- `src-tauri/src/utils/business_days.rs`

**Test Coverage:**
```rust
// Test file: tests/rust/unit/business_days_test.rs

#[test_suite]
mod business_days {
    // Basic calculations
    - test_same_day_business_hours()
    - test_single_business_day()
    - test_multiple_business_days()
    - test_full_week_calculation()
    - test_multiple_weeks()

    // Weekend handling
    - test_weekend_start_date()
    - test_weekend_end_date()
    - test_span_includes_weekend()
    - test_friday_to_monday()

    // Holiday handling
    - test_us_federal_holidays()
    - test_custom_holiday_list()
    - test_holiday_on_weekend()
    - test_consecutive_holidays()

    // Edge cases
    - test_zero_duration()
    - test_negative_duration_error()
    - test_leap_year()
    - test_year_boundary()
    - test_dst_transitions()

    // Business hours
    - test_before_business_hours()
    - test_after_business_hours()
    - test_partial_day_calculation()
    - test_custom_business_hours()
}
```

**Success Criteria:**
- 100% line coverage
- All US federal holidays tested
- Timezone edge cases covered
- Performance: <1ms for typical date ranges

---

### 1.3 Database Operations (Target: 90%)
**Priority:** P0 - CRITICAL

**Files to Test:**
- `src-tauri/src/db/queries.rs`
- `src-tauri/src/db/user_queries.rs`
- `src-tauri/src/db/project_queries.rs`
- `src-tauri/src/db/metrics_queries.rs`
- `src-tauri/src/db/migrations.rs`

**Test Coverage:**
```rust
// Test file: tests/rust/unit/db_test.rs

#[test_suite]
mod database_operations {
    // CRUD operations
    - test_create_user()
    - test_read_user()
    - test_update_user()
    - test_delete_user()
    - test_create_duplicate_user_error()

    - test_create_project()
    - test_read_project()
    - test_update_project()
    - test_delete_project()

    - test_create_activity()
    - test_read_activities_by_user()
    - test_read_activities_by_date_range()
    - test_update_activity()
    - test_delete_activity()

    // Relationships
    - test_user_activities_relationship()
    - test_project_members_relationship()
    - test_squad_users_relationship()
    - test_cascade_delete_user()
    - test_orphaned_activity_handling()

    // Queries
    - test_filter_by_date_range()
    - test_filter_by_squad()
    - test_filter_by_repository()
    - test_combined_filters()
    - test_pagination()
    - test_sorting()

    // Transactions
    - test_transaction_commit()
    - test_transaction_rollback()
    - test_concurrent_transactions()

    // Migrations
    - test_migration_up()
    - test_migration_down()
    - test_migration_idempotency()
    - test_schema_version_tracking()
}

#[test_suite]
mod metrics_queries {
    - test_get_user_daily_metrics()
    - test_get_user_weekly_metrics()
    - test_get_squad_metrics()
    - test_get_project_metrics()
    - test_get_amplifier_metrics()
    - test_metrics_with_filters()
    - test_metrics_performance_large_dataset()
}
```

**Success Criteria:**
- 90% line coverage
- All CRUD operations tested
- Transaction integrity verified
- Migration rollback safety confirmed
- Performance benchmarks for common queries

---

### 1.4 GitHub Sync Operations (Target: 85%)
**Priority:** P0 - CRITICAL

**Files to Test:**
- `src-tauri/src/github/sync.rs`
- `src-tauri/src/github/sync_user.rs`
- `src-tauri/src/github/mod.rs`

**Test Coverage:**
```rust
// Test file: tests/rust/unit/github_client_test.rs

#[test_suite]
mod github_sync {
    // API calls
    - test_fetch_user_data()
    - test_fetch_pull_requests()
    - test_fetch_commits()
    - test_fetch_reviews()
    - test_handle_rate_limit()
    - test_handle_api_error()
    - test_retry_on_network_failure()

    // Data parsing
    - test_parse_pr_response()
    - test_parse_commit_response()
    - test_parse_review_response()
    - test_parse_incomplete_data()
    - test_parse_malformed_json()

    // Sync logic
    - test_incremental_sync()
    - test_full_sync()
    - test_sync_new_user()
    - test_sync_existing_user()
    - test_sync_multiple_repositories()
    - test_sync_deduplication()

    // State management
    - test_sync_progress_tracking()
    - test_sync_cancellation()
    - test_sync_resume_after_failure()
    - test_last_sync_timestamp()
}
```

**Success Criteria:**
- 85% line coverage
- Mock GitHub API responses
- Rate limit handling verified
- Error recovery tested
- Sync idempotency confirmed

---

## Phase 2: Integration & Data Flow Testing

### 2.1 Full Sync Pipeline (Target: 90%)
**Priority:** P1 - HIGH

**Test Coverage:**
```rust
// Test file: tests/rust/integration/test_full_sync.rs

#[test_suite]
mod full_sync_integration {
    // End-to-end sync
    - test_sync_user_complete_workflow()
    - test_sync_multiple_users()
    - test_sync_with_existing_data()
    - test_sync_data_integrity()

    // Error scenarios
    - test_sync_partial_failure_recovery()
    - test_sync_database_error_handling()
    - test_sync_network_interruption()

    // Performance
    - test_sync_performance_100_prs()
    - test_sync_performance_1000_commits()
    - test_concurrent_user_syncs()
}
```

---

### 2.2 Metrics Pipeline (Target: 90%)
**Priority:** P1 - HIGH

**Test Coverage:**
```rust
// Test file: tests/rust/integration/test_metrics_pipeline.rs

#[test_suite]
mod metrics_pipeline_integration {
    - test_raw_data_to_metrics_complete_flow()
    - test_metrics_aggregation_pipeline()
    - test_metrics_refresh_on_new_data()
    - test_metrics_cache_invalidation()
    - test_benchmark_comparison_pipeline()
}
```

---

### 2.3 Search & Embeddings (Target: 80%)
**Priority:** P1 - HIGH

**Files to Test:**
- `src-tauri/src/search/mod.rs`
- `src-tauri/src/embeddings/mod.rs`

**Test Coverage:**
```rust
// Test file: tests/rust/unit/search_test.rs

#[test_suite]
mod search_functionality {
    // Search operations
    - test_search_by_keyword()
    - test_search_by_date_range()
    - test_search_by_user()
    - test_search_by_project()
    - test_combined_search_filters()
    - test_search_pagination()
    - test_search_ranking()

    // Semantic search
    - test_semantic_search_similar_prs()
    - test_semantic_search_relevance()
    - test_embedding_generation()
    - test_embedding_similarity_calculation()
}
```

---

## Phase 3: User Interface Testing

### 3.1 Core Components (Target: 80%)
**Priority:** P2 - MEDIUM

**Components to Test:**

#### Dashboard Components
```typescript
// tests/frontend/unit/components/Dashboard.test.tsx
describe('Dashboard', () => {
  // Rendering
  - 'renders without crashing'
  - 'displays loading state'
  - 'displays error state'
  - 'renders with data'

  // Data fetching
  - 'fetches dashboard data on mount'
  - 'refetches on date range change'
  - 'handles fetch errors gracefully'

  // Interactions
  - 'updates filters'
  - 'triggers sync action'
  - 'navigates to detail views'
})
```

#### Metrics Components
```typescript
// tests/frontend/unit/components/metrics/ProductivityOverview.test.tsx
describe('ProductivityOverview', () => {
  - 'renders all metric cards'
  - 'displays correct metric values'
  - 'shows benchmark comparisons'
  - 'handles missing data'
  - 'formats numbers correctly'
  - 'displays trend indicators'
})

// tests/frontend/unit/components/metrics/CycleTimeChart.test.tsx
// tests/frontend/unit/components/metrics/ThroughputChart.test.tsx
// tests/frontend/unit/components/metrics/CodeReviewMetrics.test.tsx
// tests/frontend/unit/components/metrics/AmplifierDashboard.test.tsx
```

#### Filter Components
```typescript
// tests/frontend/unit/components/filters/DateRangeFilter.test.tsx
// tests/frontend/unit/components/filters/SquadFilter.test.tsx
// tests/frontend/unit/components/filters/UserFilter.test.tsx
// tests/frontend/unit/components/filters/RepositoryFilter.test.tsx
```

#### Project Components
```typescript
// tests/frontend/unit/components/project/Timeline.test.tsx
// tests/frontend/unit/components/project/RepositoryManager.test.tsx
```

---

### 3.2 Pages (Target: 75%)
**Priority:** P2 - MEDIUM

```typescript
// tests/frontend/unit/pages/Projects.test.tsx
describe('Projects Page', () => {
  - 'renders project list'
  - 'filters projects by squad'
  - 'navigates to project detail'
  - 'handles empty state'
})

// tests/frontend/unit/pages/Settings.test.tsx
describe('Settings Page', () => {
  - 'renders all settings sections'
  - 'saves GitHub token'
  - 'validates input'
  - 'shows success/error messages'
})

// tests/frontend/unit/pages/UserDetail.test.tsx
describe('User Detail Page', () => {
  - 'fetches and displays user data'
  - 'renders activity timeline'
  - 'displays user metrics'
  - 'handles navigation'
})
```

---

### 3.3 Stores (Target: 85%)
**Priority:** P2 - MEDIUM

```typescript
// tests/frontend/unit/stores/configStore.test.ts
describe('ConfigStore', () => {
  // Already has TODO tests defined
  - Implement all 5 test suites
  - Add tests for persistence
  - Add tests for validation
})

// Additional store tests needed:
// tests/frontend/unit/stores/dataStore.test.ts
// tests/frontend/unit/stores/syncStore.test.ts
// tests/frontend/unit/stores/metricsStore.test.ts
```

---

### 3.4 Hooks (Target: 80%)
**Priority:** P3 - LOW

```typescript
// tests/frontend/unit/hooks/useDebounce.test.ts
// tests/frontend/unit/hooks/useMetrics.test.ts
```

---

## Phase 4: End-to-End User Workflows

### 4.1 Critical User Journeys (Target: 100%)
**Priority:** P1 - HIGH

```typescript
// tests/e2e/specs/auth.spec.ts
describe('Authentication', () => {
  - 'complete GitHub OAuth flow'
  - 'persists authentication state'
  - 'handles authentication errors'
  - 'logs out successfully'
  - 'redirects unauthenticated users'
})

// tests/e2e/specs/dashboard.spec.ts
describe('Dashboard Workflow', () => {
  - 'loads dashboard with default filters'
  - 'applies date range filter'
  - 'applies squad filter'
  - 'applies repository filter'
  - 'views metric details'
  - 'exports metrics data'
})

// tests/e2e/specs/sync.spec.ts
describe('GitHub Sync', () => {
  - 'initiates manual sync'
  - 'displays sync progress'
  - 'updates UI after sync completion'
  - 'handles sync errors'
  - 'shows last sync timestamp'
})

// tests/e2e/specs/search.spec.ts
describe('Search Functionality', () => {
  - 'searches activities by keyword'
  - 'filters search results'
  - 'navigates to search result detail'
  - 'handles empty search results'
  - 'performs semantic search'
})

// tests/e2e/specs/settings.spec.ts
describe('Settings Management', () => {
  - 'updates GitHub configuration'
  - 'manages team members'
  - 'configures squads'
  - 'sets up repositories'
  - 'validates required fields'
})

// tests/e2e/specs/filters.spec.ts
describe('Filter Interactions', () => {
  - 'combines multiple filters'
  - 'clears all filters'
  - 'persists filter state on navigation'
  - 'resets filters on logout'
})
```

---

## Implementation Guidelines

### Testing Best Practices

#### Rust Tests
```rust
// Use test fixtures
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_example() {
        let db = setup_test_db();
        // Test implementation
        teardown_test_db(db);
    }
}

// Use mockall for mocking
use mockall::{automock, predicate::*};

// Use proptest for property-based testing
use proptest::prelude::*;
```

#### TypeScript Tests
```typescript
// Use React Testing Library best practices
import { render, screen, fireEvent, waitFor } from '@testing-library/react';

// Use MSW for API mocking
import { rest } from 'msw';
import { setupServer } from 'msw/node';

// Test user behavior, not implementation
test('user can submit form', async () => {
  render(<MyComponent />);

  const input = screen.getByLabelText(/name/i);
  const button = screen.getByRole('button', { name: /submit/i });

  fireEvent.change(input, { target: { value: 'John' } });
  fireEvent.click(button);

  await waitFor(() => {
    expect(screen.getByText(/success/i)).toBeInTheDocument();
  });
});
```

### Test Organization

```
tests/
├── frontend/
│   ├── unit/
│   │   ├── components/
│   │   ├── stores/
│   │   ├── hooks/
│   │   └── utils/
│   ├── integration/
│   └── setup.ts
├── e2e/
│   ├── specs/
│   └── fixtures/
└── rust/
    ├── unit/
    ├── integration/
    ├── fixtures/
    └── test_utils.rs
```

### CI/CD Integration

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  frontend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: npm install
      - run: npm run test:coverage
      - run: npm run test:e2e

  backend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - run: cd src-tauri && cargo test

  coverage-report:
    needs: [frontend-tests, backend-tests]
    runs-on: ubuntu-latest
    steps:
      - run: npm run test:coverage
      - uses: codecov/codecov-action@v3
```

---

## Success Metrics

### Coverage Targets (by End of Plan)
- **Metrics Calculations:** 100%
- **Business Days Logic:** 100%
- **Database Operations:** 90%
- **GitHub Sync:** 85%
- **Search & Embeddings:** 80%
- **UI Components:** 80%
- **Pages:** 75%
- **Stores:** 85%
- **E2E Critical Paths:** 100%

### Quality Gates
- All tests must pass before merging to main
- Code coverage must not decrease
- No skipped tests in CI
- Performance tests must meet benchmarks
- E2E tests must run on every PR

### Timeline Estimates

**Phase 1 (Critical):** Core business logic testing
- Estimated effort: Comprehensive implementation required

**Phase 2 (High):** Integration testing
- Estimated effort: Moderate implementation required

**Phase 3 (Medium):** UI testing
- Estimated effort: Extensive implementation required

**Phase 4 (High):** E2E testing
- Estimated effort: Significant implementation required

---

## Maintenance & Monitoring

### Ongoing Practices
1. Write tests for all new features
2. Update tests when modifying existing code
3. Run tests locally before committing
4. Review coverage reports weekly
5. Refactor tests alongside code
6. Keep test dependencies updated
7. Monitor test execution time

### Coverage Monitoring
- Generate coverage reports: `npm run test:coverage`
- Review in CI/CD pipeline
- Set up coverage badges in README
- Track coverage trends over time

---

## Appendix: Test Commands

### Frontend
```bash
# Run all unit tests
npm run test

# Run with UI
npm run test:ui

# Generate coverage report
npm run test:coverage

# Run E2E tests
npm run test:e2e

# Run specific test file
npm run test -- Dashboard.test.tsx

# Watch mode
npm run test -- --watch
```

### Backend
```bash
# Run all tests
cd src-tauri && cargo test

# Run specific test
cargo test test_calculate_metrics

# Run with output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored

# Run benchmarks
cargo test --benches
```

---

## Notes

- All test files follow established patterns in `tests/README.md`
- Use existing mock infrastructure in `tests/frontend/mocks/`
- Leverage GitHub response fixtures in `tests/rust/fixtures/`
- Follow TDD practices for new features
- Prioritize tests that prevent regressions in production

---

**Document Version:** 1.0
**Last Updated:** 2025-12-26
**Status:** Ready for Implementation
