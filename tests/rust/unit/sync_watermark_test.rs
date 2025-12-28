//! Unit tests for watermark-based incremental sync
//!
//! Tests for:
//! - get_issues_watermark returns correct timestamp
//! - get_prs_watermark returns correct timestamp
//! - get_reviews_watermark returns correct timestamp
//! - Sync only fetches new items after watermark
//! - First sync (no watermark) returns None
//! - Handles empty watermark table
//! - Watermark updates correctly after sync

use rusqlite::{Connection, Result};
use made_activity_tracker::db::queries::*;

// ============================================================================
// TEST DATABASE SETUP
// ============================================================================

fn setup_test_db() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;

    conn.execute_batch(
        "CREATE TABLE repositories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            owner TEXT NOT NULL,
            name TEXT NOT NULL,
            github_id INTEGER,
            enabled INTEGER DEFAULT 1,
            last_synced_at TEXT,
            UNIQUE(owner, name)
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
            sync_updated_at TEXT,
            closed_at TEXT,
            labels TEXT DEFAULT '[]',
            FOREIGN KEY (repo_id) REFERENCES repositories(id)
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
            FOREIGN KEY (repo_id) REFERENCES repositories(id)
        );

        CREATE TABLE pr_reviews (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            github_id INTEGER UNIQUE NOT NULL,
            pr_id INTEGER NOT NULL,
            reviewer_id INTEGER NOT NULL,
            state TEXT NOT NULL,
            body TEXT,
            submitted_at TEXT NOT NULL,
            sync_updated_at TEXT,
            FOREIGN KEY (pr_id) REFERENCES pull_requests(id)
        );"
    )?;

    Ok(conn)
}

fn create_repo(conn: &Connection, owner: &str, name: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO repositories (owner, name, github_id) VALUES (?1, ?2, ?3)",
        [owner, name, "12345"],
    )?;
    Ok(conn.last_insert_rowid())
}

