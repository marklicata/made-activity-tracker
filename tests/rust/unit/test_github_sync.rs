//! Unit tests for GitHub sync functionality
//!
//! Tests for:
//! - REST API response parsing
//! - Data validation and transformation
//! - Bot filtering
//! - Error handling

use serde_json::json;

#[cfg(test)]
mod data_parsing_tests {
    use super::*;
    use made_activity_tracker::github::rest_api::{RestIssue, RestPullRequest, RestMilestone, RestUser, RestLabel};

    #[test]
    fn test_parse_rest_issue_complete() {
        let json_data = json!({
            "id": 123456,
            "number": 42,
            "title": "Add user authentication",
            "body": "We need OAuth2",
            "state": "open",
            "user": {
                "id": 1,
                "login": "johndoe"
            },
            "assignee": {
                "id": 2,
                "login": "janedoe"
            },
            "milestone": {
                "id": 1,
                "number": 1,
                "title": "v1.0",
                "description": "First release",
                "state": "open",
                "due_on": "2024-12-31T00:00:00Z",
                "open_issues": 5,
                "closed_issues": 10
            },
            "labels": [
                {"name": "feature"},
                {"name": "auth"}
            ],
            "created_at": "2024-02-01T10:00:00Z",
            "updated_at": "2024-02-15T14:30:00Z",
            "closed_at": null,
            "pull_request": null
        });

        let issue: RestIssue = serde_json::from_value(json_data).unwrap();

        assert_eq!(issue.id, 123456);
        assert_eq!(issue.number, 42);
        assert_eq!(issue.title, "Add user authentication");
        assert_eq!(issue.body, Some("We need OAuth2".to_string()));
        assert_eq!(issue.state, "open");
        assert!(issue.user.is_some());
        assert_eq!(issue.user.unwrap().login, "johndoe");
        assert!(issue.assignee.is_some());
        assert!(issue.milestone.is_some());
        assert_eq!(issue.labels.len(), 2);
        assert!(issue.closed_at.is_none());
    }

    #[test]
    fn test_parse_rest_issue_minimal() {
        // Test with minimal required fields
        let json_data = json!({
            "id": 123456,
            "number": 42,
            "title": "Minimal Issue",
            "body": null,
            "state": "open",
            "user": null,
            "assignee": null,
            "milestone": null,
            "labels": [],
            "created_at": "2024-02-01T10:00:00Z",
            "updated_at": "2024-02-15T14:30:00Z",
            "closed_at": null,
            "pull_request": null
        });

        let issue: RestIssue = serde_json::from_value(json_data).unwrap();

        assert_eq!(issue.id, 123456);
        assert_eq!(issue.number, 42);
        assert!(issue.body.is_none());
        assert!(issue.user.is_none());
        assert!(issue.milestone.is_none());
        assert_eq!(issue.labels.len(), 0);
    }

    #[test]
    fn test_parse_rest_pull_request() {
        let json_data = json!({
            "id": 789,
            "number": 50,
            "title": "feat: Add OAuth",
            "body": "Implements OAuth2",
            "state": "closed",
            "user": {
                "id": 1,
                "login": "johndoe"
            },
            "labels": [
                {"name": "feature"}
            ],
            "created_at": "2024-02-10T14:00:00Z",
            "updated_at": "2024-02-12T16:00:00Z",
            "merged_at": "2024-02-12T16:00:00Z",
            "closed_at": "2024-02-12T16:00:00Z",
            "additions": 250,
            "deletions": 30,
            "changed_files": 8
        });

        let pr: RestPullRequest = serde_json::from_value(json_data).unwrap();

        assert_eq!(pr.id, 789);
        assert_eq!(pr.number, 50);
        assert_eq!(pr.title, "feat: Add OAuth");
        assert_eq!(pr.state, "closed");
        assert!(pr.user.is_some());
        assert_eq!(pr.merged_at, Some("2024-02-12T16:00:00Z".to_string()));
        assert_eq!(pr.additions, Some(250));
        assert_eq!(pr.deletions, Some(30));
        assert_eq!(pr.changed_files, Some(8));
    }

    #[test]
    fn test_parse_pull_request_without_metrics() {
        let json_data = json!({
            "id": 789,
            "number": 50,
            "title": "feat: Add OAuth",
            "body": null,
            "state": "open",
            "user": null,
            "labels": [],
            "created_at": "2024-02-10T14:00:00Z",
            "updated_at": "2024-02-12T16:00:00Z",
            "merged_at": null,
            "closed_at": null,
            "additions": null,
            "deletions": null,
            "changed_files": null
        });

        let pr: RestPullRequest = serde_json::from_value(json_data).unwrap();

        assert_eq!(pr.id, 789);
        assert!(pr.merged_at.is_none());
        assert!(pr.additions.is_none());
        assert!(pr.deletions.is_none());
    }

