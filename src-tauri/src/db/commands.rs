use super::models::*;
use super::queries;
use crate::db::AppState;
use rusqlite::params;
use tauri::State;

// ============================================================================
// QUERY HELPER COMMANDS
// ============================================================================

/// Get sync statistics
#[tauri::command]
pub async fn get_sync_stats(state: State<'_, AppState>) -> Result<queries::SyncStats, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;
    queries::get_sync_stats(&conn).map_err(|e| e.to_string())
}

/// Get all non-bot users for filtering
#[tauri::command]
pub async fn get_all_users(state: State<'_, AppState>) -> Result<Vec<User>, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;
    queries::get_all_users(&conn).map_err(|e| e.to_string())
}

/// Get all repositories for filtering
#[tauri::command]
pub async fn get_all_repositories(state: State<'_, AppState>) -> Result<Vec<Repository>, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;
    queries::get_all_repositories(&conn).map_err(|e| e.to_string())
}

// ============================================================================
// REPOSITORY COMMANDS
// ============================================================================

#[tauri::command]
pub async fn add_repository(
    owner: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;
    queries::upsert_repository(&conn, &owner, &name, None, true)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_repository(
    owner: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    // First, get the repository ID
    let repo_id: i64 = conn
        .query_row(
            "SELECT id FROM repositories WHERE owner = ?1 AND name = ?2",
            params![owner, name],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Delete related records first (foreign key constraints)
    // Delete PR reviews (references pull_requests)
    conn.execute(
        "DELETE FROM pr_reviews WHERE pr_id IN (SELECT id FROM pull_requests WHERE repo_id = ?1)",
        params![repo_id],
    )
    .map_err(|e| e.to_string())?;

    // Delete pull requests
    conn.execute(
        "DELETE FROM pull_requests WHERE repo_id = ?1",
        params![repo_id],
    )
    .map_err(|e| e.to_string())?;

    // Delete issues
    conn.execute(
        "DELETE FROM issues WHERE repo_id = ?1",
        params![repo_id],
    )
    .map_err(|e| e.to_string())?;

    // Delete milestones
    conn.execute(
        "DELETE FROM milestones WHERE repo_id = ?1",
        params![repo_id],
    )
    .map_err(|e| e.to_string())?;

    // Delete sync log
    conn.execute(
        "DELETE FROM sync_log WHERE repo_id = ?1",
        params![repo_id],
    )
    .map_err(|e| e.to_string())?;

    // Finally, delete the repository
    conn.execute(
        "DELETE FROM repositories WHERE id = ?1",
        params![repo_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn toggle_repository(
    owner: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE repositories SET enabled = NOT enabled WHERE owner = ?1 AND name = ?2",
        params![owner, name],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// ============================================================================
// SQUAD COMMANDS
// ============================================================================

#[tauri::command]
pub async fn add_squad(
    name: String,
    members: Vec<String>,
    color: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    queries::upsert_squad(&conn, &id, &name, Some(&color))
        .map_err(|e| e.to_string())?;

    queries::set_squad_members(&conn, &id, &members)
        .map_err(|e| e.to_string())?;

    Ok(id)
}

#[tauri::command]
pub async fn update_squad(
    id: String,
    name: Option<String>,
    members: Option<Vec<String>>,
    color: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    // Get current squad
    let current_squad = queries::get_all_squads(&conn)
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|s| s.id == id)
        .ok_or_else(|| "Squad not found".to_string())?;

    let final_name = name.unwrap_or(current_squad.name);
    let final_color = color.or(current_squad.color);

    queries::upsert_squad(&conn, &id, &final_name, final_color.as_deref())
        .map_err(|e| e.to_string())?;

    if let Some(member_list) = members {
        queries::set_squad_members(&conn, &id, &member_list)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn remove_squad(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    // Remove squad members first (foreign key constraint)
    conn.execute(
        "DELETE FROM squad_members WHERE squad_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    // Remove squad
    conn.execute(
        "DELETE FROM squads WHERE id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_all_squads_command(
    state: State<'_, AppState>,
) -> Result<Vec<Squad>, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;
    queries::get_all_squads(&conn).map_err(|e| e.to_string())
}

// ============================================================================
// USER COMMANDS (tracked users)
// ============================================================================

#[tauri::command]
pub async fn toggle_user_tracked(
    username: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE users
         SET tracked = NOT tracked,
             tracked_at = CASE WHEN NOT tracked THEN datetime('now') ELSE tracked_at END
         WHERE login = ?1",
        params![username],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Fix users with invalid github_id (0 or negative)
#[tauri::command]
pub async fn fix_invalid_users(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    use crate::github::auth;
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct GithubUser {
        id: i64,
        login: String,
    }

    let token = auth::get_token()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Not authenticated".to_string())?;

    // Find users with invalid github_id (in separate scope to drop lock)
    let invalid_users: Vec<(i64, String)> = {
        let conn = state.sqlite.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, login FROM users WHERE github_id <= 0"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?;

        let results: Result<Vec<_>, _> = rows.collect();
        results.map_err(|e| e.to_string())?
    }; // Lock is dropped here

    let mut fixed_users = Vec::new();
    let client = Client::new();

    for (db_id, login) in invalid_users {
        tracing::info!("Fixing user '{}' with invalid github_id", login);

        // Fetch correct github_id from GitHub API
        let url = format!("https://api.github.com/users/{}", login);
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "MADE-Activity-Tracker")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            let gh_user: GithubUser = response.json().await.map_err(|e| e.to_string())?;

            // Update the user with correct github_id (in separate scope to drop lock)
            {
                let conn = state.sqlite.lock().map_err(|e| e.to_string())?;
                conn.execute(
                    "UPDATE users SET github_id = ?1 WHERE id = ?2",
                    params![gh_user.id, db_id],
                )
                .map_err(|e| format!("Failed to update user: {}", e))?;
            } // Lock is dropped here

            fixed_users.push(format!("{} (github_id: {})", login, gh_user.id));
            tracing::info!("Fixed user '{}' with github_id {}", login, gh_user.id);
        } else {
            tracing::error!("Failed to fetch GitHub user '{}': {}", login, response.status());
        }
    }

    Ok(fixed_users)
}

// ============================================================================
// SETTINGS COMMANDS
// ============================================================================

#[tauri::command]
pub async fn get_settings(
    state: State<'_, AppState>,
) -> Result<Settings, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;
    queries::get_settings(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_settings(
    history_days: i32,
    excluded_bots: Vec<String>,
    bug_labels: Vec<String>,
    feature_labels: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;
    queries::update_settings(
        &conn,
        history_days,
        &excluded_bots,
        &bug_labels,
        &feature_labels,
    )
    .map_err(|e| e.to_string())
}
