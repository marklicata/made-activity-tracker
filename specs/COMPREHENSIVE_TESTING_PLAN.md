# Comprehensive Testing Plan for MADE Activity Tracker

## Executive Summary

**Current State:** 15,000+ lines of code with ~30% test coverage
**Target State:** 85%+ test coverage across all critical paths
**Timeline:** 4 weeks to close critical gaps
**Priority:** P0 - PR-based metrics system (677 lines, 0% coverage)

## Critical Findings

### 🔴 Priority 0 - CRITICAL GAPS (Week 1)

#### 1. PR-Based Metrics System - ZERO TESTS
**File:** `src-tauri/src/db/metrics_queries.rs` (677 lines)
**Risk:** 🔴 CRITICAL - Production code with complex calculations, completely untested
**Impact:** High - Powers entire Amplifier metrics dashboard

**Missing Tests:**
- [ ] Speed metrics calculations
  - [ ] PRs per day per developer
  - [ ] PR turnaround time (hours)
  - [ ] LOC per day calculation
  - [ ] Cycle time distribution bucketing (< 4h, 4-12h, 12-24h, > 24h)
- [ ] Ease metrics calculations
  - [ ] Concurrent repositories count
  - [ ] Repos per developer
  - [ ] Active repositories aggregation
  - [ ] Repository distribution (org vs personal)
  - [ ] Work pattern heatmap generation (day/hour grid)
  - [ ] PR switch frequency calculation
- [ ] Quality metrics calculations
  - [ ] PR merge rate (merged / total closed)
  - [ ] Bug PR percentage (classification logic)
  - [ ] Feature PR percentage (classification logic)
  - [ ] Files per PR distribution (bucketing)
  - [ ] Review cycle time (time to first review)
  - [ ] Merge rate trend (weekly buckets)
- [ ] Overview metrics
  - [ ] Productivity multiplier formula (weighted: 35%+25%+25%+15%)
  - [ ] Active developers count
  - [ ] Total PRs calculation
- [ ] PR type classification
  - [ ] Feature detection (feat, feature, add, enhancement keywords)
  - [ ] Bug fix detection (fix, bug keywords)
  - [ ] Refactor detection
  - [ ] Test detection
  - [ ] Docs detection
- [ ] Edge cases
  - [ ] Empty data handling (no PRs)
  - [ ] Single PR edge case
  - [ ] Division by zero protection
  - [ ] Null/missing data handling
- [ ] Benchmark comparisons
  - [ ] Industry benchmark values
  - [ ] Elite benchmark values
  - [ ] Percentage calculations

#### 2. Frontend Metrics Components - ZERO TESTS
**Files:** `src/components/metrics/*.tsx` (1,000+ lines)
**Risk:** 🔴 CRITICAL - User-facing, brand new, untested

**Missing Tests:**
- [ ] **AmplifierMetricsView**
  - [ ] Renders loading state
  - [ ] Renders error state
  - [ ] Renders metrics sections
  - [ ] Time period selector works (7d/30d/90d)
  - [ ] Fetches correct data for selected period
- [ ] **ProductivityOverview**
  - [ ] Displays multiplier correctly
  - [ ] Color-codes by tier (red/yellow/blue/purple)
  - [ ] Shows period, PRs, developers
  - [ ] Handles decimal formatting
- [ ] **BenchmarkMetricCard**
  - [ ] Displays value with correct formatting
  - [ ] Shows tier badge
  - [ ] Displays vs industry/elite percentages
  - [ ] Handles positive/negative trends
  - [ ] Handles missing comparison data
- [ ] **SpeedSection**
  - [ ] Renders all 4 metric cards
  - [ ] Renders cycle time distribution chart
  - [ ] Formats hours correctly (m/h/d/w)
  - [ ] Shows benchmark comparisons
