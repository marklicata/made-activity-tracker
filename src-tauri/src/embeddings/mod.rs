pub mod generator;

use anyhow::Result;

/// Stub implementation for Phase 1
/// Phase 3 will implement actual FastEmbed integration

/// Generate embeddings for a list of texts (STUBBED)
pub fn generate_embeddings(texts: &[String]) -> Result<Vec<Vec<f32>>> {
    // Stub: Return empty 384-dimensional vectors for Phase 1
    // Real implementation in Phase 3 will use FastEmbed
    if texts.is_empty() {
        return Ok(vec![]);
    }
    
    Ok(texts.iter().map(|_| vec![0.0; 384]).collect())
}

/// Generate a single embedding (STUBBED)
pub fn generate_embedding(text: &str) -> Result<Vec<f32>> {
    let _ = text; // Prevent unused warning
    // Stub: Return empty 384-dimensional vector
    Ok(vec![0.0; 384])
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
    }

    #[test]
    fn test_empty_batch() {
        let embeddings = generate_embeddings(&[]).unwrap();
        assert_eq!(embeddings.len(), 0);
    }
}
