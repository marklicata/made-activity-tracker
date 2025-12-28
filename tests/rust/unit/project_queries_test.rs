//! Unit tests for project_queries.rs
//!
//! Tests for:
//! - get_timeline_events with filters and ordering
//! - get_contributor_stats aggregation
//! - get_activity_heatmap bucketing
//! - get_lifecycle_metrics calculations
//! - get_project_summary aggregation

use rusqlite::{Connection, Result};
use made_activity_tracker::db::project_queries::*;

// ============================================================================
// TEST DATABASE SETUP
// ============================================================================

fn setup_test_db() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;

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
            merged_at TEXT,
            closed_at TEXT,
            additions INTEGER DEFAULT 0,
            deletions INTEGER DEFAULT 0,
            changed_files INTEGER DEFAULT 0,
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
            FOREIGN KEY (repo_id) REFERENCES repositories(id),
            FOREIGN KEY (author_id) REFERENCES users(id)
        );

        CREATE TABLE commits (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sha TEXT UNIQUE NOT NULL,
            repo_id INTEGER NOT NULL,
            author_id INTEGER,
            message TEXT NOT NULL,
            created_at TEXT NOT NULL,
            additions INTEGER DEFAULT 0,
            deletions INTEGER DEFAULT 0,
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
        "INSERT INTO users (github_id, login, name) VALUES (?1, ?2, ?3)",
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
) -> Result<i64> {
    let state = if merged_at.is_some() { "closed" } else { "open" };

    conn.execute(
        "INSERT INTO pull_requests (
            github_id, repo_id, number, title, state, author_id,
            created_at, updated_at, merged_at, additions, deletions
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
            100,
            50
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
            closed_at
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

fn create_commit(
    conn: &Connection,
    repo_id: i64,
    author_id: i64,
    sha: &str,
    created_at: &str,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO commits (sha, repo_id, author_id, message, created_at, additions, deletions)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            sha,
            repo_id,
            author_id,
            "Commit message",
            created_at,
            50,
            25
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

// ============================================================================
// GET_TIMELINE_EVENTS TESTS
// ============================================================================

#[cfg(test)]
mod timeline_tests {
    use super::*;

    #[test]
    fn test_get_timeline_events_basic() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        create_pr(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", None).unwrap();
        create_issue(&conn, repo_id, user_id, 1, "2024-01-02T10:00:00Z", None).unwrap();

        let events = get_timeline_events(&conn, repo_id, None, None, 10).unwrap();

        assert!(events.len() >= 2);
    }

    #[test]
    fn test_get_timeline_events_ordered_by_date() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        create_pr(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", None).unwrap();
        create_pr(&conn, repo_id, user_id, 2, "2024-01-05T10:00:00Z", None).unwrap();
        create_pr(&conn, repo_id, user_id, 3, "2024-01-03T10:00:00Z", None).unwrap();

        let events = get_timeline_events(&conn, repo_id, None, None, 10).unwrap();

        // Should be in descending order (newest first)
        if events.len() >= 2 {
            assert!(events[0].timestamp >= events[1].timestamp);
        }
    }

    #[test]
    fn test_get_timeline_events_with_date_filter() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        create_pr(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", None).unwrap();
        create_pr(&conn, repo_id, user_id, 2, "2024-02-01T10:00:00Z", None).unwrap();

        let events = get_timeline_events(
            &conn,
            repo_id,
            Some("2024-01-01T00:00:00Z"),
            Some("2024-01-31T23:59:59Z"),
            10,
        ).unwrap();

        // Should only have January events
        for event in &events {
            assert!(event.timestamp.starts_with("2024-01"));
        }
    }

    #[test]
    fn test_get_timeline_events_respects_limit() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // Create 10 events
        for i in 1..=10 {
            let date = format!("2024-01-{:02}T10:00:00Z", i);
            create_pr(&conn, repo_id, user_id, i, &date, None).unwrap();
        }

        let events = get_timeline_events(&conn, repo_id, None, None, 5).unwrap();

        assert!(events.len() <= 5);
    }

    #[test]
    fn test_get_timeline_events_empty_repo() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        let events = get_timeline_events(&conn, repo_id, None, None, 10).unwrap();

        assert_eq!(events.len(), 0);
    }
}

// ============================================================================
// GET_CONTRIBUTOR_STATS TESTS
// ============================================================================

#[cfg(test)]
mod contributor_stats_tests {
    use super::*;

    #[test]
    fn test_get_contributor_stats_basic() {
        let conn = setup_test_db().unwrap();
        let user1 = create_user(&conn, 1, "user1").unwrap();
        let user2 = create_user(&conn, 2, "user2").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        create_pr(&conn, repo_id, user1, 1, "2024-01-01T10:00:00Z", None).unwrap();
        create_pr(&conn, repo_id, user2, 2, "2024-01-01T10:00:00Z", None).unwrap();
        create_issue(&conn, repo_id, user1, 1, "2024-01-01T10:00:00Z", None).unwrap();

        let stats = get_contributor_stats(&conn, repo_id, None, None).unwrap();

        assert_eq!(stats.len(), 2);

        let user1_stats = stats.iter().find(|s| s.user.login == "user1").unwrap();
        assert_eq!(user1_stats.pr_count, 1);
        assert_eq!(user1_stats.issue_count, 1);
    }

    #[test]
    fn test_get_contributor_stats_deduplication() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // Same user with multiple contributions
        create_pr(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", None).unwrap();
        create_pr(&conn, repo_id, user_id, 2, "2024-01-01T10:00:00Z", None).unwrap();
        create_issue(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", None).unwrap();

        let stats = get_contributor_stats(&conn, repo_id, None, None).unwrap();

        // Should only have one entry for the user
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].pr_count, 2);
        assert_eq!(stats[0].issue_count, 1);
    }

    #[test]
    fn test_get_contributor_stats_ordered_by_contribution() {
        let conn = setup_test_db().unwrap();
        let user1 = create_user(&conn, 1, "lowcontrib").unwrap();
        let user2 = create_user(&conn, 2, "highcontrib").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // User1: 1 PR
        create_pr(&conn, repo_id, user1, 1, "2024-01-01T10:00:00Z", None).unwrap();

        // User2: 5 PRs
        for i in 2..=6 {
            create_pr(&conn, repo_id, user2, i, "2024-01-01T10:00:00Z", None).unwrap();
        }

        let stats = get_contributor_stats(&conn, repo_id, None, None).unwrap();

        // High contributor should be first
        assert_eq!(stats[0].user.login, "highcontrib");
        assert_eq!(stats[1].user.login, "lowcontrib");
    }

    #[test]
    fn test_get_contributor_stats_with_date_filter() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        create_pr(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", None).unwrap();
        create_pr(&conn, repo_id, user_id, 2, "2024-02-01T10:00:00Z", None).unwrap();

        let stats = get_contributor_stats(
            &conn,
            repo_id,
            Some("2024-01-01T00:00:00Z"),
            Some("2024-01-31T23:59:59Z"),
        ).unwrap();

        // Should only count January contributions
        assert_eq!(stats[0].pr_count, 1);
    }
}

