# Feature Spec: Project Deep Dive

## Overview
Add a dedicated page for in-depth analysis of a single repository, showing detailed activity timelines, contributor breakdowns, lifecycle metrics, and activity heatmaps. This provides granular insights beyond the aggregate metrics shown on the dashboard.

## Priority
**MEDIUM** - Implement after GitHub CLI fallback

## Problem Statement
The current dashboard shows aggregate metrics across multiple repositories, but doesn't provide detailed insights into what happened in a specific project. Users need to:
- Understand the full history of work on a project
- See who contributed what and when
- Identify bottlenecks in issue/PR lifecycles
- Visualize activity patterns over time

## User Story
As a team lead, I want to dive deep into a specific repository's activity, so that I can understand the project's history, identify contributors' work patterns, and spot process bottlenecks.

## Requirements

### Functional Requirements

#### 1. Navigation
- **FR-1.1**: Click on any repository card on the dashboard to navigate to its deep dive page
- **FR-1.2**: Deep dive page accessible via route: `/projects/:owner/:repo`
- **FR-1.3**: Breadcrumb navigation: Dashboard > owner/repo
- **FR-1.4**: Back button returns to dashboard with filters preserved

#### 2. Timeline View
- **FR-2.1**: Chronological activity feed showing all events (commits, PRs, issues, reviews, merges)
- **FR-2.2**: Filter timeline by:
  - Event type (commits, PRs, issues, reviews)
  - User (contributor filter)
  - Date range
- **FR-2.3**: Each timeline item shows:
  - Event icon and type
  - Title/description
  - Author with avatar
  - Timestamp (relative and absolute)
  - Quick stats (lines changed, files modified, etc.)
- **FR-2.4**: Click timeline item to see full details in a drawer/modal
- **FR-2.5**: Infinite scroll or pagination for large timelines
- **FR-2.6**: Timeline zoom controls (day/week/month/all granularity)

#### 3. Contributor Breakdown
- **FR-3.1**: Table showing all contributors to the repository
- **FR-3.2**: For each contributor, show:
  - Name, avatar, GitHub username
  - Total commits
  - Total PRs created
  - Total PRs reviewed
  - Total issues created
  - Total issues commented on
  - Lines added/deleted
  - Files changed
  - First contribution date
  - Last contribution date
  - Activity trend (increasing/decreasing)
- **FR-3.3**: Sort by any column
- **FR-3.4**: Filter by date range
- **FR-3.5**: Click contributor to filter entire page by that user
- **FR-3.6**: Export contributor data to CSV

#### 4. Activity Heatmaps
- **FR-4.1**: **Calendar Heatmap**: GitHub-style contribution graph showing daily activity
  - Color intensity based on event count
  - Hover shows exact count and date
  - Click to filter timeline by that day
- **FR-4.2**: **Time-of-Day Heatmap**: When during the day are people working?
  - 24-hour grid showing activity by hour
  - Identifies peak working hours
- **FR-4.3**: **Day-of-Week Heatmap**: Which days see most activity?
  - Bar chart or grid showing Monday-Sunday distribution
- **FR-4.4**: **User Activity Matrix**: Who worked when?
  - Rows: users, Columns: time periods
  - Shows individual and team patterns

#### 5. Issue/PR Lifecycle Analysis
- **FR-5.1**: **Time to Merge** metrics:
  - Average time from PR open to merge
  - Median, P50, P90, P99
  - Trend over time (getting faster/slower?)
  - Breakdown by PR size (small/medium/large)
- **FR-5.2**: **Time to Close** metrics:
  - For issues: open to close time
  - For PRs: open to close time (including rejected)
- **FR-5.3**: **Review Time** metrics:
  - Time from PR open to first review
  - Average review cycle time
  - Number of review cycles before merge
- **FR-5.4**: **Bottleneck Identification**:
  - PRs waiting longest for review
  - Issues open longest without activity
  - PRs with most back-and-forth
- **FR-5.5**: **State Distribution**:
  - Pie chart: Open vs Closed vs Merged
  - Historical trend of open items over time
- **FR-5.6**: **Size Analysis**:
  - Distribution of PR sizes (lines changed)
  - Average size over time
  - Correlation between size and merge time

