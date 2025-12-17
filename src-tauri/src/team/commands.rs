use crate::db::{models::User, AppState};
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
