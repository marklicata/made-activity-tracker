# Dashboard Metrics Integration Spec

## Overview
This spec defines how to integrate the metrics from the Amplifier Impact Analysis into the existing Made Activity Tracker dashboard, organizing them into the Speed, Ease, and Quality framework.

## Design Philosophy
- **Data-driven insights**: Surface meaningful comparisons against industry benchmarks
- **Progressive disclosure**: Show high-level metrics with ability to drill into details
- **Actionable intelligence**: Help users understand their productivity patterns
- **Visual clarity**: Use charts and visualizations to make trends obvious

---

## Metric Categories

### SPEED Metrics
Speed measures how fast work gets done - velocity, throughput, and cycle time.

#### Primary Metrics

**1. Commits per Day**
- **Definition**: Average number of commits per developer per day
- **Calculation**: Total commits / (number of developers × days in period)
- **Benchmarks**:
  - Industry Average: 1.2
  - Elite Performers: 2.5
  - Amplifier Target: 3.7+
- **Display**: Large metric card with trend line and benchmark comparison bar
- **Color coding**:
  - Red: < 1.2 (below industry)
  - Yellow: 1.2-2.5 (industry to elite)
  - Green: > 2.5 (elite tier)

**2. PR Turnaround Time**
- **Definition**: Average time from PR creation to merge
- **Calculation**: Average hours between PR opened and merged timestamps
- **Benchmarks**:
  - Industry Average: 89h
  - Elite Performers: 24h
  - Amplifier Target: < 15h
- **Display**: Time metric with comparison to previous period
- **Breakdown**: Show distribution (< 4h, 4-12h, 12-24h, > 24h)

**3. Lines of Code per Day**
- **Definition**: Net lines added per developer per day
- **Calculation**: (Total additions - deletions) / (developers × days)
- **Benchmarks**:
  - Industry Average: 500
  - Elite Performers: 1,500
  - Amplifier Target: 3,000+
- **Display**: Bar chart with benchmark overlay
- **Note**: Include caveat that this is an output metric, not a quality metric

**4. Commit Frequency**
- **Definition**: Percentage of commits happening within 4 hours of previous commit
- **Calculation**: (Commits with < 4h gap / total commits) × 100
- **Target**: > 35% indicates high momentum
- **Display**: Percentage with time-of-day heatmap
- **Insight**: Higher percentage = better flow state, less context switching

#### Secondary Metrics

**5. Deployment Frequency**
- Track if available from git tags or CI/CD
- Elite target: Multiple deploys per day

**6. Lead Time for Changes**
- Time from commit to production
- Elite target: < 1 day

---

### EASE Metrics
Ease measures capacity for parallel work, sustainability, and reduced friction.

#### Primary Metrics

**1. Concurrent Project Capacity**
- **Definition**: Number of repositories with commits in the measurement period
- **Calculation**: Count of unique repos with activity / number of developers
- **Benchmarks**:
  - Industry Average: 2.1
  - Elite Performers: 3.5
  - Amplifier Target: 10+
- **Display**: Metric card with list of active projects
- **Breakdown**: Show which repos and commit counts

**2. Context Switch Frequency**
- **Definition**: How often developers switch between repositories
- **Calculation**: Number of times consecutive commits are in different repos
- **Display**: Daily pattern chart
- **Insight**: Lower switching = better sustained focus (BUT higher capacity with Amplifier means more switching is actually good)

**3. Active Repositories**
- **Definition**: Total number of repositories with any activity
- **Time periods**: 7d, 30d, 90d, all-time
- **Display**: Trend over time with breakdown by repo type (personal vs org)

**4. Repository Distribution**
- **Breakdown by type**:
  - Core organization repos
  - Personal projects
  - Community/fork contributions
- **Display**: Pie chart or stacked bar

#### Secondary Metrics

**5. Average Commits per Repository**
- Shows depth of engagement per project
- Helps identify deep work vs breadth of contributions

**6. Weekend/Evening Activity Ratio**
- Sustainability indicator
- Target: < 20% to avoid burnout
- Display: Weekly rhythm heatmap

---

### QUALITY Metrics
Quality measures the standard of work - merge success, bug ratio, and feature focus.

#### Primary Metrics

**1. PR Merge Rate**
- **Definition**: Percentage of PRs that get merged (vs closed/abandoned)
- **Calculation**: (Merged PRs / Total PRs) × 100
- **Benchmarks**:
  - Industry Average: 68%
  - Elite Performers: 85%
  - Amplifier Target: 90%+
