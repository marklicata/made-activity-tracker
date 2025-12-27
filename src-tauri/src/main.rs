#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use made_activity_tracker::*;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex as TokioMutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // Load API keys from project keys.env file if it exists
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let keys_env_path = manifest_dir.parent().unwrap().join("keys.env");

    if keys_env_path.exists() {
        if let Err(e) = dotenvy::from_path(&keys_env_path) {
            eprintln!("Warning: Failed to load API keys from {:?}: {}", keys_env_path, e);
        } else {
            println!("Loaded API keys from {:?}", keys_env_path);
        }
    } else {
        eprintln!("Warning: keys.env file not found at {:?}", keys_env_path);
        eprintln!("AI chat features may not work without API keys.");
    }

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
            tracing::info!("=== Initializing AI Features ===");
            let amplifier_client = tauri::async_runtime::block_on(async {
                // Get database path
                tracing::info!("Getting database path for Amplifier...");
                let db_path = match db::get_db_path(&app_handle) {
                    Ok(path) => {
                        tracing::info!("✓ Database path: {:?}", path);
                        path
                    }
                    Err(e) => {
                        tracing::error!("✗ Failed to get database path: {}", e);
                        tracing::error!("AI features will be disabled");
                        return None;
                    }
                };

                // Start sidecar
                tracing::info!("Starting Amplifier sidecar process...");
                let mut sidecar = ai::AmplifierSidecar::new();
                match sidecar.start(db_path) {
                    Ok(()) => {
                        tracing::info!("✓ Amplifier sidecar started successfully on port {}", sidecar.port);

                        // Create client
                        tracing::info!("Creating Amplifier HTTP client...");
                        let client = ai::AmplifierClient::new(sidecar.port, sidecar.auth_token.clone());

                        // Health check
                        tracing::info!("Running initial health check...");
                        match client.health_check().await {
                            Ok(true) => {
                                tracing::info!("✓ Amplifier health check passed - AI features ready");
                            }
                            Ok(false) => {
                                tracing::warn!("⚠ Amplifier health check returned false - server may not be ready");
                            }
                            Err(e) => {
                                tracing::error!("✗ Amplifier health check error: {}", e);
                                tracing::error!("AI features may not work correctly");
                            }
                        }

                        // Keep sidecar alive by moving it into app state
                        tracing::debug!("Registering sidecar in app state...");
                        app.manage(sidecar);

                        Some(client)
                    }
                    Err(e) => {
                        tracing::error!("✗ Failed to start Amplifier sidecar: {}", e);
                        tracing::warn!("AI features will be disabled");
                        None
                    }
                }
            });

            // Create and manage AI state if we have a client
            if let Some(client) = amplifier_client {
                tracing::info!("Creating AI state and registering commands...");
                let ai_state = AiState {
                    amplifier_client: Arc::new(TokioMutex::new(client)),
                };
                app.manage(ai_state);
                tracing::info!("✓ AI features initialized successfully");
            } else {
                tracing::warn!("⚠ AI features are not available - client not created");
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

            // Database CRUD commands
            db::commands::get_settings,
            db::commands::update_settings,
            db::commands::add_repository,
            db::commands::remove_repository,
            db::commands::toggle_repository,
            db::commands::add_squad,
            db::commands::update_squad,
            db::commands::remove_squad,
            db::commands::get_all_squads_command,
            db::commands::toggle_user_tracked,
            db::commands::fix_invalid_users,

            // Query helper commands
            db::commands::get_sync_stats,
            db::commands::get_all_users,
            db::commands::get_all_repositories,

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
            team::commands::update_user_tracked_status,
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
