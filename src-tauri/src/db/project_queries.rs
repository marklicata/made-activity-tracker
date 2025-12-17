use super::models::*;
use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// DATA MODELS FOR PROJECT DEEP DIVE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityHeatmapData {
    pub daily_counts: HashMap<String, i32>,   // date -> count
    pub hourly_counts: HashMap<u8, i32>,      // hour -> count
    pub weekday_counts: HashMap<String, i32>, // weekday -> count
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleMetrics {
    pub avg_time_to_merge: f64,       // hours
    pub median_time_to_merge: f64,
    pub p90_time_to_merge: f64,
    pub avg_time_to_first_review: f64,
    pub avg_review_cycles: f64,
    pub open_prs_count: i32,
    pub open_issues_count: i32,
    pub bottleneck_prs: Vec<PullRequest>,
    pub bottleneck_issues: Vec<Issue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub total_contributors: i32,
    pub total_commits: i32,
    pub total_prs: i32,
    pub total_issues: i32,
    pub last_synced_at: Option<String>,
}

// ============================================================================
// TIMELINE QUERIES
// ============================================================================

/// Get timeline events for a repository
/// Combines PRs, issues, and reviews into a unified timeline
pub fn get_timeline_events(
    conn: &Connection,
    repo_id: i64,
    start_date: Option<&str>,
    end_date: Option<&str>,
    event_types: Option<&Vec<String>>,
    user_id: Option<i64>,
    limit: i32,
) -> Result<Vec<TimelineEvent>> {
    let mut events = Vec::new();

    // Build the date filter clause
    let date_filter = match (start_date, end_date) {
        (Some(start), Some(end)) => format!(" AND timestamp >= '{}' AND timestamp <= '{}'", start, end),
        (Some(start), None) => format!(" AND timestamp >= '{}'", start),
        (None, Some(end)) => format!(" AND timestamp <= '{}'", end),
        (None, None) => String::new(),
    };

    // Build the user filter clause
    let user_filter = match user_id {
        Some(uid) => format!(" AND author_id = {}", uid),
        None => String::new(),
    };

    // Check which event types to include
    let include_issues = event_types.is_none() || event_types.unwrap().iter().any(|t| t.starts_with("issue"));
    let include_prs = event_types.is_none() || event_types.unwrap().iter().any(|t| t.starts_with("pr_"));
    let include_reviews = event_types.is_none() || event_types.unwrap().iter().any(|t| t == "review");

    // Fetch issue events (opened and closed)
    if include_issues {
        // Issue opened events
        let query = format!(
            "SELECT i.id, i.number, i.title, i.body, i.created_at, i.author_id,
                    u.id, u.github_id, u.login, u.name, u.avatar_url, u.is_bot
             FROM issues i
             LEFT JOIN users u ON i.author_id = u.id
             WHERE i.repo_id = ?1{}{}
             ORDER BY i.created_at DESC
             LIMIT {}",
            date_filter, user_filter, limit
        );

        let mut stmt = conn.prepare(&query)?;
        let issue_iter = stmt.query_map([repo_id], |row| {
            let user = User {
                id: row.get(6)?,
                github_id: row.get(7)?,
                login: row.get(8)?,
                name: row.get(9)?,
                avatar_url: row.get(10)?,
                is_bot: row.get(11)?,
            };

            let metadata = serde_json::json!({
                "issue_number": row.get::<_, i32>(1)?,
                "state": "opened"
            });

            Ok(TimelineEvent {
                id: format!("issue-{}-opened", row.get::<_, i64>(0)?),
                event_type: "issue_opened".to_string(),
                timestamp: row.get(4)?,
                author: user,
                title: row.get(2)?,
                description: row.get(3)?,
                url: None,
                metadata,
            })
        })?;

        for event in issue_iter {
            events.push(event?);
        }

        // Issue closed events
        let query = format!(
            "SELECT i.id, i.number, i.title, i.body, i.closed_at, i.author_id,
                    u.id, u.github_id, u.login, u.name, u.avatar_url, u.is_bot
             FROM issues i
             LEFT JOIN users u ON i.author_id = u.id
             WHERE i.repo_id = ?1 AND i.closed_at IS NOT NULL{}{}
             ORDER BY i.closed_at DESC
             LIMIT {}",
            date_filter.replace("timestamp", "i.closed_at"),
            user_filter,
            limit
        );

        let mut stmt = conn.prepare(&query)?;
        let issue_closed_iter = stmt.query_map([repo_id], |row| {
            let user = User {
                id: row.get(6)?,
                github_id: row.get(7)?,
                login: row.get(8)?,
                name: row.get(9)?,
                avatar_url: row.get(10)?,
                is_bot: row.get(11)?,
            };

            let metadata = serde_json::json!({
                "issue_number": row.get::<_, i32>(1)?,
                "state": "closed"
            });

            Ok(TimelineEvent {
                id: format!("issue-{}-closed", row.get::<_, i64>(0)?),
                event_type: "issue_closed".to_string(),
                timestamp: row.get::<_, String>(4)?,
                author: user,
                title: row.get(2)?,
                description: row.get(3)?,
                url: None,
                metadata,
            })
        })?;

        for event in issue_closed_iter {
            events.push(event?);
        }
    }

    // Fetch PR events (opened, merged, closed)
    if include_prs {
        // PR opened events
        let query = format!(
            "SELECT pr.id, pr.number, pr.title, pr.body, pr.created_at, pr.additions, pr.deletions, pr.changed_files, pr.author_id,
                    u.id, u.github_id, u.login, u.name, u.avatar_url, u.is_bot
             FROM pull_requests pr
             LEFT JOIN users u ON pr.author_id = u.id
             WHERE pr.repo_id = ?1{}{}
             ORDER BY pr.created_at DESC
             LIMIT {}",
            date_filter, user_filter, limit
        );

        let mut stmt = conn.prepare(&query)?;
        let pr_iter = stmt.query_map([repo_id], |row| {
            let user = User {
                id: row.get(9)?,
                github_id: row.get(10)?,
                login: row.get(11)?,
                name: row.get(12)?,
                avatar_url: row.get(13)?,
                is_bot: row.get(14)?,
            };

            let metadata = serde_json::json!({
                "pr_number": row.get::<_, i32>(1)?,
                "additions": row.get::<_, i32>(5)?,
                "deletions": row.get::<_, i32>(6)?,
                "changed_files": row.get::<_, i32>(7)?,
                "state": "opened"
            });

            Ok(TimelineEvent {
                id: format!("pr-{}-opened", row.get::<_, i64>(0)?),
                event_type: "pr_opened".to_string(),
                timestamp: row.get(4)?,
                author: user,
                title: row.get(2)?,
                description: row.get(3)?,
                url: None,
                metadata,
            })
        })?;

        for event in pr_iter {
            events.push(event?);
        }

        // PR merged events
        let query = format!(
            "SELECT pr.id, pr.number, pr.title, pr.body, pr.merged_at, pr.additions, pr.deletions, pr.changed_files, pr.author_id,
                    u.id, u.github_id, u.login, u.name, u.avatar_url, u.is_bot
             FROM pull_requests pr
             LEFT JOIN users u ON pr.author_id = u.id
             WHERE pr.repo_id = ?1 AND pr.merged_at IS NOT NULL{}{}
             ORDER BY pr.merged_at DESC
             LIMIT {}",
            date_filter.replace("timestamp", "pr.merged_at"),
            user_filter,
            limit
        );

        let mut stmt = conn.prepare(&query)?;
        let pr_merged_iter = stmt.query_map([repo_id], |row| {
            let user = User {
                id: row.get(9)?,
                github_id: row.get(10)?,
                login: row.get(11)?,
                name: row.get(12)?,
                avatar_url: row.get(13)?,
                is_bot: row.get(14)?,
            };

            let metadata = serde_json::json!({
                "pr_number": row.get::<_, i32>(1)?,
                "additions": row.get::<_, i32>(5)?,
                "deletions": row.get::<_, i32>(6)?,
                "changed_files": row.get::<_, i32>(7)?,
                "state": "merged"
            });

            Ok(TimelineEvent {
                id: format!("pr-{}-merged", row.get::<_, i64>(0)?),
                event_type: "pr_merged".to_string(),
                timestamp: row.get::<_, String>(4)?,
                author: user,
                title: row.get(2)?,
                description: row.get(3)?,
                url: None,
                metadata,
            })
        })?;

        for event in pr_merged_iter {
            events.push(event?);
        }
    }

    // Fetch review events
    if include_reviews {
        let query = format!(
            "SELECT r.id, r.submitted_at, r.state, r.reviewer_id,
                    pr.number, pr.title,
                    u.id, u.github_id, u.login, u.name, u.avatar_url, u.is_bot
             FROM pr_reviews r
             JOIN pull_requests pr ON r.pr_id = pr.id
             LEFT JOIN users u ON r.reviewer_id = u.id
             WHERE pr.repo_id = ?1{}{}
             ORDER BY r.submitted_at DESC
             LIMIT {}",
            date_filter.replace("timestamp", "r.submitted_at"),
            user_filter.replace("author_id", "reviewer_id"),
            limit
        );

        let mut stmt = conn.prepare(&query)?;
        let review_iter = stmt.query_map([repo_id], |row| {
            let user = User {
                id: row.get(6)?,
                github_id: row.get(7)?,
                login: row.get(8)?,
                name: row.get(9)?,
                avatar_url: row.get(10)?,
                is_bot: row.get(11)?,
            };

            let metadata = serde_json::json!({
                "pr_number": row.get::<_, i32>(4)?,
                "review_state": row.get::<_, String>(2)?
            });

            Ok(TimelineEvent {
                id: format!("review-{}", row.get::<_, i64>(0)?),
                event_type: "review".to_string(),
                timestamp: row.get(1)?,
                author: user,
                title: row.get(5)?,
                description: None,
                url: None,
                metadata,
            })
        })?;

        for event in review_iter {
            events.push(event?);
        }
    }

    // Sort all events by timestamp DESC
    events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Apply limit
    events.truncate(limit as usize);

    Ok(events)
}

