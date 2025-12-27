use crate::ai::{AmplifierClient, ChatRequest, ChatResponse};
use crate::AiState;
use serde::{Deserialize, Serialize};
use tauri::State;
use std::env;
use std::ops::Deref;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyStatus {
    pub has_anthropic: bool,
    pub has_openai: bool,
}

#[tauri::command]
pub async fn send_chat_message(
    state: State<'_, AiState>,
    request: ChatRequest,
) -> Result<ChatResponse, String> {
    tracing::info!("[Command] send_chat_message invoked");
    tracing::debug!("  Message: {}...", request.message.chars().take(50).collect::<String>());

    tracing::debug!("  Acquiring client lock...");
    let client_guard = state.amplifier_client.lock().await;
    tracing::debug!("  ✓ Client lock acquired");

    let result = client_guard.chat(request).await;
    match &result {
        Ok(_) => tracing::info!("[Command] ✓ send_chat_message completed successfully"),
        Err(e) => tracing::error!("[Command] ✗ send_chat_message failed: {}", e),
    }
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_amplifier_health(
    state: State<'_, AiState>,
) -> Result<bool, String> {
    tracing::info!("[Command] check_amplifier_health invoked");

    tracing::debug!("  Acquiring client lock...");
    let client_guard = state.amplifier_client.lock().await;
    tracing::debug!("  ✓ Client lock acquired");

    let result = client_guard.health_check().await;
    match &result {
        Ok(true) => tracing::info!("[Command] ✓ Health check passed"),
        Ok(false) => tracing::warn!("[Command] ⚠ Health check returned false"),
        Err(e) => tracing::error!("[Command] ✗ Health check error: {}", e),
    }
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_api_keys() -> ApiKeyStatus {
    tracing::info!("[Command] check_api_keys invoked");
    let has_anthropic = env::var("ANTHROPIC_API_KEY").is_ok();
    let has_openai = env::var("OPENAI_API_KEY").is_ok();

    tracing::info!("  ANTHROPIC_API_KEY: {}", if has_anthropic { "✓ Set" } else { "✗ Not set" });
    tracing::info!("  OPENAI_API_KEY: {}", if has_openai { "✓ Set" } else { "✗ Not set" });

    ApiKeyStatus {
        has_anthropic,
        has_openai,
    }
}

#[tauri::command]
pub fn set_api_key(provider: String, key: String) -> Result<(), String> {
    tracing::info!("[Command] set_api_key invoked for provider: {}", provider);

    match provider.as_str() {
        "anthropic" => {
            tracing::info!("  Setting ANTHROPIC_API_KEY");
            env::set_var("ANTHROPIC_API_KEY", key);
            tracing::info!("  ✓ ANTHROPIC_API_KEY set successfully");
            Ok(())
        }
        "openai" => {
            tracing::info!("  Setting OPENAI_API_KEY");
            env::set_var("OPENAI_API_KEY", key);
            tracing::info!("  ✓ OPENAI_API_KEY set successfully");
            Ok(())
        }
        _ => {
            tracing::error!("  ✗ Unknown provider: {}", provider);
            Err(format!("Unknown provider: {}", provider))
        }
    }
}
