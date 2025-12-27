use crate::db::{
    models::User,
    project_queries::TimelineEvent,
    queries,
    user_queries::{ActivityDataPoint, CollaborationMatrix, FocusMetrics, RepositoryContribution, UserSummary},
    AppState,
};
use crate::github::auth;
use reqwest::Client;
use rusqlite::params;
use serde::Deserialize;
use tauri::{Manager, State};

/// Add a user to the tracked users list
#[tauri::command]
pub async fn add_tracked_user(
    username: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<User, String> {
    let token = auth::get_token()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Not authenticated".to_string())?;

    // Try to find the user locally first
    let user_exists = {
        let conn = state.sqlite.lock().map_err(|e| e.to_string())?;
        queries::get_user_by_login(&conn, &username)
            .map_err(|e| e.to_string())?
    };

    // If not found locally, fetch from GitHub
    let gh_user = if user_exists.is_none() {
        Some(fetch_github_user(&username, &token).await?)
    } else {
        None
    };

    // Now do all database operations without holding lock across await
    let user = {
        let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

        // Insert fetched user if needed
        if let Some(gh_user) = gh_user {
            let _ = queries::get_or_create_user(
                &conn,
                gh_user.id,
                &gh_user.login,
                gh_user.name.as_deref(),
                Some(&gh_user.avatar_url),
                None,
                None,
                None,
            )
            .map_err(|e| e.to_string())?;
        }

        // Get the user (either existing or just created)
        let user = queries::get_user_by_login(&conn, &username)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("User '{}' not found", username))?;

        // Mark user as tracked in users table (idempotent)
        let now = chrono::Utc::now().to_rfc3339();
        let rows_updated = conn.execute(
            "UPDATE users SET tracked = 1, tracked_at = COALESCE(tracked_at, ?2) WHERE id = ?1",
            params![user.id, now],
        )
        .map_err(|e| format!("Failed to mark user tracked: {}", e))?;

        tracing::info!("Marked user '{}' (id: {}) as tracked. Rows updated: {}", user.login, user.id, rows_updated);

        user
    }; // conn is dropped here automatically

    // Trigger a user-centric sync to fetch this user's activity data
    tracing::info!("Fetching activity data for tracked user '{}'", username);
    if let Err(e) = crate::github::sync_user::sync_tracked_user(&app, &state, &token, &username).await {
        tracing::error!("Failed to sync user activity: {}", e);
        // Don't fail the command - user was still added successfully
    }

    Ok(user)
}

#[derive(Debug, Deserialize)]
struct GithubUserResponse {
    id: i64,
    login: String,
    name: Option<String>,
    avatar_url: String,
}

async fn fetch_github_user(username: &str, token: &str) -> Result<GithubUserResponse, String> {
    let client = Client::new();
    let url = format!("https://api.github.com/users/{}", username);

    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "MADE-Activity-Tracker")
        .send()
        .await
        .map_err(|e| format!("Failed to call GitHub: {}", e))?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(format!("GitHub user '{}' not found", username));
    }

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("GitHub API error ({}): {}", status, body));
    }

    response
        .json::<GithubUserResponse>()
        .await
        .map_err(|e| format!("Failed to parse GitHub user response: {}", e))
}

/// Remove a user from the tracked users list and delete all their data
#[tauri::command]
pub async fn remove_tracked_user(
    username: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    // First, verify the user exists and is tracked
    let user_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM users WHERE login = ?1 AND tracked = 1",
            params![username],
            |row| row.get(0),
        )
        .ok();

    let user_id = match user_id {
        Some(id) => id,
        None => return Err(format!("User '{}' was not being tracked", username)),
    };

    tracing::info!("Removing user '{}' (id: {}) from tracked users list", username, user_id);

    // Remove from squad memberships
    let squad_memberships = conn
        .execute(
            "DELETE FROM squad_members WHERE user_id = ?1",
            params![user_id],
        )
        .map_err(|e| format!("Failed to delete squad memberships: {}", e))?;
    if squad_memberships > 0 {
        tracing::info!("Removed user '{}' from {} squads", username, squad_memberships);
    }

    // Delete from deprecated tracked_users table if it exists
    let _ = conn.execute(
        "DELETE FROM tracked_users WHERE user_id = ?1",
        params![user_id],
    );

    // Mark user as not tracked (but keep their data)
    conn.execute(
        "UPDATE users SET tracked = 0, tracked_at = NULL WHERE id = ?1",
        params![user_id],
    )
    .map_err(|e| format!("Failed to untrack user: {}", e))?;

    tracing::info!("Successfully removed user '{}' from tracked users list (their contributions remain visible)", username);
    Ok(())
}

