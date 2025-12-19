use anyhow::Result;
use rusqlite::Connection;

/// Run all database migrations
pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA)?;

    // Run migrations for existing databases
    migrate_add_embedding_columns(conn)?;
    migrate_add_tracked_users_table(conn)?; // deprecated but retained
    migrate_add_sync_updated_at_columns(conn)?;
    migrate_add_user_tracked_columns(conn)?;
    migrate_add_milestone_repo_github_index(conn)?;
    migrate_backfill_tracked_users(conn)?;

    tracing::info!("Database migrations completed");
    Ok(())
}

/// Add embedding columns to existing databases (Phase 2A migration)
fn migrate_add_embedding_columns(conn: &Connection) -> Result<()> {
    // Check if issues table has embedding column
    let has_issue_embedding: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('issues') WHERE name='embedding'",
            [],
            |row| row.get(0),
        )
        .map(|count: i32| count > 0)
        .unwrap_or(false);

    if !has_issue_embedding {
        tracing::info!("Adding embedding column to issues table...");
        conn.execute("ALTER TABLE issues ADD COLUMN embedding BLOB", [])?;
    }

    // Check if pull_requests table has embedding column
    let has_pr_embedding: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('pull_requests') WHERE name='embedding'",
            [],
            |row| row.get(0),
        )
        .map(|count: i32| count > 0)
        .unwrap_or(false);

    if !has_pr_embedding {
        tracing::info!("Adding embedding column to pull_requests table...");
        conn.execute("ALTER TABLE pull_requests ADD COLUMN embedding BLOB", [])?;
    }

    Ok(())
}

/// Add tracked_users table for user-centric view
fn migrate_add_tracked_users_table(conn: &Connection) -> Result<()> {
    // Check if tracked_users table exists (deprecated, kept for backwards compatibility)
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tracked_users'",
            [],
            |row| row.get(0),
        )
        .map(|count: i32| count > 0)
        .unwrap_or(false);

    if !table_exists {
        tracing::info!("Creating tracked_users table (deprecated)...");
        conn.execute(
            "CREATE TABLE tracked_users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL REFERENCES users(id),
                added_at TEXT NOT NULL,
                UNIQUE(user_id)
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_tracked_users_added ON tracked_users(added_at)",
            [],
        )?;
    }

    Ok(())
}

/// Add sync_updated_at columns to issues, pull_requests, pr_reviews
fn migrate_add_sync_updated_at_columns(conn: &Connection) -> Result<()> {
    // issues
    let has_issue_sync: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('issues') WHERE name='sync_updated_at'",
            [],
            |row| row.get(0),
        )
        .map(|count: i32| count > 0)
        .unwrap_or(false);
    if !has_issue_sync {
        tracing::info!("Adding sync_updated_at to issues...");
        conn.execute("ALTER TABLE issues ADD COLUMN sync_updated_at TEXT", [])?;
        conn.execute("UPDATE issues SET sync_updated_at = updated_at WHERE sync_updated_at IS NULL", [])?;
    }

    // pull_requests
    let has_pr_sync: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('pull_requests') WHERE name='sync_updated_at'",
            [],
            |row| row.get(0),
        )
        .map(|count: i32| count > 0)
        .unwrap_or(false);
    if !has_pr_sync {
        tracing::info!("Adding sync_updated_at to pull_requests...");
        conn.execute("ALTER TABLE pull_requests ADD COLUMN sync_updated_at TEXT", [])?;
        conn.execute("UPDATE pull_requests SET sync_updated_at = updated_at WHERE sync_updated_at IS NULL", [])?;
    }

    // pr_reviews
    let has_review_sync: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('pr_reviews') WHERE name='sync_updated_at'",
            [],
            |row| row.get(0),
        )
        .map(|count: i32| count > 0)
        .unwrap_or(false);
    if !has_review_sync {
        tracing::info!("Adding sync_updated_at to pr_reviews...");
        conn.execute("ALTER TABLE pr_reviews ADD COLUMN sync_updated_at TEXT", [])?;
        conn.execute(
            "UPDATE pr_reviews SET sync_updated_at = submitted_at WHERE sync_updated_at IS NULL",
            [],
        )?;
    }

    Ok(())
}

