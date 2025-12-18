use super::models::User;
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
        "SELECT id, github_id, login, name, avatar_url, is_bot FROM users WHERE id = ?1",
        params![user_id],
        |row| {
            Ok(User {
                id: row.get(0)?,
                github_id: row.get(1)?,
                login: row.get(2)?,
                name: row.get(3)?,
                avatar_url: row.get(4)?,
                is_bot: row.get(5)?,
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
            let days_since = (now - last_date).num_days();

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
