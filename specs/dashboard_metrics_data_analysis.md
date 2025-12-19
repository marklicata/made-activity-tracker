# Dashboard Metrics Data Analysis

## Current Database Schema Assessment

### What We Already Have ✅

Your existing database schema contains:

**Tables:**
- `repositories` - Tracked GitHub repositories
- `users` - GitHub users with tracking status
- `pull_requests` - **Complete PR data including:**
  - Timestamps: `created_at`, `merged_at`, `closed_at`, `updated_at`
  - Metrics: `additions`, `deletions`, `changed_files`, `review_comments`
  - Classification: `labels`, `title`, `body`
  - State tracking: `state` (open/merged/closed)
- `pr_reviews` - Review activity with timestamps
- `issues` - Issue tracking
- `milestones` - Cycle/sprint tracking
- `squads` / `squad_members` - Team organization
- `metrics_snapshots` - Pre-computed metrics storage

### What We're Missing ❌

**Critical gap: NO COMMITS TABLE**

Your code currently returns:
```rust
total_commits: 0, // We don't track individual commits yet
```

---

## Metrics We CAN Calculate (Without Schema Changes)

### SPEED Metrics

#### ✅ PR Turnaround Time
**Calculation:** `merged_at - created_at` for merged PRs
```sql
SELECT
  AVG(CAST((julianday(merged_at) - julianday(created_at)) * 24 AS REAL)) as avg_hours
FROM pull_requests
WHERE merged_at IS NOT NULL
  AND created_at > date('now', '-30 days')
```
**Data source:** `pull_requests.created_at`, `pull_requests.merged_at`
**Benchmark ready:** ✅ Yes

#### ✅ PR Cycle Time Distribution
**What % of PRs merge within:** < 4h, 4-12h, 12-24h, > 24h
```sql
SELECT
  CASE
    WHEN hours_to_merge < 4 THEN '< 4h'
    WHEN hours_to_merge < 12 THEN '4-12h'
    WHEN hours_to_merge < 24 THEN '12-24h'
    ELSE '> 24h'
  END as time_bucket,
  COUNT(*) as count,
  ROUND(COUNT(*) * 100.0 / SUM(COUNT(*)) OVER(), 1) as percentage
FROM (
  SELECT (julianday(merged_at) - julianday(created_at)) * 24 as hours_to_merge
  FROM pull_requests
  WHERE merged_at IS NOT NULL
)
GROUP BY time_bucket
```
**Data source:** `pull_requests` timestamps

#### ✅ Lines of Code per Day (PR-based)
**Calculation:** Sum of additions/deletions per day
```sql
SELECT
  DATE(created_at) as day,
  SUM(additions + deletions) as total_loc,
  COUNT(DISTINCT author_id) as active_authors
FROM pull_requests
WHERE created_at > date('now', '-30 days')
GROUP BY DATE(created_at)
```
**Data source:** `pull_requests.additions`, `pull_requests.deletions`
**Note:** This is PR-based LOC, not commit-based, but still valid metric

#### ❌ Commits per Day
**Problem:** No commits table exists
**Workaround:** Use "PRs per day" as proxy metric
```sql
SELECT
  COUNT(*) / 30.0 as prs_per_day_per_dev,
  COUNT(DISTINCT DATE(created_at)) as active_days,
  COUNT(*) * 1.0 / COUNT(DISTINCT DATE(created_at)) as prs_per_active_day
FROM pull_requests
WHERE created_at > date('now', '-30 days')
  AND author_id IN (SELECT id FROM users WHERE tracked = 1)
```
**Alternative metric:** "PRs opened per day" (typically elite teams do 0.8-1.5 PRs/day)

#### ❌ Commit Frequency (39.3% within 4 hours)
**Problem:** Requires commit timestamps
**Not calculable** without commits table

### EASE Metrics

