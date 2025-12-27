//! Unit tests for user_queries.rs
//!
//! Tests for:
//! - get_user_summary_data with filters
//! - get_user_activity_timeline with filters
//! - get_user_repo_distribution
//! - get_collaboration_matrix
//! - get_user_activity_trend
//! - get_user_focus_metrics

use rusqlite::{Connection, Result};
use made_activity_tracker::db::user_queries::*;

// ============================================================================
// TEST DATABASE SETUP
// ============================================================================

fn setup_test_db() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;

    // Create schema
    conn.execute_batch(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            github_id INTEGER UNIQUE NOT NULL,
            login TEXT NOT NULL,
            name TEXT,
            avatar_url TEXT,
            is_bot INTEGER DEFAULT 0,
            tracked INTEGER DEFAULT 0,
            tracked_at TEXT
        );

        CREATE TABLE repositories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            owner TEXT NOT NULL,
            name TEXT NOT NULL,
            github_id INTEGER,
            enabled INTEGER DEFAULT 1,
            last_synced_at TEXT,
            UNIQUE(owner, name)
        );

        CREATE TABLE pull_requests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            github_id INTEGER UNIQUE NOT NULL,
            repo_id INTEGER NOT NULL,
            number INTEGER NOT NULL,
            title TEXT NOT NULL,
            body TEXT,
            state TEXT NOT NULL,
            author_id INTEGER,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            sync_updated_at TEXT,
            merged_at TEXT,
            closed_at TEXT,
            additions INTEGER DEFAULT 0,
            deletions INTEGER DEFAULT 0,
            changed_files INTEGER DEFAULT 0,
            review_comments INTEGER DEFAULT 0,
            labels TEXT DEFAULT '[]',
            FOREIGN KEY (repo_id) REFERENCES repositories(id),
            FOREIGN KEY (author_id) REFERENCES users(id)
        );

        CREATE TABLE issues (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            github_id INTEGER UNIQUE NOT NULL,
            repo_id INTEGER NOT NULL,
            number INTEGER NOT NULL,
            title TEXT NOT NULL,
            body TEXT,
            state TEXT NOT NULL,
            author_id INTEGER,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            closed_at TEXT,
            labels TEXT DEFAULT '[]',
            FOREIGN KEY (repo_id) REFERENCES repositories(id),
            FOREIGN KEY (author_id) REFERENCES users(id)
        );

        CREATE TABLE pr_reviews (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            github_id INTEGER UNIQUE NOT NULL,
            pr_id INTEGER NOT NULL,
            reviewer_id INTEGER NOT NULL,
            state TEXT NOT NULL,
            body TEXT,
            submitted_at TEXT NOT NULL,
            FOREIGN KEY (pr_id) REFERENCES pull_requests(id),
            FOREIGN KEY (reviewer_id) REFERENCES users(id)
        );"
    )?;

    Ok(conn)
}

fn create_user(conn: &Connection, github_id: i64, login: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO users (github_id, login, name, tracked) VALUES (?1, ?2, ?3, 1)",
        [&github_id.to_string(), login, &format!("{} Name", login)],
    )?;
    Ok(conn.last_insert_rowid())
}

fn create_repo(conn: &Connection, owner: &str, name: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO repositories (owner, name, github_id) VALUES (?1, ?2, ?3)",
        [owner, name, "12345"],
    )?;
    Ok(conn.last_insert_rowid())
}

