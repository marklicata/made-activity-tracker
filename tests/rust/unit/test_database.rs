//! Unit tests for database operations
//!
//! Tests for:
//! - CRUD operations (Create, Read, Update, Delete)
//! - Data integrity and constraints
//! - Query operations and filtering

use made_activity_tracker::db::queries;
use rusqlite::{Connection, Result};
use chrono::Utc;

// Helper to set up test database with schema
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

        CREATE TABLE issues (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
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
            sync_updated_at TEXT,
            closed_at TEXT,
            FOREIGN KEY (repo_id) REFERENCES repositories(id),
            FOREIGN KEY (author_id) REFERENCES users(id)
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
            FOREIGN KEY (repo_id) REFERENCES repositories(id),
            FOREIGN KEY (author_id) REFERENCES users(id)
        );

        CREATE TABLE pr_reviews (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            github_id INTEGER UNIQUE NOT NULL,
            pr_id INTEGER NOT NULL,
            reviewer_id INTEGER,
            state TEXT NOT NULL,
            submitted_at TEXT NOT NULL,
            sync_updated_at TEXT,
            FOREIGN KEY (pr_id) REFERENCES pull_requests(id) ON DELETE CASCADE,
            FOREIGN KEY (reviewer_id) REFERENCES users(id)
        );

        CREATE TABLE squads (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            color TEXT
        );

        CREATE TABLE squad_members (
            squad_id TEXT NOT NULL,
            user_id INTEGER NOT NULL,
            role TEXT,
            joined_at TEXT NOT NULL,
            PRIMARY KEY (squad_id, user_id),
            FOREIGN KEY (squad_id) REFERENCES squads(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        CREATE INDEX idx_users_github_id ON users(github_id);
        CREATE INDEX idx_repositories_owner_name ON repositories(owner, name);
        CREATE INDEX idx_issues_repo_id ON issues(repo_id);
        CREATE INDEX idx_pull_requests_repo_id ON pull_requests(repo_id);"
    )?;

    Ok(conn)
}

#[cfg(test)]
mod repository_crud_tests {
    use super::*;