#### ✅ Concurrent Project Capacity
**Calculation:** Count of repositories with activity in period
```sql
SELECT
  COUNT(DISTINCT repo_id) as concurrent_repos,
  COUNT(DISTINCT repo_id) * 1.0 / COUNT(DISTINCT author_id) as repos_per_dev
FROM pull_requests
WHERE created_at > date('now', '-30 days')
  AND author_id IN (SELECT id FROM users WHERE tracked = 1)
```
**Data source:** `pull_requests.repo_id`, `pull_requests.author_id`
**Benchmark ready:** ✅ Yes (compare to 2.1 industry / 3.5 elite)

#### ✅ Active Repositories List
**Show which repos and activity counts**
```sql
SELECT
  r.owner || '/' || r.name as repo_name,
  COUNT(DISTINCT pr.id) as pr_count,
  SUM(pr.additions + pr.deletions) as total_loc,
  COUNT(DISTINCT pr.author_id) as contributor_count,
  MAX(pr.created_at) as last_activity
FROM pull_requests pr
JOIN repositories r ON pr.repo_id = r.id
WHERE pr.created_at > date('now', '-30 days')
GROUP BY r.id, r.owner, r.name
ORDER BY pr_count DESC
```
**Data source:** `pull_requests`, `repositories`

#### ✅ Repository Distribution (Org vs Personal)
**Calculation:** Count repos by owner
```sql
SELECT
  CASE
    WHEN r.owner = 'microsoft' THEN 'Organization'
    ELSE 'Personal'
  END as repo_type,
  COUNT(DISTINCT r.id) as repo_count
FROM repositories r
JOIN pull_requests pr ON pr.repo_id = r.id
WHERE pr.created_at > date('now', '-30 days')
GROUP BY repo_type
```
**Data source:** `repositories.owner`

#### ❌ Context Switch Frequency
**Problem:** Requires sequential commit data
**Workaround:** Calculate PR switching frequency (less granular but still useful)
```sql
-- Show how often consecutive PRs are in different repos
WITH ordered_prs AS (
  SELECT
    author_id,
    repo_id,
    created_at,
    LAG(repo_id) OVER (PARTITION BY author_id ORDER BY created_at) as prev_repo_id
  FROM pull_requests
  WHERE created_at > date('now', '-30 days')
)
SELECT
  COUNT(CASE WHEN repo_id != prev_repo_id THEN 1 END) * 100.0 / COUNT(*) as switch_percentage
FROM ordered_prs
WHERE prev_repo_id IS NOT NULL
```
**Alternative metric:** "PR context switches" instead of commit-level

#### ✅ Work Pattern Heatmap
**Can visualize by:** Hour of day, day of week for PR creation
```sql
SELECT
  CAST(strftime('%w', created_at) AS INTEGER) as day_of_week, -- 0=Sunday
  CAST(strftime('%H', created_at) AS INTEGER) as hour_of_day,
  COUNT(*) as activity_count
FROM pull_requests
WHERE created_at > date('now', '-30 days')
GROUP BY day_of_week, hour_of_day
```
**Data source:** `pull_requests.created_at`
**Note:** Shows PR creation patterns, not commit patterns

### QUALITY Metrics

#### ✅ PR Merge Rate
**Calculation:** Merged PRs / All PRs with resolution
```sql
SELECT
  COUNT(CASE WHEN merged_at IS NOT NULL THEN 1 END) * 100.0 /
  COUNT(CASE WHEN state != 'open' THEN 1 END) as merge_rate
FROM pull_requests
WHERE created_at > date('now', '-30 days')
  AND state != 'open' -- Only count closed PRs (merged or rejected)
```
**Data source:** `pull_requests.merged_at`, `pull_requests.state`
**Benchmark ready:** ✅ Yes (68% avg / 85% elite / 90%+ target)

#### ✅ PR Merge Rate Trend
**Show trend over time**
```sql
SELECT
  DATE(created_at, 'start of week') as week,
  COUNT(CASE WHEN merged_at IS NOT NULL THEN 1 END) * 100.0 /
  COUNT(CASE WHEN state != 'open' THEN 1 END) as merge_rate
FROM pull_requests
WHERE created_at > date('now', '-90 days')
  AND state != 'open'
GROUP BY week
ORDER BY week
```

