use super::AppConfig;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

fn get_config_path(app: &AppHandle) -> PathBuf {
    app.path_resolver()
        .app_config_dir()
        .expect("Failed to get config directory")
        .join("config.json")
}

/// Load configuration from disk (internal use, returns Result)
pub fn load_config_internal(app: &AppHandle) -> Result<AppConfig> {
    let config_path = get_config_path(app);
    
    if !config_path.exists() {
        return Ok(AppConfig::default());
    }
    
    let contents = fs::read_to_string(&config_path)?;
    let config: AppConfig = serde_json::from_str(&contents)?;
    
    Ok(config)
}

/// Load configuration from disk
#[tauri::command]
pub async fn load_config(app: AppHandle) -> Result<AppConfig, String> {
    load_config_internal(&app).map_err(|e| e.to_string())
}

/// Save configuration to disk
#[tauri::command]
pub async fn save_config(config: AppConfig, app: AppHandle) -> Result<(), String> {
    let config_path = get_config_path(&app);
    
    // Ensure directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    
    let contents = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&config_path, contents).map_err(|e| e.to_string())?;
    
    tracing::info!("Config saved to {:?}", config_path);
    Ok(())
}

/// Get sync statistics
#[tauri::command]
pub async fn get_sync_stats(state: tauri::State<'_, crate::db::AppState>) -> Result<crate::db::queries::SyncStats, String> {
    let conn = state.sqlite.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_sync_stats(&conn).map_err(|e| e.to_string())
}
