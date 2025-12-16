use anyhow::{anyhow, Result};
use keyring::Entry;
use serde::{Deserialize, Serialize};

const SERVICE_NAME: &str = "made-activity-tracker";
const ACCOUNT_NAME: &str = "github-token";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub user: GitHubUser,
    pub access_token: String,
}

/// Device Flow URLs for GitHub OAuth
const DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const ACCESS_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const USER_API_URL: &str = "https://api.github.com/user";

/// Store the access token securely in the system keychain
pub fn store_token(token: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME)?;
    entry.set_password(token)?;
    Ok(())
}

/// Retrieve the access token from the system keychain
pub fn get_token() -> Result<Option<String>> {
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME)?;
    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(anyhow!("Failed to get token: {}", e)),
    }
}

/// Delete the access token from the system keychain
pub fn delete_token() -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME)?;
    match entry.delete_password() { // Fixed: Changed from delete_credential() to delete_password()
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(anyhow!("Failed to delete token: {}", e)),
    }
}

/// Initiate GitHub Device Flow authentication
pub async fn initiate_device_flow(client_id: &str) -> Result<DeviceFlowResponse> {
    let client = reqwest::Client::new();
    
    let response = client
        .post(DEVICE_CODE_URL)
        .header("Accept", "application/json")
        .form(&[("client_id", client_id), ("scope", "repo read:user")])
        .send()
        .await?
        .json::<DeviceFlowResponse>()
        .await?;
    
    Ok(response)
}

#[derive(Debug, Deserialize)]
pub struct DeviceFlowResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: i32,
    pub interval: i32,
}

/// Poll for access token after user authorizes
pub async fn poll_for_token(client_id: &str, device_code: &str, interval: u64) -> Result<String> {
    let client = reqwest::Client::new();
    
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
        
        let response = client
            .post(ACCESS_TOKEN_URL)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", client_id),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await?
            .json::<TokenPollResponse>()
            .await?;
        
        if let Some(token) = response.access_token {
            return Ok(token);
        }
        
        match response.error.as_deref() {
            Some("authorization_pending") => continue,
            Some("slow_down") => {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
            Some("expired_token") => return Err(anyhow!("Device code expired")),
            Some("access_denied") => return Err(anyhow!("Access denied by user")),
            Some(error) => return Err(anyhow!("OAuth error: {}", error)),
            None => continue,
        }
    }
}

#[derive(Debug, Deserialize)]
struct TokenPollResponse {
    access_token: Option<String>,
    error: Option<String>,
}

/// Fetch the authenticated user's profile
pub async fn get_authenticated_user(access_token: &str) -> Result<GitHubUser> {
    let client = reqwest::Client::new();
    
    let user = client
        .get(USER_API_URL)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "MADE-Activity-Tracker")
        .send()
        .await?
        .json::<GitHubUser>()
        .await?;
    
    Ok(user)
}
