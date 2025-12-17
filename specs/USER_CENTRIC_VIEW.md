# Feature Spec: User-Centric View

## Overview
Add a new top-level navigation section that allows tracking specific users and their work across all repositories. This shifts the perspective from "what happened in this repo" to "what is this person/team working on across all repos."

## Priority
**MEDIUM** - Implement after GitHub CLI fallback

## Problem Statement
The current application is repository-centric: you select repositories to track and see metrics for those repos. However, managers and team leads often think in terms of people, not repos. They need to answer questions like:
- What has Alice been working on lately?
- Where is the backend team spending their time?
- Is Bob spreading himself too thin across too many repos?
- What are the collaboration patterns between team members?

## User Story
As a team lead, I want to track specific engineers and see their work across all repositories, so that I can understand what they're working on, identify patterns, and provide better support.

## Requirements

### Functional Requirements

#### 1. Navigation
- **FR-1.1**: New top-level navigation item: "Team View" or "Users"
- **FR-1.2**: Route: `/team` or `/users`
- **FR-1.3**: Distinct from dashboard (different primary focus)
- **FR-1.4**: Can switch between "Repos" (dashboard) and "Team" views

#### 2. User Selection & Management
- **FR-2.1**: Input field to add GitHub usernames (manual entry)
- **FR-2.2**: Add multiple users at once (comma-separated or one per line)
- **FR-2.3**: User list persisted in local storage/database
- **FR-2.4**: Remove users from tracked list
- **FR-2.5**: Visual indicator for users with no data found
- **FR-2.6**: Bulk operations: clear all, import from CSV

#### 3. User Overview Cards
Display a card for each tracked user showing:
- **FR-3.1**: Avatar, name, GitHub username
- **FR-3.2**: Summary stats:
  - Total PRs created (in date range)
  - Total PRs reviewed
  - Total commits
  - Total issues opened/closed
  - Number of repositories they've touched
- **FR-3.3**: Activity indicator (active/quiet/idle)
- **FR-3.4**: Last activity timestamp
- **FR-3.5**: Click card to see detailed view for that user

#### 4. Repository Distribution View
For each user, show which repositories they're working in:
- **FR-4.1**: List of repositories with contribution counts
- **FR-4.2**: Visual breakdown (pie chart or bar chart)
- **FR-4.3**: Percentage of time spent in each repo
- **FR-4.4**: Link to each repository's deep dive page
- **FR-4.5**: Filter by date range to see focus changes over time

#### 5. Aggregate Metrics Per User
When clicking a user card, show detailed page with:
- **FR-5.1**: Total contributions across all repos
- **FR-5.2**: Breakdown by activity type:
  - Commits (count and lines changed)
  - PRs opened (by state: merged, open, closed)
  - PRs reviewed (by outcome: approved, changes requested, commented)
  - Issues opened/closed
  - Comments on issues/PRs
- **FR-5.3**: Activity timeline (same as project deep dive, but user-filtered)
- **FR-5.4**: Repository heatmap showing where they work most
- **FR-5.5**: Time-based trends (velocity over time)
- **FR-5.6**: Average PR size, review time, merge time

#### 6. Collaboration Patterns
Show how users interact with each other:
- **FR-6.1**: **Review Network**: Who reviews whose code?
  - Directed graph or matrix
  - Shows review frequency between user pairs
  - Identifies review bottlenecks (one person reviewing everything)