- [ ] **EaseSection**
  - [ ] Renders all 4 metric cards
  - [ ] Renders repo distribution chart
  - [ ] Renders top repos table
  - [ ] Handles empty repos list
- [ ] **QualitySection**
  - [ ] Renders all 5 metric cards
  - [ ] Renders PR type distribution
  - [ ] Renders files per PR distribution
  - [ ] Handles missing review data
- [ ] **DistributionChart**
  - [ ] Renders bars with correct widths
  - [ ] Shows labels and percentages
  - [ ] Handles 0% data
  - [ ] Handles 100% data

#### 3. usePRMetrics Hook - ZERO TESTS
**File:** `src/hooks/usePRMetrics.ts`
**Risk:** 🟡 HIGH - Critical data fetching logic

**Missing Tests:**
- [ ] Fetches data on mount
- [ ] Updates when days parameter changes
- [ ] Handles loading state correctly
- [ ] Handles error state correctly
- [ ] Auto-refresh works if enabled
- [ ] Cleanup on unmount

---

### 🟡 Priority 1 - HIGH RISK (Week 2)

#### 4. GitHub Sync Reliability
**Files:** `src-tauri/src/github/*.rs`
**Risk:** 🟡 HIGH - Core functionality, data integrity

**Missing Tests:**
- [ ] **SAML Fallback Chain**
  - [ ] Detects SAML errors in GraphQL response
  - [ ] Falls back to REST API
  - [ ] Falls back to CLI if REST fails
  - [ ] Handles CLI not installed
- [ ] **Watermark-Based Sync**
  - [ ] get_issues_watermark returns correct timestamp
  - [ ] get_prs_watermark returns correct timestamp
  - [ ] Sync only fetches new items after watermark
  - [ ] First sync (no watermark) fetches all
  - [ ] Handles empty watermark table
- [ ] **Embedding Generation**
  - [ ] Triggers after sync completes
  - [ ] Generates embeddings for new items only
  - [ ] Handles embedding failures gracefully
  - [ ] Batch size respects limits
- [ ] **Milestone Ordering**
  - [ ] Milestones sync before issues
  - [ ] Foreign key constraints work
  - [ ] Milestone not found error handling
- [ ] **Bot Exclusion**
  - [ ] Bots detected by is_bot flag
  - [ ] Bots filtered from metrics
  - [ ] Bot PRs/issues stored but not counted

#### 5. Database Query Modules
**Files:** `src-tauri/src/db/user_queries.rs`, `project_queries.rs`
**Risk:** 🟡 HIGH - Data accuracy

**Missing Tests:**
- [ ] **user_queries.rs**
  - [ ] get_user_activity_timeline with filters
  - [ ] get_tracked_users returns only tracked
  - [ ] toggle_user_tracking updates correctly
  - [ ] get_user_metrics aggregates correctly
- [ ] **project_queries.rs**
  - [ ] get_project_timeline orders correctly
  - [ ] get_project_contributors deduplicates
  - [ ] get_project_activity_heatmap buckets correctly
  - [ ] get_project_lifecycle_metrics calculates correctly

#### 6. Integration Tests
**Missing Coverage:**
- [ ] **Full Sync Pipeline**
  - [ ] End-to-end sync from GitHub to database
  - [ ] Verifies data integrity
  - [ ] Tests rollback on failure
  - [ ] Tests incremental sync after initial
- [ ] **Metrics Calculation Pipeline**
  - [ ] Database → metrics → frontend flow
  - [ ] Filter application throughout
  - [ ] Timeseries generation
- [ ] **Search Indexing Pipeline**
  - [ ] Sync → embedding generation → search indexing
  - [ ] Hybrid search returns relevant results
  - [ ] Duplicate detection works

---

### 🟢 Priority 2 - MEDIUM RISK (Week 3)

#### 7. AI Integration
**Files:** `src-tauri/src/ai/*.rs`
**Risk:** 🟡 MEDIUM - New feature, untested