fn create_issue_with_watermark(
    conn: &Connection,
    repo_id: i64,
    number: i32,
    sync_updated_at: &str,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO issues (github_id, repo_id, number, title, state, created_at, updated_at, sync_updated_at)
         VALUES (?1, ?2, ?3, ?4, 'open', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', ?5)",
        rusqlite::params![
            (repo_id * 1000 + number as i64),
            repo_id,
            number,
            format!("Issue #{}", number),
            sync_updated_at
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

fn create_pr_with_watermark(
    conn: &Connection,
    repo_id: i64,
    number: i32,
    sync_updated_at: &str,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO pull_requests (github_id, repo_id, number, title, state, created_at, updated_at, sync_updated_at)
         VALUES (?1, ?2, ?3, ?4, 'open', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', ?5)",
        rusqlite::params![
            (repo_id * 2000 + number as i64),
            repo_id,
            number,
            format!("PR #{}", number),
            sync_updated_at
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

fn create_review_with_watermark(
    conn: &Connection,
    pr_id: i64,
    reviewer_id: i64,
    sync_updated_at: &str,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO pr_reviews (github_id, pr_id, reviewer_id, state, submitted_at, sync_updated_at)
         VALUES (?1, ?2, ?3, 'approved', '2024-01-01T00:00:00Z', ?4)",
        rusqlite::params![
            (pr_id * 3000 + reviewer_id),
            pr_id,
            reviewer_id,
            sync_updated_at
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

// ============================================================================
// GET_ISSUES_WATERMARK TESTS
// ============================================================================

#[cfg(test)]
mod issues_watermark_tests {
    use super::*;

    #[test]
    fn test_get_issues_watermark_returns_latest() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // Create issues with different watermark timestamps
        create_issue_with_watermark(&conn, repo_id, 1, "2024-01-01T10:00:00Z").unwrap();
        create_issue_with_watermark(&conn, repo_id, 2, "2024-01-05T10:00:00Z").unwrap();
        create_issue_with_watermark(&conn, repo_id, 3, "2024-01-03T10:00:00Z").unwrap();

        let watermark = get_issues_watermark(&conn, repo_id).unwrap();

        // Should return the latest timestamp
        assert_eq!(watermark, Some("2024-01-05T10:00:00Z".to_string()));
    }

    #[test]
    fn test_get_issues_watermark_no_issues() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        let watermark = get_issues_watermark(&conn, repo_id).unwrap();

        // Should return None when no issues exist
        assert_eq!(watermark, None);
    }

    #[test]
    fn test_get_issues_watermark_null_sync_updated_at() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // Create issue without sync_updated_at
        conn.execute(
            "INSERT INTO issues (github_id, repo_id, number, title, state, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'open', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')",
            rusqlite::params![1001, repo_id, 1, "Issue #1"],
        ).unwrap();

        let watermark = get_issues_watermark(&conn, repo_id).unwrap();

        // Should return None when sync_updated_at is NULL
        assert_eq!(watermark, None);
    }

    #[test]
    fn test_get_issues_watermark_mixed_null_and_values() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // Create issue with watermark
        create_issue_with_watermark(&conn, repo_id, 1, "2024-01-05T10:00:00Z").unwrap();

        // Create issue without watermark
        conn.execute(
            "INSERT INTO issues (github_id, repo_id, number, title, state, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'open', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')",
            rusqlite::params![1002, repo_id, 2, "Issue #2"],
        ).unwrap();

        let watermark = get_issues_watermark(&conn, repo_id).unwrap();

        // Should return the latest non-null watermark
        assert_eq!(watermark, Some("2024-01-05T10:00:00Z".to_string()));
    }

    #[test]
    fn test_get_issues_watermark_only_for_specific_repo() {
        let conn = setup_test_db().unwrap();
        let repo1 = create_repo(&conn, "org", "repo1").unwrap();
        let repo2 = create_repo(&conn, "org", "repo2").unwrap();

        // Create issues in different repos
        create_issue_with_watermark(&conn, repo1, 1, "2024-01-01T10:00:00Z").unwrap();
        create_issue_with_watermark(&conn, repo2, 2, "2024-01-10T10:00:00Z").unwrap();

        let watermark1 = get_issues_watermark(&conn, repo1).unwrap();
        let watermark2 = get_issues_watermark(&conn, repo2).unwrap();

        // Each repo should have its own watermark
        assert_eq!(watermark1, Some("2024-01-01T10:00:00Z".to_string()));
        assert_eq!(watermark2, Some("2024-01-10T10:00:00Z".to_string()));
    }

    #[test]
    fn test_get_issues_watermark_nonexistent_repo() {
        let conn = setup_test_db().unwrap();

        let watermark = get_issues_watermark(&conn, 99999).unwrap();

        assert_eq!(watermark, None);
    }
}

// ============================================================================
// GET_PRS_WATERMARK TESTS
// ============================================================================

#[cfg(test)]
mod prs_watermark_tests {
    use super::*;

    #[test]
    fn test_get_prs_watermark_returns_latest() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // Create PRs with different watermark timestamps
        create_pr_with_watermark(&conn, repo_id, 1, "2024-01-01T10:00:00Z").unwrap();
        create_pr_with_watermark(&conn, repo_id, 2, "2024-01-08T10:00:00Z").unwrap();
        create_pr_with_watermark(&conn, repo_id, 3, "2024-01-05T10:00:00Z").unwrap();

        let watermark = get_prs_watermark(&conn, repo_id).unwrap();

        // Should return the latest timestamp
        assert_eq!(watermark, Some("2024-01-08T10:00:00Z".to_string()));
    }

    #[test]
    fn test_get_prs_watermark_no_prs() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        let watermark = get_prs_watermark(&conn, repo_id).unwrap();

        // Should return None when no PRs exist
        assert_eq!(watermark, None);
    }

    #[test]
    fn test_get_prs_watermark_null_sync_updated_at() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // Create PR without sync_updated_at
        conn.execute(
            "INSERT INTO pull_requests (github_id, repo_id, number, title, state, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'open', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')",
            rusqlite::params![2001, repo_id, 1, "PR #1"],
        ).unwrap();

        let watermark = get_prs_watermark(&conn, repo_id).unwrap();

        // Should return None when sync_updated_at is NULL
        assert_eq!(watermark, None);
    }

    #[test]
    fn test_get_prs_watermark_only_for_specific_repo() {
        let conn = setup_test_db().unwrap();
        let repo1 = create_repo(&conn, "org", "repo1").unwrap();
        let repo2 = create_repo(&conn, "org", "repo2").unwrap();

        create_pr_with_watermark(&conn, repo1, 1, "2024-01-01T10:00:00Z").unwrap();
        create_pr_with_watermark(&conn, repo2, 2, "2024-01-15T10:00:00Z").unwrap();

        let watermark1 = get_prs_watermark(&conn, repo1).unwrap();
        let watermark2 = get_prs_watermark(&conn, repo2).unwrap();

        assert_eq!(watermark1, Some("2024-01-01T10:00:00Z".to_string()));
        assert_eq!(watermark2, Some("2024-01-15T10:00:00Z".to_string()));
    }
}

// ============================================================================
// GET_REVIEWS_WATERMARK TESTS
// ============================================================================

#[cfg(test)]
mod reviews_watermark_tests {
    use super::*;

    #[test]
    fn test_get_reviews_watermark_returns_latest() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // Create PR
        let pr_id = create_pr_with_watermark(&conn, repo_id, 1, "2024-01-01T10:00:00Z").unwrap();

        // Create reviews with different watermark timestamps
        create_review_with_watermark(&conn, pr_id, 1, "2024-01-02T10:00:00Z").unwrap();
        create_review_with_watermark(&conn, pr_id, 2, "2024-01-10T10:00:00Z").unwrap();
        create_review_with_watermark(&conn, pr_id, 3, "2024-01-05T10:00:00Z").unwrap();

        let watermark = get_reviews_watermark(&conn, repo_id).unwrap();

        // Should return the latest timestamp
        assert_eq!(watermark, Some("2024-01-10T10:00:00Z".to_string()));
    }

    #[test]
    fn test_get_reviews_watermark_no_reviews() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        let watermark = get_reviews_watermark(&conn, repo_id).unwrap();

        // Should return None when no reviews exist
        assert_eq!(watermark, None);
    }

    #[test]
    fn test_get_reviews_watermark_joins_through_pr() {
        let conn = setup_test_db().unwrap();
        let repo1 = create_repo(&conn, "org", "repo1").unwrap();
        let repo2 = create_repo(&conn, "org", "repo2").unwrap();

        // Create PRs in different repos
        let pr1 = create_pr_with_watermark(&conn, repo1, 1, "2024-01-01T10:00:00Z").unwrap();
        let pr2 = create_pr_with_watermark(&conn, repo2, 2, "2024-01-01T10:00:00Z").unwrap();

        // Create reviews for each PR
        create_review_with_watermark(&conn, pr1, 1, "2024-01-02T10:00:00Z").unwrap();
        create_review_with_watermark(&conn, pr2, 2, "2024-01-08T10:00:00Z").unwrap();

        // Watermarks should be separate per repo
        let watermark1 = get_reviews_watermark(&conn, repo1).unwrap();
        let watermark2 = get_reviews_watermark(&conn, repo2).unwrap();

        assert_eq!(watermark1, Some("2024-01-02T10:00:00Z".to_string()));
        assert_eq!(watermark2, Some("2024-01-08T10:00:00Z".to_string()));
    }

    #[test]
    fn test_get_reviews_watermark_null_sync_updated_at() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();
        let pr_id = create_pr_with_watermark(&conn, repo_id, 1, "2024-01-01T10:00:00Z").unwrap();

        // Create review without sync_updated_at
        conn.execute(
            "INSERT INTO pr_reviews (github_id, pr_id, reviewer_id, state, submitted_at)
             VALUES (?1, ?2, ?3, 'approved', '2024-01-01T00:00:00Z')",
            rusqlite::params![3001, pr_id, 1],
        ).unwrap();

        let watermark = get_reviews_watermark(&conn, repo_id).unwrap();

        // Should return None when sync_updated_at is NULL
        assert_eq!(watermark, None);
    }
}

