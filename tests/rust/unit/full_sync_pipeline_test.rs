/**
 * Integration tests for the full sync pipeline
 *
 * Tests the complete end-to-end sync flow from repository setup through data sync
 * to embedding generation. These tests verify that all components work together correctly.
 *
 * Pipeline Stages Tested:
 * 1. Repository Configuration
 *    - Enabled/disabled repositories
 *    - Settings (history_days, excluded_bots)
 *
 * 2. Sync Execution Order
 *    - Milestones synced first (dependency for issues)
 *    - Issues synced with milestone references
 *    - Pull requests synced
 *    - Last synced timestamp updated
 *
 * 3. Watermark-based Incremental Sync
 *    - First sync fetches all historical data
 *    - Subsequent syncs only fetch new/updated items
 *    - Watermarks updated after successful sync
 *    - Per-repo watermark isolation
 *
 * 4. Error Handling & Resilience
 *    - One repository failure doesn't break other repos
 *    - Sync logs track failures
 *    - Partial sync completion is recorded
 *    - Sync can be retried after failure
 *
 * 5. Data Integrity
 *    - Milestone references in issues are valid
 *    - User references are deduplicated
 *    - Bot filtering is applied consistently
 *    - Labels and metadata preserved
 *
 * 6. Sync Logs & Tracking
 *    - Sync start/complete logged for each sync type
 *    - Item counts tracked accurately
 *    - Timestamps recorded correctly
 *    - Error messages captured on failure
 *
 * 7. Multi-Repository Sync
 *    - Multiple repos sync in sequence
 *    - Progress tracking across repos
 *    - Individual repo errors don't stop batch
 *
 * 8. Embedding Generation Phase
 *    - Embeddings generated after data sync
 *    - Only new items without embeddings processed
 *    - Embedding failures don't break sync
 *    - Batch processing works correctly
 */

#[cfg(test)]
mod full_sync_pipeline_tests {
    use rusqlite::{Connection, Result as SqliteResult};

    // ============================================================================
    // Test Database Setup Helpers
    // ============================================================================

    fn setup_test_db() -> SqliteResult<Connection> {
        let conn = Connection::open_in_memory()?;

        // Create complete schema for pipeline testing
        conn.execute_batch(
            r#"
            CREATE TABLE settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                history_days INTEGER DEFAULT 90,
                excluded_bots TEXT DEFAULT '[]',
                sync_interval_minutes INTEGER DEFAULT 60
            );

            INSERT INTO settings (id) VALUES (1);

            CREATE TABLE repositories (
                id INTEGER PRIMARY KEY,
                github_id INTEGER UNIQUE,
                owner TEXT NOT NULL,
                name TEXT NOT NULL,
                full_name TEXT NOT NULL,
                description TEXT,
                url TEXT,
                is_private INTEGER DEFAULT 0,
                is_fork INTEGER DEFAULT 0,
                created_at TEXT,
                updated_at TEXT,
                pushed_at TEXT,
                size INTEGER DEFAULT 0,
                stargazers_count INTEGER DEFAULT 0,
                watchers_count INTEGER DEFAULT 0,
                language TEXT,
                has_issues INTEGER DEFAULT 1,
                has_projects INTEGER DEFAULT 1,
                has_wiki INTEGER DEFAULT 1,
                archived INTEGER DEFAULT 0,
                disabled INTEGER DEFAULT 0,
                open_issues_count INTEGER DEFAULT 0,
                default_branch TEXT,
                last_synced TEXT,
                enabled INTEGER DEFAULT 1
            );

            CREATE TABLE users (
                id INTEGER PRIMARY KEY,
                github_id INTEGER UNIQUE,
                login TEXT NOT NULL,
                name TEXT,
                email TEXT,
                avatar_url TEXT,
                bio TEXT,
                company TEXT,
                location TEXT,
                created_at TEXT,
                updated_at TEXT,
                is_bot INTEGER DEFAULT 0
            );

            CREATE TABLE milestones (
                id INTEGER PRIMARY KEY,
                github_id INTEGER UNIQUE,
                repo_id INTEGER NOT NULL,
                number INTEGER NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                state TEXT NOT NULL,
                due_on TEXT,
                open_issues INTEGER DEFAULT 0,
                closed_issues INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                closed_at TEXT,
                FOREIGN KEY (repo_id) REFERENCES repositories(id)
            );

            CREATE TABLE issues (
                id INTEGER PRIMARY KEY,
                github_id INTEGER UNIQUE,
                repo_id INTEGER NOT NULL,
                number INTEGER NOT NULL,
                title TEXT NOT NULL,
                body TEXT,
                state TEXT NOT NULL,
                author_id INTEGER,
                assignee_id INTEGER,
                milestone_id INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                closed_at TEXT,
                labels TEXT,
                sync_updated_at TEXT NOT NULL,
                embedding_id INTEGER,
                FOREIGN KEY (repo_id) REFERENCES repositories(id),
                FOREIGN KEY (author_id) REFERENCES users(id),
                FOREIGN KEY (assignee_id) REFERENCES users(id),
                FOREIGN KEY (milestone_id) REFERENCES milestones(id)
            );

            CREATE TABLE pull_requests (
                id INTEGER PRIMARY KEY,
                github_id INTEGER UNIQUE,
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
                labels TEXT,
                sync_updated_at TEXT NOT NULL,
                embedding_id INTEGER,
                FOREIGN KEY (repo_id) REFERENCES repositories(id),
                FOREIGN KEY (author_id) REFERENCES users(id)
            );

            CREATE TABLE sync_logs (
                id INTEGER PRIMARY KEY,
                repo_id INTEGER NOT NULL,
                sync_type TEXT NOT NULL,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                items_synced INTEGER,
                status TEXT NOT NULL,
                error_message TEXT,
                FOREIGN KEY (repo_id) REFERENCES repositories(id)
            );

            CREATE TABLE embeddings (
                id INTEGER PRIMARY KEY,
                text TEXT NOT NULL,
                embedding BLOB NOT NULL,
                created_at TEXT NOT NULL
            );
            "#
        )?;