// ============================================================================
// GET_ACTIVITY_HEATMAP TESTS
// ============================================================================

#[cfg(test)]
mod activity_heatmap_tests {
    use super::*;

    #[test]
    fn test_get_activity_heatmap_basic() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        create_pr(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", None).unwrap();
        create_pr(&conn, repo_id, user_id, 2, "2024-01-01T14:00:00Z", None).unwrap();

        let heatmap = get_activity_heatmap(&conn, repo_id, None, None).unwrap();

        assert!(!heatmap.is_empty());
    }

    #[test]
    fn test_get_activity_heatmap_bucketing() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // Multiple events at same hour
        create_pr(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", None).unwrap();
        create_pr(&conn, repo_id, user_id, 2, "2024-01-01T10:30:00Z", None).unwrap();
        create_pr(&conn, repo_id, user_id, 3, "2024-01-01T10:45:00Z", None).unwrap();

        let heatmap = get_activity_heatmap(&conn, repo_id, None, None).unwrap();

        // Should bucket by hour
        let hour_10_cells: Vec<_> = heatmap.iter().filter(|cell| cell.hour == 10).collect();
        let total_activity: i32 = hour_10_cells.iter().map(|c| c.activity_count).sum();

        assert_eq!(total_activity, 3);
    }

    #[test]
    fn test_get_activity_heatmap_empty() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        let heatmap = get_activity_heatmap(&conn, repo_id, None, None).unwrap();

        assert_eq!(heatmap.len(), 0);
    }
}

// ============================================================================
// GET_LIFECYCLE_METRICS TESTS
// ============================================================================

#[cfg(test)]
mod lifecycle_metrics_tests {
    use super::*;

    #[test]
    fn test_get_lifecycle_metrics_basic() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        create_pr(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", Some("2024-01-02T10:00:00Z")).unwrap();
        create_pr(&conn, repo_id, user_id, 2, "2024-01-01T10:00:00Z", None).unwrap();

        let metrics = get_lifecycle_metrics(&conn, repo_id, None, None).unwrap();

        assert_eq!(metrics.total_prs, 2);
        assert_eq!(metrics.merged_prs, 1);
        assert_eq!(metrics.open_prs, 1);
    }