// ============================================================================
// INCREMENTAL SYNC BEHAVIOR TESTS
// ============================================================================

#[cfg(test)]
mod incremental_sync_tests {
    use super::*;

    #[test]
    fn test_first_sync_returns_no_watermark() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // First sync - no watermark should exist
        let issues_wm = get_issues_watermark(&conn, repo_id).unwrap();
        let prs_wm = get_prs_watermark(&conn, repo_id).unwrap();
        let reviews_wm = get_reviews_watermark(&conn, repo_id).unwrap();

        assert_eq!(issues_wm, None);
        assert_eq!(prs_wm, None);
        assert_eq!(reviews_wm, None);
    }

    #[test]
    fn test_watermark_updates_after_sync() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // First sync
        create_issue_with_watermark(&conn, repo_id, 1, "2024-01-01T10:00:00Z").unwrap();
        let wm1 = get_issues_watermark(&conn, repo_id).unwrap();
        assert_eq!(wm1, Some("2024-01-01T10:00:00Z".to_string()));

        // Second sync - new item with later watermark
        create_issue_with_watermark(&conn, repo_id, 2, "2024-01-05T10:00:00Z").unwrap();
        let wm2 = get_issues_watermark(&conn, repo_id).unwrap();
        assert_eq!(wm2, Some("2024-01-05T10:00:00Z".to_string()));

        // Watermark should have advanced
        assert!(wm2 > wm1);
    }

    #[test]
    fn test_watermark_identifies_new_items() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // Initial sync
        create_issue_with_watermark(&conn, repo_id, 1, "2024-01-01T10:00:00Z").unwrap();
        create_issue_with_watermark(&conn, repo_id, 2, "2024-01-02T10:00:00Z").unwrap();

        let watermark = get_issues_watermark(&conn, repo_id).unwrap().unwrap();

        // Simulate next sync - only items updated after watermark should be fetched
        // This would be done in the actual sync code with: WHERE sync_updated_at > ?watermark
        let new_items: Vec<i32> = conn.prepare(
            "SELECT number FROM issues WHERE repo_id = ?1 AND sync_updated_at > ?2"
        ).unwrap()
        .query_map(rusqlite::params![repo_id, watermark], |row| row.get(0))
        .unwrap()
        .collect::<Result<Vec<i32>, _>>()
        .unwrap();

        // No items should be newer than the watermark yet
        assert_eq!(new_items.len(), 0);

        // Add new item after watermark
        create_issue_with_watermark(&conn, repo_id, 3, "2024-01-05T10:00:00Z").unwrap();

        let new_items: Vec<i32> = conn.prepare(
            "SELECT number FROM issues WHERE repo_id = ?1 AND sync_updated_at > ?2"
        ).unwrap()
        .query_map(rusqlite::params![repo_id, watermark], |row| row.get(0))
        .unwrap()
        .collect::<Result<Vec<i32>, _>>()
        .unwrap();

        // New item should be found
        assert_eq!(new_items, vec![3]);
    }

    #[test]
    fn test_watermark_handles_updates_to_existing_items() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // Create initial issue
        let issue_id = create_issue_with_watermark(&conn, repo_id, 1, "2024-01-01T10:00:00Z").unwrap();
        let initial_wm = get_issues_watermark(&conn, repo_id).unwrap();

        // Update the same issue with new watermark
        conn.execute(
            "UPDATE issues SET sync_updated_at = ?1 WHERE id = ?2",
            rusqlite::params!["2024-01-10T10:00:00Z", issue_id],
        ).unwrap();

        let updated_wm = get_issues_watermark(&conn, repo_id).unwrap();

        // Watermark should reflect the update
        assert_eq!(updated_wm, Some("2024-01-10T10:00:00Z".to_string()));
        assert!(updated_wm > initial_wm);
    }
}