#### ✅ Files per PR
**Calculation:** Average changed_files
```sql
SELECT
  AVG(changed_files) as avg_files,
  MIN(changed_files) as min_files,
  MAX(changed_files) as max_files,
  COUNT(*) as total_prs
FROM pull_requests
WHERE created_at > date('now', '-30 days')
```
**Data source:** `pull_requests.changed_files`
**Benchmark ready:** ✅ Yes (8 files industry avg)

#### ✅ Files per PR Distribution
**Show histogram: How many PRs have 1-3, 4-8, 9-15, 16+ files**
```sql
SELECT
  CASE
    WHEN changed_files <= 3 THEN '1-3'
    WHEN changed_files <= 8 THEN '4-8'
    WHEN changed_files <= 15 THEN '9-15'
    ELSE '16+'
  END as file_range,
  COUNT(*) as pr_count,
  ROUND(COUNT(*) * 100.0 / SUM(COUNT(*)) OVER(), 1) as percentage
FROM pull_requests
WHERE created_at > date('now', '-30 days')
GROUP BY file_range
ORDER BY MIN(changed_files)
```

#### ⚠️ Bug Fix Ratio (Limited)
**Problem:** Requires commit message analysis
**Workaround:** Analyze PR titles and labels
```sql
SELECT
  COUNT(CASE
    WHEN title LIKE '%fix%' OR title LIKE '%bug%'
      OR labels LIKE '%bug%' OR labels LIKE '%fix%'
    THEN 1
  END) * 100.0 / COUNT(*) as bug_pr_percentage
FROM pull_requests
WHERE created_at > date('now', '-30 days')
```
**Data source:** `pull_requests.title`, `pull_requests.labels`
**Limitation:** Less accurate than commit-message-based classification
**Benchmark:** Still comparable (25% industry / 15% elite / <10% target)

#### ⚠️ Feature Work Percentage (Limited)
**Workaround:** Analyze PR titles and labels for feature keywords
```sql
SELECT
  CASE
    WHEN title LIKE '%feat%' OR title LIKE '%feature%' OR title LIKE '%add%'
      OR labels LIKE '%feature%' OR labels LIKE '%enhancement%'
    THEN 'feature'
    WHEN title LIKE '%fix%' OR title LIKE '%bug%'
      OR labels LIKE '%bug%'
    THEN 'bug_fix'
    WHEN title LIKE '%refactor%' OR title LIKE '%improve%'
    THEN 'refactor'
    WHEN title LIKE '%test%' OR title LIKE '%spec%'
    THEN 'test'
    WHEN title LIKE '%doc%' OR labels LIKE '%documentation%'
    THEN 'docs'
    ELSE 'other'
  END as pr_type,
  COUNT(*) as count,
  ROUND(COUNT(*) * 100.0 / SUM(COUNT(*)) OVER(), 1) as percentage
FROM pull_requests
WHERE created_at > date('now', '-30 days')
GROUP BY pr_type
```
**Data source:** `pull_requests.title`, `pull_requests.labels`
**Limitation:** Classification based on PR, not individual commits
**Still useful:** Shows overall work distribution

#### ✅ Review Cycle Time
**Calculation:** Time from PR creation to first review
```sql
SELECT
  AVG(CAST((julianday(r.submitted_at) - julianday(pr.created_at)) * 24 AS REAL)) as avg_hours_to_first_review
FROM pull_requests pr
JOIN pr_reviews r ON r.pr_id = pr.id
WHERE pr.created_at > date('now', '-30 days')
  AND r.submitted_at = (
    SELECT MIN(submitted_at)
    FROM pr_reviews
    WHERE pr_id = pr.id
  )
```
**Data source:** `pr_reviews.submitted_at`, `pull_requests.created_at`
**Benchmark:** Elite target < 4 hours

#### ✅ Review Comments per PR
**Already captured!**
```sql
SELECT AVG(review_comments) as avg_comments
FROM pull_requests
WHERE created_at > date('now', '-30 days')
```
**Data source:** `pull_requests.review_comments`

