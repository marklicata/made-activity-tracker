#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod config;
mod db;
mod embeddings;
mod github;
mod metrics;
mod project;
mod search;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Log version info at startup
    tracing::info!("Made Activity Tracker v{} starting...", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();
            
            // Initialize databases on startup
            tauri::async_runtime::block_on(async {
                if let Err(e) = db::init_databases(&app_handle).await {
                    tracing::error!("Failed to initialize databases: {}", e);
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Auth commands
            github::commands::github_login,
            github::commands::github_logout,
            github::commands::check_auth,
            
            // Sync commands
            github::commands::sync_github_data,
            github::commands::sync_repository,
            
            // Config commands
            config::commands::load_config,
            config::commands::save_config,
            config::commands::get_sync_stats,
            config::commands::get_all_users,
            config::commands::get_all_repositories,

            // Metrics commands
            metrics::commands::get_dashboard_metrics,
            metrics::commands::get_dashboard_metrics_filtered,
            metrics::commands::get_metrics_timeseries,
            metrics::commands::get_user_metrics,
            metrics::commands::get_squad_metrics,
            
            // Search commands
            search::commands::hybrid_search,
            search::commands::find_duplicates,
            
            // Roadmap commands
            github::commands::get_roadmap,

            // Project deep dive commands
            project::commands::get_project_timeline,
            project::commands::get_project_contributors,
            project::commands::get_project_activity_heatmap,
            project::commands::get_project_lifecycle_metrics,
            project::commands::get_project_summary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