- **Display**: Percentage with trend line
- **Breakdown**: Show reasons for non-merged PRs if available

**2. Bug Fix Ratio**
- **Definition**: Percentage of commits that are bug fixes
- **Calculation**: (Bug fix commits / total commits) × 100
- **Classification**: Use commit message keywords: "fix", "bug", "patch", "hotfix"
- **Benchmarks**:
  - Industry Average: 25%
  - Elite Performers: 15%
  - Amplifier Target: < 10%
- **Display**: Percentage card with trend
- **Lower is better**: Indicates fewer bugs or higher first-time quality

**3. Feature Work Percentage**
- **Definition**: Percentage of commits adding new functionality
- **Calculation**: (Feature commits / total commits) × 100
- **Classification**: Keywords: "feat", "add", "implement", "new"
- **Benchmarks**:
  - Industry Average: 45%
  - Elite Performers: 55%
  - Amplifier Target: 60%+
- **Display**: Pie chart of commit types
- **Higher is better**: More time on valuable new features

**4. Files per PR**
- **Definition**: Average number of files changed per PR
- **Calculation**: Total files changed / number of PRs
- **Benchmark**: Industry average: 8 files
- **Display**: Average with distribution histogram
- **Interpretation**:
  - Too low (< 3): Might be too granular
  - Good range (3-15): Well-scoped changes
  - Too high (> 20): May be hard to review

#### Secondary Metrics

**5. Review Cycle Time**
- Time from PR creation to first review
- Elite target: < 4 hours

**6. Code Churn Rate**
- Lines of code rewritten within 3 weeks
- Target: < 20% (indicates stable, well-thought-out code)

**7. Commit Message Quality Score**
- Analysis of commit message patterns
- Criteria: Length, formatting, conventional commits, issue references

---

## Dashboard Layout Proposal

### Top-Level Dashboard View

