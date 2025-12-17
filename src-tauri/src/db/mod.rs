pub mod migrations;
pub mod models;
pub mod queries;
pub mod project_queries;

use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager}; // Added Manager import

/// Application state holding database connections
pub struct AppState {
    pub sqlite: Mutex<Connection>,
    pub lancedb_path: PathBuf, // Kept for future use
}

/// Initialize SQLite database
pub async fn init_databases(app: &AppHandle) -> Result<()> {
    let app_dir = app
        .path_resolver()
        .app_data_dir()
        .expect("Failed to get app data directory");
    
    std::fs::create_dir_all(&app_dir)?;
    
    // Initialize SQLite
    let sqlite_path = app_dir.join("made.db");
    let conn = Connection::open(&sqlite_path)?;
    migrations::run_migrations(&conn)?;
    
    // LanceDB path for future use (Phase 3)
    let lancedb_path = app_dir.join("vectors");
    std::fs::create_dir_all(&lancedb_path)?;
    
    // Store in app state
    let state = AppState {
        sqlite: Mutex::new(conn),
        lancedb_path,
    };
    
    app.manage(state);
    
    tracing::info!("Databases initialized at {:?}", app_dir);
    Ok(())
}