/// Get all tracked users
#[tauri::command]
pub async fn get_tracked_users(state: State<'_, AppState>) -> Result<Vec<User>, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, github_id, login, name, avatar_url, is_bot, tracked, tracked_at
             FROM users
             WHERE tracked = 1
             ORDER BY tracked_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let users = stmt
        .query_map([], |row| {
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
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(users)
}

/// Get summary statistics for a user
#[tauri::command]
pub async fn get_user_summary(
    username: String,
    start_date: Option<String>,
    end_date: Option<String>,
    state: State<'_, AppState>,
) -> Result<UserSummary, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    // Find user by username
    let user_id: i64 = conn
        .query_row(
            "SELECT id FROM users WHERE login = ?1",
            params![username],
            |row| row.get(0),
        )
        .map_err(|e| format!("User '{}' not found: {}", username, e))?;

    // Get user summary
    crate::db::user_queries::get_user_summary_data(
        &conn,
        user_id,
        start_date.as_deref(),
        end_date.as_deref(),
    )
    .map_err(|e| format!("Failed to get user summary: {}", e))
}

/// Get activity timeline for a user
#[tauri::command]
pub async fn get_user_activity_timeline(
    username: String,
    start_date: Option<String>,
    end_date: Option<String>,
    limit: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<TimelineEvent>, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    let limit = limit.unwrap_or(100); // Default to 100 events

    // Find user by username
    let user_id: i64 = conn
        .query_row(
            "SELECT id FROM users WHERE login = ?1",
            params![username],
            |row| row.get(0),
        )
        .map_err(|e| format!("User '{}' not found: {}", username, e))?;

    // Get user timeline
    crate::db::user_queries::get_user_activity_timeline(
        &conn,
        user_id,
        start_date.as_deref(),
        end_date.as_deref(),
        limit,
    )
    .map_err(|e| format!("Failed to get user timeline: {}", e))
}

/// Get repository distribution for a user
#[tauri::command]
pub async fn get_user_repository_distribution(
    username: String,
    start_date: Option<String>,
    end_date: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<RepositoryContribution>, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    // Find user by username
    let user_id: i64 = conn
        .query_row(
            "SELECT id FROM users WHERE login = ?1",
            params![username],
            |row| row.get(0),
        )
        .map_err(|e| format!("User '{}' not found: {}", username, e))?;

    // Get repository distribution
    crate::db::user_queries::get_user_repo_distribution(
        &conn,
        user_id,
        start_date.as_deref(),
        end_date.as_deref(),
    )
    .map_err(|e| format!("Failed to get repository distribution: {}", e))
}

/// Get collaboration matrix showing interactions between tracked users
#[tauri::command]
pub async fn get_team_collaboration_matrix(
    usernames: Vec<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    state: State<'_, AppState>,
) -> Result<CollaborationMatrix, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    // Convert usernames to user IDs
    let placeholders = usernames.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let query = format!("SELECT id FROM users WHERE login IN ({})", placeholders);

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    let username_params: Vec<&dyn rusqlite::ToSql> = usernames
        .iter()
        .map(|u| u as &dyn rusqlite::ToSql)
        .collect();

    let user_ids: Vec<i64> = stmt
        .query_map(&username_params[..], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    if user_ids.is_empty() {
        return Err("No valid users found".to_string());
    }

    // Get collaboration matrix
    crate::db::user_queries::get_collaboration_matrix(
        &conn,
        user_ids,
        start_date.as_deref(),
        end_date.as_deref(),
    )
    .map_err(|e| format!("Failed to get collaboration matrix: {}", e))
}

/// Get activity trend for a user over time
#[tauri::command]
pub async fn get_user_activity_trend(
    username: String,
    start_date: Option<String>,
    end_date: Option<String>,
    granularity: String, // "day", "week", "month"
    state: State<'_, AppState>,
) -> Result<Vec<ActivityDataPoint>, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    // Find user by username
    let user_id: i64 = conn
        .query_row(
            "SELECT id FROM users WHERE login = ?1",
            params![username],
            |row| row.get(0),
        )
        .map_err(|e| format!("User '{}' not found: {}", username, e))?;

    // Get activity trend
    crate::db::user_queries::get_user_activity_trend(
        &conn,
        user_id,
        start_date.as_deref(),
        end_date.as_deref(),
        &granularity,
    )
    .map_err(|e| format!("Failed to get activity trend: {}", e))
}

/// Get focus metrics for a user
#[tauri::command]
pub async fn get_user_focus_metrics(
    username: String,
    start_date: Option<String>,
    end_date: Option<String>,
    state: State<'_, AppState>,
) -> Result<FocusMetrics, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    // Find user by username
    let user_id: i64 = conn
        .query_row(
            "SELECT id FROM users WHERE login = ?1",
            params![username],
            |row| row.get(0),
        )
        .map_err(|e| format!("User '{}' not found: {}", username, e))?;

    // Get focus metrics
    crate::db::user_queries::get_user_focus_metrics(
        &conn,
        user_id,
        start_date.as_deref(),
        end_date.as_deref(),
    )
    .map_err(|e| format!("Failed to get focus metrics: {}", e))
}
