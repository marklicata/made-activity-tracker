use crate::ai::{AmplifierClient, ChatRequest, ChatResponse};
use crate::AiState;
use tauri::State;

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
