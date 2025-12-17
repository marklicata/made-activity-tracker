use crate::db::project_queries::*;
use crate::db::AppState;
use tauri::State;

/// Get timeline events for a project
#[tauri::command]
pub async fn get_project_timeline(
    repo_id: i64,
    start_date: Option<String>,
    end_date: Option<String>,
    event_types: Option<Vec<String>>,
    user_id: Option<i64>,
    limit: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<TimelineEvent>, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    let limit = limit.unwrap_or(1000); // Default to 1000 events (from spec)

    crate::db::project_queries::get_timeline_events(
        &conn,
        repo_id,
        start_date.as_deref(),
        end_date.as_deref(),
        event_types.as_ref(),
        user_id,
        limit,
    )
    .map_err(|e| e.to_string())
}

/// Get contributor statistics for a project
#[tauri::command]
pub async fn get_project_contributors(
    repo_id: i64,
    start_date: Option<String>,
    end_date: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<ContributorStats>, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    crate::db::project_queries::get_contributor_stats(
        &conn,
        repo_id,
        start_date.as_deref(),
        end_date.as_deref(),
    )
    .map_err(|e| e.to_string())
}

/// Get activity heatmap data for a project
#[tauri::command]
pub async fn get_project_activity_heatmap(
    repo_id: i64,
    start_date: Option<String>,
    end_date: Option<String>,
    state: State<'_, AppState>,
) -> Result<ActivityHeatmapData, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    crate::db::project_queries::get_activity_heatmap(
        &conn,
        repo_id,
        start_date.as_deref(),
        end_date.as_deref(),
    )
    .map_err(|e| e.to_string())
}

/// Get lifecycle metrics for a project
#[tauri::command]
pub async fn get_project_lifecycle_metrics(
    repo_id: i64,
    start_date: Option<String>,
    end_date: Option<String>,
    state: State<'_, AppState>,
) -> Result<LifecycleMetrics, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    crate::db::project_queries::get_lifecycle_metrics(
        &conn,
        repo_id,
        start_date.as_deref(),
        end_date.as_deref(),
    )
    .map_err(|e| e.to_string())
}

/// Get project summary statistics
#[tauri::command]
pub async fn get_project_summary(
    repo_id: i64,
    start_date: Option<String>,
    end_date: Option<String>,
    state: State<'_, AppState>,
) -> Result<ProjectSummary, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    crate::db::project_queries::get_project_summary(
        &conn,
        repo_id,
        start_date.as_deref(),
        end_date.as_deref(),
    )
    .map_err(|e| e.to_string())
}
