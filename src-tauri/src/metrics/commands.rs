use super::calculator::{calculate_dashboard_metrics, DashboardMetrics};
use crate::db::AppState;
use chrono::{Duration, Utc};
use tauri::State;

/// Get metrics for the main dashboard
#[tauri::command]
pub async fn get_dashboard_metrics(
    state: State<'_, AppState>,
) -> Result<DashboardMetrics, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;
    
    // Default to 90 days of history
    let since = (Utc::now() - Duration::days(90))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    
    // TODO: Load these from config
    let excluded_bots = vec![
        "dependabot[bot]".to_string(),
        "renovate[bot]".to_string(),
    ];
    let bug_labels = vec!["bug".to_string(), "defect".to_string()];
    
    let issues = crate::db::queries::get_issues_for_metrics(&conn, &since, &excluded_bots)
        .map_err(|e| e.to_string())?;
    
    let prs = crate::db::queries::get_prs_for_metrics(&conn, &since, &excluded_bots)
        .map_err(|e| e.to_string())?;
    
    let metrics = calculate_dashboard_metrics(&issues, &prs, &bug_labels, 90);
    
    Ok(metrics)
}

/// Get metrics for a specific user
#[tauri::command]
pub async fn get_user_metrics(
    _username: String,
    state: State<'_, AppState>,
) -> Result<DashboardMetrics, String> {
    // TODO: Implement user-specific filtering
    get_dashboard_metrics(state).await
}

/// Get metrics for a specific squad
#[tauri::command]
pub async fn get_squad_metrics(
    _squad_id: String,
    state: State<'_, AppState>,
) -> Result<DashboardMetrics, String> {
    // TODO: Implement squad-specific filtering
    get_dashboard_metrics(state).await
}