---

## Schema Changes Needed for Full Metrics

### Option 1: Add Commits Table (Comprehensive)

If you want **commit-level metrics** (like the Amplifier analysis), you need:

```sql
CREATE TABLE IF NOT EXISTS commits (
    id INTEGER PRIMARY KEY,
    sha TEXT UNIQUE NOT NULL,
    repo_id INTEGER NOT NULL REFERENCES repositories(id),
    author_id INTEGER REFERENCES users(id),
    committer_id INTEGER REFERENCES users(id),
    message TEXT NOT NULL,
    committed_at TEXT NOT NULL,
    additions INTEGER DEFAULT 0,
    deletions INTEGER DEFAULT 0,
    changed_files INTEGER DEFAULT 0,
    pr_id INTEGER REFERENCES pull_requests(id), -- Link to PR if available
    parent_shas TEXT, -- JSON array of parent commit SHAs
    UNIQUE(repo_id, sha)
);

CREATE INDEX IF NOT EXISTS idx_commits_repo ON commits(repo_id);
CREATE INDEX IF NOT EXISTS idx_commits_author ON commits(author_id);
CREATE INDEX IF NOT EXISTS idx_commits_date ON commits(committed_at);
CREATE INDEX IF NOT EXISTS idx_commits_pr ON commits(pr_id);
```

**Benefits:**
- ✅ True commits per day metric
- ✅ Commit frequency analysis (time between commits)
- ✅ Accurate bug fix ratio (analyze commit messages)
- ✅ Accurate feature work percentage
- ✅ Context switching at commit level
- ✅ Full compatibility with Amplifier analysis methodology

**Costs:**
- ❌ Large data volume (commits >> PRs)
- ❌ More GitHub API calls to fetch commits
- ❌ Slower sync times
- ❌ More storage needed

### Option 2: PR-Based Metrics Only (Recommended)

**Skip the commits table** and use PR-based proxies:

**What changes:**
- "Commits per day" → **"PRs per day"** (0.8-1.5 for elite teams)
- "Commit frequency" → **"PR creation frequency"**
- "Bug fix ratio" → **"Bug PR percentage"** (based on PR titles/labels)
- "Feature work %" → **"Feature PR percentage"** (based on PR titles/labels)
- "Lines of code" → **"PR LOC"** (already have additions/deletions)

**Schema changes needed:** **NONE** ✅

**But you could add these optional enhancements:**

```sql
-- Optional: Better PR classification
ALTER TABLE pull_requests ADD COLUMN pr_type TEXT; -- 'feature', 'bugfix', 'refactor', 'docs', 'test'
CREATE INDEX IF NOT EXISTS idx_prs_type ON pull_requests(pr_type);

-- Optional: First review timestamp (denormalized for performance)
ALTER TABLE pull_requests ADD COLUMN first_reviewed_at TEXT;
CREATE INDEX IF NOT EXISTS idx_prs_first_review ON pull_requests(first_reviewed_at);
```

These would be **populated during sync**, not requiring new API calls:
- `pr_type`: Classify based on labels and title during sync
- `first_reviewed_at`: Calculate from `pr_reviews` during sync

### Option 3: Hybrid Approach

Track **commits only for PRs** (not all commits):

```sql
CREATE TABLE IF NOT EXISTS pr_commits (
    id INTEGER PRIMARY KEY,
    sha TEXT NOT NULL,
    pr_id INTEGER NOT NULL REFERENCES pull_requests(id),
    message TEXT NOT NULL,
    committed_at TEXT NOT NULL,
    author_id INTEGER REFERENCES users(id),
    UNIQUE(pr_id, sha)
);

CREATE INDEX IF NOT EXISTS idx_pr_commits_pr ON pr_commits(pr_id);
CREATE INDEX IF NOT EXISTS idx_pr_commits_date ON pr_commits(committed_at);
```

**Benefits:**
- ✅ More granular activity data within PRs
- ✅ Can analyze commit frequency within PR development
- ✅ Better commit message classification
- ✅ Smaller dataset than all commits
- ✅ Can calculate "commits per PR" metric

