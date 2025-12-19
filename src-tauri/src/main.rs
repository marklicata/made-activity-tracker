#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod ai;
mod config;
mod db;
mod embeddings;
mod github;
mod metrics;
mod project;
mod search;
mod team;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex as TokioMutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// AI-specific application state
pub struct AiState {
    pub amplifier_client: Arc<TokioMutex<ai::AmplifierClient>>,
}

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

            // Initialize Amplifier sidecar
            let amplifier_client = tauri::async_runtime::block_on(async {
                // Get database path
                let db_path = db::get_db_path(&app_handle)
                    .expect("Failed to get database path");

                // Start sidecar
                let mut sidecar = ai::AmplifierSidecar::new();
                match sidecar.start(db_path) {
                    Ok(()) => {
                        tracing::info!("Amplifier sidecar started on port {}", sidecar.port);

                        // Create client
                        let client = ai::AmplifierClient::new(sidecar.port, sidecar.auth_token.clone());

                        // Health check
                        match client.health_check().await {
                            Ok(true) => {
                                tracing::info!("Amplifier health check passed");
                            }
                            Ok(false) => {
                                tracing::warn!("Amplifier health check failed");
                            }
                            Err(e) => {
                                tracing::error!("Amplifier health check error: {}", e);
                            }
                        }

                        // Keep sidecar alive by moving it into app state
                        app.manage(sidecar);

                        Some(client)
                    }
                    Err(e) => {
                        tracing::error!("Failed to start Amplifier sidecar: {}", e);
                        tracing::warn!("AI features will be disabled");
                        None
                    }
                }
            });

            // Create and manage AI state if we have a client
            if let Some(client) = amplifier_client {
                let ai_state = AiState {
                    amplifier_client: Arc::new(TokioMutex::new(client)),
                };
                app.manage(ai_state);
            }

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
            metrics::commands::get_pr_based_metrics,
            
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

            // Team/user-centric commands
            team::commands::add_tracked_user,
            team::commands::remove_tracked_user,
            team::commands::get_tracked_users,
            team::commands::get_user_summary,
            team::commands::get_user_activity_timeline,
            team::commands::get_user_repository_distribution,
            team::commands::get_team_collaboration_matrix,
            team::commands::get_user_activity_trend,
            team::commands::get_user_focus_metrics,

            // AI commands
            ai::commands::send_chat_message,
            ai::commands::check_amplifier_health,
            ai::commands::check_api_keys,
            ai::commands::set_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