**Missing Tests:**
- [ ] **Sidecar Lifecycle**
  - [ ] Sidecar starts successfully
  - [ ] Health check succeeds
  - [ ] Port allocation works
  - [ ] Shutdown cleans up process
- [ ] **Error Handling**
  - [ ] Handles sidecar startup failure
  - [ ] Handles sidecar crash during operation
  - [ ] Health check timeout handling
  - [ ] Request retry logic

#### 8. Project/Team Commands
**Files:** `src-tauri/src/project/commands.rs`, `team/commands.rs`
**Risk:** 🟡 MEDIUM - User-facing analytics

**Missing Tests:**
- [ ] Project timeline generation
- [ ] Contributor aggregation
- [ ] Activity heatmap data structure
- [ ] Collaboration matrix calculation
- [ ] Focus metrics calculation

#### 9. E2E Tests for New Features
**Missing Flows:**
- [ ] **PR-Based Metrics Dashboard**
  - [ ] Navigate to dashboard
  - [ ] Toggle to Amplifier view
  - [ ] Select 30-day period
  - [ ] Verify metrics display
  - [ ] Verify benchmark badges
- [ ] **Project Deep-Dive**
  - [ ] Navigate from projects list
  - [ ] Timeline loads
  - [ ] Contributors table populates
  - [ ] Activity heatmap renders
- [ ] **Team View**
  - [ ] Squad filter works
  - [ ] User cards render
  - [ ] Collaboration graph displays
- [ ] **User Tracking**
  - [ ] Toggle user tracking
  - [ ] Tracked users appear in metrics
  - [ ] Untracked users excluded

---

### 🔵 Priority 3 - NICE TO HAVE (Week 4)

#### 10. Property-Based Testing
- [ ] Metrics calculations with random datasets
- [ ] Date range edge cases (leap years, DST, timezones)
- [ ] Filter combinations (all permutations)

#### 11. Performance Tests
- [ ] Large dataset queries (10k+ PRs)
- [ ] Embedding generation at scale
- [ ] Search performance with large corpus

#### 12. Error Scenario Tests
- [ ] Network failures during sync
- [ ] Database corruption recovery
- [ ] Invalid API responses
- [ ] Rate limiting handling

---

## Test File Structure

### Backend Rust Tests

```
tests/
├── rust/
│   ├── unit/
│   │   ├── pr_metrics_test.rs           [NEW - P0]
│   │   ├── pr_metrics_edge_cases_test.rs [NEW - P0]
│   │   ├── user_queries_test.rs         [NEW - P1]
│   │   ├── project_queries_test.rs      [NEW - P1]
│   │   ├── sync_watermark_test.rs       [NEW - P1]
│   │   ├── saml_fallback_test.rs        [NEW - P1]
│   │   ├── ai_sidecar_test.rs           [NEW - P2]
│   │   └── ... (existing tests)
│   └── integration/
│       ├── full_sync_pipeline_test.rs   [NEW - P1]
│       ├── metrics_pipeline_test.rs     [NEW - P1]
│       ├── search_pipeline_test.rs      [NEW - P2]
│       └── ... (existing tests)
```

### Frontend Tests

