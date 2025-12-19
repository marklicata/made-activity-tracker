use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

// ============================================================================
// METRIC MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetrics {
    pub speed: SpeedMetrics,
    pub ease: EaseMetrics,
    pub quality: QualityMetrics,
    pub overview: OverviewMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewMetrics {
    pub productivity_multiplier: f64,
    pub period_days: i32,
    pub total_prs: i32,
    pub active_developers: i32,
}

// ============================================================================
// SPEED METRICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedMetrics {
    pub prs_per_day: f64,
    pub prs_per_day_per_dev: f64,
    pub pr_turnaround_hours: f64,
    pub loc_per_day: f64,
    pub cycle_time_distribution: CycleTimeDistribution,
    pub benchmark_comparison: SpeedBenchmarks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleTimeDistribution {
    pub under_4h: i32,
    pub under_4h_pct: f64,
    pub h4_to_12: i32,
    pub h4_to_12_pct: f64,
    pub h12_to_24: i32,
    pub h12_to_24_pct: f64,
    pub over_24h: i32,
    pub over_24h_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedBenchmarks {
    pub prs_per_day_industry: f64,
    pub prs_per_day_elite: f64,
    pub pr_turnaround_industry: f64,
    pub pr_turnaround_elite: f64,
}

// ============================================================================
// EASE METRICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EaseMetrics {
    pub concurrent_repos: i32,
    pub repos_per_dev: f64,
    pub total_active_repos: i32,
    pub active_repos: Vec<ActiveRepository>,
    pub repo_distribution: RepoDistribution,
    pub work_pattern: Vec<WorkPatternCell>,
    pub pr_switch_frequency: f64,
    pub benchmark_comparison: EaseBenchmarks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveRepository {
    pub repo_name: String,
    pub pr_count: i32,
    pub total_loc: i32,
    pub contributor_count: i32,
    pub last_activity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoDistribution {
    pub org_repos: i32,
    pub org_repos_pct: f64,
    pub personal_repos: i32,
    pub personal_repos_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPatternCell {
    pub day_of_week: i32, // 0=Sunday, 1=Monday, etc.
    pub hour_of_day: i32, // 0-23
    pub activity_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EaseBenchmarks {
    pub concurrent_repos_industry: f64,
    pub concurrent_repos_elite: f64,
}

// ============================================================================
// QUALITY METRICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub pr_merge_rate: f64,
    pub avg_files_per_pr: f64,
    pub bug_pr_percentage: f64,
    pub feature_pr_percentage: f64,
    pub avg_review_cycle_hours: f64,
    pub avg_review_comments: f64,
    pub pr_type_distribution: Vec<PrTypeBreakdown>,
    pub files_per_pr_distribution: FilesPerPrDistribution,
    pub merge_rate_trend: Vec<MergeRateTrend>,
    pub benchmark_comparison: QualityBenchmarks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrTypeBreakdown {
    pub pr_type: String,
    pub count: i32,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesPerPrDistribution {
    pub range_1_3: i32,
    pub range_1_3_pct: f64,
    pub range_4_8: i32,
    pub range_4_8_pct: f64,
    pub range_9_15: i32,
    pub range_9_15_pct: f64,
    pub range_16_plus: i32,
    pub range_16_plus_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRateTrend {
    pub week: String,
    pub merge_rate: f64,
    pub total_prs: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityBenchmarks {
    pub merge_rate_industry: f64,
    pub merge_rate_elite: f64,
    pub bug_ratio_industry: f64,
    pub bug_ratio_elite: f64,
    pub files_per_pr_industry: f64,
}

// ============================================================================
// QUERY FUNCTIONS
// ============================================================================

/// Get complete dashboard metrics for a given time period
pub fn get_dashboard_metrics(conn: &Connection, days: i32) -> Result<DashboardMetrics> {
    let speed = get_speed_metrics(conn, days)?;
    let ease = get_ease_metrics(conn, days)?;
    let quality = get_quality_metrics(conn, days)?;
    let overview = get_overview_metrics(conn, days, &speed, &ease, &quality)?;

    Ok(DashboardMetrics {
        speed,
        ease,
        quality,
        overview,
    })
}

/// Calculate overview metrics including productivity multiplier
fn get_overview_metrics(
    conn: &Connection,
    days: i32,
    speed: &SpeedMetrics,
    ease: &EaseMetrics,
    quality: &QualityMetrics,
) -> Result<OverviewMetrics> {
    let (total_prs, active_developers): (i32, i32) = conn.query_row(
        "SELECT
            COUNT(*) as total_prs,
            COUNT(DISTINCT author_id) as active_developers
         FROM pull_requests
         WHERE created_at > datetime('now', '-' || ?1 || ' days')
           AND author_id IN (SELECT id FROM users WHERE tracked = 1)",
        params![days],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    // Calculate productivity multiplier
    // Formula: Weighted average of performance vs industry benchmarks
    let pr_velocity_ratio = speed.prs_per_day_per_dev / speed.benchmark_comparison.prs_per_day_industry;
    let pr_speed_ratio = speed.benchmark_comparison.pr_turnaround_industry / speed.pr_turnaround_hours;
    let repo_capacity_ratio = ease.repos_per_dev / ease.benchmark_comparison.concurrent_repos_industry;
    let quality_ratio = quality.pr_merge_rate / quality.benchmark_comparison.merge_rate_industry;

    let productivity_multiplier =
        (pr_velocity_ratio * 0.35) +
        (pr_speed_ratio * 0.25) +
        (repo_capacity_ratio * 0.25) +
        (quality_ratio * 0.15);

    Ok(OverviewMetrics {
        productivity_multiplier,
        period_days: days,
        total_prs,
        active_developers,
    })
}

/// Get Speed metrics
fn get_speed_metrics(conn: &Connection, days: i32) -> Result<SpeedMetrics> {
    // PRs per day calculations
    let (total_prs, active_developers, active_days): (f64, f64, f64) = conn.query_row(
        "SELECT
            COUNT(*) as total_prs,
            COUNT(DISTINCT author_id) as active_developers,
            COUNT(DISTINCT DATE(created_at)) as active_days
         FROM pull_requests
         WHERE created_at > datetime('now', '-' || ?1 || ' days')
           AND author_id IN (SELECT id FROM users WHERE tracked = 1)",
        params![days],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;

    let prs_per_day = if active_days > 0.0 { total_prs / active_days } else { 0.0 };
    let prs_per_day_per_dev = if active_developers > 0.0 && active_days > 0.0 {
        total_prs / (active_developers * active_days)
    } else {
        0.0
    };

    // PR turnaround time (merged PRs only)
    let pr_turnaround_hours: f64 = conn.query_row(
        "SELECT AVG((julianday(merged_at) - julianday(created_at)) * 24.0)
         FROM pull_requests
         WHERE merged_at IS NOT NULL
           AND created_at > datetime('now', '-' || ?1 || ' days')
           AND author_id IN (SELECT id FROM users WHERE tracked = 1)",
        params![days],
        |row| row.get(0),
    ).unwrap_or(0.0);

    // Lines of code per day
    let loc_per_day: f64 = conn.query_row(
        "SELECT SUM(additions + deletions) * 1.0 / ?1
         FROM pull_requests
         WHERE created_at > datetime('now', '-' || ?1 || ' days')
           AND author_id IN (SELECT id FROM users WHERE tracked = 1)",
        params![days],
        |row| row.get(0),
    ).unwrap_or(0.0);

    // Cycle time distribution
    let cycle_time_distribution = get_cycle_time_distribution(conn, days)?;

    // Benchmarks (industry standards)
    let benchmark_comparison = SpeedBenchmarks {
        prs_per_day_industry: 0.8,
        prs_per_day_elite: 1.5,
        pr_turnaround_industry: 89.0,
        pr_turnaround_elite: 24.0,
    };

    Ok(SpeedMetrics {
        prs_per_day,
        prs_per_day_per_dev,
        pr_turnaround_hours,
        loc_per_day,
        cycle_time_distribution,
        benchmark_comparison,
    })
}

/// Get cycle time distribution
fn get_cycle_time_distribution(conn: &Connection, days: i32) -> Result<CycleTimeDistribution> {
    let mut stmt = conn.prepare(
        "SELECT
            SUM(CASE WHEN hours_to_merge < 4 THEN 1 ELSE 0 END) as under_4h,
            SUM(CASE WHEN hours_to_merge >= 4 AND hours_to_merge < 12 THEN 1 ELSE 0 END) as h4_to_12,
            SUM(CASE WHEN hours_to_merge >= 12 AND hours_to_merge < 24 THEN 1 ELSE 0 END) as h12_to_24,
            SUM(CASE WHEN hours_to_merge >= 24 THEN 1 ELSE 0 END) as over_24h,
            COUNT(*) as total
         FROM (
            SELECT (julianday(merged_at) - julianday(created_at)) * 24.0 as hours_to_merge
            FROM pull_requests
            WHERE merged_at IS NOT NULL
              AND created_at > datetime('now', '-' || ?1 || ' days')
              AND author_id IN (SELECT id FROM users WHERE tracked = 1)
         )"
    )?;

    let (under_4h, h4_to_12, h12_to_24, over_24h, total): (i32, i32, i32, i32, i32) =
        stmt.query_row(params![days], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })?;

    let total_f = total as f64;
    Ok(CycleTimeDistribution {
        under_4h,
        under_4h_pct: if total > 0 { (under_4h as f64 / total_f) * 100.0 } else { 0.0 },
        h4_to_12,
        h4_to_12_pct: if total > 0 { (h4_to_12 as f64 / total_f) * 100.0 } else { 0.0 },
        h12_to_24,
        h12_to_24_pct: if total > 0 { (h12_to_24 as f64 / total_f) * 100.0 } else { 0.0 },
        over_24h,
        over_24h_pct: if total > 0 { (over_24h as f64 / total_f) * 100.0 } else { 0.0 },
    })
}

/// Get Ease metrics
fn get_ease_metrics(conn: &Connection, days: i32) -> Result<EaseMetrics> {
    // Concurrent repositories
    let (concurrent_repos, active_developers): (i32, i32) = conn.query_row(
        "SELECT
            COUNT(DISTINCT repo_id) as concurrent_repos,
            COUNT(DISTINCT author_id) as active_developers
         FROM pull_requests
         WHERE created_at > datetime('now', '-' || ?1 || ' days')
           AND author_id IN (SELECT id FROM users WHERE tracked = 1)",
        params![days],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let repos_per_dev = if active_developers > 0 {
        concurrent_repos as f64 / active_developers as f64
    } else {
        0.0
    };

    // Active repositories list
    let active_repos = get_active_repositories(conn, days)?;
    let total_active_repos = active_repos.len() as i32;

    // Repository distribution
    let repo_distribution = get_repo_distribution(conn, days)?;

    // Work pattern heatmap
    let work_pattern = get_work_pattern(conn, days)?;

    // PR switch frequency
    let pr_switch_frequency = get_pr_switch_frequency(conn, days)?;

    // Benchmarks
    let benchmark_comparison = EaseBenchmarks {
        concurrent_repos_industry: 2.1,
        concurrent_repos_elite: 3.5,
    };

    Ok(EaseMetrics {
        concurrent_repos,
        repos_per_dev,
        total_active_repos,
        active_repos,
        repo_distribution,
        work_pattern,
        pr_switch_frequency,
        benchmark_comparison,
    })
}

/// Get active repositories list
fn get_active_repositories(conn: &Connection, days: i32) -> Result<Vec<ActiveRepository>> {
    let mut stmt = conn.prepare(
        "SELECT
            r.owner || '/' || r.name as repo_name,
            COUNT(DISTINCT pr.id) as pr_count,
            SUM(pr.additions + pr.deletions) as total_loc,
            COUNT(DISTINCT pr.author_id) as contributor_count,
            MAX(pr.created_at) as last_activity
         FROM pull_requests pr
         JOIN repositories r ON pr.repo_id = r.id
         WHERE pr.created_at > datetime('now', '-' || ?1 || ' days')
           AND pr.author_id IN (SELECT id FROM users WHERE tracked = 1)
         GROUP BY r.id, r.owner, r.name
         ORDER BY pr_count DESC
         LIMIT 20"
    )?;

    let repos = stmt.query_map(params![days], |row| {
        Ok(ActiveRepository {
            repo_name: row.get(0)?,
            pr_count: row.get(1)?,
            total_loc: row.get(2).unwrap_or(0),
            contributor_count: row.get(3)?,
            last_activity: row.get(4)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(repos)
}

/// Get repository distribution (org vs personal)
fn get_repo_distribution(conn: &Connection, days: i32) -> Result<RepoDistribution> {
    let (org_repos, personal_repos): (i32, i32) = conn.query_row(
        "SELECT
            COUNT(DISTINCT CASE WHEN r.owner IN ('microsoft', 'Microsoft') THEN r.id END) as org_repos,
            COUNT(DISTINCT CASE WHEN r.owner NOT IN ('microsoft', 'Microsoft') THEN r.id END) as personal_repos
         FROM repositories r
         JOIN pull_requests pr ON pr.repo_id = r.id
         WHERE pr.created_at > datetime('now', '-' || ?1 || ' days')
           AND pr.author_id IN (SELECT id FROM users WHERE tracked = 1)",
        params![days],
        |row| Ok((row.get(0).unwrap_or(0), row.get(1).unwrap_or(0))),
    )?;

    let total = (org_repos + personal_repos) as f64;
    Ok(RepoDistribution {
        org_repos,
        org_repos_pct: if total > 0.0 { (org_repos as f64 / total) * 100.0 } else { 0.0 },
        personal_repos,
        personal_repos_pct: if total > 0.0 { (personal_repos as f64 / total) * 100.0 } else { 0.0 },
    })
}

/// Get work pattern heatmap
fn get_work_pattern(conn: &Connection, days: i32) -> Result<Vec<WorkPatternCell>> {
    let mut stmt = conn.prepare(
        "SELECT
            CAST(strftime('%w', created_at) AS INTEGER) as day_of_week,
            CAST(strftime('%H', created_at) AS INTEGER) as hour_of_day,
            COUNT(*) as activity_count
         FROM pull_requests
         WHERE created_at > datetime('now', '-' || ?1 || ' days')
           AND author_id IN (SELECT id FROM users WHERE tracked = 1)
         GROUP BY day_of_week, hour_of_day
         ORDER BY day_of_week, hour_of_day"
    )?;

    let pattern = stmt.query_map(params![days], |row| {
        Ok(WorkPatternCell {
            day_of_week: row.get(0)?,
            hour_of_day: row.get(1)?,
            activity_count: row.get(2)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(pattern)
}

/// Get PR switch frequency (how often consecutive PRs are in different repos)
fn get_pr_switch_frequency(conn: &Connection, days: i32) -> Result<f64> {
    let switch_pct: f64 = conn.query_row(
        "WITH ordered_prs AS (
            SELECT
                author_id,
                repo_id,
                created_at,
                LAG(repo_id) OVER (PARTITION BY author_id ORDER BY created_at) as prev_repo_id
            FROM pull_requests
            WHERE created_at > datetime('now', '-' || ?1 || ' days')
              AND author_id IN (SELECT id FROM users WHERE tracked = 1)
        )
        SELECT
            CASE WHEN COUNT(*) > 0
            THEN (COUNT(CASE WHEN repo_id != prev_repo_id THEN 1 END) * 100.0 / COUNT(*))
            ELSE 0.0
            END as switch_percentage
        FROM ordered_prs
        WHERE prev_repo_id IS NOT NULL",
        params![days],
        |row| row.get(0),
    ).unwrap_or(0.0);

    Ok(switch_pct)
}

/// Get Quality metrics
fn get_quality_metrics(conn: &Connection, days: i32) -> Result<QualityMetrics> {
    // PR merge rate
    let pr_merge_rate: f64 = conn.query_row(
        "SELECT
            CASE WHEN COUNT(CASE WHEN state != 'open' THEN 1 END) > 0
            THEN (COUNT(CASE WHEN merged_at IS NOT NULL THEN 1 END) * 100.0 /
                  COUNT(CASE WHEN state != 'open' THEN 1 END))
            ELSE 0.0
            END as merge_rate
         FROM pull_requests
         WHERE created_at > datetime('now', '-' || ?1 || ' days')
           AND author_id IN (SELECT id FROM users WHERE tracked = 1)",
        params![days],
        |row| row.get(0),
    ).unwrap_or(0.0);

    // Average files per PR
    let avg_files_per_pr: f64 = conn.query_row(
        "SELECT AVG(changed_files)
         FROM pull_requests
         WHERE created_at > datetime('now', '-' || ?1 || ' days')
           AND author_id IN (SELECT id FROM users WHERE tracked = 1)",
        params![days],
        |row| row.get(0),
    ).unwrap_or(0.0);

    // PR type distribution
    let pr_type_distribution = get_pr_type_distribution(conn, days)?;

    // Calculate bug and feature percentages from distribution
    let bug_pr_percentage = pr_type_distribution.iter()
        .find(|p| p.pr_type == "bug_fix")
        .map(|p| p.percentage)
        .unwrap_or(0.0);

    let feature_pr_percentage = pr_type_distribution.iter()
        .find(|p| p.pr_type == "feature")
        .map(|p| p.percentage)
        .unwrap_or(0.0);

    // Average review cycle time
    let avg_review_cycle_hours: f64 = conn.query_row(
        "SELECT AVG((julianday(r.submitted_at) - julianday(pr.created_at)) * 24.0)
         FROM pull_requests pr
         JOIN pr_reviews r ON r.pr_id = pr.id
         WHERE pr.created_at > datetime('now', '-' || ?1 || ' days')
           AND pr.author_id IN (SELECT id FROM users WHERE tracked = 1)
           AND r.submitted_at = (
                SELECT MIN(submitted_at)
                FROM pr_reviews
                WHERE pr_id = pr.id
           )",
        params![days],
        |row| row.get(0),
    ).unwrap_or(0.0);

    // Average review comments
    let avg_review_comments: f64 = conn.query_row(
        "SELECT AVG(review_comments)
         FROM pull_requests
         WHERE created_at > datetime('now', '-' || ?1 || ' days')
           AND author_id IN (SELECT id FROM users WHERE tracked = 1)",
        params![days],
        |row| row.get(0),
    ).unwrap_or(0.0);

    // Files per PR distribution
    let files_per_pr_distribution = get_files_per_pr_distribution(conn, days)?;

    // Merge rate trend
    let merge_rate_trend = get_merge_rate_trend(conn, 90)?; // Always show 90 days for trend

    // Benchmarks
    let benchmark_comparison = QualityBenchmarks {
        merge_rate_industry: 68.0,
        merge_rate_elite: 85.0,
        bug_ratio_industry: 25.0,
        bug_ratio_elite: 15.0,
        files_per_pr_industry: 8.0,
    };

    Ok(QualityMetrics {
        pr_merge_rate,
        avg_files_per_pr,
        bug_pr_percentage,
        feature_pr_percentage,
        avg_review_cycle_hours,
        avg_review_comments,
        pr_type_distribution,
        files_per_pr_distribution,
        merge_rate_trend,
        benchmark_comparison,
    })
}

/// Classify PR type based on title and labels
fn get_pr_type_distribution(conn: &Connection, days: i32) -> Result<Vec<PrTypeBreakdown>> {
    let mut stmt = conn.prepare(
        "SELECT
            CASE
                WHEN LOWER(title) LIKE '%feat%' OR LOWER(title) LIKE '%feature%'
                     OR LOWER(title) LIKE '%add%' OR LOWER(labels) LIKE '%feature%'
                     OR LOWER(labels) LIKE '%enhancement%'
                THEN 'feature'
                WHEN LOWER(title) LIKE '%fix%' OR LOWER(title) LIKE '%bug%'
                     OR LOWER(labels) LIKE '%bug%'
                THEN 'bug_fix'
                WHEN LOWER(title) LIKE '%refactor%' OR LOWER(title) LIKE '%improve%'
                THEN 'refactor'
                WHEN LOWER(title) LIKE '%test%' OR LOWER(title) LIKE '%spec%'
                THEN 'test'
                WHEN LOWER(title) LIKE '%doc%' OR LOWER(labels) LIKE '%documentation%'
                THEN 'docs'
                ELSE 'other'
            END as pr_type,
            COUNT(*) as count
         FROM pull_requests
         WHERE created_at > datetime('now', '-' || ?1 || ' days')
           AND author_id IN (SELECT id FROM users WHERE tracked = 1)
         GROUP BY pr_type"
    )?;

    let types: Vec<(String, i32)> = stmt.query_map(params![days], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?
    .collect::<Result<Vec<_>, _>>()?;

    let total: i32 = types.iter().map(|(_, count)| count).sum();
    let total_f = total as f64;

    let breakdown = types.into_iter().map(|(pr_type, count)| {
        PrTypeBreakdown {
            pr_type,
            count,
            percentage: if total > 0 { (count as f64 / total_f) * 100.0 } else { 0.0 },
        }
    }).collect();

    Ok(breakdown)
}

/// Get files per PR distribution
fn get_files_per_pr_distribution(conn: &Connection, days: i32) -> Result<FilesPerPrDistribution> {
    let (range_1_3, range_4_8, range_9_15, range_16_plus, total): (i32, i32, i32, i32, i32) = conn.query_row(
        "SELECT
            SUM(CASE WHEN changed_files <= 3 THEN 1 ELSE 0 END) as range_1_3,
            SUM(CASE WHEN changed_files > 3 AND changed_files <= 8 THEN 1 ELSE 0 END) as range_4_8,
            SUM(CASE WHEN changed_files > 8 AND changed_files <= 15 THEN 1 ELSE 0 END) as range_9_15,
            SUM(CASE WHEN changed_files > 15 THEN 1 ELSE 0 END) as range_16_plus,
            COUNT(*) as total
         FROM pull_requests
         WHERE created_at > datetime('now', '-' || ?1 || ' days')
           AND author_id IN (SELECT id FROM users WHERE tracked = 1)",
        params![days],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
    )?;

    let total_f = total as f64;
    Ok(FilesPerPrDistribution {
        range_1_3,
        range_1_3_pct: if total > 0 { (range_1_3 as f64 / total_f) * 100.0 } else { 0.0 },
        range_4_8,
        range_4_8_pct: if total > 0 { (range_4_8 as f64 / total_f) * 100.0 } else { 0.0 },
        range_9_15,
        range_9_15_pct: if total > 0 { (range_9_15 as f64 / total_f) * 100.0 } else { 0.0 },
        range_16_plus,
        range_16_plus_pct: if total > 0 { (range_16_plus as f64 / total_f) * 100.0 } else { 0.0 },
    })
}

/// Get merge rate trend over time (weekly buckets)
fn get_merge_rate_trend(conn: &Connection, days: i32) -> Result<Vec<MergeRateTrend>> {
    let mut stmt = conn.prepare(
        "SELECT
            DATE(created_at, 'weekday 0', '-6 days') as week,
            (COUNT(CASE WHEN merged_at IS NOT NULL THEN 1 END) * 100.0 /
             CASE WHEN COUNT(CASE WHEN state != 'open' THEN 1 END) > 0
                  THEN COUNT(CASE WHEN state != 'open' THEN 1 END)
                  ELSE 1
             END) as merge_rate,
            COUNT(*) as total_prs
         FROM pull_requests
         WHERE created_at > datetime('now', '-' || ?1 || ' days')
           AND author_id IN (SELECT id FROM users WHERE tracked = 1)
           AND state != 'open'
         GROUP BY week
         ORDER BY week"
    )?;

    let trend = stmt.query_map(params![days], |row| {
        Ok(MergeRateTrend {
            week: row.get(0)?,
            merge_rate: row.get(1)?,
            total_prs: row.get(2)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(trend)
}