#### 6. Summary Cards
At the top of the page, show high-level stats:
- **FR-6.1**: Total contributors (all time and in selected date range)
- **FR-6.2**: Total commits, PRs, issues
- **FR-6.3**: Activity trend indicator (up/down/stable)
- **FR-6.4**: Last sync timestamp
- **FR-6.5**: Repository health score (based on activity, review time, etc.)

#### 7. Date Range Filter
- **FR-7.1**: Global date range filter affects all sections
- **FR-7.2**: Preset ranges: Last 7 days, 30 days, 90 days, 6 months, 1 year, All time
- **FR-7.3**: Custom date range picker
- **FR-7.4**: Date range persisted in URL for sharing

#### 8. Export & Sharing
- **FR-8.1**: Export full report as PDF
- **FR-8.2**: Share link with current filters
- **FR-8.3**: Export data tables to CSV

### Non-Functional Requirements

#### Performance
- **NFR-1**: Page load time < 2 seconds for repositories with < 10k events
- **NFR-2**: Timeline rendering virtualized for smooth scrolling with 1000+ events
- **NFR-3**: Heatmap calculations cached per repository
- **NFR-4**: Lazy load sections as user scrolls

#### Usability
- **NFR-5**: Responsive design works on desktop and tablet
- **NFR-6**: Accessible keyboard navigation
- **NFR-7**: Clear loading states for each section
- **NFR-8**: Empty states when no data available

#### Data
- **NFR-9**: All data pulled from existing database (no new sync required)
- **NFR-10**: Calculations performed on-demand or cached

## Technical Design

### New Routes

```typescript
// src/App.tsx
<Route path="/projects/:owner/:repo" element={<ProjectDeepDive />} />
```

### New Pages

#### `src/pages/ProjectDeepDive.tsx`
Main container component that:
- Fetches repository details from Tauri
- Manages date range filter state
- Manages selected contributor filter
- Renders all sections

### New Components

#### `src/components/project/ProjectHeader.tsx`
- Repository name, description
- Summary stats cards
- Sync status
- Date range filter
- Back button

#### `src/components/project/Timeline.tsx`
- Activity feed with infinite scroll
- Event type filters
- Event cards with icons
- Detail drawer/modal

#### `src/components/project/ContributorTable.tsx`
- Sortable, filterable table
- Avatar, name, stats columns
- Click to filter by user
- Export to CSV button

#### `src/components/project/ActivityHeatmap.tsx`
- Calendar heatmap (recharts or custom)
- Tooltip on hover
- Click to filter timeline

#### `src/components/project/TimeOfDayHeatmap.tsx`
- 24-hour heat grid
- Color-coded intensity

#### `src/components/project/LifecycleMetrics.tsx`
- Time to merge charts
- Time to close charts
- Review time charts
- Bottleneck lists

#### `src/components/project/BottleneckList.tsx`
- List of items needing attention
- Sort by wait time
- Link to GitHub

### New Tauri Commands

#### `src-tauri/src/metrics/commands.rs`

```rust
#[tauri::command]
pub async fn get_project_timeline(
    repo_id: i64,
    start_date: Option<String>,
    end_date: Option<String>,
    event_types: Option<Vec<String>>,
    user_id: Option<i64>,
) -> Result<Vec<TimelineEvent>, String> {
    // Fetch all events for repository in date range
    // Merge PRs, issues, reviews, commits into single timeline
    // Sort by timestamp
}

#[tauri::command]
pub async fn get_project_contributors(
    repo_id: i64,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<ContributorStats>, String> {
    // Aggregate stats per contributor
    // Calculate totals, averages, trends
}

#[tauri::command]
pub async fn get_project_activity_heatmap(
    repo_id: i64,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<ActivityHeatmapData, String> {
    // Calculate daily activity counts
    // Return as { date: count } map
}

#[tauri::command]
pub async fn get_project_lifecycle_metrics(
    repo_id: i64,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<LifecycleMetrics, String> {
    // Calculate time to merge, time to close
    // Calculate review times
    // Identify bottlenecks
}
```

### New Data Models

#### `src-tauri/src/metrics/mod.rs`