```
tests/
├── frontend/
│   ├── unit/
│   │   ├── components/
│   │   │   ├── metrics/
│   │   │   │   ├── AmplifierMetricsView.test.tsx     [NEW - P0]
│   │   │   │   ├── ProductivityOverview.test.tsx     [NEW - P0]
│   │   │   │   ├── BenchmarkMetricCard.test.tsx      [NEW - P0]
│   │   │   │   ├── SpeedSection.test.tsx             [NEW - P0]
│   │   │   │   ├── EaseSection.test.tsx              [NEW - P0]
│   │   │   │   ├── QualitySection.test.tsx           [NEW - P0]
│   │   │   │   └── DistributionChart.test.tsx        [NEW - P0]
│   │   │   ├── project/
│   │   │   │   ├── Timeline.test.tsx                 [NEW - P2]
│   │   │   │   ├── ActivityHeatmap.test.tsx          [NEW - P2]
│   │   │   │   └── ContributorTable.test.tsx         [NEW - P2]
│   │   │   └── team/
│   │   │       ├── UserCard.test.tsx                 [NEW - P2]
│   │   │       ├── CollaborationGraph.test.tsx       [NEW - P2]
│   │   │       └── FocusAnalysis.test.tsx            [NEW - P2]
│   │   ├── hooks/
│   │   │   └── usePRMetrics.test.ts                  [NEW - P0]
│   │   └── types/
│   │       └── metrics.test.ts                       [NEW - P0]
│   └── integration/
│       ├── dashboard_flow.test.tsx                   [NEW - P1]
│       └── metrics_display.test.tsx                  [NEW - P1]
```

### E2E Tests

```
tests/
└── e2e/
    └── specs/
        ├── amplifier_metrics.spec.ts                 [NEW - P1]
        ├── project_deep_dive.spec.ts                 [NEW - P2]
        ├── team_view.spec.ts                         [NEW - P2]
        ├── user_tracking.spec.ts                     [NEW - P2]
        └── ... (existing tests)
```

---

## Implementation Schedule

### Week 1: Critical Path - PR-Based Metrics

**Monday-Tuesday:**
- [ ] Create `pr_metrics_test.rs` with speed metrics tests
- [ ] Create `pr_metrics_edge_cases_test.rs` with edge cases
- [ ] Test productivity multiplier formula

**Wednesday-Thursday:**
- [ ] Create ease metrics tests (concurrent repos, work pattern)
- [ ] Create quality metrics tests (merge rate, PR classification)

**Friday:**
- [ ] Frontend: `AmplifierMetricsView.test.tsx`
- [ ] Frontend: `BenchmarkMetricCard.test.tsx`
- [ ] Frontend: `usePRMetrics.test.ts`

**Deliverable:** PR-based metrics system at 90%+ test coverage

---

### Week 2: Integration & Sync Reliability

**Monday-Tuesday:**
- [ ] Create `full_sync_pipeline_test.rs`
- [ ] Create `sync_watermark_test.rs`
- [ ] Create `saml_fallback_test.rs`

**Wednesday-Thursday:**
- [ ] Create `user_queries_test.rs`
- [ ] Create `project_queries_test.rs`
- [ ] Create `metrics_pipeline_test.rs`

**Friday:**
- [ ] E2E: `amplifier_metrics.spec.ts`
- [ ] E2E: Dashboard flow validation

**Deliverable:** Sync reliability at 80%+ coverage, critical user flows tested

---

### Week 3: Feature Coverage

**Monday-Tuesday:**
- [ ] Frontend section tests (SpeedSection, EaseSection, QualitySection)
- [ ] DistributionChart tests
- [ ] ProductivityOverview tests

**Wednesday-Thursday:**
- [ ] AI integration tests (`ai_sidecar_test.rs`)
- [ ] Project/team command tests
- [ ] Search pipeline integration test

**Friday:**
- [ ] E2E: `project_deep_dive.spec.ts`
- [ ] E2E: `team_view.spec.ts`
- [ ] E2E: `user_tracking.spec.ts`

**Deliverable:** All major features have test coverage

---

### Week 4: Polish & Performance

**Monday-Tuesday:**
- [ ] Property-based tests for metrics
- [ ] Edge case tests for filters
- [ ] Date handling tests

**Wednesday-Thursday:**
- [ ] Performance tests for large datasets
- [ ] Memory leak tests
- [ ] Error scenario tests

**Friday:**
- [ ] Test documentation
- [ ] Coverage report generation
- [ ] CI/CD integration

**Deliverable:** 85%+ coverage across codebase, CI pipeline green

---

## Coverage Targets by Component