    #[test]
    fn test_get_lifecycle_metrics_issue_counts() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        create_issue(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", Some("2024-01-02T10:00:00Z")).unwrap();
        create_issue(&conn, repo_id, user_id, 2, "2024-01-01T10:00:00Z", None).unwrap();
        create_issue(&conn, repo_id, user_id, 3, "2024-01-01T10:00:00Z", None).unwrap();

        let metrics = get_lifecycle_metrics(&conn, repo_id, None, None).unwrap();

        assert_eq!(metrics.total_issues, 3);
        assert_eq!(metrics.closed_issues, 1);
        assert_eq!(metrics.open_issues, 2);
    }

    #[test]
    fn test_get_lifecycle_metrics_merge_rate_calculation() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // 3 merged out of 4 PRs = 75% merge rate
        create_pr(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", Some("2024-01-02T10:00:00Z")).unwrap();
        create_pr(&conn, repo_id, user_id, 2, "2024-01-01T10:00:00Z", Some("2024-01-02T10:00:00Z")).unwrap();
        create_pr(&conn, repo_id, user_id, 3, "2024-01-01T10:00:00Z", Some("2024-01-02T10:00:00Z")).unwrap();
        create_pr(&conn, repo_id, user_id, 4, "2024-01-01T10:00:00Z", None).unwrap();

        let metrics = get_lifecycle_metrics(&conn, repo_id, None, None).unwrap();

        assert_eq!(metrics.merge_rate, 75.0);
    }

    #[test]
    fn test_get_lifecycle_metrics_zero_prs() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        let metrics = get_lifecycle_metrics(&conn, repo_id, None, None).unwrap();

        assert_eq!(metrics.total_prs, 0);
        assert_eq!(metrics.merge_rate, 0.0);
    }
}

// ============================================================================
// GET_PROJECT_SUMMARY TESTS
// ============================================================================

#[cfg(test)]
mod project_summary_tests {
    use super::*;

    #[test]
    fn test_get_project_summary_basic() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        create_pr(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", None).unwrap();
        create_issue(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", None).unwrap();
        create_commit(&conn, repo_id, user_id, "abc123", "2024-01-01T10:00:00Z").unwrap();

        let summary = get_project_summary(&conn, repo_id, None, None).unwrap();

        assert_eq!(summary.total_prs, 1);
        assert_eq!(summary.total_issues, 1);
        assert_eq!(summary.total_commits, 1);
    }

    #[test]
    fn test_get_project_summary_contributor_count() {
        let conn = setup_test_db().unwrap();
        let user1 = create_user(&conn, 1, "user1").unwrap();
        let user2 = create_user(&conn, 2, "user2").unwrap();
        let user3 = create_user(&conn, 3, "user3").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        create_pr(&conn, repo_id, user1, 1, "2024-01-01T10:00:00Z", None).unwrap();
        create_pr(&conn, repo_id, user2, 2, "2024-01-01T10:00:00Z", None).unwrap();
        create_issue(&conn, repo_id, user3, 1, "2024-01-01T10:00:00Z", None).unwrap();

        let summary = get_project_summary(&conn, repo_id, None, None).unwrap();

        assert_eq!(summary.unique_contributors, 3);
    }

    #[test]
    fn test_get_project_summary_lines_changed() {
        let conn = setup_test_db().unwrap();
        let user_id = create_user(&conn, 1, "testuser").unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // Each PR has 100 additions, 50 deletions
        create_pr(&conn, repo_id, user_id, 1, "2024-01-01T10:00:00Z", None).unwrap();
        create_pr(&conn, repo_id, user_id, 2, "2024-01-01T10:00:00Z", None).unwrap();

        let summary = get_project_summary(&conn, repo_id, None, None).unwrap();

        assert_eq!(summary.lines_added, 200);
        assert_eq!(summary.lines_deleted, 100);
    }

    #[test]
    fn test_get_project_summary_empty_project() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        let summary = get_project_summary(&conn, repo_id, None, None).unwrap();

        assert_eq!(summary.total_prs, 0);
        assert_eq!(summary.total_issues, 0);
        assert_eq!(summary.unique_contributors, 0);
    }
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_nonexistent_repo() {
        let conn = setup_test_db().unwrap();

        let events = get_timeline_events(&conn, 99999, None, None, 10).unwrap();
        assert_eq!(events.len(), 0);

        let stats = get_contributor_stats(&conn, 99999, None, None).unwrap();
        assert_eq!(stats.len(), 0);
    }

    #[test]
    fn test_invalid_date_format() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // Should not crash with invalid dates
        let result = get_timeline_events(&conn, repo_id, Some("invalid-date"), None, 10);
        assert!(result.is_ok());
    }
}
