use super::calculator::{calculate_dashboard_metrics, DashboardMetrics};
use super::filter_params::MetricsFilters;
use crate::db::AppState;
use crate::db::metrics_queries;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
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

/// Get metrics with filters applied
#[tauri::command]
pub async fn get_dashboard_metrics_filtered(
    filters: MetricsFilters,
    state: State<'_, AppState>,
) -> Result<DashboardMetrics, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    // Determine date range
    let (since, until) = if let Some(range) = filters.date_range {
        (range.start, Some(range.end))
    } else {
        // Default to 90 days
        let since = (Utc::now() - Duration::days(90))
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        (since, None)
    };

    // TODO: Load from config
    let excluded_bots = vec![
        "dependabot[bot]".to_string(),
        "renovate[bot]".to_string(),
    ];
    let bug_labels = vec!["bug".to_string(), "defect".to_string()];

    // Get squad member IDs if squad filter is set
    let squad_member_ids = if let Some(ref squad_id) = filters.squad_id {
        Some(crate::db::queries::get_squad_member_ids(&conn, squad_id)
            .map_err(|e| e.to_string())?)
    } else {
        None
    };

    // Fetch filtered data
    let issues = crate::db::queries::get_issues_for_metrics_filtered(
        &conn,
        &since,
        until.as_deref(),
        &excluded_bots,
        filters.repository_ids.as_deref(),
        filters.user_id,
        squad_member_ids.as_deref(),
    ).map_err(|e| e.to_string())?;

    let prs = crate::db::queries::get_prs_for_metrics_filtered(
        &conn,
        &since,
        until.as_deref(),
        &excluded_bots,
        filters.repository_ids.as_deref(),
        filters.user_id,
        squad_member_ids.as_deref(),
    ).map_err(|e| e.to_string())?;

    // Calculate days in period
    // TODO: Calculate actual days from date range instead of hardcoding
    let days_in_period = 90;

    let metrics = calculate_dashboard_metrics(&issues, &prs, &bug_labels, days_in_period);

    Ok(metrics)
}

// Timeseries data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeseriesDataPoint {
    pub date: String,
    pub speed: super::calculator::SpeedMetrics,
    pub ease: super::calculator::EaseMetrics,
    pub quality: super::calculator::QualityMetrics,
}

/// Get timeseries data for charts
#[tauri::command]
pub async fn get_metrics_timeseries(
    filters: MetricsFilters,
    granularity: String,
    state: State<'_, AppState>,
) -> Result<Vec<TimeseriesDataPoint>, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    // Determine date range
    let (start_date, end_date) = if let Some(range) = filters.date_range {
        (range.start, range.end)
    } else {
        let end = Utc::now();
        let start = end - Duration::days(90);
        (
            start.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            end.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        )
    };

    // Generate date buckets based on granularity
    let date_buckets = generate_date_buckets(&start_date, &end_date, &granularity);

    let excluded_bots = vec!["dependabot[bot]".to_string(), "renovate[bot]".to_string()];
    let bug_labels = vec!["bug".to_string(), "defect".to_string()];

    let squad_member_ids = if let Some(ref squad_id) = filters.squad_id {
        Some(crate::db::queries::get_squad_member_ids(&conn, squad_id)
            .map_err(|e| e.to_string())?)
    } else {
        None
    };

    let mut timeseries = Vec::new();

    for (bucket_start, bucket_end) in date_buckets {
        let issues = crate::db::queries::get_issues_for_metrics_filtered(
            &conn,
            &bucket_start,
            Some(&bucket_end),
            &excluded_bots,
            filters.repository_ids.as_deref(),
            filters.user_id,
            squad_member_ids.as_deref(),
        ).map_err(|e| e.to_string())?;

        let prs = crate::db::queries::get_prs_for_metrics_filtered(
            &conn,
            &bucket_start,
            Some(&bucket_end),
            &excluded_bots,
            filters.repository_ids.as_deref(),
            filters.user_id,
            squad_member_ids.as_deref(),
        ).map_err(|e| e.to_string())?;

        let days = 7; // Simplified for weekly
        let metrics = calculate_dashboard_metrics(&issues, &prs, &bug_labels, days);

        timeseries.push(TimeseriesDataPoint {
            date: bucket_start[..10].to_string(),
            speed: metrics.speed,
            ease: metrics.ease,
            quality: metrics.quality,
        });
    }

    Ok(timeseries)
}

fn generate_date_buckets(start: &str, end: &str, granularity: &str) -> Vec<(String, String)> {
    let start_dt = DateTime::parse_from_rfc3339(start).unwrap().with_timezone(&Utc);
    let end_dt = DateTime::parse_from_rfc3339(end).unwrap().with_timezone(&Utc);

    let mut buckets = Vec::new();
    let mut current = start_dt;

    let step = match granularity {
        "daily" => Duration::days(1),
        "monthly" => Duration::days(30),
        _ => Duration::days(7), // weekly default
    };

    while current < end_dt {
        let next = (current + step).min(end_dt);
        buckets.push((
            current.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            next.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        ));
        current = next;
    }

    buckets
}

/// Get PR-based dashboard metrics (Amplifier-style)
/// This uses PR activity instead of commit data for Speed/Ease/Quality metrics
#[tauri::command]
pub async fn get_pr_based_metrics(
    days: Option<i32>,
    state: State<'_, AppState>,
) -> Result<metrics_queries::DashboardMetrics, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;

    let days = days.unwrap_or(30); // Default to 30 days

    metrics_queries::get_dashboard_metrics(&conn, days)
        .map_err(|e| e.to_string())
}
