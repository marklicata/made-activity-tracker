use super::auth::{self, AuthResult};
use super::sync;
use crate::db::AppState;
use tauri::{AppHandle, Manager, State}; // Added Manager import

// TODO: Replace with your GitHub OAuth App Client ID
const GITHUB_CLIENT_ID: &str = "Ov23liO78BuaPSWYJI0w";

/// Initiate GitHub Device Flow login
#[tauri::command]
pub async fn github_login(app: AppHandle) -> Result<AuthResult, String> {
    // Start device flow
    let device_response = auth::initiate_device_flow(GITHUB_CLIENT_ID)
        .await
        .map_err(|e| format!("Failed to start device flow: {}", e))?;

    // Open browser for user to authorize
    let verification_url = format!(
        "{}?user_code={}",
        device_response.verification_uri, device_response.user_code
    );

    // Try to open browser (fail silently - user can use the UI link)
    let _ = tauri::api::shell::open(&app.shell_scope(), &verification_url, None);

    // Emit event with user code for display
    app.emit_all("device-code", &device_response.user_code)
        .map_err(|e| e.to_string())?;

    // Poll for token
    let access_token = auth::poll_for_token(
        GITHUB_CLIENT_ID,
        &device_response.device_code,
        device_response.interval as u64,
    )
    .await
    .map_err(|e| format!("Authorization failed: {}", e))?;

    // Get user info
    let user = auth::get_authenticated_user(&access_token)
        .await
        .map_err(|e| format!("Failed to get user info: {}", e))?;

    // Store token securely
    auth::store_token(&access_token)
        .map_err(|e| format!("Failed to save credentials: {}", e))?;

    Ok(AuthResult { user, access_token })
}

/// Log out by deleting stored credentials
#[tauri::command]
pub async fn github_logout() -> Result<(), String> {
    auth::delete_token().map_err(|e| e.to_string())
}

/// Check if user is already authenticated
#[tauri::command]
pub async fn check_auth() -> Result<Option<AuthResult>, String> {
    let token = auth::get_token().map_err(|e| e.to_string())?;

    match token {
        Some(access_token) => {
            // Verify token is still valid by fetching user
            match auth::get_authenticated_user(&access_token).await {
                Ok(user) => Ok(Some(AuthResult { user, access_token })),
                Err(_) => {
                    // Token invalid, clean up
                    auth::delete_token().ok();
                    Ok(None)
                }
            }
        }
        None => Ok(None),
    }
}

/// Sync all GitHub data for configured repositories
#[tauri::command]
pub async fn sync_github_data(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let token = auth::get_token()
        .map_err(|e| e.to_string())?
        .ok_or("Not authenticated")?;

    sync::sync_all_repos(&app, &state, &token)
        .await
        .map_err(|e| e.to_string())
}

/// Get roadmap data (milestones grouped by cycle)
#[tauri::command]
pub async fn get_roadmap(state: State<'_, AppState>) -> Result<Vec<CycleGroup>, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;
    
    let milestones = crate::db::queries::get_milestones_by_cycle(&conn)
        .map_err(|e| e.to_string())?;
    
    // Group milestones by title (cycle name)
    let mut cycles: std::collections::HashMap<String, CycleGroup> = std::collections::HashMap::new();
    
    for milestone in milestones {
        let entry = cycles.entry(milestone.title.clone()).or_insert_with(|| CycleGroup {
            title: milestone.title.clone(),
            due_date: milestone.due_on.clone(),
            milestones: Vec::new(),
            total_open: 0,
            total_closed: 0,
        });
        
        entry.total_open += milestone.open_issues;
        entry.total_closed += milestone.closed_issues;
        entry.milestones.push(MilestoneInfo {
            id: milestone.id.to_string(),
            title: milestone.title,
            description: milestone.description,
            due_date: milestone.due_on,
            repo: format!("repo-{}", milestone.repo_id), // TODO: Join with repo table
            open_issues: milestone.open_issues,
            closed_issues: milestone.closed_issues,
            state: milestone.state,
        });
    }
    
    let mut result: Vec<CycleGroup> = cycles.into_values().collect();
    result.sort_by(|a, b| a.due_date.cmp(&b.due_date));
    
    Ok(result)
}

#[derive(serde::Serialize)]
pub struct CycleGroup {
    pub title: String,
    pub due_date: Option<String>,
    pub milestones: Vec<MilestoneInfo>,
    pub total_open: i32,
    pub total_closed: i32,
}

#[derive(serde::Serialize)]
pub struct MilestoneInfo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub repo: String,
    pub open_issues: i32,
    pub closed_issues: i32,
    pub state: String,
}