| Component | Current | Week 1 | Week 2 | Week 3 | Week 4 | Priority |
|-----------|---------|--------|--------|--------|--------|----------|
| PR-based metrics (metrics_queries.rs) | 0% | 90% | 95% | 95% | 95% | P0 |
| Frontend metrics components | 0% | 80% | 85% | 90% | 90% | P0 |
| usePRMetrics hook | 0% | 85% | 85% | 85% | 85% | P0 |
| GitHub sync | 30% | 40% | 75% | 80% | 85% | P1 |
| Database queries (new) | 0% | 20% | 80% | 85% | 85% | P1 |
| Integration tests | 20% | 25% | 60% | 75% | 80% | P1 |
| AI integration | 0% | 0% | 10% | 70% | 75% | P2 |
| Project/team commands | 0% | 0% | 10% | 70% | 75% | P2 |
| E2E new features | 0% | 10% | 50% | 80% | 85% | P1 |
| Original metrics calculator | 80% | 85% | 90% | 90% | 90% | P3 |
| Search & embeddings | 60% | 60% | 65% | 80% | 85% | P2 |

**Overall Target:** 30% → 50% → 70% → 80% → 85%

---

## Testing Tools & Setup

### Backend (Rust)
- **Framework:** Built-in `#[test]` with `#[cfg(test)]`
- **Assertions:** Standard `assert_eq!`, `assert!`
- **Mocking:** Manual test fixtures, in-memory SQLite
- **Coverage:** `cargo tarpaulin` or `cargo-llvm-cov`

### Frontend (TypeScript/React)
- **Framework:** Vitest (already configured)
- **Testing Library:** @testing-library/react
- **Assertions:** Vitest expect
- **Mocking:** vi.mock() for Tauri commands
- **Coverage:** Vitest built-in coverage

### E2E
- **Framework:** Playwright (already configured)
- **Browser:** Chromium, Firefox, WebKit
- **Assertions:** Playwright expect
- **Screenshots:** On failure

---

## Success Criteria

### Week 1 Success
- ✅ PR-based metrics system has 90%+ test coverage
- ✅ All metric calculations verified with unit tests
- ✅ Frontend metrics components render correctly
- ✅ No critical bugs found in PR-based metrics

### Week 2 Success
- ✅ Sync pipeline has 75%+ test coverage
- ✅ Watermark logic verified
- ✅ SAML fallback chain tested
- ✅ E2E test for Amplifier dashboard passing

### Week 3 Success
- ✅ All major features have test coverage
- ✅ AI integration tested
- ✅ Project/team modules tested
- ✅ E2E tests for all new pages

### Week 4 Success
- ✅ Overall test coverage at 85%+
- ✅ CI pipeline runs all tests
- ✅ Performance benchmarks established
- ✅ Test documentation complete

---

## Risk Mitigation

### Risk: Tests take too long to write
**Mitigation:**
- Start with critical path (P0) tests only
- Use test generators for repetitive tests
- Parallel test writing with AI assistance

### Risk: Tests are flaky
**Mitigation:**
- Use deterministic test data
- Mock time-dependent functions
- Retry failed tests once in CI
- Isolate tests (no shared state)

### Risk: Breaking changes during testing
**Mitigation:**
- Freeze feature development for Week 1
- Create feature branch for test additions
- Merge tests incrementally

### Risk: Low ROI on some tests
**Mitigation:**
- Focus on high-risk, high-value code first
- Skip trivial getters/setters
- Test business logic, not framework code

---

## Maintenance Plan

### Ongoing
- [ ] Require tests for all new features
- [ ] Run tests in CI before merge
- [ ] Maintain 85%+ coverage target
- [ ] Review test failures weekly
- [ ] Update tests when requirements change

### Monthly
- [ ] Review flaky tests
- [ ] Update test data fixtures
- [ ] Optimize slow tests
- [ ] Add property-based tests for new edge cases