```rust
#[derive(Debug, Clone, Serialize)]
pub struct TimelineEvent {
    pub id: String,
    pub event_type: String, // "commit", "pr_opened", "pr_merged", "issue_opened", etc.
    pub timestamp: String,
    pub author: User,
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub metadata: serde_json::Value, // Type-specific data
}

#[derive(Debug, Clone, Serialize)]
pub struct ContributorStats {
    pub user: User,
    pub total_commits: i32,
    pub total_prs: i32,
    pub total_prs_reviewed: i32,
    pub total_issues: i32,
    pub lines_added: i32,
    pub lines_deleted: i32,
    pub files_changed: i32,
    pub first_contribution: String,
    pub last_contribution: String,
    pub activity_trend: String, // "increasing", "stable", "decreasing"
}

#[derive(Debug, Clone, Serialize)]
pub struct ActivityHeatmapData {
    pub daily_counts: HashMap<String, i32>, // date -> count
    pub hourly_counts: HashMap<u8, i32>, // hour -> count
    pub weekday_counts: HashMap<String, i32>, // weekday -> count
}

#[derive(Debug, Clone, Serialize)]
pub struct LifecycleMetrics {
    pub avg_time_to_merge: f64, // hours
    pub median_time_to_merge: f64,
    pub p90_time_to_merge: f64,
    pub avg_time_to_first_review: f64,
    pub avg_review_cycles: f64,
    pub open_prs_count: i32,
    pub open_issues_count: i32,
    pub bottleneck_prs: Vec<PullRequest>, // PRs open longest
    pub bottleneck_issues: Vec<Issue>, // Issues open longest
}
```

### Database Queries

All data comes from existing tables. New query module:

#### `src-tauri/src/db/project_queries.rs`

```rust
pub fn get_timeline_events(
    conn: &Connection,
    repo_id: i64,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<Vec<TimelineEvent>> {
    // UNION queries across:
    // - pull_requests (opened, merged, closed events)
    // - issues (opened, closed events)
    // - pr_reviews (review submitted events)
    // Order by timestamp DESC
}

pub fn get_contributor_stats(
    conn: &Connection,
    repo_id: i64,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<Vec<ContributorStats>> {
    // Aggregate queries with GROUP BY user_id
    // Calculate totals and trends
}

pub fn get_lifecycle_metrics(
    conn: &Connection,
    repo_id: i64,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<LifecycleMetrics> {
    // Calculate time differences
    // Use percentile functions
    // Identify items with longest wait times
}
```

## UI Mockup Structure

```
┌─────────────────────────────────────────────────────────────┐
│  < Back to Dashboard                                        │
│  Dashboard > owner/repo                                      │
├─────────────────────────────────────────────────────────────┤
│  📦 owner/repo                                    [Sync ✓]  │
│  Description of the repository                              │
│                                                             │
│  [Date Range: Last 30 days ▼]                              │
│                                                             │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  │   145    │ │    23    │ │    89    │ │    12    │     │
│  │ Commits  │ │   PRs    │ │  Issues  │ │  Users   │     │
│  │  +12%    │ │   -5%    │ │   +8%    │ │   →      │     │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘     │
├─────────────────────────────────────────────────────────────┤
│  📅 Activity Calendar                                       │
│  ┌─────────────────────────────────────────────────────┐  │
│  │  [GitHub-style contribution heatmap]                 │  │
│  └─────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  📊 Lifecycle Metrics                                       │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐      │
│  │ Avg Time to  │ │  Avg Time to │ │  Avg Review  │      │
│  │    Merge     │ │  First Review│ │    Cycles    │      │
│  │   2.3 days   │ │   4.2 hours  │ │     2.1      │      │
│  └──────────────┘ └──────────────┘ └──────────────┘      │
│                                                             │
│  ⚠️ Bottlenecks (3)                                         │
│  • PR #123: Waiting for review (5 days)                   │
│  • Issue #456: No activity (12 days)                      │
│  • PR #789: Many review cycles (7 cycles)                 │
├─────────────────────────────────────────────────────────────┤
│  👥 Contributors (Showing 5 of 12)                [Export]│
│  ┌──────────────────────────────────────────────────────┐ │
│  │ Avatar │ Name      │ Commits│ PRs │ Reviews│ Lines  │ │
│  │   👤   │ Alice     │   45   │  12 │   23   │ +2.3k  │ │
│  │   👤   │ Bob       │   32   │   8 │   15   │ +1.1k  │ │
│  │   ...                                                 │ │
│  └──────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  📋 Timeline                                                │
│  [All ▼] [User: All ▼]                                    │
│                                                             │
│  ○─ alice merged PR #234 "Add new feature"                │
│     2 hours ago • +234 -12 • 5 files                      │
│                                                             │
│  ○─ bob opened issue #456 "Bug in login"                  │
│     3 hours ago                                            │
│                                                             │
│  ○─ carol reviewed PR #234                                │
│     4 hours ago • Approved                                 │
│                                                             │
│  [Load more...]                                            │
└─────────────────────────────────────────────────────────────┘
```

