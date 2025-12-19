use crate::ai::{AmplifierClient, ChatRequest, ChatResponse};
use crate::AiState;
use serde::{Deserialize, Serialize};
use tauri::State;
use std::env;

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
    let client = state.amplifier_client.lock().await;

    client
        .chat(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_amplifier_health(
    state: State<'_, AiState>,
) -> Result<bool, String> {
    let client = state.amplifier_client.lock().await;

    client
        .health_check()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_api_keys() -> ApiKeyStatus {
    ApiKeyStatus {
        has_anthropic: env::var("ANTHROPIC_API_KEY").is_ok(),
        has_openai: env::var("OPENAI_API_KEY").is_ok(),
    }
}

#[tauri::command]
pub fn set_api_key(provider: String, key: String) -> Result<(), String> {
    match provider.as_str() {
        "anthropic" => {
            env::set_var("ANTHROPIC_API_KEY", key);
            Ok(())
        }
        "openai" => {
            env::set_var("OPENAI_API_KEY", key);
            Ok(())
        }
        _ => Err(format!("Unknown provider: {}", provider)),
    }
}