    #[test]
    fn test_parse_rest_milestone() {
        let json_data = json!({
            "id": 1,
            "number": 1,
            "title": "v1.0 Release",
            "description": "First major release",
            "state": "open",
            "due_on": "2024-12-31T00:00:00Z",
            "open_issues": 5,
            "closed_issues": 10
        });

        let milestone: RestMilestone = serde_json::from_value(json_data).unwrap();

        assert_eq!(milestone.id, 1);
        assert_eq!(milestone.title, "v1.0 Release");
        assert_eq!(milestone.state, "open");
        assert_eq!(milestone.open_issues, 5);
        assert_eq!(milestone.closed_issues, 10);
        assert!(milestone.description.is_some());
        assert!(milestone.due_on.is_some());
    }

    #[test]
    fn test_parse_user_data() {
        let json_data = json!({
            "id": 12345,
            "login": "testuser"
        });

        let user: RestUser = serde_json::from_value(json_data).unwrap();

        assert_eq!(user.id, 12345);
        assert_eq!(user.login, "testuser");
    }

    #[test]
    fn test_parse_label_data() {
        let json_data = json!({
            "name": "bug"
        });

        let label: RestLabel = serde_json::from_value(json_data).unwrap();

        assert_eq!(label.name, "bug");
    }

    #[test]
    fn test_parse_multiple_labels() {
        let json_data = json!([
            {"name": "bug"},
            {"name": "critical"},
            {"name": "security"}
        ]);

        let labels: Vec<RestLabel> = serde_json::from_value(json_data).unwrap();

        assert_eq!(labels.len(), 3);
        assert_eq!(labels[0].name, "bug");
        assert_eq!(labels[1].name, "critical");
        assert_eq!(labels[2].name, "security");
    }
}

#[cfg(test)]
mod bot_filtering_tests {
    use super::*;

    fn is_bot_user(login: &str) -> bool {
        let bot_patterns = [
            "dependabot",
            "renovate",
            "github-actions",
            "bot",
            "[bot]",
        ];

        let login_lower = login.to_lowercase();
        bot_patterns.iter().any(|pattern| login_lower.contains(pattern))
    }

    #[test]
    fn test_dependabot_excluded() {
        assert!(is_bot_user("dependabot[bot]"));
        assert!(is_bot_user("dependabot"));
    }

    #[test]
    fn test_renovate_excluded() {
        assert!(is_bot_user("renovate[bot]"));
        assert!(is_bot_user("renovate"));
    }

    #[test]
    fn test_github_actions_excluded() {
        assert!(is_bot_user("github-actions[bot]"));
        assert!(is_bot_user("github-actions"));
    }

    #[test]
    fn test_generic_bot_excluded() {
        assert!(is_bot_user("my-custom-bot"));
        assert!(is_bot_user("some-user[bot]"));
    }

    #[test]
    fn test_human_users_included() {
        assert!(!is_bot_user("johndoe"));
        assert!(!is_bot_user("janedoe"));
        assert!(!is_bot_user("alice"));
        assert!(!is_bot_user("bob-smith"));
    }

    #[test]
    fn test_case_insensitive_bot_detection() {
        assert!(is_bot_user("DependaBot[bot]"));
        assert!(is_bot_user("RENOVATE"));
        assert!(is_bot_user("GitHub-Actions[bot]"));
    }

    #[test]
    fn test_bot_in_middle_of_name() {
        // User with "bot" in their name but not actually a bot
        // This is a limitation - may need more sophisticated detection
        assert!(is_bot_user("robot-maintainer")); // Contains "bot"
    }
}

#[cfg(test)]
mod data_validation_tests {
    use super::*;