## Implementation Plan

### Phase 1: Core Infrastructure
1. Create routing for `/projects/:owner/:repo`
2. Create `ProjectDeepDive.tsx` page component
3. Create Tauri command `get_project_timeline`
4. Create Tauri command `get_project_contributors`
5. Implement database queries for timeline and contributors

### Phase 2: Basic UI
1. Build `ProjectHeader` component with summary cards
2. Build `Timeline` component with basic event list
3. Build `ContributorTable` component
4. Connect to Tauri commands
5. Add loading and error states

### Phase 3: Visualizations
1. Implement `ActivityHeatmap` with calendar view
2. Implement `TimeOfDayHeatmap`
3. Add heatmap data Tauri command and queries

### Phase 4: Lifecycle Metrics
1. Implement `LifecycleMetrics` calculations in Rust
2. Build `LifecycleMetrics` component
3. Build `BottleneckList` component
4. Add charts for trends over time

### Phase 5: Filters & Interactions
1. Add date range filter affecting all sections
2. Add contributor filter from table click
3. Add timeline event type filters
4. Add URL parameter persistence

### Phase 6: Polish & Export
1. Add export to CSV for contributor table
2. Add export to PDF for full report
3. Add share link generation
4. Performance optimization and caching
5. Add keyboard navigation
6. Responsive design tweaks

## Testing Strategy

### Unit Tests
- Timeline event aggregation logic
- Contributor stats calculations
- Lifecycle metric calculations
- Percentile functions
- Date range filtering

### Integration Tests
- Full page load with mock data
- Filter interactions
- Component communication
- Tauri command calls

### Manual Testing Checklist
- [ ] Navigate from dashboard to deep dive
- [ ] All summary cards show correct counts
- [ ] Timeline shows all event types correctly
- [ ] Timeline filters work (type, user, date)
- [ ] Contributor table sorts correctly
- [ ] Click contributor filters entire page
- [ ] Activity heatmap renders correctly
- [ ] Heatmap click filters timeline
- [ ] Lifecycle metrics calculate correctly
- [ ] Bottleneck list shows appropriate items
- [ ] Date range filter affects all sections
- [ ] Export to CSV works
- [ ] Share link preserves filters
- [ ] Page loads in < 2 seconds for 1k events
- [ ] Infinite scroll performs well

## Success Metrics
- Users spend 3+ minutes on deep dive pages (indicates engagement)
- 70% of dashboard clicks navigate to deep dive
- Users identify at least 1 bottleneck per session
- Page load time < 2 seconds for 90% of repositories

## Future Enhancements
1. AI-generated insights ("This PR took 3x longer than average to merge")
2. Comparison mode (compare two time periods)
3. Goals and targets (set target merge time, track progress)
4. Custom event types from webhooks
5. Real-time updates as sync happens
6. Embedded GitHub links and previews
7. Code quality metrics integration (test coverage, etc.)

## Dependencies
- Existing database schema (no changes needed)
- React Router for navigation
- Recharts or D3.js for visualizations
- date-fns for date manipulation
- Existing Tauri infrastructure

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Large repos with 100k+ events | HIGH | Pagination, virtualization, date range limits |
| Complex SQL queries slow | MEDIUM | Indexes on timestamp fields, query optimization, caching |
| Heatmap rendering slow | MEDIUM | Canvas-based rendering, data pre-aggregation |
| Too much data to display | MEDIUM | Smart defaults, progressive disclosure, summaries |

## Open Questions
1. Should we cache calculations or compute on-demand? - Cache for 24 hrs. And give the user a button to refresh the stats.
2. What's the maximum useful timeline length? (cap at 1000 events?). Sure 1000 sounds fine. Put this in a config though.
3. Should we show commits or just PRs/issues? Commits too.
4. Real-time updates or static snapshot? Real-time updates, cached/stored for 24 hours. But the user can click a button to force an update.
5. Should bottleneck thresholds be configurable? - not sure what this means? Do what you think is best.