    #[test]
    fn test_create_repository() {
        let conn = setup_test_db().unwrap();

        let repo_id = queries::upsert_repository(&conn, "owner", "repo", Some(12345), true).unwrap();

        assert!(repo_id > 0);

        // Verify it was created
        let repo = queries::get_repository_by_id(&conn, repo_id).unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.name, "repo");
        assert_eq!(repo.github_id, Some(12345));
        assert!(repo.enabled);
    }

    #[test]
    fn test_read_repository_by_id() {
        let conn = setup_test_db().unwrap();
        let repo_id = queries::upsert_repository(&conn, "owner", "repo", Some(12345), true).unwrap();

        let repo = queries::get_repository_by_id(&conn, repo_id).unwrap();

        assert_eq!(repo.id, repo_id);
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.name, "repo");
    }

    #[test]
    fn test_read_repository_by_name() {
        let conn = setup_test_db().unwrap();
        queries::upsert_repository(&conn, "owner", "repo", Some(12345), true).unwrap();

        let repo = queries::get_repository_by_name(&conn, "owner", "repo").unwrap();

        assert!(repo.is_some());
        assert_eq!(repo.unwrap().name, "repo");
    }

    #[test]
    fn test_update_repository() {
        let conn = setup_test_db().unwrap();

        // Create initial repo
        let repo_id1 = queries::upsert_repository(&conn, "owner", "repo", Some(12345), true).unwrap();

        // Upsert with new data
        let repo_id2 = queries::upsert_repository(&conn, "owner", "repo", Some(67890), false).unwrap();

        // Should be the same ID
        assert_eq!(repo_id1, repo_id2);

        let repo = queries::get_repository_by_id(&conn, repo_id1).unwrap();
        assert_eq!(repo.github_id, Some(67890));
        assert!(!repo.enabled);
    }

    #[test]
    fn test_unique_owner_name_constraint() {
        let conn = setup_test_db().unwrap();

        queries::upsert_repository(&conn, "owner", "repo", Some(12345), true).unwrap();

        // Attempting to insert same owner/name should upsert, not fail
        let result = queries::upsert_repository(&conn, "owner", "repo", Some(67890), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_repo_synced_at() {
        let conn = setup_test_db().unwrap();
        let repo_id = queries::upsert_repository(&conn, "owner", "repo", Some(12345), true).unwrap();

        queries::update_repo_synced_at(&conn, repo_id).unwrap();

        let repo = queries::get_repository_by_id(&conn, repo_id).unwrap();
        assert!(repo.last_synced_at.is_some());
    }

    #[test]
    fn test_get_enabled_repositories() {
        let conn = setup_test_db().unwrap();

        queries::upsert_repository(&conn, "owner1", "repo1", Some(1), true).unwrap();
        queries::upsert_repository(&conn, "owner2", "repo2", Some(2), false).unwrap();
        queries::upsert_repository(&conn, "owner3", "repo3", Some(3), true).unwrap();

        let repos = queries::get_enabled_repositories(&conn).unwrap();

        assert_eq!(repos.len(), 2);
        assert!(repos.iter().any(|r| r.name == "repo1"));
        assert!(repos.iter().any(|r| r.name == "repo3"));
        assert!(!repos.iter().any(|r| r.name == "repo2"));
    }
}

#[cfg(test)]
mod user_crud_tests {
    use super::*;

    #[test]
    fn test_create_user() {
        let conn = setup_test_db().unwrap();

        let user_id = queries::get_or_create_user(
            &conn,
            12345,
            "testuser",
            Some("Test User"),
            Some("https://avatar.url"),
            Some(false),
            None,
            None,
            None,
        )
        .unwrap();

        assert!(user_id > 0);

        // Verify user was created
        let user: String = conn
            .query_row("SELECT login FROM users WHERE id = ?1", [user_id], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(user, "testuser");
    }

    #[test]
    fn test_get_existing_user() {
        let conn = setup_test_db().unwrap();

        // Create user
        let user_id1 = queries::get_or_create_user(
            &conn,
            12345,
            "testuser",
            Some("Test User"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        // Get same user
        let user_id2 = queries::get_or_create_user(
            &conn,
            12345,
            "testuser",
            Some("Updated Name"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(user_id1, user_id2);
    }

    #[test]
    fn test_user_github_id_unique_constraint() {
        let conn = setup_test_db().unwrap();

        queries::get_or_create_user(&conn, 12345, "user1", None, None, None, None, None, None).unwrap();

        // Same github_id should return same user (upsert behavior)
        let result = queries::get_or_create_user(&conn, 12345, "user2", None, None, None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_github_id() {
        let conn = setup_test_db().unwrap();

        // Negative github_id should fail
        let result = queries::get_or_create_user(&conn, -1, "user", None, None, None, None, None, None);
        assert!(result.is_err());

        // Zero github_id should fail
        let result = queries::get_or_create_user(&conn, 0, "user", None, None, None, None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_bot_user_flag() {
        let conn = setup_test_db().unwrap();

        let bot_id = queries::get_or_create_user(&conn, 12345, "bot", None, None, Some(true), None, None, None).unwrap();

        let is_bot: bool = conn
            .query_row("SELECT is_bot FROM users WHERE id = ?1", [bot_id], |row| {
                row.get(0)
            })
            .unwrap();

        assert!(is_bot);
    }

    #[test]
    fn test_user_tracking() {
        let conn = setup_test_db().unwrap();

        let user_id = queries::get_or_create_user(
            &conn,
            12345,
            "user",
            None,
            None,
            None,
            Some(true),
            Some("2024-01-01T00:00:00Z"),
            None,
        )
        .unwrap();

        let (tracked, tracked_at): (bool, Option<String>) = conn
            .query_row(
                "SELECT tracked, tracked_at FROM users WHERE id = ?1",
                [user_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert!(tracked);
        assert!(tracked_at.is_some());
    }
}

#[cfg(test)]
mod issue_crud_tests {
    use super::*;

    fn create_test_issue(conn: &Connection, repo_id: i64, github_id: i64, number: i32) -> Result<i64> {
        conn.execute(
            "INSERT INTO issues (github_id, repo_id, number, title, state, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            [
                &github_id.to_string(),
                &repo_id.to_string(),
                &number.to_string(),
                "Test Issue",
                "open",
                &Utc::now().to_rfc3339(),
                &Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    #[test]
    fn test_create_issue() {
        let conn = setup_test_db().unwrap();
        let repo_id = queries::upsert_repository(&conn, "owner", "repo", Some(1), true).unwrap();

        let issue_id = create_test_issue(&conn, repo_id, 12345, 1).unwrap();

        assert!(issue_id > 0);

        let title: String = conn
            .query_row("SELECT title FROM issues WHERE id = ?1", [issue_id], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(title, "Test Issue");
    }

    #[test]
    fn test_issue_github_id_unique() {
        let conn = setup_test_db().unwrap();
        let repo_id = queries::upsert_repository(&conn, "owner", "repo", Some(1), true).unwrap();

        create_test_issue(&conn, repo_id, 12345, 1).unwrap();

        // Duplicate github_id should fail
        let result = create_test_issue(&conn, repo_id, 12345, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_issue_foreign_key_repo() {
        let conn = setup_test_db().unwrap();

        // Creating issue with non-existent repo_id should fail
        let result = create_test_issue(&conn, 9999, 12345, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_issue_state() {
        let conn = setup_test_db().unwrap();
        let repo_id = queries::upsert_repository(&conn, "owner", "repo", Some(1), true).unwrap();
        let issue_id = create_test_issue(&conn, repo_id, 12345, 1).unwrap();

        conn.execute(
            "UPDATE issues SET state = 'closed', closed_at = ?1 WHERE id = ?2",
            [&Utc::now().to_rfc3339(), &issue_id.to_string()],
        )
        .unwrap();

        let (state, closed_at): (String, Option<String>) = conn
            .query_row(
                "SELECT state, closed_at FROM issues WHERE id = ?1",
                [issue_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(state, "closed");
        assert!(closed_at.is_some());
    }
}

#[cfg(test)]
mod pull_request_crud_tests {
    use super::*;

    fn create_test_pr(
        conn: &Connection,
        repo_id: i64,
        github_id: i64,
        number: i32,
        author_id: Option<i64>,
    ) -> Result<i64> {
        conn.execute(
            "INSERT INTO pull_requests (github_id, repo_id, number, title, state, created_at, updated_at, author_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            [
                &github_id.to_string(),
                &repo_id.to_string(),
                &number.to_string(),
                "Test PR",
                "open",
                &Utc::now().to_rfc3339(),
                &Utc::now().to_rfc3339(),
                &author_id.map(|id| id.to_string()).unwrap_or_default(),
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    #[test]
    fn test_create_pull_request() {
        let conn = setup_test_db().unwrap();
        let repo_id = queries::upsert_repository(&conn, "owner", "repo", Some(1), true).unwrap();

        let pr_id = create_test_pr(&conn, repo_id, 12345, 1, None).unwrap();

        assert!(pr_id > 0);
    }

    #[test]
    fn test_pr_with_author() {
        let conn = setup_test_db().unwrap();
        let repo_id = queries::upsert_repository(&conn, "owner", "repo", Some(1), true).unwrap();
        let author_id = queries::get_or_create_user(&conn, 11111, "author", None, None, None, None, None, None).unwrap();

        let pr_id = create_test_pr(&conn, repo_id, 12345, 1, Some(author_id)).unwrap();

        let retrieved_author_id: Option<i64> = conn
            .query_row(
                "SELECT author_id FROM pull_requests WHERE id = ?1",
                [pr_id],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(retrieved_author_id, Some(author_id));
    }

    #[test]
    fn test_pr_merge_data() {
        let conn = setup_test_db().unwrap();
        let repo_id = queries::upsert_repository(&conn, "owner", "repo", Some(1), true).unwrap();
        let pr_id = create_test_pr(&conn, repo_id, 12345, 1, None).unwrap();

        let merged_at = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE pull_requests SET state = 'closed', merged_at = ?1, closed_at = ?1 WHERE id = ?2",
            [&merged_at, &pr_id.to_string()],
        )
        .unwrap();

        let (state, merged): (String, Option<String>) = conn
            .query_row(
                "SELECT state, merged_at FROM pull_requests WHERE id = ?1",
                [pr_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(state, "closed");
        assert!(merged.is_some());
    }

    #[test]
    fn test_pr_code_metrics() {
        let conn = setup_test_db().unwrap();
        let repo_id = queries::upsert_repository(&conn, "owner", "repo", Some(1), true).unwrap();

        conn.execute(
            "INSERT INTO pull_requests (github_id, repo_id, number, title, state, created_at, updated_at, additions, deletions, changed_files)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            [
                "12345",
                &repo_id.to_string(),
                "1",
                "Test PR",
                "open",
                &Utc::now().to_rfc3339(),
                &Utc::now().to_rfc3339(),
                "100",
                "50",
                "5",
            ],
        )
        .unwrap();

        let pr_id = conn.last_insert_rowid();

        let (additions, deletions, changed_files): (i32, i32, i32) = conn
            .query_row(
                "SELECT additions, deletions, changed_files FROM pull_requests WHERE id = ?1",
                [pr_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        assert_eq!(additions, 100);
        assert_eq!(deletions, 50);
        assert_eq!(changed_files, 5);
    }
}

#[cfg(test)]
mod relationship_tests {
    use super::*;

    #[test]
    fn test_cascade_delete_pr_reviews() {
        let conn = setup_test_db().unwrap();
        let repo_id = queries::upsert_repository(&conn, "owner", "repo", Some(1), true).unwrap();

        conn.execute(
            "INSERT INTO pull_requests (github_id, repo_id, number, title, state, created_at, updated_at)
             VALUES (12345, ?1, 1, 'Test', 'open', ?2, ?2)",
            [&repo_id.to_string(), &Utc::now().to_rfc3339()],
        )
        .unwrap();

        let pr_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO pr_reviews (github_id, pr_id, state, submitted_at)
             VALUES (1, ?1, 'approved', ?2)",
            [&pr_id.to_string(), &Utc::now().to_rfc3339()],
        )
        .unwrap();

        // Delete PR
        conn.execute("DELETE FROM pull_requests WHERE id = ?1", [pr_id])
            .unwrap();

        // Reviews should be cascade deleted
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM pr_reviews WHERE pr_id = ?1", [pr_id], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn test_squad_member_relationship() {
        let conn = setup_test_db().unwrap();

        // Create squad
        conn.execute(
            "INSERT INTO squads (id, name) VALUES ('team-a', 'Team A')",
            [],
        )
        .unwrap();

        // Create user
        let user_id = queries::get_or_create_user(&conn, 12345, "user", None, None, None, None, None, None).unwrap();

        // Add user to squad
        conn.execute(
            "INSERT INTO squad_members (squad_id, user_id, joined_at) VALUES ('team-a', ?1, ?2)",
            [&user_id.to_string(), &Utc::now().to_rfc3339()],
        )
        .unwrap();

        // Verify relationship
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM squad_members WHERE squad_id = 'team-a' AND user_id = ?1",
                [user_id],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn test_cascade_delete_squad_members() {
        let conn = setup_test_db().unwrap();

        conn.execute("INSERT INTO squads (id, name) VALUES ('team-a', 'Team A')", []).unwrap();

        let user_id = queries::get_or_create_user(&conn, 12345, "user", None, None, None, None, None, None).unwrap();

        conn.execute(
            "INSERT INTO squad_members (squad_id, user_id, joined_at) VALUES ('team-a', ?1, ?2)",
            [&user_id.to_string(), &Utc::now().to_rfc3339()],
        )
        .unwrap();

        // Delete squad
        conn.execute("DELETE FROM squads WHERE id = 'team-a'", []).unwrap();

        // Members should be cascade deleted
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM squad_members WHERE squad_id = 'team-a'", [], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(count, 0);
    }
}

#[cfg(test)]
mod index_tests {
    use super::*;

    #[test]
    fn test_indexes_exist() {
        let conn = setup_test_db().unwrap();

        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'index'")
            .unwrap();

        let indexes: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        // Check that key indexes exist
        assert!(indexes.iter().any(|name| name.contains("idx_users_github_id")));
        assert!(indexes.iter().any(|name| name.contains("idx_repositories_owner_name")));
        assert!(indexes.iter().any(|name| name.contains("idx_issues_repo_id")));
        assert!(indexes.iter().any(|name| name.contains("idx_pull_requests_repo_id")));
    }
}
