//! Library exports for MADE Activity Tracker
//!
//! This module exposes the internal modules for testing and potential library usage.

use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

pub mod ai;
pub mod db;
pub mod embeddings;
pub mod github;
pub mod metrics;
pub mod project;
pub mod search;
pub mod team;

/// AI-specific application state
pub struct AiState {
    pub amplifier_client: Arc<TokioMutex<ai::AmplifierClient>>,
}