// ============================================================================
// CONTRIBUTOR QUERIES
// ============================================================================

/// Get contributor statistics for a repository
pub fn get_contributor_stats(
    conn: &Connection,
    repo_id: i64,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<Vec<ContributorStats>> {
    // Build the date filter clause
    let date_filter = match (start_date, end_date) {
        (Some(start), Some(end)) => format!(" AND created_at >= '{}' AND created_at <= '{}'", start, end),
        (Some(start), None) => format!(" AND created_at >= '{}'", start),
        (None, Some(end)) => format!(" AND created_at <= '{}'", end),
        (None, None) => String::new(),
    };

    // Get all contributors from PRs and issues
    let query = format!(
        "SELECT DISTINCT u.id, u.github_id, u.login, u.name, u.avatar_url, u.is_bot
         FROM users u
         WHERE u.id IN (
             SELECT DISTINCT author_id FROM pull_requests WHERE repo_id = ?1{}
             UNION
             SELECT DISTINCT author_id FROM issues WHERE repo_id = ?1{}
             UNION
             SELECT DISTINCT reviewer_id FROM pr_reviews r
             JOIN pull_requests pr ON r.pr_id = pr.id
             WHERE pr.repo_id = ?1{}
         ) AND u.is_bot = FALSE
         ORDER BY u.login",
        date_filter, date_filter, date_filter.replace("created_at", "r.submitted_at")
    );

    let mut stmt = conn.prepare(&query)?;
    let user_rows = stmt.query_map([repo_id], |row| {
        Ok(User {
            id: row.get(0)?,
            github_id: row.get(1)?,
            login: row.get(2)?,
            name: row.get(3)?,
            avatar_url: row.get(4)?,
            is_bot: row.get(5)?,
        })
    })?;

    let mut stats = Vec::new();

    for user_result in user_rows {
        let user = user_result?;

        // Get PR stats
        let pr_query = format!(
            "SELECT COUNT(*), COALESCE(SUM(additions), 0), COALESCE(SUM(deletions), 0), COALESCE(SUM(changed_files), 0)
             FROM pull_requests
             WHERE repo_id = ?1 AND author_id = ?2{}",
            date_filter
        );
        let (total_prs, lines_added, lines_deleted, files_changed): (i32, i32, i32, i32) =
            conn.query_row(&pr_query, params![repo_id, user.id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?;

        // Get issue stats
        let issue_query = format!(
            "SELECT COUNT(*) FROM issues WHERE repo_id = ?1 AND author_id = ?2{}",
            date_filter
        );
        let total_issues: i32 = conn.query_row(&issue_query, params![repo_id, user.id], |row| row.get(0))?;

        // Get review stats
        let review_query = format!(
            "SELECT COUNT(*) FROM pr_reviews r
             JOIN pull_requests pr ON r.pr_id = pr.id
             WHERE pr.repo_id = ?1 AND r.reviewer_id = ?2{}",
            date_filter.replace("created_at", "r.submitted_at")
        );
        let total_prs_reviewed: i32 = conn.query_row(&review_query, params![repo_id, user.id], |row| row.get(0))?;

        // Get first and last contribution
        let contribution_query = format!(
            "SELECT MIN(timestamp) as first, MAX(timestamp) as last FROM (
                SELECT created_at as timestamp FROM pull_requests WHERE repo_id = ?1 AND author_id = ?2{}
                UNION ALL
                SELECT created_at as timestamp FROM issues WHERE repo_id = ?1 AND author_id = ?2{}
                UNION ALL
                SELECT r.submitted_at as timestamp FROM pr_reviews r
                JOIN pull_requests pr ON r.pr_id = pr.id
                WHERE pr.repo_id = ?1 AND r.reviewer_id = ?2{}
             )",
            date_filter, date_filter, date_filter.replace("created_at", "r.submitted_at")
        );
        let (first_contribution, last_contribution): (String, String) =
            conn.query_row(&contribution_query, params![repo_id, user.id], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?;

        // Calculate activity trend (simplified: compare first half vs second half)
        // TODO: Implement more sophisticated trend calculation
        let activity_trend = "stable".to_string();

        stats.push(ContributorStats {
            user,
            total_commits: 0, // We don't track individual commits yet
            total_prs,
            total_prs_reviewed,
            total_issues,
            lines_added,
            lines_deleted,
            files_changed,
            first_contribution,
            last_contribution,
            activity_trend,
        });
    }

    Ok(stats)
}

// ============================================================================
// ACTIVITY HEATMAP QUERIES
// ============================================================================

/// Get activity heatmap data for a repository
pub fn get_activity_heatmap(
    conn: &Connection,
    repo_id: i64,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<ActivityHeatmapData> {
    // Build the date filter clause
    let date_filter = match (start_date, end_date) {
        (Some(start), Some(end)) => format!(" AND timestamp >= '{}' AND timestamp <= '{}'", start, end),
        (Some(start), None) => format!(" AND timestamp >= '{}'", start),
        (None, Some(end)) => format!(" AND timestamp <= '{}'", end),
        (None, None) => String::new(),
    };

    // Daily counts
    let query = format!(
        "SELECT DATE(timestamp) as date, COUNT(*) as count FROM (
            SELECT created_at as timestamp FROM pull_requests WHERE repo_id = ?1{}
            UNION ALL
            SELECT created_at as timestamp FROM issues WHERE repo_id = ?1{}
            UNION ALL
            SELECT r.submitted_at as timestamp FROM pr_reviews r
            JOIN pull_requests pr ON r.pr_id = pr.id
            WHERE pr.repo_id = ?1{}
         )
         GROUP BY date
         ORDER BY date",
        date_filter, date_filter, date_filter.replace("created_at", "r.submitted_at")
    );

    let mut stmt = conn.prepare(&query)?;
    let daily_rows = stmt.query_map([repo_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
    })?;

    let mut daily_counts = HashMap::new();
    for row in daily_rows {
        let (date, count) = row?;
        daily_counts.insert(date, count);
    }

    // Hourly counts (hour of day 0-23)
    let query = format!(
        "SELECT CAST(strftime('%H', timestamp) AS INTEGER) as hour, COUNT(*) as count FROM (
            SELECT created_at as timestamp FROM pull_requests WHERE repo_id = ?1{}
            UNION ALL
            SELECT created_at as timestamp FROM issues WHERE repo_id = ?1{}
            UNION ALL
            SELECT r.submitted_at as timestamp FROM pr_reviews r
            JOIN pull_requests pr ON r.pr_id = pr.id
            WHERE pr.repo_id = ?1{}
         )
         GROUP BY hour
         ORDER BY hour",
        date_filter, date_filter, date_filter.replace("created_at", "r.submitted_at")
    );

    let mut stmt = conn.prepare(&query)?;
    let hourly_rows = stmt.query_map([repo_id], |row| {
        Ok((row.get::<_, u8>(0)?, row.get::<_, i32>(1)?))
    })?;

    let mut hourly_counts = HashMap::new();
    for row in hourly_rows {
        let (hour, count) = row?;
        hourly_counts.insert(hour, count);
    }

    // Weekday counts (0=Sunday, 6=Saturday)
    let query = format!(
        "SELECT strftime('%w', timestamp) as weekday, COUNT(*) as count FROM (
            SELECT created_at as timestamp FROM pull_requests WHERE repo_id = ?1{}
            UNION ALL
            SELECT created_at as timestamp FROM issues WHERE repo_id = ?1{}
            UNION ALL
            SELECT r.submitted_at as timestamp FROM pr_reviews r
            JOIN pull_requests pr ON r.pr_id = pr.id
            WHERE pr.repo_id = ?1{}
         )
         GROUP BY weekday
         ORDER BY weekday",
        date_filter, date_filter, date_filter.replace("created_at", "r.submitted_at")
    );

    let mut stmt = conn.prepare(&query)?;
    let weekday_rows = stmt.query_map([repo_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
    })?;

    let mut weekday_counts = HashMap::new();
    let weekday_names = vec!["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
    for row in weekday_rows {
        let (weekday_num, count) = row?;
        let weekday_index: usize = weekday_num.parse().unwrap_or(0);
        weekday_counts.insert(weekday_names[weekday_index].to_string(), count);
    }

    Ok(ActivityHeatmapData {
        daily_counts,
        hourly_counts,
        weekday_counts,
    })
}

// ============================================================================
// LIFECYCLE METRICS QUERIES
// ============================================================================

/// Get lifecycle metrics for a repository
pub fn get_lifecycle_metrics(
    conn: &Connection,
    repo_id: i64,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<LifecycleMetrics> {
    // Build the date filter clause
    let date_filter = match (start_date, end_date) {
        (Some(start), Some(end)) => format!(" AND created_at >= '{}' AND created_at <= '{}'", start, end),
        (Some(start), None) => format!(" AND created_at >= '{}'", start),
        (None, Some(end)) => format!(" AND created_at <= '{}'", end),
        (None, None) => String::new(),
    };

    // Calculate time to merge for merged PRs (in hours)
    let merge_time_query = format!(
        "SELECT
            AVG((julianday(merged_at) - julianday(created_at)) * 24) as avg_hours,
            (julianday(merged_at) - julianday(created_at)) * 24 as hours
         FROM pull_requests
         WHERE repo_id = ?1 AND merged_at IS NOT NULL{}
         ORDER BY hours",
        date_filter
    );

    let mut stmt = conn.prepare(&merge_time_query)?;
    let mut times: Vec<f64> = Vec::new();
    let mut avg_time_to_merge = 0.0;

    let time_rows = stmt.query_map([repo_id], |row| {
        Ok(row.get::<_, f64>(1)?)
    })?;

    for time in time_rows {
        let t = time?;
        times.push(t);
    }

    if !times.is_empty() {
        avg_time_to_merge = times.iter().sum::<f64>() / times.len() as f64;
    }

    // Calculate median (50th percentile)
    let median_time_to_merge = if !times.is_empty() {
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = times.len() / 2;
        if times.len() % 2 == 0 {
            (times[mid - 1] + times[mid]) / 2.0
        } else {
            times[mid]
        }
    } else {
        0.0
    };

    // Calculate 90th percentile
    let p90_time_to_merge = if !times.is_empty() {
        let idx = ((times.len() as f64) * 0.9) as usize;
        times[idx.min(times.len() - 1)]
    } else {
        0.0
    };

    // Calculate time to first review (in hours)
    let first_review_query = format!(
        "SELECT AVG((julianday(r.submitted_at) - julianday(pr.created_at)) * 24) as avg_hours
         FROM pull_requests pr
         JOIN (
             SELECT pr_id, MIN(submitted_at) as submitted_at
             FROM pr_reviews
             GROUP BY pr_id
         ) r ON pr.id = r.pr_id
         WHERE pr.repo_id = ?1{}",
        date_filter
    );

    let avg_time_to_first_review: f64 = conn.query_row(&first_review_query, [repo_id], |row| {
        row.get(0)
    }).unwrap_or(0.0);

    // Calculate average review cycles
    let review_cycles_query = format!(
        "SELECT AVG(review_count) FROM (
             SELECT pr.id, COUNT(r.id) as review_count
             FROM pull_requests pr
             LEFT JOIN pr_reviews r ON pr.id = r.pr_id
             WHERE pr.repo_id = ?1{}
             GROUP BY pr.id
         )",
        date_filter
    );

    let avg_review_cycles: f64 = conn.query_row(&review_cycles_query, [repo_id], |row| {
        row.get(0)
    }).unwrap_or(0.0);

    // Get open PRs and issues count
    let open_prs_count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM pull_requests WHERE repo_id = ?1 AND state = 'open'",
        [repo_id],
        |row| row.get(0),
    )?;

    let open_issues_count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM issues WHERE repo_id = ?1 AND state = 'open'",
        [repo_id],
        |row| row.get(0),
    )?;

    // Get bottleneck PRs (open for longest time)
    let bottleneck_prs_query =
        "SELECT id, github_id, repo_id, number, title, body, state, author_id, created_at, updated_at,
                merged_at, closed_at, additions, deletions, changed_files, review_comments, labels
         FROM pull_requests
         WHERE repo_id = ?1 AND state = 'open'
         ORDER BY created_at ASC
         LIMIT 5";

    let mut stmt = conn.prepare(bottleneck_prs_query)?;
    let pr_rows = stmt.query_map([repo_id], |row| {
        Ok(PullRequest {
            id: row.get(0)?,
            github_id: row.get(1)?,
            repo_id: row.get(2)?,
            number: row.get(3)?,
            title: row.get(4)?,
            body: row.get(5)?,
            state: row.get(6)?,
            author_id: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
            merged_at: row.get(10)?,
            closed_at: row.get(11)?,
            additions: row.get(12)?,
            deletions: row.get(13)?,
            changed_files: row.get(14)?,
            review_comments: row.get(15)?,
            labels: serde_json::from_str(&row.get::<_, String>(16)?).unwrap_or_default(),
        })
    })?;

    let mut bottleneck_prs = Vec::new();
    for pr in pr_rows {
        bottleneck_prs.push(pr?);
    }

    // Get bottleneck issues (open for longest time)
    let bottleneck_issues_query =
        "SELECT id, github_id, repo_id, number, title, body, state, author_id, assignee_id, milestone_id,
                created_at, updated_at, closed_at, labels
         FROM issues
         WHERE repo_id = ?1 AND state = 'open'
         ORDER BY created_at ASC
         LIMIT 5";

    let mut stmt = conn.prepare(bottleneck_issues_query)?;
    let issue_rows = stmt.query_map([repo_id], |row| {
        Ok(Issue {
            id: row.get(0)?,
            github_id: row.get(1)?,
            repo_id: row.get(2)?,
            number: row.get(3)?,
            title: row.get(4)?,
            body: row.get(5)?,
            state: row.get(6)?,
            author_id: row.get(7)?,
            assignee_id: row.get(8)?,
            milestone_id: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
            closed_at: row.get(12)?,
            labels: serde_json::from_str(&row.get::<_, String>(13)?).unwrap_or_default(),
        })
    })?;

    let mut bottleneck_issues = Vec::new();
    for issue in issue_rows {
        bottleneck_issues.push(issue?);
    }

    Ok(LifecycleMetrics {
        avg_time_to_merge,
        median_time_to_merge,
        p90_time_to_merge,
        avg_time_to_first_review,
        avg_review_cycles,
        open_prs_count,
        open_issues_count,
        bottleneck_prs,
        bottleneck_issues,
    })
}