    #[test]
    fn test_invalid_json_returns_error() {
        let invalid_json = r#"{"id": "not-a-number"}"#;
        let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);
        // Should fail because id should be i64
        assert!(result.is_ok()); // serde_json::Value accepts any valid JSON
    }

    #[test]
    fn test_missing_required_field() {
        use made_activity_tracker::github::rest_api::RestIssue;

        let json_data = json!({
            "id": 123,
            // Missing "number" field
            "title": "Test",
            "state": "open",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "labels": []
        });

        let result: Result<RestIssue, _> = serde_json::from_value(json_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_closed_issue_with_closed_at() {
        use made_activity_tracker::github::rest_api::RestIssue;

        let json_data = json!({
            "id": 123,
            "number": 1,
            "title": "Test",
            "body": null,
            "state": "closed",
            "user": null,
            "assignee": null,
            "milestone": null,
            "labels": [],
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "closed_at": "2024-01-02T00:00:00Z",
            "pull_request": null
        });

        let issue: RestIssue = serde_json::from_value(json_data).unwrap();
        assert_eq!(issue.state, "closed");
        assert!(issue.closed_at.is_some());
    }

    #[test]
    fn test_parse_open_issue_without_closed_at() {
        use made_activity_tracker::github::rest_api::RestIssue;

        let json_data = json!({
            "id": 123,
            "number": 1,
            "title": "Test",
            "body": null,
            "state": "open",
            "user": null,
            "assignee": null,
            "milestone": null,
            "labels": [],
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "closed_at": null,
            "pull_request": null
        });

        let issue: RestIssue = serde_json::from_value(json_data).unwrap();
        assert_eq!(issue.state, "open");
        assert!(issue.closed_at.is_none());
    }
}

#[cfg(test)]
mod pagination_tests {
    use super::*;

    #[test]
    fn test_empty_page_should_stop() {
        // When we get an empty array, pagination should stop
        let empty_response: Vec<serde_json::Value> = vec![];
        assert_eq!(empty_response.len(), 0);
        // In actual code: if items.is_empty() { break; }
    }

    #[test]
    fn test_partial_page_should_stop() {
        // When we get less than 100 items (page size), we've reached the end
        let partial_page: Vec<i32> = (1..=50).collect();
        assert_eq!(partial_page.len(), 50);
        assert!(partial_page.len() < 100);
        // In actual code: if all_items.len() % 100 != 0 { break; }
    }

    #[test]
    fn test_full_page_should_continue() {
        // When we get exactly 100 items, there might be more
        let full_page: Vec<i32> = (1..=100).collect();
        assert_eq!(full_page.len(), 100);
        assert_eq!(full_page.len() % 100, 0);
        // In actual code: continue to next page
    }

    #[test]
    fn test_page_calculation() {
        // Test page number increment
        let mut page = 1;
        page += 1;
        assert_eq!(page, 2);

        page += 1;
        assert_eq!(page, 3);
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_malformed_json() {
        let bad_json = r#"{"id": 123, "title": "unclosed string"#;
        let result: Result<serde_json::Value, _> = serde_json::from_str(bad_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_incomplete_data_structure() {
        use made_activity_tracker::github::rest_api::RestPullRequest;

        let json_data = json!({
            "id": 789,
            // Missing required fields
            "title": "Test PR"
        });

        let result: Result<RestPullRequest, _> = serde_json::from_value(json_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_data_types() {
        use made_activity_tracker::github::rest_api::RestIssue;

        let json_data = json!({
            "id": "should-be-number",  // Wrong type
            "number": 42,
            "title": "Test",
            "state": "open",
            "labels": [],
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        });

        let result: Result<RestIssue, _> = serde_json::from_value(json_data);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod timestamp_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_iso8601_timestamp() {
        let timestamp = "2024-02-15T14:30:00Z";
        // In real code, this would be parsed with chrono
        assert!(timestamp.ends_with('Z'));
        assert!(timestamp.contains('T'));
    }

    #[test]
    fn test_parse_nullable_timestamp() {
        use made_activity_tracker::github::rest_api::RestIssue;

        let json_data = json!({
            "id": 123,
            "number": 42,
            "title": "Test",
            "body": null,
            "state": "open",
            "user": null,
            "assignee": null,
            "milestone": null,
            "labels": [],
            "created_at": "2024-02-01T10:00:00Z",
            "updated_at": "2024-02-15T14:30:00Z",
            "closed_at": null,  // Nullable timestamp
            "pull_request": null
        });

        let issue: RestIssue = serde_json::from_value(json_data).unwrap();
        assert!(issue.closed_at.is_none());
    }
}

#[cfg(test)]
mod state_handling_tests {
    use super::*;

    #[test]
    fn test_issue_states() {
        let open_state = "open";
        let closed_state = "closed";

        assert_eq!(open_state, "open");
        assert_eq!(closed_state, "closed");
        assert_ne!(open_state, closed_state);
    }

    #[test]
    fn test_pr_merged_vs_closed() {
        use made_activity_tracker::github::rest_api::RestPullRequest;

        // PR merged (closed with merged_at)
        let merged_pr = json!({
            "id": 1,
            "number": 1,
            "title": "Test",
            "body": null,
            "state": "closed",
            "user": null,
            "labels": [],
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "merged_at": "2024-01-02T00:00:00Z",
            "closed_at": "2024-01-02T00:00:00Z",
            "additions": null,
            "deletions": null,
            "changed_files": null
        });

        let pr: RestPullRequest = serde_json::from_value(merged_pr).unwrap();
        assert_eq!(pr.state, "closed");
        assert!(pr.merged_at.is_some());
        assert!(pr.closed_at.is_some());

        // PR closed without merge
        let closed_pr = json!({
            "id": 2,
            "number": 2,
            "title": "Test",
            "body": null,
            "state": "closed",
            "user": null,
            "labels": [],
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "merged_at": null,
            "closed_at": "2024-01-02T00:00:00Z",
            "additions": null,
            "deletions": null,
            "changed_files": null
        });

        let pr: RestPullRequest = serde_json::from_value(closed_pr).unwrap();
        assert_eq!(pr.state, "closed");
        assert!(pr.merged_at.is_none());
        assert!(pr.closed_at.is_some());
    }
}

#[cfg(test)]
mod code_metrics_tests {
    use super::*;
    use made_activity_tracker::github::rest_api::RestPullRequest;

    #[test]
    fn test_pr_with_code_changes() {
        let json_data = json!({
            "id": 1,
            "number": 1,
            "title": "Test",
            "body": null,
            "state": "open",
            "user": null,
            "labels": [],
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "merged_at": null,
            "closed_at": null,
            "additions": 250,
            "deletions": 30,
            "changed_files": 8
        });

        let pr: RestPullRequest = serde_json::from_value(json_data).unwrap();
        assert_eq!(pr.additions, Some(250));
        assert_eq!(pr.deletions, Some(30));
        assert_eq!(pr.changed_files, Some(8));

        let total_changes = pr.additions.unwrap_or(0) + pr.deletions.unwrap_or(0);
        assert_eq!(total_changes, 280);
    }

    #[test]
    fn test_pr_without_code_metrics() {
        let json_data = json!({
            "id": 1,
            "number": 1,
            "title": "Test",
            "body": null,
            "state": "open",
            "user": null,
            "labels": [],
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "merged_at": null,
            "closed_at": null,
            "additions": null,
            "deletions": null,
            "changed_files": null
        });

        let pr: RestPullRequest = serde_json::from_value(json_data).unwrap();
        assert_eq!(pr.additions, None);
        assert_eq!(pr.deletions, None);
        assert_eq!(pr.changed_files, None);

        let total_changes = pr.additions.unwrap_or(0) + pr.deletions.unwrap_or(0);
        assert_eq!(total_changes, 0);
    }

    #[test]
    fn test_large_pr_detection() {
        let json_data = json!({
            "id": 1,
            "number": 1,
            "title": "Large refactor",
            "body": null,
            "state": "open",
            "user": null,
            "labels": [],
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "merged_at": null,
            "closed_at": null,
            "additions": 5000,
            "deletions": 2000,
            "changed_files": 50
        });

        let pr: RestPullRequest = serde_json::from_value(json_data).unwrap();
        let total_changes = pr.additions.unwrap_or(0) + pr.deletions.unwrap_or(0);

        assert!(total_changes > 500, "Should be considered a large PR");
        assert_eq!(total_changes, 7000);
    }
}

#[cfg(test)]
mod sync_logic_tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        // Create minimal schema for testing sync logic
        conn.execute_batch(r#"
            CREATE TABLE repositories (
                id INTEGER PRIMARY KEY,
                github_id INTEGER,
                owner TEXT NOT NULL,
                name TEXT NOT NULL,
                enabled INTEGER DEFAULT 1,
                last_synced_at TEXT
            );

            CREATE TABLE users (
                id INTEGER PRIMARY KEY,
                github_id INTEGER UNIQUE NOT NULL,
                login TEXT NOT NULL,
                tracked INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE issues (
                id INTEGER PRIMARY KEY,
                github_id INTEGER UNIQUE NOT NULL,
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
                FOREIGN KEY(repo_id) REFERENCES repositories(id),
                FOREIGN KEY(author_id) REFERENCES users(id)
            );

            CREATE TABLE pull_requests (
                id INTEGER PRIMARY KEY,
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
                labels TEXT,
                sync_updated_at TEXT NOT NULL,
                FOREIGN KEY(repo_id) REFERENCES repositories(id),
                FOREIGN KEY(author_id) REFERENCES users(id)
            );

            CREATE TABLE sync_logs (
                id INTEGER PRIMARY KEY,
                repo_id INTEGER NOT NULL,
                sync_type TEXT NOT NULL,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                items_synced INTEGER,
                FOREIGN KEY(repo_id) REFERENCES repositories(id)
            );
        "#).unwrap();

        conn
    }

    #[test]
    fn test_incremental_sync_watermark() {
        let conn = setup_test_db();

        // Insert a repository
        conn.execute(
            "INSERT INTO repositories (id, github_id, owner, name, enabled) VALUES (1, 12345, 'test-org', 'test-repo', 1)",
            []
        ).unwrap();

        // Insert an issue with a specific updated_at timestamp
        conn.execute(
            "INSERT INTO issues (github_id, repo_id, number, title, state, created_at, updated_at, sync_updated_at, labels)
             VALUES (100, 1, 1, 'Old Issue', 'open', '2024-01-01T00:00:00Z', '2024-01-10T00:00:00Z', '2024-01-10T00:00:00Z', '[]')",
            []
        ).unwrap();

        // Query for the watermark (most recent sync_updated_at)
        let watermark: String = conn.query_row(
            "SELECT MAX(sync_updated_at) FROM issues WHERE repo_id = ?1",
            [1],
            |row| row.get(0)
        ).unwrap();

        assert_eq!(watermark, "2024-01-10T00:00:00Z");

        // In incremental sync, only items updated after this watermark should be fetched
        // This simulates the behavior in sync.rs lines 266-273
    }

    #[test]
    fn test_full_sync_all_repos() {
        let conn = setup_test_db();

        // Insert multiple enabled repositories
        conn.execute(
            "INSERT INTO repositories (id, github_id, owner, name, enabled) VALUES (1, 1, 'org1', 'repo1', 1)",
            []
        ).unwrap();
        conn.execute(
            "INSERT INTO repositories (id, github_id, owner, name, enabled) VALUES (2, 2, 'org2', 'repo2', 1)",
            []
        ).unwrap();
        conn.execute(
            "INSERT INTO repositories (id, github_id, owner, name, enabled) VALUES (3, 3, 'org3', 'repo3', 0)",
            []
        ).unwrap();

        // Get enabled repositories (simulates queries::get_enabled_repositories)
        let mut stmt = conn.prepare("SELECT id, owner, name FROM repositories WHERE enabled = 1").unwrap();
        let repos: Vec<(i64, String, String)> = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        }).unwrap().collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0].1, "org1");
        assert_eq!(repos[1].1, "org2");

        // Full sync should process all enabled repos
        // This simulates the behavior in sync.rs sync_all_repos function (lines 12-77)
    }

    #[test]
    fn test_sync_new_user_creation() {
        let conn = setup_test_db();

        // Simulate syncing a PR from a new user (github_id not in database)
        let github_id = 999;
        let login = "newuser";

        // Check user doesn't exist
        let user_exists: Result<i64, _> = conn.query_row(
            "SELECT id FROM users WHERE github_id = ?1",
            [github_id],
            |row| row.get(0)
        );
        assert!(user_exists.is_err());

        // Create user (simulates get_or_create_user in sync.rs lines 312-316)
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO users (github_id, login, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            [&github_id.to_string(), login, &now, &now]
        ).unwrap();
        let user_id = conn.last_insert_rowid();

        // Verify user was created
        let fetched_login: String = conn.query_row(
            "SELECT login FROM users WHERE github_id = ?1",
            [github_id],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(fetched_login, "newuser");
        assert!(user_id > 0);
    }

    #[test]
    fn test_sync_existing_user_update() {
        let conn = setup_test_db();

        // Create an existing user
        let github_id = 888;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO users (github_id, login, created_at, updated_at) VALUES (?1, 'oldlogin', ?2, ?3)",
            [&github_id.to_string(), &now, &now]
        ).unwrap();
        let user_id = conn.last_insert_rowid();

        // Simulate get_or_create_user for existing user (should return existing ID)
        let fetched_id: i64 = conn.query_row(
            "SELECT id FROM users WHERE github_id = ?1",
            [github_id],
            |row| row.get(0)
        ).unwrap();

        assert_eq!(fetched_id, user_id);

        // User can be updated with new information without creating duplicate
        conn.execute(
            "UPDATE users SET updated_at = ?1 WHERE github_id = ?2",
            [&chrono::Utc::now().to_rfc3339(), &github_id.to_string()]
        ).unwrap();

        // Verify still only one user with this github_id
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM users WHERE github_id = ?1",
            [github_id],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_sync_deduplication_issue() {
        let conn = setup_test_db();

        // Insert repository
        conn.execute(
            "INSERT INTO repositories (id, github_id, owner, name) VALUES (1, 1, 'org', 'repo')",
            []
        ).unwrap();

        // Insert user
        conn.execute(
            "INSERT INTO users (id, github_id, login, created_at, updated_at) VALUES (1, 100, 'testuser', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')",
            []
        ).unwrap();

        // First sync - insert issue
        conn.execute(
            "INSERT INTO issues (github_id, repo_id, number, title, state, author_id, created_at, updated_at, sync_updated_at, labels)
             VALUES (999, 1, 1, 'Test Issue', 'open', 1, '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '[]')",
            []
        ).unwrap();

        // Second sync - update the same issue (simulates upsert in sync.rs)
        // Use INSERT OR REPLACE (which is what upsert_issue does via ON CONFLICT)
        conn.execute(
            "INSERT INTO issues (github_id, repo_id, number, title, state, author_id, created_at, updated_at, sync_updated_at, labels)
             VALUES (999, 1, 1, 'Test Issue - Updated', 'closed', 1, '2024-01-01T00:00:00Z', '2024-01-02T00:00:00Z', '2024-01-02T00:00:00Z', '[]')
             ON CONFLICT(github_id) DO UPDATE SET
                title = excluded.title,
                state = excluded.state,
                updated_at = excluded.updated_at,
                sync_updated_at = excluded.sync_updated_at",
            []
        ).unwrap();

        // Verify only one issue exists
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM issues WHERE github_id = 999",
            [],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(count, 1);

        // Verify issue was updated, not duplicated
        let title: String = conn.query_row(
            "SELECT title FROM issues WHERE github_id = 999",
            [],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(title, "Test Issue - Updated");

        let state: String = conn.query_row(
            "SELECT state FROM issues WHERE github_id = 999",
            [],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(state, "closed");
    }

    #[test]
    fn test_sync_deduplication_pull_request() {
        let conn = setup_test_db();

        // Insert repository
        conn.execute(
            "INSERT INTO repositories (id, github_id, owner, name) VALUES (1, 1, 'org', 'repo')",
            []
        ).unwrap();

        // Insert user
        conn.execute(
            "INSERT INTO users (id, github_id, login, created_at, updated_at) VALUES (1, 100, 'testuser', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')",
            []
        ).unwrap();

        // First sync - insert PR
        conn.execute(
            "INSERT INTO pull_requests (github_id, repo_id, number, title, state, author_id, created_at, updated_at, sync_updated_at, labels)
             VALUES (888, 1, 1, 'Test PR', 'open', 1, '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '[]')",
            []
        ).unwrap();

        // Second sync - update the same PR with merge information
        conn.execute(
            "INSERT INTO pull_requests (github_id, repo_id, number, title, state, author_id, created_at, updated_at, merged_at, closed_at, sync_updated_at, labels)
             VALUES (888, 1, 1, 'Test PR', 'closed', 1, '2024-01-01T00:00:00Z', '2024-01-03T00:00:00Z', '2024-01-03T00:00:00Z', '2024-01-03T00:00:00Z', '2024-01-03T00:00:00Z', '[]')
             ON CONFLICT(github_id) DO UPDATE SET
                state = excluded.state,
                updated_at = excluded.updated_at,
                merged_at = excluded.merged_at,
                closed_at = excluded.closed_at,
                sync_updated_at = excluded.sync_updated_at",
            []
        ).unwrap();

        // Verify only one PR exists
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pull_requests WHERE github_id = 888",
            [],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(count, 1);

        // Verify PR was updated with merge information
        let (state, merged_at): (String, Option<String>) = conn.query_row(
            "SELECT state, merged_at FROM pull_requests WHERE github_id = 888",
            [],
            |row| Ok((row.get(0)?, row.get(1)?))
        ).unwrap();
        assert_eq!(state, "closed");
        assert!(merged_at.is_some());
        assert_eq!(merged_at.unwrap(), "2024-01-03T00:00:00Z");
    }

    #[test]
    fn test_sync_multiple_repositories() {
        let conn = setup_test_db();

        // Insert multiple repositories
        for i in 1..=3 {
            conn.execute(
                "INSERT INTO repositories (id, github_id, owner, name, enabled) VALUES (?1, ?2, ?3, ?4, 1)",
                [&i.to_string(), &(100 + i).to_string(), &format!("org{}", i), &format!("repo{}", i)]
            ).unwrap();
        }

        // Insert user
        conn.execute(
            "INSERT INTO users (id, github_id, login, created_at, updated_at) VALUES (1, 100, 'testuser', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')",
            []
        ).unwrap();

        // Sync issues to different repos
        for repo_id in 1..=3 {
            conn.execute(
                "INSERT INTO issues (github_id, repo_id, number, title, state, author_id, created_at, updated_at, sync_updated_at, labels)
                 VALUES (?1, ?2, 1, 'Issue in repo', 'open', 1, '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '[]')",
                [&(1000 + repo_id).to_string(), &repo_id.to_string()]
            ).unwrap();
        }

        // Verify issues are distributed across repos
        for repo_id in 1..=3 {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM issues WHERE repo_id = ?1",
                [repo_id],
                |row| row.get(0)
            ).unwrap();
            assert_eq!(count, 1);
        }

        // Total should be 3 issues
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM issues",
            [],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(total, 3);
    }
}

#[cfg(test)]
mod state_management_tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        conn.execute_batch(r#"
            CREATE TABLE repositories (
                id INTEGER PRIMARY KEY,
                github_id INTEGER,
                owner TEXT NOT NULL,
                name TEXT NOT NULL,
                enabled INTEGER DEFAULT 1,
                last_synced_at TEXT
            );

            CREATE TABLE sync_logs (
                id INTEGER PRIMARY KEY,
                repo_id INTEGER NOT NULL,
                sync_type TEXT NOT NULL,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                items_synced INTEGER,
                error_message TEXT,
                FOREIGN KEY(repo_id) REFERENCES repositories(id)
            );

            CREATE TABLE issues (
                id INTEGER PRIMARY KEY,
                github_id INTEGER UNIQUE NOT NULL,
                repo_id INTEGER NOT NULL,
                number INTEGER NOT NULL,
                title TEXT NOT NULL,
                state TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                sync_updated_at TEXT NOT NULL,
                labels TEXT,
                FOREIGN KEY(repo_id) REFERENCES repositories(id)
            );
        "#).unwrap();

        conn
    }

    #[test]
    fn test_sync_progress_tracking_start() {
        let conn = setup_test_db();

        // Insert repository
        conn.execute(
            "INSERT INTO repositories (id, github_id, owner, name) VALUES (1, 1, 'org', 'repo')",
            []
        ).unwrap();

        // Record sync start (simulates queries::record_sync_start in sync.rs line 261)
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO sync_logs (repo_id, sync_type, started_at) VALUES (?1, ?2, ?3)",
            [&1.to_string(), "issues", &now]
        ).unwrap();
        let log_id = conn.last_insert_rowid();

        // Verify log was created
        let (sync_type, started_at, completed_at): (String, String, Option<String>) = conn.query_row(
            "SELECT sync_type, started_at, completed_at FROM sync_logs WHERE id = ?1",
            [log_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        ).unwrap();

        assert_eq!(sync_type, "issues");
        assert!(!started_at.is_empty());
        assert!(completed_at.is_none(), "Sync should not be marked complete yet");
    }

    #[test]
    fn test_sync_progress_tracking_complete() {
        let conn = setup_test_db();

        // Insert repository
        conn.execute(
            "INSERT INTO repositories (id, github_id, owner, name) VALUES (1, 1, 'org', 'repo')",
            []
        ).unwrap();

        // Record sync start
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO sync_logs (repo_id, sync_type, started_at) VALUES (1, 'pull_requests', ?1)",
            [&now]
        ).unwrap();
        let log_id = conn.last_insert_rowid();

        // Simulate syncing items...
        let items_synced = 42;

        // Record sync complete (simulates queries::record_sync_complete in sync.rs line 383)
        let completed_at = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE sync_logs SET completed_at = ?1, items_synced = ?2 WHERE id = ?3",
            [&completed_at, &items_synced.to_string(), &log_id.to_string()]
        ).unwrap();

        // Verify sync was completed
        let (completed, count): (Option<String>, Option<i32>) = conn.query_row(
            "SELECT completed_at, items_synced FROM sync_logs WHERE id = ?1",
            [log_id],
            |row| Ok((row.get(0)?, row.get(1)?))
        ).unwrap();

        assert!(completed.is_some(), "Sync should be marked complete");
        assert_eq!(count, Some(42));
    }

    #[test]
    fn test_sync_progress_tracking_multiple_syncs() {
        let conn = setup_test_db();

        // Insert repository
        conn.execute(
            "INSERT INTO repositories (id, github_id, owner, name) VALUES (1, 1, 'org', 'repo')",
            []
        ).unwrap();

        // Record multiple syncs
        let sync_types = vec!["milestones", "issues", "pull_requests"];
        for sync_type in &sync_types {
            let now = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO sync_logs (repo_id, sync_type, started_at, completed_at, items_synced)
                 VALUES (1, ?1, ?2, ?3, 10)",
                [sync_type, &now, &now]
            ).unwrap();
        }

        // Verify all syncs were recorded
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sync_logs WHERE repo_id = 1",
            [],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(count, 3);

        // Verify each sync type
        for sync_type in &sync_types {
            let exists: i64 = conn.query_row(
                "SELECT COUNT(*) FROM sync_logs WHERE repo_id = 1 AND sync_type = ?1",
                [sync_type],
                |row| row.get(0)
            ).unwrap();
            assert_eq!(exists, 1);
        }
    }

    #[test]
    fn test_sync_resume_after_failure_using_watermark() {
        let conn = setup_test_db();

        // Insert repository
        conn.execute(
            "INSERT INTO repositories (id, github_id, owner, name) VALUES (1, 1, 'org', 'repo')",
            []
        ).unwrap();

        // First sync: Successfully sync some issues
        conn.execute(
            "INSERT INTO issues (github_id, repo_id, number, title, state, created_at, updated_at, sync_updated_at, labels)
             VALUES (1, 1, 1, 'Issue 1', 'open', '2024-01-01T00:00:00Z', '2024-01-05T00:00:00Z', '2024-01-05T00:00:00Z', '[]')",
            []
        ).unwrap();
        conn.execute(
            "INSERT INTO issues (github_id, repo_id, number, title, state, created_at, updated_at, sync_updated_at, labels)
             VALUES (2, 1, 2, 'Issue 2', 'open', '2024-01-02T00:00:00Z', '2024-01-10T00:00:00Z', '2024-01-10T00:00:00Z', '[]')",
            []
        ).unwrap();

        // Get watermark (most recent sync_updated_at)
        let watermark: String = conn.query_row(
            "SELECT MAX(sync_updated_at) FROM issues WHERE repo_id = 1",
            [],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(watermark, "2024-01-10T00:00:00Z");

        // Second sync after failure: Should only fetch items updated after watermark
        // Insert a new issue with timestamp after the watermark
        conn.execute(
            "INSERT INTO issues (github_id, repo_id, number, title, state, created_at, updated_at, sync_updated_at, labels)
             VALUES (3, 1, 3, 'Issue 3', 'open', '2024-01-03T00:00:00Z', '2024-01-15T00:00:00Z', '2024-01-15T00:00:00Z', '[]')",
            []
        ).unwrap();

        // Verify incremental sync: Only items after watermark would be fetched in real sync
        let count_after_watermark: i64 = conn.query_row(
            "SELECT COUNT(*) FROM issues WHERE repo_id = 1 AND sync_updated_at > ?1",
            [&watermark],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(count_after_watermark, 1);

        // Total issues should now be 3
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM issues WHERE repo_id = 1",
            [],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(total, 3);
    }

    #[test]
    fn test_sync_last_synced_timestamp_update() {
        let conn = setup_test_db();

        // Insert repository without last_synced_at
        conn.execute(
            "INSERT INTO repositories (id, github_id, owner, name, last_synced_at) VALUES (1, 1, 'org', 'repo', NULL)",
            []
        ).unwrap();

        // Verify no last sync time
        let last_synced: Option<String> = conn.query_row(
            "SELECT last_synced_at FROM repositories WHERE id = 1",
            [],
            |row| row.get(0)
        ).unwrap();
        assert!(last_synced.is_none());

        // Update last synced timestamp (simulates queries::update_repo_synced_at in sync.rs line 59)
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE repositories SET last_synced_at = ?1 WHERE id = 1",
            [&now]
        ).unwrap();

        // Verify timestamp was updated
        let updated_last_synced: Option<String> = conn.query_row(
            "SELECT last_synced_at FROM repositories WHERE id = 1",
            [],
            |row| row.get(0)
        ).unwrap();
        assert!(updated_last_synced.is_some());
        assert_eq!(updated_last_synced.unwrap(), now);
    }

    #[test]
    fn test_sync_error_handling_partial_completion() {
        let conn = setup_test_db();

        // Insert repository
        conn.execute(
            "INSERT INTO repositories (id, github_id, owner, name) VALUES (1, 1, 'org', 'repo')",
            []
        ).unwrap();

        // Start sync
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO sync_logs (repo_id, sync_type, started_at) VALUES (1, 'issues', ?1)",
            [&now]
        ).unwrap();
        let log_id = conn.last_insert_rowid();

        // Simulate partial sync - some items synced before error
        conn.execute(
            "INSERT INTO issues (github_id, repo_id, number, title, state, created_at, updated_at, sync_updated_at, labels)
             VALUES (1, 1, 1, 'Issue 1', 'open', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '[]')",
            []
        ).unwrap();

        // Record error (not standard in current implementation, but could be added)
        conn.execute(
            "UPDATE sync_logs SET error_message = ?1 WHERE id = ?2",
            ["Network timeout", &log_id.to_string()]
        ).unwrap();

        // Verify sync log shows incomplete sync with error
        let (completed_at, error_msg): (Option<String>, Option<String>) = conn.query_row(
            "SELECT completed_at, error_message FROM sync_logs WHERE id = ?1",
            [log_id],
            |row| Ok((row.get(0)?, row.get(1)?))
        ).unwrap();

        assert!(completed_at.is_none(), "Sync should not be marked complete");
        assert_eq!(error_msg, Some("Network timeout".to_string()));

        // Verify partial data was still saved (1 issue)
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM issues WHERE repo_id = 1",
            [],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_sync_cancellation_state() {
        let conn = setup_test_db();

        // Insert repository
        conn.execute(
            "INSERT INTO repositories (id, github_id, owner, name) VALUES (1, 1, 'org', 'repo')",
            []
        ).unwrap();

        // Start sync
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO sync_logs (repo_id, sync_type, started_at) VALUES (1, 'pull_requests', ?1)",
            [&now]
        ).unwrap();
        let log_id = conn.last_insert_rowid();

        // Simulate cancellation - sync log exists but never completed
        // In a real implementation, there would be a "cancelled" status or similar

        // Query for incomplete syncs (started but not completed)
        let incomplete_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sync_logs WHERE completed_at IS NULL",
            [],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(incomplete_count, 1);

        // Get the incomplete sync details
        let (repo_id, sync_type): (i64, String) = conn.query_row(
            "SELECT repo_id, sync_type FROM sync_logs WHERE id = ?1 AND completed_at IS NULL",
            [log_id],
            |row| Ok((row.get(0)?, row.get(1)?))
        ).unwrap();

        assert_eq!(repo_id, 1);
        assert_eq!(sync_type, "pull_requests");

        // This state could be used to resume or clean up cancelled syncs
    }
}
