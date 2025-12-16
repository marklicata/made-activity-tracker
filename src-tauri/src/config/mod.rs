pub mod commands;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub repositories: Vec<RepositoryConfig>,
    pub squads: Vec<SquadConfig>,
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

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            repositories: vec![],
            squads: vec![],
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
