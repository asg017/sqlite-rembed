/// Mock provider for testing in CI environments
/// Returns deterministic embeddings without making real API calls

use sqlite_loadable::{Error, Result};

/// Generate a mock embedding for testing
pub fn generate_mock_embedding(text: &str, dimensions: usize) -> Result<Vec<f32>> {
    // Generate deterministic values based on text hash
    let hash = simple_hash(text);
    let mut embedding = Vec::with_capacity(dimensions);

    for i in 0..dimensions {
        // Generate pseudo-random but deterministic values
        let value = ((hash + i as u32) as f32 / u32::MAX as f32) * 2.0 - 1.0;
        embedding.push(value);
    }

    Ok(embedding)
}

/// Simple hash function for deterministic output
fn simple_hash(text: &str) -> u32 {
    text.bytes().fold(0u32, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(b as u32)
    })
}

/// Check if mock mode is enabled via environment variable
pub fn is_mock_mode() -> bool {
    std::env::var("MOCK_EMBEDDINGS").unwrap_or_default() == "true"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_embedding_deterministic() {
        let text = "hello world";
        let embedding1 = generate_mock_embedding(text, 10).unwrap();
        let embedding2 = generate_mock_embedding(text, 10).unwrap();
        assert_eq!(embedding1, embedding2);
    }

    #[test]
    fn test_mock_embedding_different_texts() {
        let embedding1 = generate_mock_embedding("hello", 10).unwrap();
        let embedding2 = generate_mock_embedding("world", 10).unwrap();
        assert_ne!(embedding1, embedding2);
    }

    #[test]
    fn test_mock_embedding_dimensions() {
        let embedding = generate_mock_embedding("test", 1536).unwrap();
        assert_eq!(embedding.len(), 1536);
    }
}