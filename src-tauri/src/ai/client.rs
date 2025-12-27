use crate::ai::types::*;
use anyhow::Result;
use reqwest;
use std::time::Duration;

pub struct AmplifierClient {
    base_url: String,
    auth_token: String,
    client: reqwest::Client,
}

impl AmplifierClient {
    pub fn new(port: u16, auth_token: String) -> Self {
        let base_url = format!("http://127.0.0.1:{}", port);
        tracing::info!("Creating Amplifier client for {}", base_url);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url,
            auth_token,
            client,
        }
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        tracing::info!("Performing health check: GET {}", url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("✗ Health check request failed: {}", e);
                tracing::error!("  URL: {}", url);
                tracing::error!("  Error details: {:?}", e);
                if e.is_connect() {
                    tracing::error!("  → Connection error: Unable to reach server at {}", self.base_url);
                } else if e.is_timeout() {
                    tracing::error!("  → Timeout error: Server did not respond within 60 seconds");
                }
                e
            })?;

        let status = response.status();
        tracing::info!("Health check response status: {}", status);

        if status.is_success() {
            tracing::info!("✓ Health check passed");
            Ok(true)
        } else {
            tracing::warn!("✗ Health check failed with status: {}", status);
            Ok(false)
        }
    }

    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/chat", self.base_url);
        tracing::info!("Sending chat message to: POST {}", url);
        tracing::debug!("  Message preview: {}...",
            request.message.chars().take(50).collect::<String>());

        let response = self.client
            .post(&url)
            .header("X-Auth-Token", &self.auth_token)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("✗ Chat request failed: {}", e);
                tracing::error!("  URL: {}", url);
                if e.is_connect() {
                    tracing::error!("  → Connection error: Unable to reach server");
                } else if e.is_timeout() {
                    tracing::error!("  → Timeout error: Server did not respond within 60 seconds");
                }
                e
            })?;

        let status = response.status();
        tracing::debug!("Chat response status: {}", status);

        if !status.is_success() {
            let error = response.text().await?;
            tracing::error!("✗ Chat failed with status {}: {}", status, error);
            return Err(anyhow::anyhow!("Chat failed: {}", error));
        }

        let result: ChatResponse = response.json().await?;
        tracing::info!("✓ Chat response received");
        Ok(result)
    }
}