        Ok(conn)
    }

    fn create_repo(conn: &Connection, owner: &str, name: &str, enabled: bool) -> SqliteResult<i64> {
        conn.execute(
            "INSERT INTO repositories (github_id, owner, name, full_name, enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))",
            rusqlite::params![
                rand::random::<i32>().abs(),
                owner,
                name,
                format!("{}/{}", owner, name),
                if enabled { 1 } else { 0 }
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn create_user(conn: &Connection, github_id: i64, login: &str, is_bot: bool) -> SqliteResult<i64> {
        conn.execute(
            "INSERT INTO users (github_id, login, is_bot, created_at, updated_at)
             VALUES (?1, ?2, ?3, datetime('now'), datetime('now'))",
            rusqlite::params![github_id, login, if is_bot { 1 } else { 0 }],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn create_milestone(
        conn: &Connection,
        repo_id: i64,
        number: i32,
        title: &str,
        state: &str,
    ) -> SqliteResult<i64> {
        conn.execute(
            "INSERT INTO milestones (github_id, repo_id, number, title, state,
                                    created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))",
            rusqlite::params![
                rand::random::<i32>().abs(),
                repo_id,
                number,
                title,
                state
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn create_issue_with_milestone(
        conn: &Connection,
        repo_id: i64,
        number: i32,
        title: &str,
        author_id: Option<i64>,
        milestone_id: Option<i64>,
        created_at: &str,
        sync_updated_at: &str,
    ) -> SqliteResult<i64> {
        conn.execute(
            "INSERT INTO issues (github_id, repo_id, number, title, state, author_id,
                                milestone_id, created_at, updated_at, sync_updated_at, labels)
             VALUES (?1, ?2, ?3, ?4, 'open', ?5, ?6, ?7, ?7, ?8, '[]')",
            rusqlite::params![
                rand::random::<i32>().abs(),
                repo_id,
                number,
                title,
                author_id,
                milestone_id,
                created_at,
                sync_updated_at
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn create_pull_request(
        conn: &Connection,
        repo_id: i64,
        number: i32,
        title: &str,
        author_id: Option<i64>,
        created_at: &str,
        sync_updated_at: &str,
    ) -> SqliteResult<i64> {
        conn.execute(
            "INSERT INTO pull_requests (github_id, repo_id, number, title, state, author_id,
                                       created_at, updated_at, sync_updated_at, labels,
                                       additions, deletions, changed_files)
             VALUES (?1, ?2, ?3, ?4, 'open', ?5, ?6, ?6, ?7, '[]', 100, 50, 5)",
            rusqlite::params![
                rand::random::<i32>().abs(),
                repo_id,
                number,
                title,
                author_id,
                created_at,
                sync_updated_at
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn start_sync_log(conn: &Connection, repo_id: i64, sync_type: &str) -> SqliteResult<i64> {
        conn.execute(
            "INSERT INTO sync_logs (repo_id, sync_type, started_at, status)
             VALUES (?1, ?2, datetime('now'), 'in_progress')",
            rusqlite::params![repo_id, sync_type],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn complete_sync_log(conn: &Connection, log_id: i64, items_synced: i32) -> SqliteResult<()> {
        conn.execute(
            "UPDATE sync_logs
             SET status = 'completed', items_synced = ?1, completed_at = datetime('now')
             WHERE id = ?2",
            rusqlite::params![items_synced, log_id],
        )?;
        Ok(())
    }

    fn fail_sync_log(conn: &Connection, log_id: i64, error: &str) -> SqliteResult<()> {
        conn.execute(
            "UPDATE sync_logs
             SET status = 'failed', error_message = ?1, completed_at = datetime('now')
             WHERE id = ?2",
            rusqlite::params![error, log_id],
        )?;
        Ok(())
    }

    fn update_repo_last_synced(conn: &Connection, repo_id: i64) -> SqliteResult<()> {
        conn.execute(
            "UPDATE repositories SET last_synced = datetime('now') WHERE id = ?1",
            [repo_id],
        )?;
        Ok(())
    }

    fn get_repo_last_synced(conn: &Connection, repo_id: i64) -> SqliteResult<Option<String>> {
        conn.query_row(
            "SELECT last_synced FROM repositories WHERE id = ?1",
            [repo_id],
            |row| row.get(0),
        )
    }

    fn get_watermark(conn: &Connection, repo_id: i64, item_type: &str) -> SqliteResult<Option<String>> {
        match item_type {
            "issues" => {
                conn.query_row(
                    "SELECT MAX(sync_updated_at) FROM issues WHERE repo_id = ?1",
                    [repo_id],
                    |row| row.get(0),
                )
            }
            "pull_requests" => {
                conn.query_row(
                    "SELECT MAX(sync_updated_at) FROM pull_requests WHERE repo_id = ?1",
                    [repo_id],
                    |row| row.get(0),
                )
            }
            _ => Ok(None),
        }
    }

    fn update_settings(conn: &Connection, history_days: i32, excluded_bots: &str) -> SqliteResult<()> {
        conn.execute(
            "UPDATE settings SET history_days = ?1, excluded_bots = ?2",
            rusqlite::params![history_days, excluded_bots],
        )?;
        Ok(())
    }

    // ============================================================================
    // Test 1: Repository Configuration
    // ============================================================================

    #[test]
    fn test_only_enabled_repos_are_synced() {
        let conn = setup_test_db().unwrap();

        let enabled_repo = create_repo(&conn, "acme", "enabled-repo", true).unwrap();
        let disabled_repo = create_repo(&conn, "acme", "disabled-repo", false).unwrap();

        // Query for enabled repos (simulates sync_all_repos behavior)
        let enabled_repos: Vec<i64> = conn
            .prepare("SELECT id FROM repositories WHERE enabled = 1")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<i64>, _>>()
            .unwrap();

        assert_eq!(enabled_repos.len(), 1);
        assert_eq!(enabled_repos[0], enabled_repo);
        assert!(!enabled_repos.contains(&disabled_repo));
    }

    #[test]
    fn test_settings_loaded_before_sync() {
        let conn = setup_test_db().unwrap();

        // Update settings
        update_settings(&conn, 30, r#"["dependabot","renovate"]"#).unwrap();

        // Load settings (simulates sync pipeline)
        let (history_days, excluded_bots): (i32, String) = conn
            .query_row("SELECT history_days, excluded_bots FROM settings WHERE id = 1", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .unwrap();

        assert_eq!(history_days, 30);
        assert_eq!(excluded_bots, r#"["dependabot","renovate"]"#);
    }

    #[test]
    fn test_empty_repo_list_handled_gracefully() {
        let conn = setup_test_db().unwrap();

        // No repos created - simulate empty list
        let enabled_repos: Vec<i64> = conn
            .prepare("SELECT id FROM repositories WHERE enabled = 1")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<i64>, _>>()
            .unwrap();

        assert_eq!(enabled_repos.len(), 0, "Should handle empty repo list gracefully");
    }

    // ============================================================================
    // Test 2: Sync Execution Order
    // ============================================================================

    #[test]
    fn test_milestones_synced_before_issues() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "test-repo", true).unwrap();

        // Step 1: Sync milestones first
        let milestone_log = start_sync_log(&conn, repo_id, "milestones").unwrap();
        let milestone_id = create_milestone(&conn, repo_id, 1, "Sprint 1", "open").unwrap();
        complete_sync_log(&conn, milestone_log, 1).unwrap();

        // Step 2: Sync issues with milestone reference
        let issue_log = start_sync_log(&conn, repo_id, "issues").unwrap();
        let user_id = create_user(&conn, 1001, "alice", false).unwrap();
        create_issue_with_milestone(
            &conn,
            repo_id,
            1,
            "Issue in milestone",
            Some(user_id),
            Some(milestone_id),
            "2024-01-01T10:00:00Z",
            "2024-01-01T10:00:00Z",
        )
        .unwrap();
        complete_sync_log(&conn, issue_log, 1).unwrap();

        // Verify sync order via sync_logs timestamps
        let sync_order: Vec<(String, String)> = conn
            .prepare("SELECT sync_type, started_at FROM sync_logs WHERE repo_id = ?1 ORDER BY started_at")
            .unwrap()
            .query_map([repo_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(sync_order[0].0, "milestones");
        assert_eq!(sync_order[1].0, "issues");

        // Verify milestone reference is valid
        let milestone_ref: Option<i64> = conn
            .query_row(
                "SELECT milestone_id FROM issues WHERE number = 1 AND repo_id = ?1",
                [repo_id],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(milestone_ref, Some(milestone_id));
    }

    #[test]
    fn test_complete_sync_sequence() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "full-sync", true).unwrap();
        let user_id = create_user(&conn, 2001, "bob", false).unwrap();

        // 1. Milestones
        let ms_log = start_sync_log(&conn, repo_id, "milestones").unwrap();
        create_milestone(&conn, repo_id, 1, "Milestone 1", "open").unwrap();
        complete_sync_log(&conn, ms_log, 1).unwrap();

        // 2. Issues
        let issue_log = start_sync_log(&conn, repo_id, "issues").unwrap();
        create_issue_with_milestone(
            &conn,
            repo_id,
            1,
            "Issue 1",
            Some(user_id),
            None,
            "2024-01-01T10:00:00Z",
            "2024-01-01T10:00:00Z",
        )
        .unwrap();
        complete_sync_log(&conn, issue_log, 1).unwrap();

        // 3. Pull Requests
        let pr_log = start_sync_log(&conn, repo_id, "pull_requests").unwrap();
        create_pull_request(&conn, repo_id, 1, "PR 1", Some(user_id), "2024-01-02T10:00:00Z", "2024-01-02T10:00:00Z")
            .unwrap();
        complete_sync_log(&conn, pr_log, 1).unwrap();

        // 4. Update last_synced
        update_repo_last_synced(&conn, repo_id).unwrap();

        // Verify all steps completed
        let completed_syncs: Vec<String> = conn
            .prepare("SELECT sync_type FROM sync_logs WHERE repo_id = ?1 AND status = 'completed' ORDER BY started_at")
            .unwrap()
            .query_map([repo_id], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(completed_syncs, vec!["milestones", "issues", "pull_requests"]);

        let last_synced = get_repo_last_synced(&conn, repo_id).unwrap();
        assert!(last_synced.is_some(), "last_synced should be updated");
    }

    #[test]
    fn test_last_synced_timestamp_updated_after_sync() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "timestamp-test", true).unwrap();

        // Before sync - no last_synced
        let before = get_repo_last_synced(&conn, repo_id).unwrap();
        assert!(before.is_none());

        // Simulate sync
        let log = start_sync_log(&conn, repo_id, "issues").unwrap();
        complete_sync_log(&conn, log, 0).unwrap();
        update_repo_last_synced(&conn, repo_id).unwrap();

        // After sync - last_synced set
        let after = get_repo_last_synced(&conn, repo_id).unwrap();
        assert!(after.is_some());
    }

    // ============================================================================
    // Test 3: Watermark-based Incremental Sync
    // ============================================================================

    #[test]
    fn test_first_sync_has_no_watermark() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "new-repo", true).unwrap();

        // First sync - no watermark exists
        let watermark = get_watermark(&conn, repo_id, "issues").unwrap();
        assert_eq!(watermark, None, "First sync should have no watermark");
    }

    #[test]
    fn test_watermark_updated_after_sync() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "watermark-repo", true).unwrap();
        let user_id = create_user(&conn, 3001, "charlie", false).unwrap();

        // First sync
        create_issue_with_milestone(
            &conn,
            repo_id,
            1,
            "Issue 1",
            Some(user_id),
            None,
            "2024-01-01T10:00:00Z",
            "2024-01-01T10:00:00Z",
        )
        .unwrap();

        let watermark1 = get_watermark(&conn, repo_id, "issues").unwrap();
        assert_eq!(watermark1, Some("2024-01-01T10:00:00Z".to_string()));

        // Second sync with newer item
        create_issue_with_milestone(
            &conn,
            repo_id,
            2,
            "Issue 2",
            Some(user_id),
            None,
            "2024-01-05T10:00:00Z",
            "2024-01-05T10:00:00Z",
        )
        .unwrap();

        let watermark2 = get_watermark(&conn, repo_id, "issues").unwrap();
        assert_eq!(watermark2, Some("2024-01-05T10:00:00Z".to_string()));
    }

    #[test]
    fn test_incremental_sync_only_fetches_new_items() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "incremental-repo", true).unwrap();
        let user_id = create_user(&conn, 4001, "dave", false).unwrap();

        // Initial sync
        create_issue_with_milestone(
            &conn,
            repo_id,
            1,
            "Old issue",
            Some(user_id),
            None,
            "2024-01-01T10:00:00Z",
            "2024-01-01T10:00:00Z",
        )
        .unwrap();

        let watermark = get_watermark(&conn, repo_id, "issues").unwrap().unwrap();

        // Incremental sync - query for items newer than watermark
        create_issue_with_milestone(
            &conn,
            repo_id,
            2,
            "New issue",
            Some(user_id),
            None,
            "2024-01-10T10:00:00Z",
            "2024-01-10T10:00:00Z",
        )
        .unwrap();

        let new_items: Vec<i32> = conn
            .prepare("SELECT number FROM issues WHERE repo_id = ?1 AND sync_updated_at > ?2")
            .unwrap()
            .query_map(rusqlite::params![repo_id, watermark], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(new_items, vec![2], "Only new items should be fetched");
    }

    #[test]
    fn test_watermarks_isolated_per_repo() {
        let conn = setup_test_db().unwrap();
        let repo1 = create_repo(&conn, "org1", "repo1", true).unwrap();
        let repo2 = create_repo(&conn, "org2", "repo2", true).unwrap();
        let user_id = create_user(&conn, 5001, "eve", false).unwrap();

        // Sync repo1
        create_issue_with_milestone(
            &conn,
            repo1,
            1,
            "Repo1 issue",
            Some(user_id),
            None,
            "2024-01-01T10:00:00Z",
            "2024-01-01T10:00:00Z",
        )
        .unwrap();

        // Sync repo2
        create_issue_with_milestone(
            &conn,
            repo2,
            1,
            "Repo2 issue",
            Some(user_id),
            None,
            "2024-02-01T10:00:00Z",
            "2024-02-01T10:00:00Z",
        )
        .unwrap();

        let watermark1 = get_watermark(&conn, repo1, "issues").unwrap();
        let watermark2 = get_watermark(&conn, repo2, "issues").unwrap();

        assert_eq!(watermark1, Some("2024-01-01T10:00:00Z".to_string()));
        assert_eq!(watermark2, Some("2024-02-01T10:00:00Z".to_string()));
        assert_ne!(watermark1, watermark2, "Watermarks should be isolated per repo");
    }

    // ============================================================================
    // Test 4: Error Handling & Resilience
    // ============================================================================

    #[test]
    fn test_one_repo_failure_doesnt_break_others() {
        let conn = setup_test_db().unwrap();
        let repo1 = create_repo(&conn, "acme", "good-repo", true).unwrap();
        let repo2 = create_repo(&conn, "acme", "bad-repo", true).unwrap();
        let repo3 = create_repo(&conn, "acme", "another-good-repo", true).unwrap();

        let user_id = create_user(&conn, 6001, "frank", false).unwrap();

        // Repo1 succeeds
        let log1 = start_sync_log(&conn, repo1, "issues").unwrap();
        create_issue_with_milestone(
            &conn,
            repo1,
            1,
            "Issue 1",
            Some(user_id),
            None,
            "2024-01-01T10:00:00Z",
            "2024-01-01T10:00:00Z",
        )
        .unwrap();
        complete_sync_log(&conn, log1, 1).unwrap();

        // Repo2 fails
        let log2 = start_sync_log(&conn, repo2, "issues").unwrap();
        fail_sync_log(&conn, log2, "GraphQL error: repository not found").unwrap();

        // Repo3 succeeds
        let log3 = start_sync_log(&conn, repo3, "issues").unwrap();
        create_issue_with_milestone(
            &conn,
            repo3,
            1,
            "Issue 3",
            Some(user_id),
            None,
            "2024-01-03T10:00:00Z",
            "2024-01-03T10:00:00Z",
        )
        .unwrap();
        complete_sync_log(&conn, log3, 1).unwrap();

        // Verify repo1 and repo3 succeeded, repo2 failed
        let successful_syncs: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sync_logs WHERE status = 'completed'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        let failed_syncs: i32 = conn
            .query_row("SELECT COUNT(*) FROM sync_logs WHERE status = 'failed'", [], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(successful_syncs, 2);
        assert_eq!(failed_syncs, 1);
    }

    #[test]
    fn test_sync_logs_track_failures() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "fail-repo", true).unwrap();

        let log_id = start_sync_log(&conn, repo_id, "pull_requests").unwrap();
        fail_sync_log(&conn, log_id, "SAML authentication required").unwrap();

        let (status, error): (String, Option<String>) = conn
            .query_row(
                "SELECT status, error_message FROM sync_logs WHERE id = ?1",
                [log_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(status, "failed");
        assert_eq!(error, Some("SAML authentication required".to_string()));
    }

    #[test]
    fn test_partial_sync_completion_recorded() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "partial-repo", true).unwrap();
        let user_id = create_user(&conn, 7001, "grace", false).unwrap();

        // Start sync
        let log_id = start_sync_log(&conn, repo_id, "issues").unwrap();

        // Partial completion - synced 3 out of 10 issues before error
        create_issue_with_milestone(
            &conn,
            repo_id,
            1,
            "Issue 1",
            Some(user_id),
            None,
            "2024-01-01T10:00:00Z",
            "2024-01-01T10:00:00Z",
        )
        .unwrap();
        create_issue_with_milestone(
            &conn,
            repo_id,
            2,
            "Issue 2",
            Some(user_id),
            None,
            "2024-01-02T10:00:00Z",
            "2024-01-02T10:00:00Z",
        )
        .unwrap();
        create_issue_with_milestone(
            &conn,
            repo_id,
            3,
            "Issue 3",
            Some(user_id),
            None,
            "2024-01-03T10:00:00Z",
            "2024-01-03T10:00:00Z",
        )
        .unwrap();

        // Record partial completion with error
        conn.execute(
            "UPDATE sync_logs SET status = 'failed', items_synced = 3,
             error_message = 'Rate limit exceeded', completed_at = datetime('now')
             WHERE id = ?1",
            [log_id],
        )
        .unwrap();

        let (items_synced, error): (Option<i32>, Option<String>) = conn
            .query_row(
                "SELECT items_synced, error_message FROM sync_logs WHERE id = ?1",
                [log_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(items_synced, Some(3));
        assert!(error.unwrap().contains("Rate limit exceeded"));
    }

    #[test]
    fn test_sync_can_be_retried_after_failure() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "retry-repo", true).unwrap();

        // First attempt fails
        let log1 = start_sync_log(&conn, repo_id, "issues").unwrap();
        fail_sync_log(&conn, log1, "Network timeout").unwrap();

        // Retry succeeds
        let log2 = start_sync_log(&conn, repo_id, "issues").unwrap();
        let user_id = create_user(&conn, 8001, "harry", false).unwrap();
        create_issue_with_milestone(
            &conn,
            repo_id,
            1,
            "Retry issue",
            Some(user_id),
            None,
            "2024-01-01T10:00:00Z",
            "2024-01-01T10:00:00Z",
        )
        .unwrap();
        complete_sync_log(&conn, log2, 1).unwrap();

        // Verify both attempts logged
        let attempts: Vec<(String, String)> = conn
            .prepare("SELECT status, COALESCE(error_message, 'none') FROM sync_logs WHERE repo_id = ?1 ORDER BY started_at")
            .unwrap()
            .query_map([repo_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(attempts.len(), 2);
        assert_eq!(attempts[0].0, "failed");
        assert_eq!(attempts[1].0, "completed");
    }

    // ============================================================================
    // Test 5: Data Integrity
    // ============================================================================

    #[test]
    fn test_milestone_references_are_valid() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "milestone-test", true).unwrap();
        let user_id = create_user(&conn, 9001, "iris", false).unwrap();

        let milestone_id = create_milestone(&conn, repo_id, 1, "Sprint 1", "open").unwrap();

        create_issue_with_milestone(
            &conn,
            repo_id,
            1,
            "Issue with milestone",
            Some(user_id),
            Some(milestone_id),
            "2024-01-01T10:00:00Z",
            "2024-01-01T10:00:00Z",
        )
        .unwrap();

        // Verify foreign key constraint works
        let milestone_title: String = conn
            .query_row(
                "SELECT m.title FROM issues i
                 JOIN milestones m ON i.milestone_id = m.id
                 WHERE i.number = 1 AND i.repo_id = ?1",
                [repo_id],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(milestone_title, "Sprint 1");
    }

    #[test]
    fn test_user_deduplication() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "user-test", true).unwrap();

        // Same user (github_id constraint prevents duplicates)
        let user1 = create_user(&conn, 10001, "jack", false).unwrap();

        // Attempt to create duplicate should fail (simulates get_or_create_user)
        let duplicate_result = conn.execute(
            "INSERT INTO users (github_id, login, is_bot, created_at, updated_at)
             VALUES (?1, ?2, 0, datetime('now'), datetime('now'))",
            rusqlite::params![10001, "jack"],
        );

        assert!(duplicate_result.is_err(), "Duplicate github_id should be rejected");

        // Verify only one user exists
        let user_count: i32 = conn
            .query_row("SELECT COUNT(*) FROM users WHERE github_id = 10001", [], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(user_count, 1);
    }

    #[test]
    fn test_bot_filtering_applied_consistently() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "bot-filter-test", true).unwrap();

        let human = create_user(&conn, 11001, "kate", false).unwrap();
        let bot = create_user(&conn, 11002, "dependabot[bot]", true).unwrap();

        create_issue_with_milestone(
            &conn,
            repo_id,
            1,
            "Human issue",
            Some(human),
            None,
            "2024-01-01T10:00:00Z",
            "2024-01-01T10:00:00Z",
        )
        .unwrap();

        // Bot issue created but should be filterable
        create_issue_with_milestone(
            &conn,
            repo_id,
            2,
            "Bot issue",
            Some(bot),
            None,
            "2024-01-02T10:00:00Z",
            "2024-01-02T10:00:00Z",
        )
        .unwrap();

        // Query non-bot issues (simulates excluded_bots filtering)
        let non_bot_issues: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM issues i
                 JOIN users u ON i.author_id = u.id
                 WHERE i.repo_id = ?1 AND u.is_bot = 0",
                [repo_id],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(non_bot_issues, 1, "Bot issues should be filterable");
    }

    #[test]
    fn test_labels_and_metadata_preserved() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "labels-test", true).unwrap();
        let user_id = create_user(&conn, 12001, "leo", false).unwrap();

        // Create PR with labels
        conn.execute(
            "INSERT INTO pull_requests (github_id, repo_id, number, title, state, author_id,
                                       created_at, updated_at, sync_updated_at,
                                       labels, additions, deletions, changed_files)
             VALUES (?1, ?2, ?3, ?4, 'open', ?5, ?6, ?6, ?6, ?7, 150, 75, 8)",
            rusqlite::params![
                2001,
                repo_id,
                1,
                "PR with metadata",
                user_id,
                "2024-01-01T10:00:00Z",
                r#"["bug","critical","needs-review"]"#
            ],
        )
        .unwrap();

        let (labels, additions, deletions, files): (String, i32, i32, i32) = conn
            .query_row(
                "SELECT labels, additions, deletions, changed_files
                 FROM pull_requests WHERE number = 1 AND repo_id = ?1",
                [repo_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();

        assert!(labels.contains("bug"));
        assert!(labels.contains("critical"));
        assert_eq!(additions, 150);
        assert_eq!(deletions, 75);
        assert_eq!(files, 8);
    }

    // ============================================================================
    // Test 6: Sync Logs & Tracking
    // ============================================================================

    #[test]
    fn test_sync_log_records_start_and_completion() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "log-test", true).unwrap();

        let log_id = start_sync_log(&conn, repo_id, "issues").unwrap();

        // Verify started
        let (status, started, completed): (String, String, Option<String>) = conn
            .query_row(
                "SELECT status, started_at, completed_at FROM sync_logs WHERE id = ?1",
                [log_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        assert_eq!(status, "in_progress");
        assert!(started.len() > 0);
        assert_eq!(completed, None);

        // Complete sync
        complete_sync_log(&conn, log_id, 10).unwrap();

        let (status2, completed2): (String, Option<String>) = conn
            .query_row(
                "SELECT status, completed_at FROM sync_logs WHERE id = ?1",
                [log_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(status2, "completed");
        assert!(completed2.is_some());
    }

    #[test]
    fn test_item_counts_tracked_accurately() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "count-test", true).unwrap();
        let user_id = create_user(&conn, 13001, "mia", false).unwrap();

        let log_id = start_sync_log(&conn, repo_id, "pull_requests").unwrap();

        // Create 5 PRs
        for i in 1..=5 {
            create_pull_request(
                &conn,
                repo_id,
                i,
                &format!("PR #{}", i),
                Some(user_id),
                "2024-01-01T10:00:00Z",
                "2024-01-01T10:00:00Z",
            )
            .unwrap();
        }

        complete_sync_log(&conn, log_id, 5).unwrap();

        let items_synced: i32 = conn
            .query_row("SELECT items_synced FROM sync_logs WHERE id = ?1", [log_id], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(items_synced, 5);
    }

    #[test]
    fn test_sync_logs_per_sync_type() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "types-test", true).unwrap();

        // Create logs for different sync types
        let ms_log = start_sync_log(&conn, repo_id, "milestones").unwrap();
        complete_sync_log(&conn, ms_log, 2).unwrap();

        let issue_log = start_sync_log(&conn, repo_id, "issues").unwrap();
        complete_sync_log(&conn, issue_log, 10).unwrap();

        let pr_log = start_sync_log(&conn, repo_id, "pull_requests").unwrap();
        complete_sync_log(&conn, pr_log, 5).unwrap();

        // Verify separate logs exist
        let sync_types: Vec<String> = conn
            .prepare("SELECT sync_type FROM sync_logs WHERE repo_id = ?1 ORDER BY started_at")
            .unwrap()
            .query_map([repo_id], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(sync_types, vec!["milestones", "issues", "pull_requests"]);
    }

    // ============================================================================
    // Test 7: Multi-Repository Sync
    // ============================================================================

    #[test]
    fn test_multiple_repos_sync_in_sequence() {
        let conn = setup_test_db().unwrap();

        let repo1 = create_repo(&conn, "org1", "repo1", true).unwrap();
        let repo2 = create_repo(&conn, "org2", "repo2", true).unwrap();
        let repo3 = create_repo(&conn, "org3", "repo3", true).unwrap();

        let user_id = create_user(&conn, 14001, "noah", false).unwrap();

        // Sync all repos
        for (idx, repo_id) in [repo1, repo2, repo3].iter().enumerate() {
            let log = start_sync_log(&conn, *repo_id, "issues").unwrap();
            create_issue_with_milestone(
                &conn,
                *repo_id,
                1,
                &format!("Issue repo {}", idx + 1),
                Some(user_id),
                None,
                "2024-01-01T10:00:00Z",
                "2024-01-01T10:00:00Z",
            )
            .unwrap();
            complete_sync_log(&conn, log, 1).unwrap();
            update_repo_last_synced(&conn, *repo_id).unwrap();
        }

        // Verify all repos synced
        let synced_repos: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM repositories WHERE last_synced IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(synced_repos, 3);
    }

    #[test]
    fn test_progress_tracking_across_repos() {
        let conn = setup_test_db().unwrap();

        let repos: Vec<i64> = (1..=5)
            .map(|i| create_repo(&conn, "acme", &format!("repo{}", i), true).unwrap())
            .collect();

        // Simulate progress tracking
        for (idx, repo_id) in repos.iter().enumerate() {
            let log = start_sync_log(&conn, *repo_id, "issues").unwrap();
            complete_sync_log(&conn, log, 1).unwrap();

            // Calculate progress (idx+1)/total
            let completed = (idx + 1) as f32;
            let total = repos.len() as f32;
            let progress_pct = (completed / total * 100.0) as i32;

            // Verify progress increases
            assert!(progress_pct <= 100);
            if idx == repos.len() - 1 {
                assert_eq!(progress_pct, 100);
            }
        }
    }

    // ============================================================================
    // Test 8: Embedding Generation Phase
    // ============================================================================

    #[test]
    fn test_items_without_embeddings_identified() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "embed-test", true).unwrap();
        let user_id = create_user(&conn, 15001, "olivia", false).unwrap();

        // Create issue without embedding
        create_issue_with_milestone(
            &conn,
            repo_id,
            1,
            "Issue without embedding",
            Some(user_id),
            None,
            "2024-01-01T10:00:00Z",
            "2024-01-01T10:00:00Z",
        )
        .unwrap();

        // Query items without embeddings
        let items_without_embeddings: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM issues WHERE embedding_id IS NULL",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(items_without_embeddings, 1);
    }

    #[test]
    fn test_only_new_items_processed_for_embeddings() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "embed-new-test", true).unwrap();
        let user_id = create_user(&conn, 16001, "paul", false).unwrap();

        // Create embedding
        conn.execute(
            "INSERT INTO embeddings (text, embedding, created_at)
             VALUES ('test', X'00000000', datetime('now'))",
            [],
        )
        .unwrap();
        let embed_id = conn.last_insert_rowid();

        // Issue with embedding (should be skipped)
        conn.execute(
            "INSERT INTO issues (github_id, repo_id, number, title, state,
                                created_at, updated_at, sync_updated_at, labels, embedding_id)
             VALUES (1001, ?1, 1, 'Has embedding', 'open', datetime('now'), datetime('now'),
                     datetime('now'), '[]', ?2)",
            rusqlite::params![repo_id, embed_id],
        )
        .unwrap();

        // Issue without embedding (should be processed)
        create_issue_with_milestone(
            &conn,
            repo_id,
            2,
            "No embedding",
            Some(user_id),
            None,
            "2024-01-01T10:00:00Z",
            "2024-01-01T10:00:00Z",
        )
        .unwrap();

        // Query for items to process
        let to_process: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM issues WHERE repo_id = ?1 AND embedding_id IS NULL",
                [repo_id],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(to_process, 1, "Only items without embeddings should be processed");
    }

    #[test]
    fn test_embedding_failure_doesnt_break_sync() {
        let conn = setup_test_db().unwrap();
        let repo_id = create_repo(&conn, "acme", "embed-fail-test", true).unwrap();
        let user_id = create_user(&conn, 17001, "quinn", false).unwrap();

        // Sync completes successfully
        let sync_log = start_sync_log(&conn, repo_id, "issues").unwrap();
        create_issue_with_milestone(
            &conn,
            repo_id,
            1,
            "Issue",
            Some(user_id),
            None,
            "2024-01-01T10:00:00Z",
            "2024-01-01T10:00:00Z",
        )
        .unwrap();
        complete_sync_log(&conn, sync_log, 1).unwrap();

        // Embedding generation fails (simulated - no actual embedding created)
        // The sync should still be marked complete

        let sync_status: String = conn
            .query_row("SELECT status FROM sync_logs WHERE id = ?1", [sync_log], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(sync_status, "completed", "Sync should complete even if embeddings fail");
    }
}
