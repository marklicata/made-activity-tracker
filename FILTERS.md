# Dashboard Filters API Reference

Complete guide to using dashboard filters in the MADE Activity Tracker.

---

## Table of Contents

1. [Overview](#overview)
2. [Filter Types](#filter-types)
3. [Backend API](#backend-api)
4. [Frontend Usage](#frontend-usage)
5. [Examples](#examples)
6. [Advanced Topics](#advanced-topics)

---

## Overview

The dashboard filtering system allows users to filter metrics by date range, repository, squad, or user. Filters can be combined (except squad and user, which are mutually exclusive) and persist across page refreshes.

### Key Features

- **4 Filter Types**: Date range, Repository (multi-select), Squad (single-select), User (single-select)
- **Filter Persistence**: State saved to localStorage
- **Real-time Updates**: Metrics and charts update automatically (300ms debounce)
- **Mutual Exclusivity**: Squad and user filters can't both be active
- **Type-safe**: Full TypeScript support with matching Rust types

---

## Filter Types

### 1. Date Range Filter

**Purpose**: Limit metrics to a specific time period

**Format**: ISO 8601 strings (`YYYY-MM-DDTHH:MM:SSZ`)

**Presets**:
- Last 7 days
- Last 30 days
- Last 90 days (default)
- Last 6 months (180 days)
- Last year (365 days)

**TypeScript Interface**:
```typescript
interface DateRange {
  start: string; // ISO 8601
  end: string;   // ISO 8601
}
```

**Rust Struct**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateRange {
    pub start: String,
    pub end: String,
}
```

### 2. Repository Filter

**Purpose**: Filter metrics to specific repositories

**Format**: Array of repository IDs (integers)

**Behavior**: Multi-select - can select multiple repositories

**Example**: `[1, 2, 3]` filters to repositories with IDs 1, 2, and 3

### 3. Squad Filter

**Purpose**: Filter metrics to members of a specific squad

**Format**: String (squad ID)

**Behavior**: Single-select - only one squad at a time

**Mutually Exclusive**: Clears user filter when set

**Example**: `"frontend"` filters to all members of the frontend squad

### 4. User Filter

**Purpose**: Filter metrics to a specific user

**Format**: Integer (user ID)

**Behavior**: Single-select - only one user at a time

**Mutually Exclusive**: Clears squad filter when set

**Example**: `42` filters to user with ID 42

---

## Backend API

### Rust Types

**MetricsFilters Struct**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsFilters {
    pub date_range: Option<DateRange>,
    pub repository_ids: Option<Vec<i64>>,
    pub squad_id: Option<String>,
    pub user_id: Option<i64>,
}
```

### Tauri Commands

#### 1. Get Filtered Dashboard Metrics

**Command**: `get_dashboard_metrics_filtered`

**Parameters**:
- `filters: MetricsFilters` - Filter parameters

**Returns**: `DashboardMetrics`

**Usage**:
```typescript
const metrics = await invoke<DashboardMetrics>('get_dashboard_metrics_filtered', {
  filters: {
    dateRange: { start: '2024-01-01T00:00:00Z', end: '2024-01-31T00:00:00Z' },
    repositoryIds: [1, 2],
    squadId: 'frontend',
    userId: null,
  },
});
```

#### 2. Get Metrics Timeseries

**Command**: `get_metrics_timeseries`

**Parameters**:
- `filters: MetricsFilters` - Filter parameters
- `granularity: string` - 'daily', 'weekly', or 'monthly'

**Returns**: `TimeseriesDataPoint[]`

**Usage**:
```typescript
const timeseriesData = await invoke<TimeseriesDataPoint[]>('get_metrics_timeseries', {
  filters: {
    dateRange: { start: '2024-01-01T00:00:00Z', end: '2024-03-31T00:00:00Z' },
  },
  granularity: 'weekly',
});
```

#### 3. Get All Repositories

**Command**: `get_all_repositories`

**Parameters**: None

**Returns**: `Repository[]`

**Usage**:
```typescript
const repos = await invoke<Repository[]>('get_all_repositories');
```

#### 4. Get All Users

**Command**: `get_all_users`

**Parameters**: None

**Returns**: `User[]` (excludes bots)

**Usage**:
```typescript
const users = await invoke<User[]>('get_all_users');
```

### Database Queries

**Filtered Issue Query**:
```rust
pub fn get_issues_for_metrics_filtered(
    conn: &Connection,
    since: &str,
    until: Option<&str>,
    excluded_bots: &[String],
    repo_ids: Option<&[i64]>,
    user_id: Option<i64>,
    squad_member_ids: Option<&[i64]>,
) -> Result<Vec<Issue>>
```

**Key Features**:
- Dynamic SQL generation based on provided filters
- Parameterized queries (SQL injection safe)
- Optional parameters handled with `Option<T>`
- Bot exclusion built-in

---

## Frontend Usage

### Filter Store

**Import**:
```typescript
import { useDashboardFilterStore } from '@stores/dashboardFilterStore';
```

**State Access**:
```typescript
const { filters, hasActiveFilters, clearFilters } = useDashboardFilterStore();
```

**Actions**:

1. **Set Date Range**:
```typescript
const { setDateRange, setDateRangePreset } = useDashboardFilterStore();

// Custom range
setDateRange({
  start: '2024-01-01T00:00:00Z',
  end: '2024-01-31T00:00:00Z',
});

// Preset
setDateRangePreset(DATE_RANGE_PRESETS.LAST_30_DAYS);

// Clear
setDateRange(null);
```

2. **Set Repositories**:
```typescript
const { setRepositories } = useDashboardFilterStore();

// Select multiple
setRepositories([1, 2, 3]);

// Clear
setRepositories(null);
```

3. **Set Squad**:
```typescript
const { setSquad } = useDashboardFilterStore();

// Select squad (clears user filter)
setSquad('frontend');

// Clear
setSquad(null);
```

4. **Set User**:
```typescript
const { setUser } = useDashboardFilterStore();

// Select user (clears squad filter)
setUser(42);

// Clear
setUser(null);
```

5. **Clear All Filters**:
```typescript
const { clearFilters } = useDashboardFilterStore();

// Clears all except date range (resets to 90-day default)
clearFilters();
```

### Filter Components

**Import**:
```typescript
import { DateRangeFilter, RepositoryFilter, SquadFilter, UserFilter } from '@components/filters';
```

**Usage**:
```tsx
<div className="filter-bar">
  <DateRangeFilter />
  <RepositoryFilter />
  <SquadFilter />
  <UserFilter />

  {hasActiveFilters() && (
    <button onClick={clearFilters}>Clear filters</button>
  )}
</div>
```

**Component Props**:
- All filter components are self-contained (no props required)
- They connect to the filter store internally
- State changes trigger automatic re-renders

### Chart Component

**Import**:
```typescript
import { MetricsTrendChart } from '@components/charts';
```

**Usage**:
```tsx
<MetricsTrendChart
  data={timeseriesData}
  metric="speed" // or "ease" or "quality"
  title="Speed Trends Over Time"
/>
```

**Props**:
- `data: TimeseriesDataPoint[]` - Timeseries data from backend
- `metric: 'speed' | 'ease' | 'quality'` - Which metric category to display
- `title: string` - Chart title

---

## Examples

### Example 1: Filter by Date Range

```typescript
import { useDashboardFilterStore } from '@stores/dashboardFilterStore';
import { DATE_RANGE_PRESETS } from '@types/filters';

function MyComponent() {
  const { setDateRangePreset } = useDashboardFilterStore();

  const handleLast30Days = () => {
    setDateRangePreset(DATE_RANGE_PRESETS.LAST_30_DAYS);
  };

  return <button onClick={handleLast30Days}>Last 30 Days</button>;
}
```

### Example 2: Filter by Repository

```typescript
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { useDashboardFilterStore } from '@stores/dashboardFilterStore';

function MyComponent() {
  const [repos, setRepos] = useState<Repository[]>([]);
  const { setRepositories } = useDashboardFilterStore();

  useEffect(() => {
    invoke<Repository[]>('get_all_repositories').then(setRepos);
  }, []);

  const handleSelectRepo = (repoId: number) => {
    setRepositories([repoId]);
  };

  return (
    <div>
      {repos.map(repo => (
        <button key={repo.id} onClick={() => handleSelectRepo(repo.id)}>
          {repo.owner}/{repo.name}
        </button>
      ))}
    </div>
  );
}
```

### Example 3: Combined Filters

```typescript
import { useDashboardFilterStore } from '@stores/dashboardFilterStore';
import { DATE_RANGE_PRESETS } from '@types/filters';

function MyComponent() {
  const { setDateRangePreset, setRepositories, setSquad } = useDashboardFilterStore();

  const applyFilters = () => {
    // Last 30 days
    setDateRangePreset(DATE_RANGE_PRESETS.LAST_30_DAYS);

    // Specific repositories
    setRepositories([1, 2, 3]);

    // Frontend squad
    setSquad('frontend');
  };

  return <button onClick={applyFilters}>Apply Filters</button>;
}
```

### Example 4: Load and Display Metrics

```typescript
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { useDashboardFilterStore } from '@stores/dashboardFilterStore';

function MyComponent() {
  const [metrics, setMetrics] = useState<DashboardMetrics | null>(null);
  const { filters } = useDashboardFilterStore();

  useEffect(() => {
    const timer = setTimeout(() => {
      invoke<DashboardMetrics>('get_dashboard_metrics_filtered', { filters })
        .then(setMetrics);
    }, 300); // Debounce

    return () => clearTimeout(timer);
  }, [filters]);

  if (!metrics) return <div>Loading...</div>;

  return (
    <div>
      <p>Cycle Time: {metrics.speed.avg_cycle_time_days} days</p>
      <p>PR Lead Time: {metrics.speed.avg_pr_lead_time_hours}h</p>
    </div>
  );
}
```

### Example 5: Display Chart with Filters

```typescript
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { useDashboardFilterStore } from '@stores/dashboardFilterStore';
import { MetricsTrendChart } from '@components/charts';

function MyComponent() {
  const [timeseriesData, setTimeseriesData] = useState<TimeseriesDataPoint[]>([]);
  const { filters } = useDashboardFilterStore();

  useEffect(() => {
    const timer = setTimeout(() => {
      invoke<TimeseriesDataPoint[]>('get_metrics_timeseries', {
        filters,
        granularity: 'weekly',
      }).then(setTimeseriesData);
    }, 300);

    return () => clearTimeout(timer);
  }, [filters]);

  return (
    <MetricsTrendChart
      data={timeseriesData}
      metric="speed"
      title="Speed Trends Over Time"
    />
  );
}
```

---

## Advanced Topics

### Filter Persistence

Filters are automatically persisted to localStorage using Zustand's persist middleware:

```typescript
export const useDashboardFilterStore = create<DashboardFilterState>()(
  persist(
    (set, get) => ({
      filters: { dateRange: createDateRangeFromDays(90) },
      // ...
    }),
    {
      name: 'made-dashboard-filters', // localStorage key
    }
  )
);
```

**Storage Location**: `localStorage['made-dashboard-filters']`

**What's Persisted**: All filter state (date range, repositories, squad, user)

**When It's Loaded**: On app startup, before first render

### Mutual Exclusivity Implementation

Squad and user filters are mutually exclusive. This is enforced in the store:

```typescript
setSquad: (squadId) => {
  set({
    filters: {
      ...get().filters,
      squadId: squadId || undefined,
      userId: squadId ? undefined : get().filters.userId, // Clear user when setting squad
    },
  });
},

setUser: (userId) => {
  set({
    filters: {
      ...get().filters,
      userId: userId || undefined,
      squadId: userId ? undefined : get().filters.squadId, // Clear squad when setting user
    },
  });
},
```

**UI Behavior**: The inactive filter component is hidden using conditional rendering:

```typescript
// In SquadFilter.tsx
if (filters.userId) {
  return null; // Hide squad filter when user filter is active
}

// In UserFilter.tsx
if (filters.squadId) {
  return null; // Hide user filter when squad filter is active
}
```

### Debouncing Strategy

Filter changes trigger debounced API calls to prevent excessive backend requests:

```typescript
useEffect(() => {
  if (!loading) {
    const timer = setTimeout(() => {
      loadData();
      loadChartData();
    }, 300); // 300ms debounce

    return () => clearTimeout(timer);
  }
}, [filters]);
```

**Why 300ms?**:
- Fast enough for responsive UX
- Slow enough to batch rapid changes
- Industry standard for search/filter inputs

### Dynamic SQL Query Building

The backend builds SQL queries dynamically based on provided filters:

```rust
let mut query = String::from("SELECT ... WHERE created_at >= ?1");
let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(since.to_string())];
let mut param_idx = 2;

if let Some(end) = until {
    query.push_str(&format!(" AND created_at <= ?{}", param_idx));
    params_vec.push(Box::new(end.to_string()));
    param_idx += 1;
}

if let Some(repos) = repo_ids {
    let placeholders = repos.iter().enumerate()
        .map(|(i, _)| format!("?{}", param_idx + i))
        .collect::<Vec<_>>()
        .join(", ");
    query.push_str(&format!(" AND repo_id IN ({})", placeholders));
    for repo_id in repos {
        params_vec.push(Box::new(*repo_id));
    }
    param_idx += repos.len();
}

// ... more filters ...
```

**Key Points**:
- SQL is built only with active filters
- Parameters are properly indexed
- Type-safe parameter binding
- No SQL injection risk

### Date Range Helpers

Helper functions for creating date ranges:

```typescript
// Create range from days back
export function createDateRangeFromDays(days: number): DateRange {
  const end = new Date();
  const start = new Date();
  start.setDate(start.getDate() - days);

  return {
    start: start.toISOString(),
    end: end.toISOString(),
  };
}

// Format range for display
export function formatDateRange(range: DateRange): string {
  const start = new Date(range.start);
  const end = new Date(range.end);

  const formatDate = (date: Date) => {
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  };

  return `${formatDate(start)} - ${formatDate(end)}`;
}
```

---

## Troubleshooting

### Filters Not Persisting

**Problem**: Filters reset on page reload

**Solution**: Check that localStorage is available and not blocked:

```typescript
// Test localStorage
try {
  localStorage.setItem('test', 'test');
  localStorage.removeItem('test');
  console.log('localStorage is available');
} catch (e) {
  console.error('localStorage is blocked:', e);
}
```

### Chart Not Updating

**Problem**: Chart doesn't update when filters change

**Solution**: Verify debounce logic is working:

```typescript
console.log('Filters changed:', filters);

useEffect(() => {
  console.log('Effect triggered, starting timer');
  const timer = setTimeout(() => {
    console.log('Debounce complete, loading data');
    loadChartData();
  }, 300);

  return () => {
    console.log('Cleaning up timer');
    clearTimeout(timer);
  };
}, [filters]);
```

### Squad Filter Not Working

**Problem**: Squad filter doesn't filter correctly

**Solution**: Verify squad members are configured:

```typescript
const squads = useConfigStore((state) => state.squads);
console.log('Squads:', squads);

// Check squad has members
const squad = squads.find(s => s.id === 'frontend');
console.log('Squad members:', squad?.members);
```

### Backend Query Slow

**Problem**: Filtered queries take too long

**Solution**: Add database indexes:

```rust
conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_issues_repo_created
     ON issues(repo_id, created_at)",
    [],
)?;

conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_issues_author_created
     ON issues(author_id, created_at)",
    [],
)?;
```

---

## Best Practices

1. **Always Debounce**: Use 300ms debounce for filter changes
2. **Handle Loading States**: Show spinners while data loads
3. **Validate Dates**: Ensure date ranges are valid (start < end)
4. **Test Edge Cases**: Empty data, single repository, etc.
5. **Clear Filters Explicitly**: Provide clear visual way to reset
6. **Preserve User Intent**: Don't auto-clear filters unexpectedly
7. **Show Filter State**: Visual indication of active filters
8. **Optimize Queries**: Add indexes for commonly filtered columns

---

## API Reference Summary

### Backend Commands

| Command | Parameters | Returns |
|---------|-----------|---------|
| `get_dashboard_metrics_filtered` | `filters: MetricsFilters` | `DashboardMetrics` |
| `get_metrics_timeseries` | `filters: MetricsFilters`, `granularity: string` | `TimeseriesDataPoint[]` |
| `get_all_repositories` | None | `Repository[]` |
| `get_all_users` | None | `User[]` |
| `get_squad_member_ids` | `squad_id: string` | `i64[]` |

### Frontend Store Actions

| Action | Parameters | Effect |
|--------|-----------|--------|
| `setDateRange` | `range: DateRange \| null` | Sets custom date range |
| `setDateRangePreset` | `days: number` | Sets preset date range |
| `setRepositories` | `ids: number[] \| null` | Sets repository filter |
| `setSquad` | `id: string \| null` | Sets squad filter (clears user) |
| `setUser` | `id: number \| null` | Sets user filter (clears squad) |
| `clearFilters` | None | Resets all filters to default |
| `hasActiveFilters` | None | Returns `boolean` |

---

## Related Documentation

- [PHASE2_STATUS.md](./PHASE2_STATUS.md) - Phase 2 implementation details
- [README.md](./README.md) - Project overview and setup
- [PLAN.md](./PLAN.md) - Full project plan and architecture

---

**Last Updated**: December 2024
**Version**: Phase 2 Complete
