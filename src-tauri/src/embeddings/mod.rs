pub mod generator;

use anyhow::{Context, Result};
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use std::sync::Mutex;

/// Global embedding model instance (lazy-initialized)
static EMBEDDING_MODEL: Mutex<Option<TextEmbedding>> = Mutex::new(None);

/// Initialize or get the embedding model
fn get_model() -> Result<()> {
    let mut model_lock = EMBEDDING_MODEL.lock().unwrap();

    if model_lock.is_none() {
        tracing::info!("Initializing FastEmbed model (all-MiniLM-L6-v2)...");
        let options = InitOptions::new(EmbeddingModel::AllMiniLML6V2)
            .with_show_download_progress(true);

        let model = TextEmbedding::try_new(options)
            .context("Failed to initialize FastEmbed model. Please check your internet connection for first-time model download.")?;
        *model_lock = Some(model);
    }

    Ok(())
}

/// Generate embeddings for a list of texts using FastEmbed
pub fn generate_embeddings(texts: &[String]) -> Result<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Ok(vec![]);
    }

    let start = std::time::Instant::now();
    tracing::info!("Generating embeddings for {} texts", texts.len());

    // Ensure model is initialized
    get_model()?;

    // Access model from Mutex
    let model_lock = EMBEDDING_MODEL.lock().unwrap();
    let model = model_lock.as_ref().unwrap(); // Safe because get_model() succeeded

    let embeddings = model.embed(texts.to_vec(), None)
        .context("Failed to generate embeddings")?;

    tracing::info!("Generated {} embeddings in {:?}", embeddings.len(), start.elapsed());

    Ok(embeddings)
}

/// Generate a single embedding for a text string
pub fn generate_embedding(text: &str) -> Result<Vec<f32>> {
    let embeddings = generate_embeddings(&[text.to_string()])?;
    embeddings.into_iter().next()
        .ok_or_else(|| anyhow::anyhow!("Failed to generate embedding for text"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_generation() {
        let text = "This is a test issue about user authentication";
        let embedding = generate_embedding(text).unwrap();

        // MiniLM-L6-v2 produces 384-dimensional embeddings
        assert_eq!(embedding.len(), 384);

        // Verify it's not all zeros (real embedding)
        let has_nonzero = embedding.iter().any(|&v| v != 0.0);
        assert!(has_nonzero, "Embedding should contain non-zero values");
    }

    #[test]
    fn test_batch_embeddings() {
        let texts = vec![
            "Issue about login".to_string(),
            "PR for database migration".to_string(),
        ];
        let embeddings = generate_embeddings(&texts).unwrap();

        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 384);
        assert_eq!(embeddings[1].len(), 384);

        // Verify real embeddings (not all zeros)
        let has_nonzero_0 = embeddings[0].iter().any(|&v| v != 0.0);
        let has_nonzero_1 = embeddings[1].iter().any(|&v| v != 0.0);
        assert!(has_nonzero_0 && has_nonzero_1, "Embeddings should contain non-zero values");
    }

    #[test]
    fn test_empty_batch() {
        let embeddings = generate_embeddings(&[]).unwrap();
        assert_eq!(embeddings.len(), 0);
    }
}
