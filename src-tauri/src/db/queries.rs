use super::models::*;
use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};

// ============================================================================
// REPOSITORY QUERIES
// ============================================================================

/// Get all enabled repositories
pub fn get_enabled_repositories(conn: &Connection) -> Result<Vec<Repository>> {
    let mut stmt = conn.prepare(
        "SELECT id, owner, name, github_id, enabled, last_synced_at 
         FROM repositories WHERE enabled = TRUE"
    )?;
    
    let repos = stmt.query_map([], |row| {
        Ok(Repository {
            id: row.get(0)?,
            owner: row.get(1)?,
            name: row.get(2)?,
            github_id: row.get(3)?,
            enabled: row.get(4)?,
            last_synced_at: row.get(5)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    
    Ok(repos)
}

/// Get a repository by ID
pub fn get_repository_by_id(conn: &Connection, id: i64) -> Result<Repository> {
    let repo = conn.query_row(
        "SELECT id, owner, name, github_id, enabled, last_synced_at 
         FROM repositories WHERE id = ?1",
        params![id],
        |row| {
            Ok(Repository {
                id: row.get(0)?,
                owner: row.get(1)?,
                name: row.get(2)?,
                github_id: row.get(3)?,
                enabled: row.get(4)?,
                last_synced_at: row.get(5)?,
            })
        },
    )?;
    
    Ok(repo)
}

/// Insert or update a repository
pub fn upsert_repository(
    conn: &Connection,
    owner: &str,
    name: &str,
    github_id: Option<i64>,
    enabled: bool,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO repositories (owner, name, github_id, enabled)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(owner, name) DO UPDATE SET
            github_id = COALESCE(excluded.github_id, github_id),
            enabled = excluded.enabled",
        params![owner, name, github_id, enabled],
    )?;
    
    let id: i64 = conn.query_row(
        "SELECT id FROM repositories WHERE owner = ?1 AND name = ?2",
        params![owner, name],
        |row| row.get(0),
    )?;
    
    Ok(id)
}

/// Update repository's last synced timestamp
pub fn update_repo_synced_at(conn: &Connection, repo_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE repositories SET last_synced_at = datetime('now') WHERE id = ?1",
        params![repo_id],
    )?;
    Ok(())
}

/// Get repository by owner and name
pub fn get_repository_by_name(conn: &Connection, owner: &str, name: &str) -> Result<Option<Repository>> {
    let result = conn.query_row(
        "SELECT id, owner, name, github_id, enabled, last_synced_at 
         FROM repositories WHERE owner = ?1 AND name = ?2",
        params![owner, name],
        |row| Ok(Repository {
            id: row.get(0)?,
            owner: row.get(1)?,
            name: row.get(2)?,
            github_id: row.get(3)?,
            enabled: row.get(4)?,
            last_synced_at: row.get(5)?,
        }),
    ).optional()?;
    
    Ok(result)
}

// ============================================================================
// USER QUERIES
// ============================================================================

/// Get or create a user by GitHub login
pub fn get_or_create_user(
    conn: &Connection,
    github_id: i64,
    login: &str,
    name: Option<&str>,
    avatar_url: Option<&str>,
    is_bot: Option<bool>,
    tracked: Option<bool>,
    tracked_at: Option<&str>,
    track_if_new: Option<bool>,
) -> Result<i64> {
    // Validate github_id
    if github_id <= 0 {
        return Err(anyhow::anyhow!(
            "Invalid github_id ({}) for user '{}'. GitHub IDs must be positive integers.",
            github_id,
            login
        ));
    }

    // Determine tracked value for INSERT: prefer track_if_new, then tracked, then default to false
    let insert_tracked = track_if_new.or(tracked).unwrap_or(false);

    // Generate tracked_at timestamp if tracked is true and no timestamp provided
    let insert_tracked_at = if insert_tracked && tracked_at.is_none() {
        Some(chrono::Utc::now().to_rfc3339())
    } else {
        tracked_at.map(|s| s.to_string())
    };

    conn.execute(
        "INSERT INTO users (github_id, login, name, avatar_url, is_bot, tracked, tracked_at)
         VALUES (?1, ?2, ?3, ?4, COALESCE(?5, FALSE), ?6, ?7)
         ON CONFLICT(github_id) DO UPDATE SET
            login = excluded.login,
            name = COALESCE(excluded.name, name),
            avatar_url = COALESCE(excluded.avatar_url, avatar_url),
            is_bot = COALESCE(excluded.is_bot, is_bot)
            -- Don't update tracked/tracked_at on conflict to preserve explicit tracking status",
        params![
            github_id,
            login,
            name,
            avatar_url,
            is_bot,
            insert_tracked,
            insert_tracked_at,
        ],
    )?;

    let id: i64 = conn.query_row(
        "SELECT id FROM users WHERE github_id = ?1",
        params![github_id],
        |row| row.get(0),
    )?;

    Ok(id)
}

/// Get user by login (for looking up by username)
pub fn get_user_by_login(conn: &Connection, login: &str) -> Result<Option<User>> {
    let result = conn
        .query_row(
            "SELECT id, github_id, login, name, avatar_url, is_bot, tracked, tracked_at FROM users WHERE login = ?1",
            params![login],
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
        )
        .optional()?;

    Ok(result)
}

/// Check if a login is a known bot
pub fn is_bot_user(login: &str, excluded_bots: &[String]) -> bool {
    excluded_bots.iter().any(|bot| {
        login.eq_ignore_ascii_case(bot) || 
        login.ends_with("[bot]") ||
        login.contains("bot")
    })
}

// ============================================================================
// ISSUE QUERIES
// ============================================================================

/// Upsert an issue
pub fn upsert_issue(
    conn: &Connection,
    github_id: i64,
    repo_id: i64,
    number: i32,
    title: &str,
    body: Option<&str>,
    state: &str,
    author_id: Option<i64>,
    assignee_id: Option<i64>,
    milestone_id: Option<i64>,
    created_at: &str,
    updated_at: &str,
    closed_at: Option<&str>,
    labels: &[String],
    sync_updated_at: &str,
) -> Result<i64> {
    let labels_json = serde_json::to_string(labels)?;

    conn.execute(
        "INSERT INTO issues (github_id, repo_id, number, title, body, state, author_id,
                            assignee_id, milestone_id, created_at, updated_at, closed_at, labels, sync_updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
         ON CONFLICT(github_id) DO UPDATE SET
            title = excluded.title,
            body = excluded.body,
            state = excluded.state,
            author_id = COALESCE(excluded.author_id, author_id),
            assignee_id = COALESCE(excluded.assignee_id, assignee_id),
            milestone_id = COALESCE(excluded.milestone_id, milestone_id),
            updated_at = excluded.updated_at,
            closed_at = excluded.closed_at,
            labels = excluded.labels,
            sync_updated_at = excluded.sync_updated_at
         WHERE sync_updated_at IS NULL OR excluded.sync_updated_at >= sync_updated_at",
        params![
            github_id,
            repo_id,
            number,
            title,
            body,
            state,
            author_id,
            assignee_id,
            milestone_id,
            created_at,
            updated_at,
            closed_at,
            labels_json,
            sync_updated_at,
        ],
    )?;

    let id: i64 = conn.query_row(
        "SELECT id FROM issues WHERE github_id = ?1",
        params![github_id],
        |row| row.get(0),
    )?;

    Ok(id)
}

/// Get issues within a date range, excluding bots
pub fn get_issues_for_metrics(
    conn: &Connection,
    since: &str,
    excluded_bots: &[String],
) -> Result<Vec<Issue>> {
    // Build query with bot exclusion
    let query = "
        SELECT i.id, i.github_id, i.repo_id, i.number, i.title, i.body, i.state,
               i.author_id, i.assignee_id, i.milestone_id, i.created_at, i.updated_at,
               i.closed_at, i.labels, u.login
        FROM issues i
        LEFT JOIN users u ON i.author_id = u.id
        WHERE i.created_at >= ?1
    ";

    let mut stmt = conn.prepare(query)?;

    let issues = stmt.query_map(params![since], |row| {
        let labels_json: String = row.get(13)?;
        let labels: Vec<String> = serde_json::from_str(&labels_json).unwrap_or_default();
        let author_login: Option<String> = row.get(14)?;

        Ok((Issue {
            id: row.get(0)?,
            github_id: row.get(1)?,
            repo_id: row.get(2)?,
            number: row.get(3)?,
            title: row.get(4)?,
            body: row.get(5)?,
            state: row.get(6)?,
            author_id: row.get(7)?,
            assignee_id: row.get(8)?,
            milestone_id: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
            sync_updated_at: None,
            closed_at: row.get(12)?,
            labels,
        }, author_login))
    })?
    .filter_map(|result| {
        match result {
            Ok((issue, author_login)) => {
                // Filter out bot authors
                if let Some(login) = author_login {
                    if is_bot_user(&login, excluded_bots) {
                        return None;
                    }
                }
                Some(Ok(issue))
            }
            Err(e) => Some(Err(e)),
        }
    })
    .collect::<Result<Vec<_>, _>>()?;
    
    Ok(issues)
}

/// Get issues without embeddings
pub fn get_issues_without_embeddings(conn: &Connection, limit: i64) -> Result<Vec<Issue>> {
    let mut stmt = conn.prepare(
        "SELECT id, github_id, repo_id, number, title, body, state, author_id,
                assignee_id, milestone_id, created_at, updated_at, closed_at, labels
         FROM issues
         WHERE embedding IS NULL
         LIMIT ?1"
    )?;

    let issues = stmt.query_map(params![limit], |row| {
        let labels_json: String = row.get(13)?;
        let labels: Vec<String> = serde_json::from_str(&labels_json).unwrap_or_default();

        Ok(Issue {
            id: row.get(0)?,
            github_id: row.get(1)?,
            repo_id: row.get(2)?,
            number: row.get(3)?,
            title: row.get(4)?,
            body: row.get(5)?,
            state: row.get(6)?,
            author_id: row.get(7)?,
            assignee_id: row.get(8)?,
            milestone_id: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
            sync_updated_at: None,
            closed_at: row.get(12)?,
            labels,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(issues)
}

/// Store embedding vector for an issue
pub fn set_issue_embedding(conn: &Connection, issue_id: i64, embedding: &[f32]) -> Result<()> {
    // Convert f32 vector to bytes
    let bytes: Vec<u8> = embedding
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect();

    conn.execute(
        "UPDATE issues SET embedding = ?1 WHERE id = ?2",
        params![bytes, issue_id],
    )?;
    Ok(())
}

/// Get embedding vector for an issue
pub fn get_issue_embedding(conn: &Connection, issue_id: i64) -> Result<Option<Vec<f32>>> {
    let embedding_bytes: Option<Vec<u8>> = conn
        .query_row(
            "SELECT embedding FROM issues WHERE id = ?1",
            params![issue_id],
            |row| row.get(0),
        )
        .optional()?;

    Ok(embedding_bytes.map(|bytes| {
        bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect()
    }))
}

// ============================================================================
// PULL REQUEST QUERIES
// ============================================================================

/// Upsert a pull request
pub fn upsert_pull_request(
    conn: &Connection,
    github_id: i64,
    repo_id: i64,
    number: i32,
    title: &str,
    body: Option<&str>,
    state: &str,
    author_id: Option<i64>,
    created_at: &str,
    updated_at: &str,
    merged_at: Option<&str>,
    closed_at: Option<&str>,
    additions: i32,
    deletions: i32,
    changed_files: i32,
    labels: &[String],
    sync_updated_at: &str,
) -> Result<i64> {
    let labels_json = serde_json::to_string(labels)?;
    
    conn.execute(
        "INSERT INTO pull_requests (github_id, repo_id, number, title, body, state, author_id,
                                   created_at, updated_at, merged_at, closed_at, 
                                   additions, deletions, changed_files, labels, sync_updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
         ON CONFLICT(github_id) DO UPDATE SET
            title = excluded.title,
            body = excluded.body,
            state = excluded.state,
            author_id = COALESCE(excluded.author_id, author_id),
            updated_at = excluded.updated_at,
            merged_at = excluded.merged_at,
            closed_at = excluded.closed_at,
            additions = excluded.additions,
            deletions = excluded.deletions,
            changed_files = excluded.changed_files,
            labels = excluded.labels,
            sync_updated_at = excluded.sync_updated_at
         WHERE excluded.sync_updated_at >= COALESCE(sync_updated_at, excluded.sync_updated_at) OR sync_updated_at IS NULL",
        params![github_id, repo_id, number, title, body, state, author_id,
                created_at, updated_at, merged_at, closed_at, additions, deletions, 
                changed_files, labels_json, sync_updated_at],
    )?;
    
    let id: i64 = conn.query_row(
        "SELECT id FROM pull_requests WHERE github_id = ?1",
        params![github_id],
        |row| row.get(0),
    )?;
    
    Ok(id)
}

/// Get PRs within a date range, excluding bots
pub fn get_prs_for_metrics(
    conn: &Connection,
    since: &str,
    excluded_bots: &[String],
) -> Result<Vec<PullRequest>> {
    let query = "
        SELECT p.id, p.github_id, p.repo_id, p.number, p.title, p.body, p.state,
               p.author_id, p.created_at, p.updated_at, p.merged_at, p.closed_at,
               p.additions, p.deletions, p.changed_files, p.review_comments,
               p.labels, u.login
        FROM pull_requests p
        LEFT JOIN users u ON p.author_id = u.id
        WHERE p.created_at >= ?1
    ";

    let mut stmt = conn.prepare(query)?;

    let prs = stmt.query_map(params![since], |row| {
        let labels_json: String = row.get(16)?;
        let labels: Vec<String> = serde_json::from_str(&labels_json).unwrap_or_default();
        let author_login: Option<String> = row.get(17)?;

        Ok((PullRequest {
            id: row.get(0)?,
            github_id: row.get(1)?,
            repo_id: row.get(2)?,
            number: row.get(3)?,
            title: row.get(4)?,
            body: row.get(5)?,
            state: row.get(6)?,
            author_id: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
            sync_updated_at: None,
            merged_at: row.get(10)?,
            closed_at: row.get(11)?,
            additions: row.get(12)?,
            deletions: row.get(13)?,
            changed_files: row.get(14)?,
            review_comments: row.get(15)?,
            labels,
        }, author_login))
    })?
    .filter_map(|result| {
        match result {
            Ok((pr, author_login)) => {
                if let Some(login) = author_login {
                    if is_bot_user(&login, excluded_bots) {
                        return None;
                    }
                }
                Some(Ok(pr))
            }
            Err(e) => Some(Err(e)),
        }
    })
    .collect::<Result<Vec<_>, _>>()?;
    
    Ok(prs)
}

/// Get PRs without embeddings
pub fn get_prs_without_embeddings(conn: &Connection, limit: i64) -> Result<Vec<PullRequest>> {
    let mut stmt = conn.prepare(
        "SELECT id, github_id, repo_id, number, title, body, state, author_id,
                created_at, updated_at, merged_at, closed_at, additions, deletions,
                changed_files, review_comments, labels
         FROM pull_requests
         WHERE embedding IS NULL
         LIMIT ?1"
    )?;

    let prs = stmt.query_map(params![limit], |row| {
        let labels_json: String = row.get(16)?;
        let labels: Vec<String> = serde_json::from_str(&labels_json).unwrap_or_default();

        Ok(PullRequest {
            id: row.get(0)?,
            github_id: row.get(1)?,
            repo_id: row.get(2)?,
            number: row.get(3)?,
            title: row.get(4)?,
            body: row.get(5)?,
            state: row.get(6)?,
            author_id: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
            sync_updated_at: None,
            merged_at: row.get(10)?,
            closed_at: row.get(11)?,
            additions: row.get(12)?,
            deletions: row.get(13)?,
            changed_files: row.get(14)?,
            review_comments: row.get(15)?,
            labels,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(prs)
}

/// Store embedding vector for a PR
pub fn set_pr_embedding(conn: &Connection, pr_id: i64, embedding: &[f32]) -> Result<()> {
    // Convert f32 vector to bytes
    let bytes: Vec<u8> = embedding
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect();

    conn.execute(
        "UPDATE pull_requests SET embedding = ?1 WHERE id = ?2",
        params![bytes, pr_id],
    )?;
    Ok(())
}

/// Get embedding vector for a PR
pub fn get_pr_embedding(conn: &Connection, pr_id: i64) -> Result<Option<Vec<f32>>> {
    let embedding_bytes: Option<Vec<u8>> = conn
        .query_row(
            "SELECT embedding FROM pull_requests WHERE id = ?1",
            params![pr_id],
            |row| row.get(0),
        )
        .optional()?;

    Ok(embedding_bytes.map(|bytes| {
        bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect()
    }))
}

// ============================================================================
// PR REVIEW QUERIES
// ============================================================================

/// Upsert a PR review
pub fn upsert_pr_review(
    conn: &Connection,
    github_id: i64,
    pr_id: i64,
    reviewer_id: Option<i64>,
    state: &str,
    submitted_at: &str,
    sync_updated_at: &str,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO pr_reviews (github_id, pr_id, reviewer_id, state, submitted_at, sync_updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(github_id) DO UPDATE SET
            state = excluded.state,
            reviewer_id = COALESCE(excluded.reviewer_id, reviewer_id),
            sync_updated_at = excluded.sync_updated_at
         WHERE sync_updated_at IS NULL OR excluded.sync_updated_at >= sync_updated_at",
        params![github_id, pr_id, reviewer_id, state, submitted_at, sync_updated_at],
    )?;

    let id: i64 = conn.query_row(
        "SELECT id FROM pr_reviews WHERE github_id = ?1",
        params![github_id],
        |row| row.get(0),
    )?;

    Ok(id)
}

/// Get review rounds for a PR (distinct review submissions)
pub fn get_pr_review_count(conn: &Connection, pr_id: i64) -> Result<i32> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(DISTINCT reviewer_id || '-' || date(submitted_at)) 
         FROM pr_reviews WHERE pr_id = ?1",
        params![pr_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// Get first review timestamp for a PR
pub fn get_first_review_time(conn: &Connection, pr_id: i64) -> Result<Option<String>> {
    let result = conn.query_row(
        "SELECT MIN(submitted_at) FROM pr_reviews WHERE pr_id = ?1",
        params![pr_id],
        |row| row.get(0),
    ).optional()?;
    Ok(result.flatten())
}

// ============================================================================
// WATERMARK QUERIES (for incremental sync)
// ============================================================================

/// Get the maximum sync_updated_at for issues in a given repo (for incremental sync)
pub fn get_issues_watermark(conn: &Connection, repo_id: i64) -> Result<Option<String>> {
    let result = conn.query_row(
        "SELECT MAX(sync_updated_at) FROM issues WHERE repo_id = ?1",
        params![repo_id],
        |row| row.get(0),
    ).optional()?;
    Ok(result.flatten())
}

/// Get the maximum sync_updated_at for pull requests in a given repo (for incremental sync)
pub fn get_prs_watermark(conn: &Connection, repo_id: i64) -> Result<Option<String>> {
    let result = conn.query_row(
        "SELECT MAX(sync_updated_at) FROM pull_requests WHERE repo_id = ?1",
        params![repo_id],
        |row| row.get(0),
    ).optional()?;
    Ok(result.flatten())
}

/// Get the maximum sync_updated_at for PR reviews in a given repo (for incremental sync)
pub fn get_reviews_watermark(conn: &Connection, repo_id: i64) -> Result<Option<String>> {
    let result = conn.query_row(
        "SELECT MAX(pr_reviews.sync_updated_at)
         FROM pr_reviews
         JOIN pull_requests ON pr_reviews.pr_id = pull_requests.id
         WHERE pull_requests.repo_id = ?1",
        params![repo_id],
        |row| row.get(0),
    ).optional()?;
    Ok(result.flatten())
}

// ============================================================================
// MILESTONE QUERIES
// ============================================================================

/// Upsert a milestone
pub fn upsert_milestone(
    conn: &Connection,
    github_id: i64,
    repo_id: i64,
    title: &str,
    description: Option<&str>,
    state: &str,
    due_on: Option<&str>,
    open_issues: i32,
    closed_issues: i32,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO milestones (github_id, repo_id, title, description, state, due_on, 
                                open_issues, closed_issues)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(github_id) DO UPDATE SET
            title = excluded.title,
            description = excluded.description,
            state = excluded.state,
            due_on = excluded.due_on,
            open_issues = excluded.open_issues,
            closed_issues = excluded.closed_issues",
        params![github_id, repo_id, title, description, state, due_on, open_issues, closed_issues],
    )?;
    
    let id: i64 = conn.query_row(
        "SELECT id FROM milestones WHERE github_id = ?1",
        params![github_id],
        |row| row.get(0),
    )?;
    
    Ok(id)
}

/// Get milestone ID by GitHub ID
pub fn get_milestone_id_by_github_id(conn: &Connection, github_id: i64) -> Result<Option<i64>> {
    let result = conn.query_row(
        "SELECT id FROM milestones WHERE github_id = ?1",
        params![github_id],
        |row| row.get(0),
    ).optional()?;
    Ok(result)
}

/// Get all open milestones grouped by title (for roadmap)
pub fn get_milestones_by_cycle(conn: &Connection) -> Result<Vec<Milestone>> {
    let mut stmt = conn.prepare(
        "SELECT id, github_id, repo_id, title, description, state, due_on, open_issues, closed_issues
         FROM milestones
         ORDER BY due_on ASC NULLS LAST, title ASC"
    )?;
    
    let milestones = stmt.query_map([], |row| {
        Ok(Milestone {
            id: row.get(0)?,
            github_id: row.get(1)?,
            repo_id: row.get(2)?,
            title: row.get(3)?,
            description: row.get(4)?,
            state: row.get(5)?,
            due_on: row.get(6)?,
            open_issues: row.get(7)?,
            closed_issues: row.get(8)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    
    Ok(milestones)
}

// ============================================================================
// SQUAD QUERIES
// ============================================================================

/// Upsert a squad
pub fn upsert_squad(
    conn: &Connection,
    id: &str,
    name: &str,
    color: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO squads (id, name, color)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            color = excluded.color",
        params![id, name, color],
    )?;
    Ok(())
}

/// Set squad members (replaces existing)
pub fn set_squad_members(conn: &Connection, squad_id: &str, member_logins: &[String]) -> Result<()> {
    // Remove existing members
    conn.execute("DELETE FROM squad_members WHERE squad_id = ?1", params![squad_id])?;
    
    // Add new members
    for login in member_logins {
        if let Some(user) = get_user_by_login(conn, login)? {
            conn.execute(
                "INSERT OR IGNORE INTO squad_members (squad_id, user_id) VALUES (?1, ?2)",
                params![squad_id, user.id],
            )?;
        }
    }
    
    Ok(())
}

/// Get all squads with their members
pub fn get_all_squads(conn: &Connection) -> Result<Vec<Squad>> {
    let mut stmt = conn.prepare("SELECT id, name, color FROM squads")?;
    
    let squads = stmt.query_map([], |row| {
        Ok(Squad {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            members: Vec::new(), // Filled below
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    
    // Fill in members
    let squads_with_members: Vec<Squad> = squads.into_iter().map(|mut squad| {
        let mut member_stmt = conn.prepare(
            "SELECT u.login FROM squad_members sm 
             JOIN users u ON sm.user_id = u.id 
             WHERE sm.squad_id = ?1"
        ).unwrap();
        
        squad.members = member_stmt.query_map(params![squad.id], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        
        squad
    }).collect();
    
    Ok(squads_with_members)
}

// ============================================================================
// SYNC LOG QUERIES
// ============================================================================

/// Record sync start
pub fn record_sync_start(conn: &Connection, repo_id: i64, sync_type: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO sync_log (repo_id, sync_type, started_at) VALUES (?1, ?2, datetime('now'))",
        params![repo_id, sync_type],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Record sync completion
pub fn record_sync_complete(conn: &Connection, log_id: i64, items_synced: i32) -> Result<()> {
    conn.execute(
        "UPDATE sync_log SET completed_at = datetime('now'), items_synced = ?1 WHERE id = ?2",
        params![items_synced, log_id],
    )?;
    Ok(())
}

/// Record sync error
pub fn record_sync_error(conn: &Connection, log_id: i64, error: &str) -> Result<()> {
    conn.execute(
        "UPDATE sync_log SET completed_at = datetime('now'), error = ?1 WHERE id = ?2",
        params![error, log_id],
    )?;
    Ok(())
}

// ============================================================================
// STATS QUERIES
// ============================================================================

/// Get counts for dashboard summary
pub fn get_sync_stats(conn: &Connection) -> Result<SyncStats> {
    let issue_count: i64 = conn.query_row("SELECT COUNT(*) FROM issues", [], |row| row.get(0))?;
    let pr_count: i64 = conn.query_row("SELECT COUNT(*) FROM pull_requests", [], |row| row.get(0))?;
    let user_count: i64 = conn.query_row("SELECT COUNT(*) FROM users WHERE is_bot = FALSE", [], |row| row.get(0))?;
    let repo_count: i64 = conn.query_row("SELECT COUNT(*) FROM repositories WHERE enabled = TRUE", [], |row| row.get(0))?;
    
    Ok(SyncStats {
        issues: issue_count,
        pull_requests: pr_count,
        users: user_count,
        repositories: repo_count,
    })
}

#[derive(Debug, serde::Serialize)]
pub struct SyncStats {
    pub issues: i64,
    pub pull_requests: i64,
    pub users: i64,
    pub repositories: i64,
}

// ============================================================================
// FILTERED METRICS QUERIES (for dashboard filters)
// ============================================================================

/// Get issues with optional filters for metrics
pub fn get_issues_for_metrics_filtered(
    conn: &Connection,
    since: &str,
    until: Option<&str>,
    excluded_bots: &[String],
    repo_ids: Option<&[i64]>,
    user_id: Option<i64>,
    squad_member_ids: Option<&[i64]>,
) -> Result<Vec<Issue>> {
    let mut query = String::from(
        "SELECT i.id, i.github_id, i.repo_id, i.number, i.title, i.body, i.state,
                i.author_id, i.assignee_id, i.milestone_id, i.created_at, i.updated_at,
                i.closed_at, i.labels, u.login
         FROM issues i
         LEFT JOIN users u ON i.author_id = u.id
         WHERE i.created_at >= ?1"
    );

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(since.to_string())];
    let mut param_idx = 2;

    // Add date range end filter
    if let Some(end) = until {
        query.push_str(&format!(" AND i.created_at <= ?{}", param_idx));
        params_vec.push(Box::new(end.to_string()));
        param_idx += 1;
    }

    // Add repository filter
    if let Some(repos) = repo_ids {
        if !repos.is_empty() {
            let placeholders = (0..repos.len())
                .map(|idx| format!("?{}", param_idx + idx))
                .collect::<Vec<_>>()
                .join(", ");
            query.push_str(&format!(" AND i.repo_id IN ({})", placeholders));
            for &repo_id in repos {
                params_vec.push(Box::new(repo_id));
            }
            param_idx += repos.len();
        }
    }

    // Add user filter
    if let Some(uid) = user_id {
        query.push_str(&format!(" AND i.author_id = ?{}", param_idx));
        params_vec.push(Box::new(uid));
        param_idx += 1;
    }

    // Add squad filter (members)
    if let Some(member_ids) = squad_member_ids {
        if !member_ids.is_empty() {
            let placeholders = (0..member_ids.len())
                .map(|idx| format!("?{}", param_idx + idx))
                .collect::<Vec<_>>()
                .join(", ");
            query.push_str(&format!(" AND i.author_id IN ({})", placeholders));
            for &member_id in member_ids {
                params_vec.push(Box::new(member_id));
            }
        }
    }

    let mut stmt = conn.prepare(&query)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter()
        .map(|p| p.as_ref() as &dyn rusqlite::ToSql)
        .collect();

    let issues = stmt.query_map(param_refs.as_slice(), |row| {
        let labels_json: String = row.get(13)?;
        let labels: Vec<String> = serde_json::from_str(&labels_json).unwrap_or_default();
        let author_login: Option<String> = row.get(14)?;

        Ok((Issue {
            id: row.get(0)?,
            github_id: row.get(1)?,
            repo_id: row.get(2)?,
            number: row.get(3)?,
            title: row.get(4)?,
            body: row.get(5)?,
            state: row.get(6)?,
            author_id: row.get(7)?,
            assignee_id: row.get(8)?,
            milestone_id: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
            sync_updated_at: None,
            closed_at: row.get(12)?,
            labels,
        }, author_login))
    })?
    .filter_map(|result| {
        match result {
            Ok((issue, author_login)) => {
                if let Some(login) = author_login {
                    if is_bot_user(&login, excluded_bots) {
                        return None;
                    }
                }
                Some(Ok(issue))
            }
            Err(e) => Some(Err(e)),
        }
    })
    .collect::<Result<Vec<_>, _>>()?;

    Ok(issues)
}

/// Get PRs with optional filters for metrics
pub fn get_prs_for_metrics_filtered(
    conn: &Connection,
    since: &str,
    until: Option<&str>,
    excluded_bots: &[String],
    repo_ids: Option<&[i64]>,
    user_id: Option<i64>,
    squad_member_ids: Option<&[i64]>,
) -> Result<Vec<PullRequest>> {
    let mut query = String::from(
        "SELECT p.id, p.github_id, p.repo_id, p.number, p.title, p.body, p.state,
                p.author_id, p.created_at, p.updated_at, p.merged_at, p.closed_at,
                p.additions, p.deletions, p.changed_files, p.review_comments,
                p.labels, u.login
         FROM pull_requests p
         LEFT JOIN users u ON p.author_id = u.id
         WHERE p.created_at >= ?1"
    );

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(since.to_string())];
    let mut param_idx = 2;

    // Add date range end filter
    if let Some(end) = until {
        query.push_str(&format!(" AND p.created_at <= ?{}", param_idx));
        params_vec.push(Box::new(end.to_string()));
        param_idx += 1;
    }

    // Add repository filter
    if let Some(repos) = repo_ids {
        if !repos.is_empty() {
            let placeholders = (0..repos.len())
                .map(|idx| format!("?{}", param_idx + idx))
                .collect::<Vec<_>>()
                .join(", ");
            query.push_str(&format!(" AND p.repo_id IN ({})", placeholders));
            for &repo_id in repos {
                params_vec.push(Box::new(repo_id));
            }
            param_idx += repos.len();
        }
    }

    // Add user filter
    if let Some(uid) = user_id {
        query.push_str(&format!(" AND p.author_id = ?{}", param_idx));
        params_vec.push(Box::new(uid));
        param_idx += 1;
    }

    // Add squad filter (members)
    if let Some(member_ids) = squad_member_ids {
        if !member_ids.is_empty() {
            let placeholders = (0..member_ids.len())
                .map(|idx| format!("?{}", param_idx + idx))
                .collect::<Vec<_>>()
                .join(", ");
            query.push_str(&format!(" AND p.author_id IN ({})", placeholders));
            for &member_id in member_ids {
                params_vec.push(Box::new(member_id));
            }
        }
    }

    let mut stmt = conn.prepare(&query)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter()
        .map(|p| p.as_ref() as &dyn rusqlite::ToSql)
        .collect();

    let prs = stmt.query_map(param_refs.as_slice(), |row| {
        let labels_json: String = row.get(16)?;
        let labels: Vec<String> = serde_json::from_str(&labels_json).unwrap_or_default();
        let author_login: Option<String> = row.get(17)?;

        Ok((PullRequest {
            id: row.get(0)?,
            github_id: row.get(1)?,
            repo_id: row.get(2)?,
            number: row.get(3)?,
            title: row.get(4)?,
            body: row.get(5)?,
            state: row.get(6)?,
            author_id: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
            sync_updated_at: None,
            merged_at: row.get(10)?,
            closed_at: row.get(11)?,
            additions: row.get(12)?,
            deletions: row.get(13)?,
            changed_files: row.get(14)?,
            review_comments: row.get(15)?,
            labels,
        }, author_login))
    })?
    .filter_map(|result| {
        match result {
            Ok((pr, author_login)) => {
                if let Some(login) = author_login {
                    if is_bot_user(&login, excluded_bots) {
                        return None;
                    }
                }
                Some(Ok(pr))
            }
            Err(e) => Some(Err(e)),
        }
    })
    .collect::<Result<Vec<_>, _>>()?;

    Ok(prs)
}

// ============================================================================
// HELPER QUERIES FOR FILTERS
// ============================================================================

/// Get squad member user IDs
pub fn get_squad_member_ids(conn: &Connection, squad_id: &str) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare(
        "SELECT user_id FROM squad_members WHERE squad_id = ?1"
    )?;

    let ids = stmt.query_map(params![squad_id], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ids)
}

/// Get all non-bot users for filtering
pub fn get_all_users(conn: &Connection) -> Result<Vec<User>> {
    let mut stmt = conn.prepare(
        "SELECT id, github_id, login, name, avatar_url, is_bot
         FROM users
         WHERE is_bot = FALSE
         ORDER BY login ASC"
    )?;

    let users = stmt.query_map([], |row| {
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

    Ok(users)
}

/// Get all repositories for filtering
pub fn get_all_repositories(conn: &Connection) -> Result<Vec<Repository>> {
    let mut stmt = conn.prepare(
        "SELECT id, owner, name, github_id, enabled, last_synced_at
         FROM repositories
         ORDER BY owner ASC, name ASC"
    )?;

    let repos = stmt.query_map([], |row| {
        Ok(Repository {
            id: row.get(0)?,
            owner: row.get(1)?,
            name: row.get(2)?,
            github_id: row.get(3)?,
            enabled: row.get(4)?,
            last_synced_at: row.get(5)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(repos)
}

// ============================================================================
// SETTINGS QUERIES
// ============================================================================

/// Get application settings (always returns the single row)
pub fn get_settings(conn: &Connection) -> Result<Settings> {
    let row = conn.query_row(
        "SELECT id, history_days, excluded_bots, bug_labels, feature_labels, created_at, updated_at
         FROM settings WHERE id = 1",
        [],
        |row| {
            let excluded_bots_json: String = row.get(2)?;
            let bug_labels_json: String = row.get(3)?;
            let feature_labels_json: String = row.get(4)?;

            Ok(Settings {
                id: row.get(0)?,
                history_days: row.get(1)?,
                excluded_bots: serde_json::from_str(&excluded_bots_json).unwrap_or_default(),
                bug_labels: serde_json::from_str(&bug_labels_json).unwrap_or_default(),
                feature_labels: serde_json::from_str(&feature_labels_json).unwrap_or_default(),
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    )?;

    Ok(row)
}

/// Update application settings
pub fn update_settings(
    conn: &Connection,
    history_days: i32,
    excluded_bots: &[String],
    bug_labels: &[String],
    feature_labels: &[String],
) -> Result<()> {
    let excluded_bots_json = serde_json::to_string(excluded_bots)?;
    let bug_labels_json = serde_json::to_string(bug_labels)?;
    let feature_labels_json = serde_json::to_string(feature_labels)?;

    conn.execute(
        "UPDATE settings SET
            history_days = ?1,
            excluded_bots = ?2,
            bug_labels = ?3,
            feature_labels = ?4,
            updated_at = datetime('now')
         WHERE id = 1",
        params![history_days, excluded_bots_json, bug_labels_json, feature_labels_json],
    )?;

    Ok(())
}
