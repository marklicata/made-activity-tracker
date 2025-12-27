use super::models::User;
use super::project_queries::TimelineEvent;
use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

// ============================================================================
// DATA MODELS FOR USER-CENTRIC VIEW
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummary {
    pub user: User,
    pub total_commits: i32,
    pub total_prs_created: i32,
    pub total_prs_merged: i32,
    pub total_prs_reviewed: i32,
    pub total_issues_opened: i32,
    pub total_issues_closed: i32,
    pub lines_added: i32,
    pub lines_deleted: i32,
    pub repositories_touched: i32,
    pub first_activity: Option<String>,
    pub last_activity: Option<String>,
    pub activity_status: String, // "active", "quiet", "idle"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryContribution {
    pub repo_id: i64,
    pub owner: String,
    pub name: String,
    pub pr_count: i32,
    pub issue_count: i32,
    pub review_count: i32,
    pub total_contributions: i32,
    pub percentage_of_user_work: f64,
}

// ============================================================================
// USER SUMMARY QUERIES
// ============================================================================

/// Get summary statistics for a user across all repositories
pub fn get_user_summary_data(
    conn: &Connection,
    user_id: i64,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<UserSummary> {
    // Build date filter
    let date_filter = match (start_date, end_date) {
        (Some(start), Some(end)) => format!(" AND created_at >= '{}' AND created_at <= '{}'", start, end),
        (Some(start), None) => format!(" AND created_at >= '{}'", start),
        (None, Some(end)) => format!(" AND created_at <= '{}'", end),
        (None, None) => String::new(),
    };

    // Get user info
    let user: User = conn.query_row(
        "SELECT id, github_id, login, name, avatar_url, is_bot, tracked, tracked_at FROM users WHERE id = ?1",
        params![user_id],
        |row| {
            Ok(User {
                id: row.get(0)?,
                github_id: row.get(1)?,
                login: row.get(2)?,
                name: row.get(3)?,
                avatar_url: row.get(4)?,
                is_bot: row.get(5)?,
                tracked: row.get(6)?,
                tracked_at: row.get(7)?,
            })
        },
    )?;

    // Get PR stats
    let pr_query = format!(
        "SELECT COUNT(*),
                COALESCE(SUM(CASE WHEN state = 'closed' AND merged_at IS NOT NULL THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(additions), 0),
                COALESCE(SUM(deletions), 0)
         FROM pull_requests
         WHERE author_id = ?1{}",
        date_filter
    );
    let (total_prs_created, total_prs_merged, lines_added, lines_deleted): (i32, i32, i32, i32) =
        conn.query_row(&pr_query, params![user_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;

    // Get review stats
    let review_date_filter = match (start_date, end_date) {
        (Some(start), Some(end)) => format!(" AND r.submitted_at >= '{}' AND r.submitted_at <= '{}'", start, end),
        (Some(start), None) => format!(" AND r.submitted_at >= '{}'", start),
        (None, Some(end)) => format!(" AND r.submitted_at <= '{}'", end),
        (None, None) => String::new(),
    };

    let review_query = format!(
        "SELECT COUNT(*) FROM pr_reviews r WHERE r.reviewer_id = ?1{}",
        review_date_filter
    );
    let total_prs_reviewed: i32 = conn.query_row(&review_query, params![user_id], |row| row.get(0))?;

    // Get issue stats
    let issue_query = format!(
        "SELECT COUNT(*),
                COALESCE(SUM(CASE WHEN state = 'closed' THEN 1 ELSE 0 END), 0)
         FROM issues
         WHERE author_id = ?1{}",
        date_filter
    );
    let (total_issues_opened, total_issues_closed): (i32, i32) =
        conn.query_row(&issue_query, params![user_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

    // Get number of repositories touched
    let repos_query = format!(
        "SELECT COUNT(DISTINCT repo_id) FROM (
            SELECT repo_id FROM pull_requests WHERE author_id = ?1{}
            UNION
            SELECT repo_id FROM issues WHERE author_id = ?1{}
            UNION
            SELECT pr.repo_id FROM pr_reviews r
            JOIN pull_requests pr ON r.pr_id = pr.id
            WHERE r.reviewer_id = ?1{}
         )",
        date_filter, date_filter, review_date_filter
    );
    let repositories_touched: i32 = conn.query_row(&repos_query, params![user_id], |row| row.get(0))?;

    // Get first and last activity timestamps
    let activity_query = format!(
        "SELECT MIN(timestamp) as first, MAX(timestamp) as last FROM (
            SELECT created_at as timestamp FROM pull_requests WHERE author_id = ?1{}
            UNION ALL
            SELECT created_at as timestamp FROM issues WHERE author_id = ?1{}
            UNION ALL
            SELECT r.submitted_at as timestamp FROM pr_reviews r WHERE r.reviewer_id = ?1{}
         )",
        date_filter, date_filter, review_date_filter
    );
    let (first_activity, last_activity): (Option<String>, Option<String>) =
        conn.query_row(&activity_query, params![user_id], |row| {
            Ok((row.get(0).ok(), row.get(1).ok()))
        })
        .unwrap_or((None, None));

    // Calculate activity status based on last activity
    let activity_status = if let Some(ref last) = last_activity {
        if let Ok(last_date) = chrono::DateTime::parse_from_rfc3339(last) {
            let now = chrono::Utc::now();
            let last_date_utc = last_date.with_timezone(&chrono::Utc);
            let days_since = (now - last_date_utc).num_days();

            // Count recent activity (last 7 days)
            let recent_activity_query = format!(
                "SELECT COUNT(*) FROM (
                    SELECT created_at FROM pull_requests WHERE author_id = ?1 AND created_at >= datetime('now', '-7 days')
                    UNION ALL
                    SELECT created_at FROM issues WHERE author_id = ?1 AND created_at >= datetime('now', '-7 days')
                    UNION ALL
                    SELECT r.submitted_at FROM pr_reviews r WHERE r.reviewer_id = ?1 AND r.submitted_at >= datetime('now', '-7 days')
                 )"
            );
            let recent_count: i32 = conn.query_row(&recent_activity_query, params![user_id], |row| row.get(0)).unwrap_or(0);

            if recent_count >= 3 || days_since <= 3 {
                "active".to_string()
            } else if recent_count >= 1 || days_since <= 7 {
                "quiet".to_string()
            } else {
                "idle".to_string()
            }
        } else {
            "idle".to_string()
        }
    } else {
        "idle".to_string()
    };

    Ok(UserSummary {
        user,
        total_commits: 0, // We don't track individual commits yet
        total_prs_created,
        total_prs_merged,
        total_prs_reviewed,
        total_issues_opened,
        total_issues_closed,
        lines_added,
        lines_deleted,
        repositories_touched,
        first_activity,
        last_activity,
        activity_status,
    })
}

// ============================================================================
// USER TIMELINE QUERIES
// ============================================================================

/// Get activity timeline for a user across all repositories
pub fn get_user_activity_timeline(
    conn: &Connection,
    user_id: i64,
    start_date: Option<&str>,
    end_date: Option<&str>,
    limit: i32,
) -> Result<Vec<TimelineEvent>> {
    let mut events = Vec::new();

    // Build date filters
    let date_filter = match (start_date, end_date) {
        (Some(start), Some(end)) => format!(" AND created_at >= '{}' AND created_at <= '{}'", start, end),
        (Some(start), None) => format!(" AND created_at >= '{}'", start),
        (None, Some(end)) => format!(" AND created_at <= '{}'", end),
        (None, None) => String::new(),
    };

    let review_date_filter = match (start_date, end_date) {
        (Some(start), Some(end)) => format!(" AND r.submitted_at >= '{}' AND r.submitted_at <= '{}'", start, end),
        (Some(start), None) => format!(" AND r.submitted_at >= '{}'", start),
        (None, Some(end)) => format!(" AND r.submitted_at <= '{}'", end),
        (None, None) => String::new(),
    };

    // Fetch PR opened events
    let pr_query = format!(
        "SELECT pr.id, pr.number, pr.title, pr.body, pr.created_at, pr.additions, pr.deletions, pr.changed_files,
                u.id, u.github_id, u.login, u.name, u.avatar_url, u.is_bot,
                r.owner, r.name
         FROM pull_requests pr
         LEFT JOIN users u ON pr.author_id = u.id
         LEFT JOIN repositories r ON pr.repo_id = r.id
         WHERE pr.author_id = ?1{}
         ORDER BY pr.created_at DESC
         LIMIT {}",
        date_filter, limit
    );

    let mut stmt = conn.prepare(&pr_query)?;
    let pr_iter = stmt.query_map([user_id], |row| {
        let user = User {
            id: row.get(8)?,
            github_id: row.get(9)?,
            login: row.get(10)?,
            name: row.get(11)?,
            avatar_url: row.get(12)?,
            is_bot: row.get(13)?,
            tracked: false,
            tracked_at: None,
        };

        let metadata = serde_json::json!({
            "pr_number": row.get::<_, i32>(1)?,
            "additions": row.get::<_, i32>(5)?,
            "deletions": row.get::<_, i32>(6)?,
            "changed_files": row.get::<_, i32>(7)?,
            "repository": format!("{}/{}", row.get::<_, String>(14)?, row.get::<_, String>(15)?),
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

    // Fetch issue events
    let issue_query = format!(
        "SELECT i.id, i.number, i.title, i.body, i.created_at,
                u.id, u.github_id, u.login, u.name, u.avatar_url, u.is_bot,
                r.owner, r.name
         FROM issues i
         LEFT JOIN users u ON i.author_id = u.id
         LEFT JOIN repositories r ON i.repo_id = r.id
         WHERE i.author_id = ?1{}
         ORDER BY i.created_at DESC
         LIMIT {}",
        date_filter, limit
    );

    let mut stmt = conn.prepare(&issue_query)?;
    let issue_iter = stmt.query_map([user_id], |row| {
        let user = User {
            id: row.get(5)?,
            github_id: row.get(6)?,
            login: row.get(7)?,
            name: row.get(8)?,
            avatar_url: row.get(9)?,
            is_bot: row.get(10)?,
            tracked: false,
            tracked_at: None,
        };

        let metadata = serde_json::json!({
            "issue_number": row.get::<_, i32>(1)?,
            "repository": format!("{}/{}", row.get::<_, String>(11)?, row.get::<_, String>(12)?),
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

    // Fetch review events
    let review_query = format!(
        "SELECT r.id, r.submitted_at, r.state,
                u.id, u.github_id, u.login, u.name, u.avatar_url, u.is_bot,
                pr.number, pr.title,
                repo.owner, repo.name
         FROM pr_reviews r
         JOIN pull_requests pr ON r.pr_id = pr.id
         LEFT JOIN users u ON r.reviewer_id = u.id
         LEFT JOIN repositories repo ON pr.repo_id = repo.id
         WHERE r.reviewer_id = ?1{}
         ORDER BY r.submitted_at DESC
         LIMIT {}",
        review_date_filter, limit
    );

    let mut stmt = conn.prepare(&review_query)?;
    let review_iter = stmt.query_map([user_id], |row| {
        let user = User {
            id: row.get(3)?,
            github_id: row.get(4)?,
            login: row.get(5)?,
            name: row.get(6)?,
            avatar_url: row.get(7)?,
            is_bot: row.get(8)?,
            tracked: false,
            tracked_at: None,
        };

        let metadata = serde_json::json!({
            "pr_number": row.get::<_, i32>(9)?,
            "review_state": row.get::<_, String>(2)?,
            "repository": format!("{}/{}", row.get::<_, String>(11)?, row.get::<_, String>(12)?),
        });

        Ok(TimelineEvent {
            id: format!("review-{}", row.get::<_, i64>(0)?),
            event_type: "review".to_string(),
            timestamp: row.get(1)?,
            author: user,
            title: row.get(10)?,
            description: None,
            url: None,
            metadata,
        })
    })?;

    for event in review_iter {
        events.push(event?);
    }

    // Sort by timestamp descending
    events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Apply limit
    events.truncate(limit as usize);

    Ok(events)
}

// ============================================================================
// REPOSITORY DISTRIBUTION QUERIES
// ============================================================================

/// Get repository contribution breakdown for a user
pub fn get_user_repo_distribution(
    conn: &Connection,
    user_id: i64,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<Vec<RepositoryContribution>> {
    // Build date filters
    let date_filter = match (start_date, end_date) {
        (Some(start), Some(end)) => format!(" AND created_at >= '{}' AND created_at <= '{}'", start, end),
        (Some(start), None) => format!(" AND created_at >= '{}'", start),
        (None, Some(end)) => format!(" AND created_at <= '{}'", end),
        (None, None) => String::new(),
    };

    let review_date_filter = match (start_date, end_date) {
        (Some(start), Some(end)) => format!(" AND r.submitted_at >= '{}' AND r.submitted_at <= '{}'", start, end),
        (Some(start), None) => format!(" AND r.submitted_at >= '{}'", start),
        (None, Some(end)) => format!(" AND r.submitted_at <= '{}'", end),
        (None, None) => String::new(),
    };

    // Get all repositories the user has contributed to
    let repo_query = format!(
        "SELECT DISTINCT repo_id, owner, name FROM (
            SELECT pr.repo_id, r.owner, r.name
            FROM pull_requests pr
            JOIN repositories r ON pr.repo_id = r.id
            WHERE pr.author_id = ?1{}
            UNION
            SELECT i.repo_id, r.owner, r.name
            FROM issues i
            JOIN repositories r ON i.repo_id = r.id
            WHERE i.author_id = ?1{}
            UNION
            SELECT pr.repo_id, repo.owner, repo.name
            FROM pr_reviews r
            JOIN pull_requests pr ON r.pr_id = pr.id
            JOIN repositories repo ON pr.repo_id = repo.id
            WHERE r.reviewer_id = ?1{}
        )",
        date_filter, date_filter, review_date_filter
    );

    let mut stmt = conn.prepare(&repo_query)?;
    let repos: Vec<(i64, String, String)> = stmt
        .query_map([user_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect::<Result<Vec<_>, _>>()?;

    let mut contributions = Vec::new();
    let mut total_work = 0;

    // For each repo, count contributions
    for (repo_id, owner, name) in repos {
        // Count PRs
        let pr_query = format!(
            "SELECT COUNT(*) FROM pull_requests WHERE repo_id = ?1 AND author_id = ?2{}",
            date_filter
        );
        let pr_count: i32 = conn.query_row(&pr_query, params![repo_id, user_id], |row| row.get(0))?;

        // Count issues
        let issue_query = format!(
            "SELECT COUNT(*) FROM issues WHERE repo_id = ?1 AND author_id = ?2{}",
            date_filter
        );
        let issue_count: i32 = conn.query_row(&issue_query, params![repo_id, user_id], |row| row.get(0))?;

        // Count reviews
        let review_query = format!(
            "SELECT COUNT(*) FROM pr_reviews r
             JOIN pull_requests pr ON r.pr_id = pr.id
             WHERE pr.repo_id = ?1 AND r.reviewer_id = ?2{}",
            review_date_filter
        );
        let review_count: i32 = conn.query_row(&review_query, params![repo_id, user_id], |row| row.get(0))?;

        let total = pr_count + issue_count + review_count;
        total_work += total;

        contributions.push(RepositoryContribution {
            repo_id,
            owner,
            name,
            pr_count,
            issue_count,
            review_count,
            total_contributions: total,
            percentage_of_user_work: 0.0, // Will calculate after getting total
        });
    }

    // Calculate percentages
    for contrib in &mut contributions {
        contrib.percentage_of_user_work = if total_work > 0 {
            (contrib.total_contributions as f64 / total_work as f64) * 100.0
        } else {
            0.0
        };
    }

    // Sort by total contributions descending
    contributions.sort_by(|a, b| b.total_contributions.cmp(&a.total_contributions));

    Ok(contributions)
}

// ============================================================================
// ACTIVITY TRENDS & FOCUS ANALYSIS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDataPoint {
    pub timestamp: String,      // Date in YYYY-MM-DD format (or YYYY-MM for month, YYYY-Www for week)
    pub pr_count: i32,
    pub review_count: i32,
    pub issue_count: i32,
    pub total_activity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusMetrics {
    pub repos_touched: i32,
    pub top_repo_percentage: f64,     // % of work in most-worked repo
    pub concentration_score: f64,     // 0-1, higher = more focused
    pub repos_distribution: Vec<(String, i32)>, // (repo_name, contribution_count)
}

// ============================================================================
// COLLABORATION MATRIX QUERIES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionStats {
    pub reviews_given: i32,     // How many times this user reviewed the other user's PRs
    pub reviews_received: i32,  // How many times the other user reviewed this user's PRs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationMatrix {
    pub users: Vec<User>,
    pub interactions: std::collections::HashMap<String, std::collections::HashMap<String, InteractionStats>>,
}

/// Get collaboration matrix showing interactions between team members
/// This shows who reviews whose code
pub fn get_collaboration_matrix(
    conn: &Connection,
    user_ids: Vec<i64>,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<CollaborationMatrix> {
    use std::collections::HashMap;

    // First, get all users
    let placeholders = user_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let user_query = format!(
        "SELECT id, github_id, login, name, avatar_url, is_bot
         FROM users
         WHERE id IN ({})",
        placeholders
    );

    let mut stmt = conn.prepare(&user_query)?;
    let user_params: Vec<&dyn rusqlite::ToSql> = user_ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();

    let users: Vec<User> = stmt
        .query_map(&user_params[..], |row| {
            Ok(User {
                id: row.get(0)?,
                github_id: row.get(1)?,
                login: row.get(2)?,
                name: row.get(3)?,
                avatar_url: row.get(4)?,
                is_bot: row.get(5)?,
                tracked: false,
                tracked_at: None,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Build date filter for reviews
    let date_filter = match (start_date, end_date) {
        (Some(start), Some(end)) => format!(" AND r.submitted_at >= '{}' AND r.submitted_at <= '{}'", start, end),
        (Some(start), None) => format!(" AND r.submitted_at >= '{}'", start),
        (None, Some(end)) => format!(" AND r.submitted_at <= '{}'", end),
        (None, None) => String::new(),
    };

    // Initialize the interactions map
    let mut interactions: HashMap<String, HashMap<String, InteractionStats>> = HashMap::new();

    // Initialize all user pairs with zero interactions
    for user in &users {
        let mut user_interactions = HashMap::new();
        for other_user in &users {
            if user.id != other_user.id {
                user_interactions.insert(
                    other_user.login.clone(),
                    InteractionStats {
                        reviews_given: 0,
                        reviews_received: 0,
                    },
                );
            }
        }
        interactions.insert(user.login.clone(), user_interactions);
    }

    // Query for all reviews between the tracked users
    // This counts how many times reviewer_id reviewed author_id's PRs
    let review_query = format!(
        "SELECT pr.author_id, r.reviewer_id, COUNT(*) as review_count
         FROM pr_reviews r
         JOIN pull_requests pr ON r.pr_id = pr.id
         WHERE pr.author_id IN ({})
           AND r.reviewer_id IN ({})
           AND pr.author_id != r.reviewer_id{}
         GROUP BY pr.author_id, r.reviewer_id",
        placeholders, placeholders, date_filter
    );

    let mut stmt = conn.prepare(&review_query)?;
    let mut all_params = user_params.clone();
    all_params.extend(&user_params);

    let review_data: Vec<(i64, i64, i32)> = stmt
        .query_map(&all_params[..], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Create a map from user_id to login for quick lookup
    let id_to_login: HashMap<i64, String> = users.iter()
        .map(|u| (u.id, u.login.clone()))
        .collect();

    // Fill in the interaction counts
    for (pr_author_id, reviewer_id, count) in review_data {
        if let (Some(pr_author_login), Some(reviewer_login)) = (
            id_to_login.get(&pr_author_id),
            id_to_login.get(&reviewer_id),
        ) {
            // Reviewer reviewed PR author's code
            if let Some(reviewer_interactions) = interactions.get_mut(reviewer_login) {
                if let Some(stats) = reviewer_interactions.get_mut(pr_author_login) {
                    stats.reviews_given = count;
                }
            }

            // PR author received review from reviewer
            if let Some(author_interactions) = interactions.get_mut(pr_author_login) {
                if let Some(stats) = author_interactions.get_mut(reviewer_login) {
                    stats.reviews_received = count;
                }
            }
        }
    }

    Ok(CollaborationMatrix {
        users,
        interactions,
    })
}

/// Get activity trend data for a user over time
/// Granularity can be "day", "week", or "month"
pub fn get_user_activity_trend(
    conn: &Connection,
    user_id: i64,
    start_date: Option<&str>,
    end_date: Option<&str>,
    granularity: &str,
) -> Result<Vec<ActivityDataPoint>> {
    // Determine date truncation based on granularity
    let date_trunc = match granularity {
        "week" => "strftime('%Y-W%W', created_at)",
        "month" => "strftime('%Y-%m', created_at)",
        _ => "date(created_at)", // default to day
    };

    let review_date_trunc = match granularity {
        "week" => "strftime('%Y-W%W', r.submitted_at)",
        "month" => "strftime('%Y-%m', r.submitted_at)",
        _ => "date(r.submitted_at)",
    };

    // Build date filters
    let pr_date_filter = match (start_date, end_date) {
        (Some(start), Some(end)) => format!(" AND created_at >= '{}' AND created_at <= '{}'", start, end),
        (Some(start), None) => format!(" AND created_at >= '{}'", start),
        (None, Some(end)) => format!(" AND created_at <= '{}'", end),
        (None, None) => String::new(),
    };

    let review_date_filter = match (start_date, end_date) {
        (Some(start), Some(end)) => format!(" AND r.submitted_at >= '{}' AND r.submitted_at <= '{}'", start, end),
        (Some(start), None) => format!(" AND r.submitted_at >= '{}'", start),
        (None, Some(end)) => format!(" AND r.submitted_at <= '{}'", end),
        (None, None) => String::new(),
    };

    // Query PRs grouped by time period
    let pr_query = format!(
        "SELECT {} as period, COUNT(*) as count
         FROM pull_requests
         WHERE author_id = ?1{}
         GROUP BY period
         ORDER BY period",
        date_trunc, pr_date_filter
    );

    let mut pr_counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
    let mut stmt = conn.prepare(&pr_query)?;
    let pr_iter = stmt.query_map([user_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)))?;
    for result in pr_iter {
        let (period, count) = result?;
        pr_counts.insert(period, count);
    }

    // Query reviews grouped by time period
    let review_query = format!(
        "SELECT {} as period, COUNT(*) as count
         FROM pr_reviews r
         WHERE r.reviewer_id = ?1{}
         GROUP BY period
         ORDER BY period",
        review_date_trunc, review_date_filter
    );

    let mut review_counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
    let mut stmt = conn.prepare(&review_query)?;
    let review_iter = stmt.query_map([user_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)))?;
    for result in review_iter {
        let (period, count) = result?;
        review_counts.insert(period, count);
    }

    // Query issues grouped by time period
    let issue_query = format!(
        "SELECT {} as period, COUNT(*) as count
         FROM issues
         WHERE author_id = ?1{}
         GROUP BY period
         ORDER BY period",
        date_trunc, pr_date_filter
    );

    let mut issue_counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
    let mut stmt = conn.prepare(&issue_query)?;
    let issue_iter = stmt.query_map([user_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)))?;
    for result in issue_iter {
        let (period, count) = result?;
        issue_counts.insert(period, count);
    }

    // Combine all periods
    let mut all_periods: std::collections::HashSet<String> = std::collections::HashSet::new();
    all_periods.extend(pr_counts.keys().cloned());
    all_periods.extend(review_counts.keys().cloned());
    all_periods.extend(issue_counts.keys().cloned());

    let mut data_points: Vec<ActivityDataPoint> = all_periods
        .into_iter()
        .map(|period| {
            let pr_count = *pr_counts.get(&period).unwrap_or(&0);
            let review_count = *review_counts.get(&period).unwrap_or(&0);
            let issue_count = *issue_counts.get(&period).unwrap_or(&0);
            ActivityDataPoint {
                timestamp: period,
                pr_count,
                review_count,
                issue_count,
                total_activity: pr_count + review_count + issue_count,
            }
        })
        .collect();

    // Sort by timestamp
    data_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(data_points)
}

/// Get focus metrics for a user showing repository concentration
pub fn get_user_focus_metrics(
    conn: &Connection,
    user_id: i64,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<FocusMetrics> {
    // Reuse the repository distribution function
    let contributions = get_user_repo_distribution(conn, user_id, start_date, end_date)?;

    if contributions.is_empty() {
        return Ok(FocusMetrics {
            repos_touched: 0,
            top_repo_percentage: 0.0,
            concentration_score: 0.0,
            repos_distribution: vec![],
        });
    }

    let repos_touched = contributions.len() as i32;
    let top_repo_percentage = contributions[0].percentage_of_user_work;

    // Calculate concentration score using Herfindahl-Hirschman Index (HHI)
    // HHI = sum of squares of market shares (here, percentage of work)
    // Result is 0-1, where 1 = all work in one repo (highly focused)
    let hhi: f64 = contributions
        .iter()
        .map(|c| {
            let share = c.percentage_of_user_work / 100.0;
            share * share
        })
        .sum();

    // Create simplified distribution for display
    let repos_distribution: Vec<(String, i32)> = contributions
        .iter()
        .take(10) // Top 10 repos
        .map(|c| (format!("{}/{}", c.owner, c.name), c.total_contributions))
        .collect();

    Ok(FocusMetrics {
        repos_touched,
        top_repo_percentage,
        concentration_score: hhi,
        repos_distribution,
    })
}