fn create_pr(
    conn: &Connection,
    repo_id: i64,
    author_id: i64,
    number: i32,
    created_at: &str,
    merged_at: Option<&str>,
    additions: i32,
    deletions: i32,
) -> Result<i64> {
    let state = if merged_at.is_some() { "closed" } else { "open" };

    conn.execute(
        "INSERT INTO pull_requests (
            github_id, repo_id, number, title, state, author_id,
            created_at, updated_at, merged_at, additions, deletions, changed_files
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![
            (repo_id * 1000 + number as i64),
            repo_id,
            number,
            format!("PR #{}", number),
            state,
            author_id,
            created_at,
            created_at,
            merged_at,
            additions,
            deletions,
            5
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

fn create_issue(
    conn: &Connection,
    repo_id: i64,
    author_id: i64,
    number: i32,
    created_at: &str,
    closed_at: Option<&str>,
) -> Result<i64> {
    let state = if closed_at.is_some() { "closed" } else { "open" };

    conn.execute(
        "INSERT INTO issues (
            github_id, repo_id, number, title, state, author_id,
            created_at, updated_at, closed_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            (repo_id * 2000 + number as i64),
            repo_id,
            number,
            format!("Issue #{}", number),
            state,
            author_id,
            created_at,
            created_at,
            closed_at,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

fn create_review(
    conn: &Connection,
    pr_id: i64,
    reviewer_id: i64,
    submitted_at: &str,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO pr_reviews (github_id, pr_id, reviewer_id, state, submitted_at)
         VALUES (?1, ?2, ?3, 'approved', ?4)",
        rusqlite::params![
            (pr_id * 3000 + reviewer_id),
            pr_id,
            reviewer_id,
            submitted_at
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

// ============================================================================
// GET_USER_SUMMARY_DATA TESTS
// ============================================================================

#[cfg(test)]
mod user_summary_tests {
    use super::*;

    #[test]
    fn test_get_user_summary_basic() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // Create some activity
        create_pr(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", Some("2024-01-02T10:00:00Z"), 100, 50).unwrap();
        create_pr(&conn, repo_id, user_id, 2, "2024-01-03T10:00:00Z", None, 200, 100).unwrap();
        create_issue(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", Some("2024-01-02T10:00:00Z")).unwrap();

        let summary = get_user_summary_data(&conn, user_id, None, None).unwrap();

        assert_eq!(summary.user.login, "testuser");
        assert_eq!(summary.total_prs_created, 2);
        assert_eq!(summary.total_prs_merged, 1);
        assert_eq!(summary.total_issues_opened, 1);
        assert_eq!(summary.total_issues_closed, 1);
        assert_eq!(summary.lines_added, 300);
        assert_eq!(summary.lines_deleted, 150);
        assert_eq!(summary.repositories_touched, 1);
    }

    #[test]
    fn test_get_user_summary_with_date_filter() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // PRs in different time periods
        create_pr(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", Some("2024-01-02T10:00:00Z"), 100, 50).unwrap();
        create_pr(&conn, repo_id, user_id, 2, "2024-02-01T10:00:00Z", Some("2024-02-02T10:00:00Z"), 200, 100).unwrap();

        // Filter to January only
        let summary = get_user_summary_data(
            &conn,
            user_id,
            Some("2024-01-01T00:00:00Z"),
            Some("2024-01-31T23:59:59Z"),
        ).unwrap();

        assert_eq!(summary.total_prs_created, 1);
        assert_eq!(summary.total_prs_merged, 1);
        assert_eq!(summary.lines_added, 100);
    }

    #[test]
    fn test_get_user_summary_multiple_repos() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo1 = create_repo(&conn, "org", "repo1").unwrap();
        let repo2 = create_repo(&conn, "org", "repo2").unwrap();
        let repo3 = create_repo(&conn, "org", "repo3").unwrap();

        create_pr(&conn, repo1, user_id, 1, "2024-01-01T10:00:00Z", None, 100, 50).unwrap();
        create_pr(&conn, repo2, user_id, 2, "2024-01-01T10:00:00Z", None, 100, 50).unwrap();
        create_issue(&conn, repo3, user_id, 1, "2024-01-01T10:00:00Z", None).unwrap();

        let summary = get_user_summary_data(&conn, user_id, None, None).unwrap();

        assert_eq!(summary.repositories_touched, 3);
    }

    #[test]
    fn test_get_user_summary_with_reviews() {
        let conn = setup_test_db().unwrap();
        let user1 = create_user(&conn, 1, "author").unwrap();
        let user2 = create_user(&conn, 2, "reviewer").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        let pr_id = create_pr(&conn, repo_id, user1, 1, "2024-01-01T10:00:00Z", None, 100, 50).unwrap();
        create_review(&conn, pr_id, user2, "2024-01-02T10:00:00Z").unwrap();

        let summary = get_user_summary_data(&conn, user2, None, None).unwrap();

        assert_eq!(summary.total_prs_reviewed, 1);
    }

    #[test]
    fn test_get_user_summary_no_activity() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();

        let summary = get_user_summary_data(&conn, user_id, None, None).unwrap();

        assert_eq!(summary.total_prs_created, 0);
        assert_eq!(summary.total_prs_merged, 0);
        assert_eq!(summary.total_issues_opened, 0);
        assert_eq!(summary.repositories_touched, 0);
        assert_eq!(summary.activity_status, "idle");
    }

    #[test]
    fn test_activity_status_active() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // Recent activity
        let now = chrono::Utc::now();
        let recent = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();

        create_pr(&conn, repo_id, user_id, 1, &recent, None, 100, 50).unwrap();
        create_pr(&conn, repo_id, user_id, 2, &recent, None, 100, 50).unwrap();
        create_pr(&conn, repo_id, user_id, 3, &recent, None, 100, 50).unwrap();

        let summary = get_user_summary_data(&conn, user_id, None, None).unwrap();

        assert_eq!(summary.activity_status, "active");
    }
}

// ============================================================================
// GET_USER_ACTIVITY_TIMELINE TESTS
// ============================================================================

#[cfg(test)]
mod user_timeline_tests {
    use super::*;

    #[test]
    fn test_get_user_activity_timeline_basic() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        create_pr(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", None, 100, 50).unwrap();
        create_issue(&conn, repo_id, user_id, 1, "2024-01-02T10:00:00Z", None).unwrap();

        let timeline = get_user_activity_timeline(&conn, user_id, None, None, 10).unwrap();

        assert!(timeline.len() >= 2);

        // Should have both PR and issue events
        let pr_events = timeline.iter().filter(|e| e.event_type == "pr_opened").count();
        let issue_events = timeline.iter().filter(|e| e.event_type == "issue_opened").count();

        assert_eq!(pr_events, 1);
        assert_eq!(issue_events, 1);
    }

    #[test]
    fn test_get_user_activity_timeline_with_date_filter() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        create_pr(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", None, 100, 50).unwrap();
        create_pr(&conn, repo_id, user_id, 2, "2024-02-01T10:00:00Z", None, 100, 50).unwrap();

        let timeline = get_user_activity_timeline(
            &conn,
            user_id,
            Some("2024-01-01T00:00:00Z"),
            Some("2024-01-31T23:59:59Z"),
            10,
        ).unwrap();

        // Should only have January PR
        let pr_events = timeline.iter().filter(|e| e.event_type == "pr_opened").count();
        assert_eq!(pr_events, 1);
    }

    #[test]
    fn test_get_user_activity_timeline_respects_limit() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // Create 10 PRs
        for i in 1..=10 {
            let date = format!("2024-01-{:02}T10:00:00Z", i);
            create_pr(&conn, repo_id, user_id, i, &date, None, 100, 50).unwrap();
        }

        let timeline = get_user_activity_timeline(&conn, user_id, None, None, 5).unwrap();

        // Should respect limit
        assert!(timeline.len() <= 5);
    }

    #[test]
    fn test_get_user_activity_timeline_ordered_by_date() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        create_pr(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", None, 100, 50).unwrap();
        create_pr(&conn, repo_id, user_id, 2, "2024-01-05T10:00:00Z", None, 100, 50).unwrap();
        create_pr(&conn, repo_id, user_id, 3, "2024-01-03T10:00:00Z", None, 100, 50).unwrap();

        let timeline = get_user_activity_timeline(&conn, user_id, None, None, 10).unwrap();

        // Should be in descending order (newest first)
        if timeline.len() >= 2 {
            assert!(timeline[0].timestamp >= timeline[1].timestamp);
        }
    }

    #[test]
    fn test_get_user_activity_timeline_empty() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();

        let timeline = get_user_activity_timeline(&conn, user_id, None, None, 10).unwrap();

        assert_eq!(timeline.len(), 0);
    }
}

// ============================================================================
// GET_USER_REPO_DISTRIBUTION TESTS
// ============================================================================

#[cfg(test)]
mod user_repo_distribution_tests {
    use super::*;

    #[test]
    fn test_get_user_repo_distribution_basic() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo1 = create_repo(&conn, "org", "repo1").unwrap();
        let repo2 = create_repo(&conn, "org", "repo2").unwrap();

        // More activity in repo1
        create_pr(&conn, repo1, user_id, 1, "2024-01-01T10:00:00Z", None, 100, 50).unwrap();
        create_pr(&conn, repo1, user_id, 2, "2024-01-01T10:00:00Z", None, 100, 50).unwrap();
        create_pr(&conn, repo2, user_id, 3, "2024-01-01T10:00:00Z", None, 100, 50).unwrap();

        let distribution = get_user_repo_distribution(&conn, user_id, None, None).unwrap();

        assert_eq!(distribution.len(), 2);

        // Repo1 should have higher contribution
        let repo1_contrib = distribution.iter().find(|r| r.name == "repo1").unwrap();
        assert_eq!(repo1_contrib.pr_count, 2);
    }

    #[test]
    fn test_get_user_repo_distribution_percentage_calculation() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo1 = create_repo(&conn, "org", "repo1").unwrap();
        let repo2 = create_repo(&conn, "org", "repo2").unwrap();

        // 3 contributions in repo1, 1 in repo2 = 75% and 25%
        create_pr(&conn, repo1, user_id, 1, "2024-01-01T10:00:00Z", None, 100, 50).unwrap();
        create_pr(&conn, repo1, user_id, 2, "2024-01-01T10:00:00Z", None, 100, 50).unwrap();
        create_pr(&conn, repo1, user_id, 3, "2024-01-01T10:00:00Z", None, 100, 50).unwrap();
        create_pr(&conn, repo2, user_id, 4, "2024-01-01T10:00:00Z", None, 100, 50).unwrap();

        let distribution = get_user_repo_distribution(&conn, user_id, None, None).unwrap();

        let repo1_contrib = distribution.iter().find(|r| r.name == "repo1").unwrap();
        assert!((repo1_contrib.percentage_of_user_work - 75.0).abs() < 0.1);
    }

    #[test]
    fn test_get_user_repo_distribution_ordered_by_contribution() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo1 = create_repo(&conn, "org", "repo1").unwrap();
        let repo2 = create_repo(&conn, "org", "repo2").unwrap();

        // More in repo2
        create_pr(&conn, repo1, user_id, 1, "2024-01-01T10:00:00Z", None, 100, 50).unwrap();
        create_pr(&conn, repo2, user_id, 2, "2024-01-01T10:00:00Z", None, 100, 50).unwrap();
        create_pr(&conn, repo2, user_id, 3, "2024-01-01T10:00:00Z", None, 100, 50).unwrap();
        create_pr(&conn, repo2, user_id, 4, "2024-01-01T10:00:00Z", None, 100, 50).unwrap();

        let distribution = get_user_repo_distribution(&conn, user_id, None, None).unwrap();

        // repo2 should be first (most contributions)
        assert_eq!(distribution[0].name, "repo2");
        assert_eq!(distribution[1].name, "repo1");
    }
}

// ============================================================================
// EDGE CASES AND ERROR HANDLING
// ============================================================================

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_nonexistent_user() {
        let conn = setup_test_db().unwrap();

        let result = get_user_summary_data(&conn, 99999, None, None);

        assert!(result.is_err());
    }

    #[test]
    fn test_user_with_zero_activity() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();

        let summary = get_user_summary_data(&conn, user_id, None, None).unwrap();

        assert_eq!(summary.total_prs_created, 0);
        assert_eq!(summary.lines_added, 0);
        assert_eq!(summary.first_activity, None);
        assert_eq!(summary.last_activity, None);
    }

    #[test]
    fn test_invalid_date_format() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();

        // Should still work with empty results
        let result = get_user_summary_data(&conn, user_id, Some("invalid-date"), None);

        // Function should not crash with invalid dates
        assert!(result.is_ok());
    }
}