```
┌─────────────────────────────────────────────────────────────┐
│  Made Activity Tracker                    [Date Range: 30d] │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   SPEED     │  │    EASE     │  │   QUALITY   │         │
│  ├─────────────┤  ├─────────────┤  ├─────────────┤         │
│  │             │  │             │  │             │         │
│  │    3.2      │  │    12.4     │  │    91%      │         │
│  │ commits/day │  │ concurrent  │  │  PR merge   │         │
│  │             │  │   projects  │  │    rate     │         │
│  │   [vs 1.2]  │  │  [vs 2.1]   │  │  [vs 68%]   │         │
│  │             │  │             │  │             │         │
│  │  ▲ +15%     │  │  ▲ +8%      │  │  ▼ -2%      │         │
│  │  vs prev    │  │  vs prev    │  │  vs prev    │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Productivity Multiplier: 9.2×                        │   │
│  │  [===================================      ] vs avg   │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### Speed Section (Expanded)

```
┌────────────────── SPEED ───────────────────┐
│                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│  │   3.2    │  │  14.2h   │  │  4,234   │ │
│  │commits/d │  │PR cycle  │  │ LOC/day  │ │
│  └──────────┘  └──────────┘  └──────────┘ │
│                                             │
│  Commits per Day Trend                      │
│  ┌────────────────────────────────────────┐│
│  │         •••                             ││
│  │      •••   •••                          ││
│  │   •••         •••                       ││
│  │•••               ••                     ││
│  └────────────────────────────────────────┘│
│  Week 1    Week 2    Week 3    Week 4      │
│                                             │
│  Benchmark Comparison                       │
│  Industry (1.2)  [▓▓▓░░░░░░░]              │
│  Elite (2.5)     [▓▓▓▓▓▓▓░░░]              │
│  You (3.2)       [▓▓▓▓▓▓▓▓░░] 🎯           │
│                                             │
│  Commit Frequency Distribution              │
│  < 4h   [████████████████] 42%             │
│  4-12h  [██████████] 28%                   │
│  12-24h [█████] 18%                        │
│  > 24h  [███] 12%                          │
│                                             │
└─────────────────────────────────────────────┘
```

### Ease Section (Expanded)

```
┌────────────────── EASE ────────────────────┐
│                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│  │   12.4   │  │    23    │  │   0.42   │ │
│  │concurrent│  │  active  │  │ switches │ │
│  │ projects │  │  repos   │  │ per day  │ │
│  └──────────┘  └──────────┘  └──────────┘ │
│                                             │
│  Active Repositories (Last 30 Days)         │
│  ┌────────────────────────────────────────┐│
│  │ made-activity-tracker        45 commits││
│  │ amplifier-core                32 commits││
│  │ ai-chat-panel                 28 commits││
│  │ module-resolution             12 commits││
│  │ [... 8 more]           [View all →]    ││
│  └────────────────────────────────────────┘│
│                                             │
│  Repository Distribution                    │
│  ┌────────────────────────────────────────┐│
│  │ ██████████████░░░░░░░░░░░░              ││
│  │ 62% Org Repos   |   38% Personal       ││
│  └────────────────────────────────────────┘│
│                                             │
│  Work Pattern (Last 7 Days)                 │
│     M   T   W   T   F   S   S              │
│ AM  ██  ███ ██  ███ ██  ░░  ░░             │
│ PM  ███ ██  ███ ██  ███ █░  ░░             │
│ EVE ░░  █░  ░░  ░░  ░░  ░░  ░░             │
│                                             │
└─────────────────────────────────────────────┘
```

### Quality Section (Expanded)

```
┌───────────────── QUALITY ──────────────────┐
│                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│  │   91%    │  │   8.2%   │  │  62.4%   │ │
│  │PR merge  │  │   bug    │  │ feature  │ │
│  │   rate   │  │   ratio  │  │   work   │ │
│  └──────────┘  └──────────┘  └──────────┘ │
│                                             │
│  Commit Type Breakdown                      │
│  ┌────────────────────────────────────────┐│
│  │                                         ││
│  │  Features    [██████████████] 62.4%    ││
│  │  Refactor    [█████] 15.2%             ││
│  │  Docs        [███] 9.8%                ││
│  │  Bug Fixes   [██] 8.2%                 ││
│  │  Tests       [█] 4.4%                  ││
│  │                                         ││
│  └────────────────────────────────────────┘│
│                                             │
│  PR Merge Rate Trend                        │
│  ┌────────────────────────────────────────┐│
│  │ 95% ••••••••••••••••                   ││
│  │     •                •                  ││
│  │ 90% •                  •••••            ││
│  │ 85% ───────────────────────────Elite   ││
│  │ 68% ───────────────────────────Avg     ││
│  └────────────────────────────────────────┘│
│                                             │
│  Files per PR Distribution                  │
│  8.7 files/PR average                       │
│  1-3   [██] 15%                            │
│  4-8   [█████████] 48%                     │
│  9-15  [█████] 25%                         │
│  16+   [██] 12%                            │
│                                             │
└─────────────────────────────────────────────┘
```

---

## Implementation Phases

### Phase 1: Core Metrics (Week 1-2)
**Goal**: Get the foundation in place with data collection

1. **Backend Data Collection**
   - Add database schema for new metrics
   - Build aggregation queries for Speed/Ease/Quality metrics
   - Create API endpoints for metric retrieval
   - Implement caching for expensive calculations

2. **Basic Dashboard Cards**
   - Implement top-level Speed/Ease/Quality cards
   - Add simple trend indicators (up/down vs previous period)
   - Show primary metrics only

### Phase 2: Benchmarking & Comparisons (Week 3)
**Goal**: Add context through industry comparisons

1. **Benchmark Data**
   - Store industry/elite benchmark values in config
   - Add comparison visualization components
   - Implement relative scoring system (below/at/above benchmarks)

2. **Enhanced Metrics Display**
   - Add benchmark comparison bars
   - Color coding based on performance tier
   - Add "productivity multiplier" calculation

### Phase 3: Detailed Views (Week 4)
**Goal**: Enable drill-down into metric details

1. **Expandable Sections**
   - Click Speed/Ease/Quality cards to expand
   - Show secondary metrics
   - Add distribution charts and histograms

2. **Time-based Filtering**
   - Add date range selector (7d, 30d, 90d, all-time)
   - Show trend lines over time
   - Compare current period to previous period

### Phase 4: Advanced Insights (Week 5-6)
**Goal**: Provide actionable intelligence

1. **Pattern Analysis**
   - Work pattern heatmaps (time of day, day of week)
   - Commit clustering analysis
   - Identify productivity patterns

2. **Recommendations**
   - Suggest focus areas based on metrics
   - Highlight improvements or regressions
   - Set personal targets vs benchmarks

---

## Data Sources & Calculations

### Required Data Points

From Git/GitHub:
- Commit SHA, timestamp, author, message, files changed
- Lines added/deleted per commit
- PR number, created_at, merged_at, closed_at, files changed
- Repository name, organization

From Database:
- User records (for multi-developer normalization)
- Project associations
- Time period tracking

### Derived Metrics Calculations

**Velocity Multiplier Formula**:
```
velocity_multiplier = (
  (commits_per_day / 1.2) * 0.4 +
  (concurrent_projects / 2.1) * 0.3 +
  (LOC_per_day / 500) * 0.2 +
  (89 / PR_turnaround_hours) * 0.1
)
```
Weights can be adjusted based on what matters most to the team.

**Commit Classification**:
```sql
CASE
  WHEN message ILIKE '%fix%' OR message ILIKE '%bug%' THEN 'bug_fix'
  WHEN message ILIKE '%feat%' OR message ILIKE '%add%' OR message ILIKE '%implement%' THEN 'feature'
  WHEN message ILIKE '%refactor%' OR message ILIKE '%improve%' THEN 'refactor'
  WHEN message ILIKE '%test%' OR message ILIKE '%spec%' THEN 'test'
  WHEN message ILIKE '%doc%' OR message ILIKE '%readme%' THEN 'docs'
  ELSE 'other'