// ============================================================================
// EDGE CASES AND ERROR HANDLING
// ============================================================================

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_watermark_with_invalid_timestamps() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        // Create issue with malformed timestamp
        conn.execute(
            "INSERT INTO issues (github_id, repo_id, number, title, state, created_at, updated_at, sync_updated_at)
             VALUES (?1, ?2, ?3, ?4, 'open', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', ?5)",
            rusqlite::params![1001, repo_id, 1, "Issue", "invalid-timestamp"],
        ).unwrap();

        // Should still work - SQLite treats as text
        let watermark = get_issues_watermark(&conn, repo_id).unwrap();
        assert_eq!(watermark, Some("invalid-timestamp".to_string()));
    }

    #[test]
    fn test_watermark_with_very_old_timestamps() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        create_issue_with_watermark(&conn, repo_id, 1, "1970-01-01T00:00:00Z").unwrap();

        let watermark = get_issues_watermark(&conn, repo_id).unwrap();
        assert_eq!(watermark, Some("1970-01-01T00:00:00Z".to_string()));
    }

    #[test]
    fn test_watermark_with_future_timestamps() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "org", "repo").unwrap();

        create_issue_with_watermark(&conn, repo_id, 1, "2099-12-31T23:59:59Z").unwrap();

        let watermark = get_issues_watermark(&conn, repo_id).unwrap();
        assert_eq!(watermark, Some("2099-12-31T23:59:59Z".to_string()));
    }

    #[test]
    fn test_watermark_concurrent_repos() {
        let conn = setup_test_db().unwrap();
        let repo1 = create_repo(&conn, "org", "repo1").unwrap();
        let repo2 = create_repo(&conn, "org", "repo2").unwrap();
        let repo3 = create_repo(&conn, "org", "repo3").unwrap();

        // Create data in all repos with different watermarks
        create_issue_with_watermark(&conn, repo1, 1, "2024-01-01T10:00:00Z").unwrap();
        create_pr_with_watermark(&conn, repo2, 1, "2024-01-05T10:00:00Z").unwrap();
        create_issue_with_watermark(&conn, repo3, 1, "2024-01-10T10:00:00Z").unwrap();

        // All watermarks should be independent
        let wm1 = get_issues_watermark(&conn, repo1).unwrap();
        let wm2 = get_prs_watermark(&conn, repo2).unwrap();
        let wm3 = get_issues_watermark(&conn, repo3).unwrap();

        assert_eq!(wm1, Some("2024-01-01T10:00:00Z".to_string()));
        assert_eq!(wm2, Some("2024-01-05T10:00:00Z".to_string()));
        assert_eq!(wm3, Some("2024-01-10T10:00:00Z".to_string()));
    }
}