// ============================================================================
// SUMMARY QUERIES
// ============================================================================

/// Get project summary statistics
pub fn get_project_summary(
    conn: &Connection,
    repo_id: i64,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<ProjectSummary> {
    // Build the date filter clause
    let date_filter = match (start_date, end_date) {
        (Some(start), Some(end)) => format!(" AND created_at >= '{}' AND created_at <= '{}'", start, end),
        (Some(start), None) => format!(" AND created_at >= '{}'", start),
        (None, Some(end)) => format!(" AND created_at <= '{}'", end),
        (None, None) => String::new(),
    };

    // Get unique contributors count
    let contributor_query = format!(
        "SELECT COUNT(DISTINCT author_id) FROM (
            SELECT author_id FROM pull_requests WHERE repo_id = ?1{}
            UNION
            SELECT author_id FROM issues WHERE repo_id = ?1{}
         )",
        date_filter, date_filter
    );

    let total_contributors: i32 = conn.query_row(&contributor_query, [repo_id], |row| row.get(0))?;

    // Get total PRs
    let pr_query = format!("SELECT COUNT(*) FROM pull_requests WHERE repo_id = ?1{}", date_filter);
    let total_prs: i32 = conn.query_row(&pr_query, [repo_id], |row| row.get(0))?;

    // Get total issues
    let issue_query = format!("SELECT COUNT(*) FROM issues WHERE repo_id = ?1{}", date_filter);
    let total_issues: i32 = conn.query_row(&issue_query, [repo_id], |row| row.get(0))?;

    // Get last synced timestamp
    let last_synced_at: Option<String> = conn.query_row(
        "SELECT last_synced_at FROM repositories WHERE id = ?1",
        [repo_id],
        |row| row.get(0),
    ).ok();

    Ok(ProjectSummary {
        total_contributors,
        total_commits: 0, // We don't track individual commits yet
        total_prs,
        total_issues,
        last_synced_at,
    })
}