- **FR-6.2**: **Co-contribution Map**: Who works on same repos/PRs?
  - Shows which users collaborate most often
  - Identifies silos (users who don't collaborate)
- **FR-6.3**: **Response Time Matrix**: How quickly do users respond to each other's PRs?
- **FR-6.4**: **Cross-repo work**: Which users work together across multiple repos?

#### 7. Time-Based Activity Trends
- **FR-7.1**: Line chart showing activity over time for each user
- **FR-7.2**: Compare multiple users side-by-side
- **FR-7.3**: Identify velocity changes (ramping up, slowing down, consistent)
- **FR-7.4**: Weekly/monthly aggregations
- **FR-7.5**: Annotate with milestones or significant events

#### 8. Focus Analysis
- **FR-8.1**: How many repos is each user working in?
- **FR-8.2**: Concentration score: % of work in top repository
- **FR-8.3**: Context switching indicator: how often do they move between repos?
- **FR-8.4**: Identify users spread thin across many repos
- **FR-8.5**: Identify users deeply focused on one repo

#### 9. Team Summary View
Aggregate view across all tracked users:
- **FR-9.1**: Total team activity metrics
- **FR-9.2**: Most active contributors
- **FR-9.3**: Distribution of work across team
- **FR-9.4**: Team velocity trends
- **FR-9.5**: Collaboration health score

#### 10. Filters & Controls
- **FR-10.1**: Date range filter (affects all metrics)
- **FR-10.2**: Repository filter (show only activity in specific repos)
- **FR-10.3**: Activity type filter (commits/PRs/reviews/issues)
- **FR-10.4**: Sort users by various metrics
- **FR-10.5**: Search/filter within tracked users list

#### 11. Export & Sharing
- **FR-11.1**: Export team report to PDF
- **FR-11.2**: Export individual user reports to PDF
- **FR-11.3**: Export user list and metrics to CSV
- **FR-11.4**: Share link with current user selection and filters

### Non-Functional Requirements

#### Performance
- **NFR-1**: Page load time < 3 seconds for 20 tracked users
- **NFR-2**: User addition triggers background data fetch
- **NFR-3**: Lazy load detailed views (only when clicked)
- **NFR-4**: Cache aggregate calculations

#### Usability
- **NFR-5**: Clear empty state when no users tracked
- **NFR-6**: Visual feedback when adding/removing users
- **NFR-7**: Graceful handling of users with no GitHub activity
- **NFR-8**: Responsive design for desktop and tablet

#### Data
- **NFR-9**: User data pulled from existing database (no new API calls needed)
- **NFR-10**: Users without activity in tracked repos show "No data" state
- **NFR-11**: User information cached and updated during regular repo syncs

## Technical Design

### New Routes

```typescript
// src/App.tsx
<Route path="/team" element={<TeamView />} />
<Route path="/team/:username" element={<UserDetail />} />
```

### New Pages

#### `src/pages/TeamView.tsx`
Main container showing:
- User management (add/remove users)
- User overview cards
- Team summary metrics
- Filters and controls

#### `src/pages/UserDetail.tsx`
Detailed view for a single user:
- User header with summary
- Activity timeline
- Repository distribution
- Metrics and trends

### New Components

#### `src/components/team/UserManager.tsx`
- Input for adding users
- List of tracked users with remove buttons
- Import/export functionality

#### `src/components/team/UserCard.tsx`
- Avatar, name, username
- Summary stats
- Activity indicator
- Click to navigate to detail

#### `src/components/team/TeamSummary.tsx`
- Aggregate team metrics
- Top contributors
- Team trends

#### `src/components/team/UserTimeline.tsx`
- Reuse Timeline from project deep dive
- Filter by user_id

#### `src/components/team/RepositoryDistribution.tsx`
- Chart showing which repos user worked in
- Breakdown by contribution type
- Links to repo deep dives

#### `src/components/team/CollaborationGraph.tsx`
- Network diagram or matrix
- Shows review relationships
- Interactive (hover for details)

#### `src/components/team/UserActivityTrend.tsx`
- Line chart of activity over time
- Compare multiple users
- Velocity indicators

#### `src/components/team/FocusAnalysis.tsx`
- Metrics about repo concentration
- Context switching indicators
- Visual representation of focus

### New Tauri Commands

#### `src-tauri/src/metrics/commands.rs`

```rust
#[tauri::command]
pub async fn get_user_summary(
    username: String,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<UserSummary, String> {
    // Aggregate user activity across all repos
    // Return summary stats
}

#[tauri::command]
pub async fn get_user_activity_timeline(
    username: String,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<TimelineEvent>, String> {
    // Get all events for user across all repos
    // Sorted chronologically
}

#[tauri::command]
pub async fn get_user_repository_distribution(
    username: String,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<RepositoryContribution>, String> {
    // For each repo user touched, count contributions
    // Return sorted by contribution count
}

#[tauri::command]
pub async fn get_team_collaboration_matrix(
    usernames: Vec<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<CollaborationMatrix, String> {
    // Calculate review relationships between users
    // Return matrix of interactions
}

#[tauri::command]
pub async fn get_user_activity_trend(
    username: String,
    start_date: Option<String>,
    end_date: Option<String>,
    granularity: String, // "day", "week", "month"
) -> Result<Vec<ActivityDataPoint>, String> {
    // Group activity by time period
    // Return time series data
}

#[tauri::command]
pub async fn get_team_summary(
    usernames: Vec<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<TeamSummary, String> {
    // Aggregate metrics across all users
    // Calculate team-level insights
}
```

### New Data Models

#### `src-tauri/src/metrics/mod.rs`

```rust
#[derive(Debug, Clone, Serialize)]
pub struct UserSummary {
    pub user: User,
    pub total_commits: i32,
    pub total_prs_created: i32,
    pub total_prs_merged: i32,
    pub total_prs_reviewed: i32,
    pub total_issues_opened: i32,
    pub total_issues_closed: i32,
    pub total_comments: i32,
    pub lines_added: i32,
    pub lines_deleted: i32,
    pub repositories_touched: i32,
    pub first_activity: Option<String>,
    pub last_activity: Option<String>,
    pub activity_status: String, // "active", "quiet", "idle"
}

#[derive(Debug, Clone, Serialize)]
pub struct RepositoryContribution {
    pub repository: Repository,
    pub commit_count: i32,
    pub pr_count: i32,
    pub issue_count: i32,
    pub review_count: i32,
    pub total_contributions: i32,
    pub percentage_of_user_work: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CollaborationMatrix {
    pub users: Vec<User>,
    pub interactions: HashMap<String, HashMap<String, InteractionStats>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InteractionStats {
    pub reviews_given: i32, // User A reviewed User B's PRs
    pub reviews_received: i32, // User B reviewed User A's PRs
    pub co_authored_prs: i32,
    pub issue_interactions: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActivityDataPoint {
    pub timestamp: String,
    pub commit_count: i32,
    pub pr_count: i32,
    pub review_count: i32,
    pub issue_count: i32,
    pub total_activity: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamSummary {
    pub total_members: i32,
    pub total_commits: i32,
    pub total_prs: i32,
    pub total_reviews: i32,
    pub total_issues: i32,
    pub most_active_user: User,
    pub most_collaborative_pair: (User, User),
    pub average_activity_per_user: f64,
    pub team_velocity_trend: String, // "increasing", "stable", "decreasing"
}

#[derive(Debug, Clone, Serialize)]
pub struct FocusMetrics {
    pub repos_touched: i32,
    pub top_repo_percentage: f64, // % of work in most-worked repo
    pub concentration_score: f64, // 0-1, higher = more focused
    pub context_switches: i32, // Estimated switches between repos
}
```

### Database Queries

All data from existing tables, joined by user_id.

#### `src-tauri/src/db/user_queries.rs`

```rust
pub fn get_user_summary_data(
    conn: &Connection,
    user_id: i64,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<UserSummary> {
    // Aggregate across:
    // - pull_requests where author_id = user_id
    // - pr_reviews where reviewer_id = user_id
    // - issues where author_id = user_id
    // Count distinct repo_ids
}

pub fn get_user_activity_timeline(
    conn: &Connection,
    user_id: i64,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<Vec<TimelineEvent>> {
    // UNION of events across all repos where user participated
    // Similar to project timeline but filtered by user
}

pub fn get_user_repo_distribution(
    conn: &Connection,
    user_id: i64,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<Vec<RepositoryContribution>> {
    // GROUP BY repo_id
    // Count contributions per repo
    // Calculate percentages
}

pub fn get_collaboration_matrix(
    conn: &Connection,
    user_ids: Vec<i64>,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<CollaborationMatrix> {
    // Join pr_reviews with pull_requests
    // Count interactions between user pairs
}

pub fn get_user_activity_trend(
    conn: &Connection,
    user_id: i64,
    start_date: Option<&str>,
    end_date: Option<&str>,
    granularity: &str,
) -> Result<Vec<ActivityDataPoint>> {
    // GROUP BY date (truncated by granularity)
    // Count activities per period
}
```

### User Management Storage

#### Option 1: Local Storage (Frontend)
Store tracked usernames in localStorage:
```typescript
// src/stores/teamStore.ts
const trackedUsers = localStorage.getItem('trackedUsers') || '[]'
```

#### Option 2: Database (Backend)
Add `tracked_users` table:
```sql
CREATE TABLE tracked_users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    github_id INTEGER NOT NULL,
    login TEXT NOT NULL,
    added_at TEXT NOT NULL,
    UNIQUE(github_id)
);
```

**Recommendation**: Use Database (Option 2) for consistency across devices and better data integrity.

### New Zustand Store

```typescript
// src/stores/teamStore.ts
interface TeamStore {
    trackedUsers: string[], // GitHub usernames
    addUser: (username: string) => void,
    removeUser: (username: string) => void,
    clearUsers: () => void,
    importUsers: (usernames: string[]) => void,
}
```

## UI Mockup Structure

### Team View Page

```
┌─────────────────────────────────────────────────────────────┐
│  [Dashboard] [Team View] [Search] [Roadmap] [Settings]     │
├─────────────────────────────────────────────────────────────┤
│  👥 Team View                                               │
│                                                             │
│  Track specific users across all repositories              │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ Add users: alice, bob, carol                 [Add]  │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
│  Tracked Users (3)                    [Date: Last 30 days]│
│                                                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │  Team Summary                                        │  │
│  │  • 145 total commits  • 23 PRs  • 89 reviews        │  │
│  │  • Velocity: +12% ↑                                  │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌──────────────────┐ ┌──────────────────┐ ┌───────────┐ │
│  │  👤 Alice        │ │  👤 Bob          │ │  👤 Carol │ │
│  │  @alice          │ │  @bob            │ │  @carol   │ │
│  │                  │ │                  │ │           │ │
│  │  🟢 Active       │ │  🟡 Quiet        │ │  🟢 Active│ │
│  │  Last: 2h ago    │ │  Last: 3 days    │ │  Last: 1h │ │
│  │                  │ │                  │ │           │ │
│  │  45 Commits      │ │  32 Commits      │ │  68 Comm..│ │
│  │  12 PRs          │ │   8 PRs          │ │  15 PRs   │ │
│  │  23 Reviews      │ │  15 Reviews      │ │  31 Rev.. │ │
│  │   5 Issues       │ │   2 Issues       │ │   8 Iss.. │ │
│  │                  │ │                  │ │           │ │
│  │  3 Repos         │ │  2 Repos         │ │  4 Repos  │ │
│  │  [View Details]  │ │  [View Details]  │ │  [View]   │ │
│  │  [Remove]        │ │  [Remove]        │ │  [Remove] │ │
│  └──────────────────┘ └──────────────────┘ └───────────┘ │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  📊 Team Collaboration                                      │
│  ┌─────────────────────────────────────────────────────┐  │
│  │       Reviews Given →                                │  │
│  │        Alice   Bob    Carol                          │  │
│  │  Alice   -      5      12                            │  │
│  │  Bob     8      -       3                            │  │
│  │  Carol   15     7       -                            │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  📈 Activity Trends                                         │
│  ┌─────────────────────────────────────────────────────┐  │
│  │  [Line chart showing activity over time for 3 users]│  │
│  └─────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### User Detail Page

```
┌─────────────────────────────────────────────────────────────┐
│  < Back to Team View                                        │
│  Team > alice                                               │
├─────────────────────────────────────────────────────────────┤
│  👤 Alice (@alice)                              🟢 Active  │
│  GitHub profile link                                        │
│                                                             │
│  [Date Range: Last 30 days ▼]                              │
│                                                             │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  │    45    │ │    12    │ │    23    │ │     5    │     │
│  │ Commits  │ │   PRs    │ │  Reviews │ │  Repos   │     │
│  │  +15%    │ │   +8%    │ │   +12%   │ │    →     │     │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘     │
├─────────────────────────────────────────────────────────────┤
│  📦 Repository Distribution                                 │
│  ┌─────────────────────────────────────────────────────┐  │
│  │  backend-api      40% ████████                      │  │
│  │  frontend-app     30% ██████                        │  │
│  │  shared-utils     20% ████                          │  │
│  │  infrastructure   10% ██                            │  │
│  └─────────────────────────────────────────────────────┘  │
│  Focus Score: 0.7 (Moderately focused)                     │
├─────────────────────────────────────────────────────────────┤
│  📊 Activity Breakdown                                      │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐      │
│  │ Avg PR Size  │ │  Avg Time to │ │  Review Rate │      │
│  │   +234 -45   │ │     Merge    │ │   1.2 days   │      │
│  │              │ │   2.1 days   │ │              │      │
│  └──────────────┘ └──────────────┘ └──────────────┘      │
├─────────────────────────────────────────────────────────────┤
│  📋 Activity Timeline                                       │
│  [Same as Project Deep Dive timeline, filtered by user]    │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  📈 Activity Trend                                          │
│  ┌─────────────────────────────────────────────────────┐  │
│  │  [Line chart showing weekly activity]                │  │
│  └─────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Implementation Plan

### Phase 1: Core Infrastructure
1. Add navigation route for `/team`
2. Create `TeamView.tsx` page
3. Create `UserDetail.tsx` page
4. Add `tracked_users` database table
5. Create Tauri commands for user management (add/remove)

### Phase 2: User Cards & Summary
1. Build `UserManager` component
2. Build `UserCard` component
3. Implement `get_user_summary` Tauri command
4. Build `TeamSummary` component
5. Connect frontend to backend

### Phase 3: User Detail Page
1. Implement `get_user_activity_timeline` command
2. Build `UserTimeline` component (reuse from project deep dive)
3. Build `RepositoryDistribution` component
4. Implement navigation from card to detail

### Phase 4: Collaboration Features
1. Implement `get_team_collaboration_matrix` command
2. Build `CollaborationGraph` component
3. Add collaboration section to team view

### Phase 5: Trends & Analytics
1. Implement `get_user_activity_trend` command
2. Build `UserActivityTrend` component
3. Add comparison mode (multiple users)
4. Build `FocusAnalysis` component

### Phase 6: Polish & Export
1. Add date range filtering
2. Add export to PDF/CSV
3. Add share link generation
4. Performance optimization
5. Empty states and error handling
6. Responsive design

## Testing Strategy

### Unit Tests
- User summary calculations
- Repository distribution logic
- Collaboration matrix generation
- Activity trend aggregation
- Focus metrics calculations

### Integration Tests
- Add/remove users flow
- Navigation between views
- Filter interactions
- Data consistency across views

### Manual Testing Checklist
- [ ] Add users successfully
- [ ] Remove users successfully
- [ ] User cards show correct stats
- [ ] Click card navigates to detail
- [ ] Detail page shows complete data
- [ ] Repository distribution accurate
- [ ] Collaboration matrix correct
- [ ] Activity trends render correctly
- [ ] Date range filter works
- [ ] Empty states show properly
- [ ] Users with no data handled gracefully
- [ ] Export functions work
- [ ] Performance acceptable with 20 users

## Success Metrics
- 50% of users create at least one team view
- Average of 5 users tracked per team view
- Users check team view 2x per week
- Time to answer "What is X working on?" < 30 seconds

## Future Enhancements
1. GitHub org/team auto-import
2. Saved user groups (Backend Team, Frontend Team)
3. User goals and targets
4. Notifications for user activity changes
5. AI-generated user insights
6. PTO/vacation tracking integration
7. Manager dashboards with direct reports
8. Time allocation reports (what % time in each repo)
9. Skill inference from contribution patterns
10. Onboarding tracking for new team members

## Dependencies
- Existing database schema (add `tracked_users` table only)
- React Router for navigation
- Recharts for visualizations
- Existing Tauri infrastructure
- `users` table already exists with GitHub user data

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| User not in any tracked repos | MEDIUM | Clear "No data" state, suggest syncing repos |
| Privacy concerns tracking individuals | HIGH | Clear documentation, manager-only feature, opt-out? |
| Performance with many users (50+) | MEDIUM | Pagination, lazy loading, caching |
| Collaboration matrix complex with many users | MEDIUM | Limit to top N interactions, simplify visualization |

## Open Questions
1. Should users be able to hide themselves from tracking? - No
2. What defines "active" vs "quiet" vs "idle"? - Active is taking some sort of action 3+ days in the last 7 days. Quiet is 1-3 actions in the last 7 days. Idle is no actions in the last 7 days.
3. Should we track users who aren't in the tracked repos? - YES!
4. How to handle users who leave the organization? - You may presume we will update that manually. No need to worry about it.
5. Should there be role-based access (only managers see team view)? - Right now the project is run on a local machine, not a web app. I would like to creat a web app at some point. But all of this data is public and can be looked up manually. All we're doing is making it more easy to find. So no role-based access for now.
6. Should we auto-add users when syncing repos, or keep it manual? - Good idea! IF we sync a repo and find a new user we haven't sen before, alert the person viewing the app and ask if they'd like to begin tracking this person.

## Privacy & Ethics Considerations
- This feature tracks individual productivity metrics
- Must be used for support and understanding, not surveillance
- Consider adding disclaimer about appropriate use
- Consider making this opt-in or manager-only
- Should NOT be used for performance reviews without context
- Focus on patterns and collaboration, not raw numbers