**Costs:**
- ⚠️ Still requires GitHub API calls (to get PR commits)
- ⚠️ Misses commits not associated with PRs
- ⚠️ More complex to query (need to join through PRs)

---

## Recommended Approach

### Phase 1: Use PR-Based Metrics (No Schema Changes)

**Implement immediately with existing data:**

1. **Speed:**
   - PR turnaround time ✅
   - PRs per day (proxy for commits/day) ✅
   - PR cycle time distribution ✅
   - LOC per day from PRs ✅

2. **Ease:**
   - Concurrent project capacity ✅
   - Active repositories ✅
   - Repository distribution ✅
   - Work pattern heatmap (PR-based) ✅

3. **Quality:**
   - PR merge rate ✅
   - Files per PR ✅
   - Bug PR percentage (from titles/labels) ✅
   - Feature PR percentage ✅
   - Review cycle time ✅

**Adjust benchmark comparisons:**
- Instead of "3.7 commits/day vs 1.2 industry"
- Show "1.2 PRs/day vs 0.8 industry" (still impressive!)

### Phase 2: Add Optional Enhancements (Minimal Changes)

Add these columns to `pull_requests`:
```sql
ALTER TABLE pull_requests ADD COLUMN pr_type TEXT;
ALTER TABLE pull_requests ADD COLUMN first_reviewed_at TEXT;
```

Populate during sync:
- Classify PR type from labels/title
- Calculate first review time from `pr_reviews`

### Phase 3: Evaluate Commits Table (Future)

**Only add commits if:**
- Users specifically request commit-level granularity
- You want to track work outside of PRs (direct-to-main commits)
- You need exact Amplifier methodology compatibility

**Consider the tradeoffs:**
- Much more data to sync and store
- Slower sync times
- More API rate limit usage
- Minimal benefit if all work goes through PRs

---

## API Call Analysis

### Current Sync (No Commits)
Per repository:
- 1 call: List PRs
- 1 call per PR: Get PR details (files, additions, deletions)
- 1 call: List issues
- 1 call: List milestones

**Rate limit impact:** Low-moderate (depends on # of PRs)

### With Commits Table (Option 1)
Additional calls per repository:
- 1 call: List all commits (paginated, could be 100+ pages for active repo)
- OR X calls: Get commits per PR (more targeted)

**Rate limit impact:** **HIGH** - Could hit GitHub rate limits quickly

### Hybrid (Option 3)
Additional calls:
- 1 call per PR: List PR commits

**Rate limit impact:** Moderate (proportional to PR count, not total commits)

---

## Summary & Recommendation

### ✅ You CAN calculate these metrics now (no schema changes):
- PR Turnaround Time
- PRs per Day (proxy for commits)
- Lines of Code per Day
- Concurrent Project Capacity
- Active Repositories
- PR Merge Rate
- Files per PR
- Review Cycle Time
- Work Pattern Heatmaps

### ⚠️ These metrics need approximation (PR-level, not commit-level):
- Bug Fix Ratio → Use PR classification
- Feature Work % → Use PR classification
- Context Switching → Use PR switching (less granular)

### ❌ These metrics require commits table:
- True "Commits per Day" metric
- "Commit Frequency" (% within 4 hours)
- Commit-message-based classification

### 💡 My Recommendation:

**Start with PR-based metrics (Phase 1 - no schema changes)**

This gives you:
- 80% of the value
- 0% additional API calls
- Immediate implementation
- All key benchmarks still work

**Then add optional columns (Phase 2 - minimal changes)**

This gets you to:
- 90% of the value
- Better classification accuracy
- Still no new API calls
- Easy to implement during next sync

**Only add commits table if you really need it (Phase 3)**

Would you like me to:
1. Create the SQL queries for the PR-based dashboard metrics?
2. Show how to adjust the benchmarks for PR-based metrics?
3. Design the enhanced PR columns and classification logic?
4. Implement a commits table with sync logic?