/// Add tracked and tracked_at columns to users
fn migrate_add_user_tracked_columns(conn: &Connection) -> Result<()> {
    let has_tracked: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('users') WHERE name='tracked'",
            [],
            |row| row.get(0),
        )
        .map(|count: i32| count > 0)
        .unwrap_or(false);

    if !has_tracked {
        tracing::info!("Adding tracked and tracked_at to users...");
        conn.execute("ALTER TABLE users ADD COLUMN tracked BOOLEAN NOT NULL DEFAULT FALSE", [])?;
        conn.execute("ALTER TABLE users ADD COLUMN tracked_at TEXT", [])?;
    }

    Ok(())
}

/// Add UNIQUE(repo_id, github_id) index to milestones (databaseId alignment)
fn migrate_add_milestone_repo_github_index(conn: &Connection) -> Result<()> {
    // Ensure github_id column exists first (it does in base schema)
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_milestones_repo_github ON milestones(repo_id, github_id)",
        [],
    )?;
    Ok(())
}

/// Backfill users.tracked/tracked_at from tracked_users table (if present)
fn migrate_backfill_tracked_users(conn: &Connection) -> Result<()> {
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tracked_users'",
            [],
            |row| row.get(0),
        )
        .map(|count: i32| count > 0)
        .unwrap_or(false);

    if !table_exists {
        return Ok(());
    }

    tracing::info!("Backfilling users.tracked from tracked_users table...");
    conn.execute(
        "UPDATE users SET tracked = 1, tracked_at = (
            SELECT added_at FROM tracked_users tu WHERE tu.user_id = users.id
        ) WHERE EXISTS (
            SELECT 1 FROM tracked_users tu WHERE tu.user_id = users.id
        )",
        [],
    )?;

    Ok(())
}

const SCHEMA: &str = r#"
-- Repositories being tracked
CREATE TABLE IF NOT EXISTS repositories (
    id INTEGER PRIMARY KEY,
    owner TEXT NOT NULL,
    name TEXT NOT NULL,
    github_id INTEGER UNIQUE,
    enabled BOOLEAN DEFAULT TRUE,
    last_synced_at TEXT,
    UNIQUE(owner, name)
);

-- GitHub users
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    github_id INTEGER UNIQUE NOT NULL,
    login TEXT NOT NULL,
    name TEXT,
    avatar_url TEXT,
    is_bot BOOLEAN DEFAULT FALSE,
    tracked BOOLEAN NOT NULL DEFAULT FALSE,
    tracked_at TEXT
);

-- Issues
CREATE TABLE IF NOT EXISTS issues (
    id INTEGER PRIMARY KEY,
    github_id INTEGER UNIQUE NOT NULL,
    repo_id INTEGER NOT NULL REFERENCES repositories(id),
    number INTEGER NOT NULL,
    title TEXT NOT NULL,
    body TEXT,
    state TEXT NOT NULL,
    author_id INTEGER REFERENCES users(id),
    assignee_id INTEGER REFERENCES users(id),
    milestone_id INTEGER REFERENCES milestones(id),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    sync_updated_at TEXT,
    closed_at TEXT,
    labels TEXT, -- JSON array of label names
    embedding BLOB, -- 384-dimensional float32 vector (1536 bytes)
    UNIQUE(repo_id, number)
);

-- Pull Requests
CREATE TABLE IF NOT EXISTS pull_requests (
    id INTEGER PRIMARY KEY,
    github_id INTEGER UNIQUE NOT NULL,
    repo_id INTEGER NOT NULL REFERENCES repositories(id),
    number INTEGER NOT NULL,
    title TEXT NOT NULL,
    body TEXT,
    state TEXT NOT NULL,
    author_id INTEGER REFERENCES users(id),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    sync_updated_at TEXT,
    merged_at TEXT,
    closed_at TEXT,
    additions INTEGER DEFAULT 0,
    deletions INTEGER DEFAULT 0,
    changed_files INTEGER DEFAULT 0,
    review_comments INTEGER DEFAULT 0,
    labels TEXT, -- JSON array of label names
    embedding BLOB, -- 384-dimensional float32 vector (1536 bytes)
    UNIQUE(repo_id, number)
);

