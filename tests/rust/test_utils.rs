//! Common test utilities and helpers for Rust tests
//!
//! This module provides shared functionality for unit and integration tests.

use rusqlite::{Connection, Result};
use chrono::{DateTime, NaiveDate, Utc};

/// Set up an in-memory test database with the full schema
pub fn setup_test_db() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;

    // Create all tables needed for testing
    conn.execute_batch(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            github_id INTEGER UNIQUE NOT NULL,
            login TEXT NOT NULL,
            name TEXT,
            email TEXT,
            avatar_url TEXT,
            is_bot INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE projects (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            owner TEXT NOT NULL,
            repo TEXT NOT NULL,
            description TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(owner, repo)
        );

        CREATE TABLE repositories (
            id INTEGER PRIMARY KEY,
            project_id INTEGER NOT NULL,
            github_id INTEGER UNIQUE NOT NULL,
            name TEXT NOT NULL,
            full_name TEXT NOT NULL,
            owner TEXT NOT NULL,
            default_branch TEXT DEFAULT 'main',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE squads (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
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

        CREATE TABLE pull_requests (
            id INTEGER PRIMARY KEY,
            github_id INTEGER UNIQUE NOT NULL,
            repo_id INTEGER NOT NULL,
            number INTEGER NOT NULL,
            title TEXT NOT NULL,
            state TEXT NOT NULL,
            author_id INTEGER,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            merged_at TEXT,
            closed_at TEXT,
            draft INTEGER DEFAULT 0,
            additions INTEGER DEFAULT 0,
            deletions INTEGER DEFAULT 0,
            changed_files INTEGER DEFAULT 0,
            FOREIGN KEY (repo_id) REFERENCES repositories(id),
            FOREIGN KEY (author_id) REFERENCES users(id)
        );

        CREATE TABLE issues (
            id INTEGER PRIMARY KEY,
            github_id INTEGER UNIQUE NOT NULL,
            repo_id INTEGER NOT NULL,
            number INTEGER NOT NULL,
            title TEXT NOT NULL,
            state TEXT NOT NULL,
            author_id INTEGER,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            closed_at TEXT,
            FOREIGN KEY (repo_id) REFERENCES repositories(id),
            FOREIGN KEY (author_id) REFERENCES users(id)
        );

        CREATE TABLE commits (
            id INTEGER PRIMARY KEY,
            sha TEXT UNIQUE NOT NULL,
            repo_id INTEGER NOT NULL,
            author_id INTEGER,
            message TEXT NOT NULL,
            committed_at TEXT NOT NULL,
            additions INTEGER DEFAULT 0,
            deletions INTEGER DEFAULT 0,
            FOREIGN KEY (repo_id) REFERENCES repositories(id),
            FOREIGN KEY (author_id) REFERENCES users(id)
        );

        CREATE TABLE pr_commits (
            pr_id INTEGER NOT NULL,
            commit_id INTEGER NOT NULL,
            PRIMARY KEY (pr_id, commit_id),
            FOREIGN KEY (pr_id) REFERENCES pull_requests(id) ON DELETE CASCADE,
            FOREIGN KEY (commit_id) REFERENCES commits(id) ON DELETE CASCADE
        );

        CREATE TABLE reviews (
            id INTEGER PRIMARY KEY,
            github_id INTEGER UNIQUE NOT NULL,
            pr_id INTEGER NOT NULL,
            reviewer_id INTEGER,
            state TEXT NOT NULL,
            submitted_at TEXT NOT NULL,
            FOREIGN KEY (pr_id) REFERENCES pull_requests(id) ON DELETE CASCADE,
            FOREIGN KEY (reviewer_id) REFERENCES users(id)
        );

        CREATE TABLE activities (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            activity_type TEXT NOT NULL,
            repo_id INTEGER,
            pr_id INTEGER,
            issue_id INTEGER,
            commit_id INTEGER,
            occurred_at TEXT NOT NULL,
            metadata TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (repo_id) REFERENCES repositories(id),
            FOREIGN KEY (pr_id) REFERENCES pull_requests(id),
            FOREIGN KEY (issue_id) REFERENCES issues(id),
            FOREIGN KEY (commit_id) REFERENCES commits(id)
        );

        CREATE TABLE metrics_cache (
            id INTEGER PRIMARY KEY,
            metric_type TEXT NOT NULL,
            user_id INTEGER,
            squad_id TEXT,
            project_id INTEGER,
            date TEXT NOT NULL,
            value REAL NOT NULL,
            metadata TEXT,
            calculated_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (squad_id) REFERENCES squads(id) ON DELETE CASCADE,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE INDEX idx_users_github_id ON users(github_id);
        CREATE INDEX idx_users_login ON users(login);
        CREATE INDEX idx_pull_requests_repo_id ON pull_requests(repo_id);
        CREATE INDEX idx_pull_requests_author_id ON pull_requests(author_id);
        CREATE INDEX idx_pull_requests_created_at ON pull_requests(created_at);
        CREATE INDEX idx_activities_user_id ON activities(user_id);
        CREATE INDEX idx_activities_occurred_at ON activities(occurred_at);
        CREATE INDEX idx_metrics_cache_user_date ON metrics_cache(user_id, date);
        CREATE INDEX idx_metrics_cache_squad_date ON metrics_cache(squad_id, date);
        "
    )?;

    Ok(conn)
}

/// Create a test user
pub fn create_test_user(conn: &Connection, github_id: i64, login: &str) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO users (github_id, login, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        [&github_id.to_string(), login, &now, &now],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Create a test project
pub fn create_test_project(conn: &Connection, name: &str, owner: &str, repo: &str) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO projects (name, owner, repo, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        [name, owner, repo, &now, &now],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Create a test repository
pub fn create_test_repository(
    conn: &Connection,
    project_id: i64,
    github_id: i64,
    owner: &str,
    name: &str,
) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    let full_name = format!("{}/{}", owner, name);
    conn.execute(
        "INSERT INTO repositories (project_id, github_id, name, full_name, owner, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        [
            &project_id.to_string(),
            &github_id.to_string(),
            name,
            &full_name,
            owner,
            &now,
            &now,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Create a test pull request
pub fn create_test_pr(
    conn: &Connection,
    github_id: i64,
    repo_id: i64,
    number: i32,
    author_id: i64,
    state: &str,
    created_at: &str,
) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO pull_requests (github_id, repo_id, number, title, state, author_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        [
            &github_id.to_string(),
            &repo_id.to_string(),
            &number.to_string(),
            &format!("Test PR #{}", number),
            state,
            &author_id.to_string(),
            created_at,
            &now,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Create a test squad
pub fn create_test_squad(conn: &Connection, id: &str, name: &str) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO squads (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        [id, name, &now, &now],
    )?;
    Ok(())
}

/// Add a user to a squad
pub fn add_squad_member(conn: &Connection, squad_id: &str, user_id: i64) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO squad_members (squad_id, user_id, joined_at) VALUES (?1, ?2, ?3)",
        [squad_id, &user_id.to_string(), &now],
    )?;
    Ok(())
}

/// Format a date for testing (YYYY-MM-DD format)
pub fn test_date(year: i32, month: u32, day: u32) -> String {
    NaiveDate::from_ymd_opt(year, month, day)
        .unwrap()
        .format("%Y-%m-%d")
        .to_string()
}

/// Format a datetime for testing (ISO 8601 format)
pub fn test_datetime(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> String {
    NaiveDate::from_ymd_opt(year, month, day)
        .unwrap()
        .and_hms_opt(hour, min, sec)
        .unwrap()
        .and_utc()
        .to_rfc3339()
}

/// Clean up test database
pub fn teardown_test_db(conn: Connection) {
    drop(conn);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_creates_all_tables() {
        let conn = setup_test_db().unwrap();

        // Verify tables exist by querying sqlite_master
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap();

        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(tables.contains(&"users".to_string()));
        assert!(tables.contains(&"projects".to_string()));
        assert!(tables.contains(&"pull_requests".to_string()));
        assert!(tables.contains(&"squads".to_string()));
    }

    #[test]
    fn test_create_test_user() {
        let conn = setup_test_db().unwrap();
        let user_id = create_test_user(&conn, 12345, "testuser").unwrap();

        assert!(user_id > 0);

        let login: String = conn
            .query_row("SELECT login FROM users WHERE id = ?1", [user_id], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(login, "testuser");
    }

    #[test]
    fn test_date_helpers() {
        let date = test_date(2024, 12, 26);
        assert_eq!(date, "2024-12-26");

        let datetime = test_datetime(2024, 12, 26, 10, 30, 0);
        assert!(datetime.starts_with("2024-12-26T10:30:00"));
    }
}