END
```

---

## UI/UX Considerations

### Visual Design
- Use color psychology:
  - Speed: Blue (motion, progress)
  - Ease: Green (growth, sustainability)
  - Quality: Purple (excellence, craft)
- Consistent metric card styling across all three categories
- Clear visual hierarchy: primary metrics larger, secondary smaller

### Interaction Patterns
- Hover on metric cards to see definition and calculation method
- Click to expand into detailed view
- Tooltips explain benchmarks and targets
- Smooth transitions between time periods

### Performance
- Cache aggregated metrics (refresh every 15 minutes)
- Lazy load detailed charts only when section expanded
- Use progressive enhancement for complex visualizations

### Accessibility
- Screen reader friendly labels
- Keyboard navigation for all interactive elements
- High contrast mode support
- Clear text alternatives for charts

---

## Success Metrics for This Feature

How will we know this dashboard integration is successful?

1. **User Engagement**
   - % of users viewing dashboard weekly (target: 80%+)
   - Average time spent on dashboard (target: 2+ min)
   - Feature adoption rate (target: 90%+ users expand at least one section)

2. **Behavioral Changes**
   - Users cite specific metrics in discussions
   - Improvement in user's own Speed/Ease/Quality scores over time
   - Increased commits, PRs, or other desired behaviors

3. **System Performance**
   - Dashboard loads in < 2 seconds
   - Metric calculations don't impact other system performance
   - API response times < 500ms for metric endpoints

---

## Open Questions

1. **Should we allow users to customize benchmark values?**
   - Some teams may want to set their own targets.
   - Or compare against their own historical baseline
   - [ANSWER] No, let's put these in a config and let them be defined for the team.

2. **How do we handle multi-user teams?**
   - Aggregate view vs individual views - Aggregate views. We can look at individual users separately.
   - Leaderboards? (Could be demotivating) - No leaderboards
   - Team averages with individual breakdowns - Yes

3. **What about non-code contributions?**
   - PR reviews, documentation, design work - I would like to capture work that goes into REPOS. So reviews, documents, etc. all count.
   - How to quantify and include in metrics - Just like any other code file. Documentation can be lines written. PR reviews are a little harder...

4. **Privacy considerations?**
   - What if users don't want to be compared? - Doesn't matter, they're going to be.
   - Option to hide personal metrics from team views - No.
   - Anonymous aggregated insights only - No.

5. **GitHub vs local git data?**
   - Some metrics require GitHub API (PR data)
   - Can we fall back gracefully if GitHub unavailable?
   - What about non-GitHub git workflows?
   Let's identify when GitHub APIs are required. Can we use the CLI? Can we use a CURL command? Can we just load the website and somehow scrape the data?

---

## Appendix: Metric Definitions Reference

### Industry Benchmarks Source
- GitHub Octoverse 2023
- Google DORA State of DevOps Research
- LinearB Engineering Benchmarks 2024

### Metric Tiers
- **Below Average**: Red warning indicators
- **Industry Average**: Orange/yellow indicators
- **Elite Performers**: Blue indicators
- **Amplifier Target**: Green/teal indicators (exceeds elite)

### Time Windows
- **Real-time**: Last commit info
- **7 days**: Weekly patterns
- **30 days**: Monthly velocity (primary)
- **90 days**: Quarterly trends
- **All-time**: Historical total

---

## Next Steps

1. Review this spec with team for feedback
2. Validate benchmark values against actual industry research
3. Create wireframes/mockups for each section
4. Prioritize Phase 1 features for immediate implementation
5. Set up data collection infrastructure
6. Begin implementation with Speed metrics first