### Quarterly
- [ ] Full coverage audit
- [ ] Performance test suite run
- [ ] Update E2E scenarios for new features
- [ ] Test documentation review

---

## Appendix A: Test Data Fixtures

### Sample PR Data for Tests

```rust
// Complete PR with all fields
let pr = PullRequest {
    id: 1,
    github_id: 2001,
    repo_id: 1,
    number: 42,
    title: "Add new feature".to_string(),
    body: Some("Description".to_string()),
    state: "closed".to_string(),
    author_id: Some(1),
    created_at: "2024-01-01T10:00:00Z".to_string(),
    updated_at: "2024-01-03T10:00:00Z".to_string(),
    merged_at: Some("2024-01-03T10:00:00Z".to_string()),
    closed_at: Some("2024-01-03T10:00:00Z".to_string()),
    additions: 150,
    deletions: 50,
    changed_files: 5,
    review_comments: 3,
    labels: vec!["feature".to_string()],
    sync_updated_at: None,
};
```

### Sample Frontend Metrics Data

```typescript
const mockMetrics: DashboardMetrics = {
  overview: {
    productivity_multiplier: 3.7,
    period_days: 30,
    total_prs: 150,
    active_developers: 5,
  },
  speed: {
    prs_per_day: 5.0,
    prs_per_day_per_dev: 1.0,
    pr_turnaround_hours: 12.5,
    loc_per_day: 1500,
    cycle_time_distribution: {
      under_4h: 30,
      under_4h_pct: 20.0,
      h4_to_12: 60,
      h4_to_12_pct: 40.0,
      h12_to_24: 45,
      h12_to_24_pct: 30.0,
      over_24h: 15,
      over_24h_pct: 10.0,
    },
    benchmark_comparison: {
      prs_per_day_industry: 0.8,
      prs_per_day_elite: 1.5,
      pr_turnaround_industry: 89.0,
      pr_turnaround_elite: 24.0,
    },
  },
  // ... (ease and quality)
};
```

---

## Appendix B: Test Writing Guidelines

### Naming Convention
```rust
// Rust
#[test]
fn test_<component>_<scenario>_<expected_result>()

// Examples:
#[test]
fn test_productivity_multiplier_with_zero_prs_returns_zero()

#[test]
fn test_cycle_time_distribution_buckets_correctly()
```

```typescript
// TypeScript
describe('ComponentName', () => {
  describe('Scenario', () => {
    it('should expected behavior', () => {});
  });
});

// Example:
describe('BenchmarkMetricCard', () => {
  describe('when comparison is elite tier', () => {
    it('should display blue badge', () => {});
  });
});
```

### Test Structure (AAA Pattern)
```rust
#[test]
fn test_example() {
    // Arrange - Set up test data
    let data = setup_test_data();

    // Act - Execute the code under test
    let result = function_to_test(data);

    // Assert - Verify the result
    assert_eq!(result, expected_value);
}
```

### Edge Cases to Always Test
- Empty data (no records)
- Single record
- Null/None values
- Division by zero
- Very large numbers
- Very small numbers (< 0.001)
- Negative numbers (if invalid)
- Date boundaries (start of day, end of day)
- Timezone handling

---

## Conclusion

This comprehensive testing plan addresses all major gaps in the codebase with a focus on the highest-risk, highest-value areas first. The new PR-based metrics system is the most critical priority, followed by sync reliability and user-facing components.

By following this 4-week plan, we will achieve:
- ✅ 95%+ coverage on PR-based metrics (P0)
- ✅ 80%+ coverage on sync reliability (P1)
- ✅ 85%+ coverage on user-facing features (P1)
- ✅ 85%+ overall code coverage (Target)
- ✅ Comprehensive E2E test suite
- ✅ CI/CD integration with automated testing

**Next Step:** Begin Week 1 implementation with PR-based metrics unit tests.
