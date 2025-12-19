pub mod commands;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub repositories: Vec<RepositoryConfig>,
    pub squads: Vec<SquadConfig>,
    /// Preferred shape for tracked users with metadata
    #[serde(default)]
    pub users: Vec<UserTrackedConfig>,
    /// Legacy shape kept for backward compatibility
    #[serde(default)]
    pub tracked_users: Vec<UserConfig>,
    pub history_days: i32,
    pub excluded_bots: Vec<String>,
    pub bug_labels: Vec<String>,
    pub feature_labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub owner: String,
    pub name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquadConfig {
    pub id: String,
    pub name: String,
    pub members: Vec<String>,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub username: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTrackedConfig {
    pub username: String,
    #[serde(default)]
    pub tracked: bool,
    #[serde(default)]
    pub tracked_at: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            repositories: vec![],
            squads: vec![],
            users: vec![],
            tracked_users: vec![],
            history_days: 90,
            excluded_bots: vec![
                "dependabot[bot]".to_string(),
                "dependabot-preview[bot]".to_string(),
                "renovate[bot]".to_string(),
                "github-actions[bot]".to_string(),
                "codecov[bot]".to_string(),
            ],
            bug_labels: vec!["bug".to_string(), "defect".to_string(), "fix".to_string()],
            feature_labels: vec![
                "feature".to_string(),
                "enhancement".to_string(),
                "feat".to_string(),
            ],
        }
    }
}