-- PR Reviews (for calculating review rounds)
CREATE TABLE IF NOT EXISTS pr_reviews (
    id INTEGER PRIMARY KEY,
    github_id INTEGER UNIQUE NOT NULL,
    pr_id INTEGER NOT NULL REFERENCES pull_requests(id),
    reviewer_id INTEGER REFERENCES users(id),
    state TEXT NOT NULL, -- APPROVED, CHANGES_REQUESTED, COMMENTED
    submitted_at TEXT NOT NULL,
    sync_updated_at TEXT
);

-- Milestones (Cycles)
CREATE TABLE IF NOT EXISTS milestones (
    id INTEGER PRIMARY KEY,
    github_id INTEGER UNIQUE NOT NULL,
    repo_id INTEGER NOT NULL REFERENCES repositories(id),
    title TEXT NOT NULL,
    description TEXT,
    state TEXT NOT NULL,
    due_on TEXT,
    open_issues INTEGER DEFAULT 0,
    closed_issues INTEGER DEFAULT 0,
    UNIQUE(repo_id, title),
    UNIQUE(repo_id, github_id)
);

-- Squads (Team groupings)
CREATE TABLE IF NOT EXISTS squads (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT
);

-- Squad members
CREATE TABLE IF NOT EXISTS squad_members (
    squad_id TEXT NOT NULL REFERENCES squads(id),
    user_id INTEGER NOT NULL REFERENCES users(id),
    PRIMARY KEY(squad_id, user_id)
);

-- Tracked users for user-centric view
CREATE TABLE IF NOT EXISTS tracked_users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id),
    added_at TEXT NOT NULL,
    UNIQUE(user_id)
);

-- Sync log for tracking incremental updates
CREATE TABLE IF NOT EXISTS sync_log (
    id INTEGER PRIMARY KEY,
    repo_id INTEGER NOT NULL REFERENCES repositories(id),
    sync_type TEXT NOT NULL, -- issues, pull_requests, milestones
    started_at TEXT NOT NULL,
    completed_at TEXT,
    items_synced INTEGER DEFAULT 0,
    error TEXT
);

-- Metrics snapshots for historical tracking
CREATE TABLE IF NOT EXISTS metrics_snapshots (
    id INTEGER PRIMARY KEY,
    snapshot_date TEXT NOT NULL,
    scope_type TEXT NOT NULL, -- 'team', 'user', 'squad', 'repo'
    scope_id TEXT NOT NULL,
    metrics_json TEXT NOT NULL, -- Full metrics blob
    UNIQUE(snapshot_date, scope_type, scope_id)
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_issues_repo ON issues(repo_id);
CREATE INDEX IF NOT EXISTS idx_issues_author ON issues(author_id);
CREATE INDEX IF NOT EXISTS idx_issues_state ON issues(state);
CREATE INDEX IF NOT EXISTS idx_issues_created ON issues(created_at);

CREATE INDEX IF NOT EXISTS idx_prs_repo ON pull_requests(repo_id);
CREATE INDEX IF NOT EXISTS idx_prs_author ON pull_requests(author_id);
CREATE INDEX IF NOT EXISTS idx_prs_state ON pull_requests(state);
CREATE INDEX IF NOT EXISTS idx_prs_created ON pull_requests(created_at);
CREATE INDEX IF NOT EXISTS idx_prs_merged ON pull_requests(merged_at);

CREATE INDEX IF NOT EXISTS idx_milestones_repo ON milestones(repo_id);
CREATE INDEX IF NOT EXISTS idx_milestones_due ON milestones(due_on);

CREATE INDEX IF NOT EXISTS idx_tracked_users_added ON tracked_users(added_at);
"#;
