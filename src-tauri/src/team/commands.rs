use crate::db::{
    models::User,
    project_queries::TimelineEvent,
    user_queries::{ActivityDataPoint, CollaborationMatrix, FocusMetrics, RepositoryContribution, UserSummary},
    AppState,
};
use rusqlite::params;
use tauri::State;

/// Add a user to the tracked users list
#[tauri::command]
pub async fn add_tracked_user(
    username: String,
    state: State<'_, AppState>,
) -> Result<User, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    // First, find the user by username
    let user: User = conn
        .query_row(
            "SELECT id, github_id, login, name, avatar_url, is_bot FROM users WHERE login = ?1",
            params![username],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    github_id: row.get(1)?,
                    login: row.get(2)?,
                    name: row.get(3)?,
                    avatar_url: row.get(4)?,
                    is_bot: row.get(5)?,
                })
            },
        )
        .map_err(|e| format!("User '{}' not found in database: {}. Please sync repositories that include this user first.", username, e))?;

    // Add to tracked_users (will fail silently if already exists due to UNIQUE constraint)
    let now = chrono::Utc::now().to_rfc3339();
    match conn.execute(
        "INSERT INTO tracked_users (user_id, added_at) VALUES (?1, ?2)",
        params![user.id, now],
    ) {
        Ok(_) => {
            tracing::info!("Added user '{}' to tracked users", username);
            Ok(user)
        }
        Err(rusqlite::Error::SqliteFailure(err, _)) if err.extended_code == 2067 => {
            // UNIQUE constraint failed - user already tracked
            tracing::debug!("User '{}' is already tracked", username);
            Ok(user)
        }
        Err(e) => Err(format!("Failed to add tracked user: {}", e)),
    }
}

/// Remove a user from the tracked users list
#[tauri::command]
pub async fn remove_tracked_user(
    username: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    // Find user by username, then delete from tracked_users
    let rows_affected = conn
        .execute(
            "DELETE FROM tracked_users WHERE user_id = (SELECT id FROM users WHERE login = ?1)",
            params![username],
        )
        .map_err(|e| format!("Failed to remove tracked user: {}", e))?;

    if rows_affected > 0 {
        tracing::info!("Removed user '{}' from tracked users", username);
        Ok(())
    } else {
        Err(format!("User '{}' was not being tracked", username))
    }
}

/// Get all tracked users
#[tauri::command]
pub async fn get_tracked_users(state: State<'_, AppState>) -> Result<Vec<User>, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT u.id, u.github_id, u.login, u.name, u.avatar_url, u.is_bot
             FROM tracked_users t
             JOIN users u ON t.user_id = u.id
             ORDER BY t.added_at DESC",
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
