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
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url: format!("http://127.0.0.1:{}", port),
            auth_token,
            client,
        }
    }

    pub async fn health_check(&self) -> Result<bool> {
        let response = self.client
            .get(&format!("{}/health", self.base_url))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let response = self.client
            .post(&format!("{}/chat", self.base_url))
            .header("X-Auth-Token", &self.auth_token)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Chat failed: {}", error));
        }

        let result: ChatResponse = response.json().await?;
        Ok(result)
    }
}
