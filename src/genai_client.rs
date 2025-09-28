use genai::Client as GenAiClient;
use once_cell::sync::Lazy;
use sqlite_loadable::{Error, Result};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Global tokio runtime for async operations
static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create tokio runtime")
});

/// Unified client using genai for all providers
#[derive(Clone)]
pub struct EmbeddingClient {
    /// The genai client instance
    client: Arc<GenAiClient>,
    /// Model identifier (can include provider prefix like "openai::text-embedding-3-small")
    model: String,
}

impl EmbeddingClient {
    /// Create a new embedding client for the specified model
    pub fn new(model: String, api_key: Option<String>) -> Result<Self> {
        // If an API key is provided, set it as an environment variable
        // This is a workaround since genai reads from env vars
        if let Some(key) = api_key {
            // Detect provider from model name and set appropriate env var
            let provider = if let Some(idx) = model.find("::") {
                &model[..idx]
            } else {
                // Default to openai for backward compatibility
                "openai"
            };

            match provider {
                "openai" => std::env::set_var("OPENAI_API_KEY", &key),
                "gemini" => std::env::set_var("GEMINI_API_KEY", &key),
                "google" => std::env::set_var("GEMINI_API_KEY", &key), // Google uses GEMINI_API_KEY
                "cohere" => std::env::set_var("CO_API_KEY", &key),
                "anthropic" => std::env::set_var("ANTHROPIC_API_KEY", &key),
                "groq" => std::env::set_var("GROQ_API_KEY", &key),
                "deepseek" => std::env::set_var("DEEPSEEK_API_KEY", &key),
                "xai" => std::env::set_var("XAI_API_KEY", &key),
                "mistral" => std::env::set_var("MISTRAL_API_KEY", &key),
                // For unknown providers, try setting a generic pattern
                _ => std::env::set_var(&format!("{}_API_KEY", provider.to_uppercase()), &key),
            }
        }

        let client = GenAiClient::default();

        Ok(Self {
            client: Arc::new(client),
            model,
        })
    }

    /// Generate embeddings for a single text synchronously
    pub fn embed_sync(&self, text: &str) -> Result<Vec<f32>> {
        let client = self.client.clone();
        let model = self.model.clone();
        let text = text.to_string();

        // Run async operation in the runtime
        RUNTIME.block_on(async move {
            client
                .embed(&model, text, None)
                .await
                .map_err(|e| Error::new_message(format!("Embedding failed: {}", e)))
                .and_then(|response| {
                    response
                        .first_embedding()
                        .ok_or_else(|| Error::new_message("No embedding in response"))
                        .map(|embedding| {
                            // Convert f64 to f32 for compatibility with sqlite-vec
                            embedding.vector().iter().map(|&v| v as f32).collect()
                        })
                })
        })
    }

    /// Generate embeddings for multiple texts synchronously (batch processing)
    pub fn embed_batch_sync(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        let client = self.client.clone();
        let model = self.model.clone();
        let texts: Vec<String> = texts.into_iter().map(|s| s.to_string()).collect();

        // Run async operation in the runtime
        RUNTIME.block_on(async move {
            client
                .embed_batch(&model, texts, None)
                .await
                .map_err(|e| Error::new_message(format!("Batch embedding failed: {}", e)))
                .map(|response| {
                    response
                        .embeddings
                        .into_iter()
                        .map(|embedding| {
                            embedding.vector().iter().map(|&v| v as f32).collect()
                        })
                        .collect()
                })
        })
    }
}

/// Parsed client configuration from SQL
#[derive(Debug, PartialEq)]
pub struct ClientConfig {
    pub model: String,
    pub api_key: Option<String>,
}

/// Helper to parse client options and extract model + api key
pub fn parse_client_options(name: &str, options: &str) -> Result<ClientConfig> {
    // Check if options contains JSON-like structure with key
    if options.contains('{') && options.contains('}') {
        // Try to parse as JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(options) {
            let model = json.get("model")
                .or_else(|| json.get("provider"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| name.to_string());

            let api_key = json.get("key")
                .or_else(|| json.get("api_key"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            return Ok(ClientConfig { model, api_key });
        }
    }

    // Check if it's a simple "provider:key" format
    if options.contains(':') && !options.contains("::") {
        let parts: Vec<&str> = options.splitn(2, ':').collect();
        if parts.len() == 2 {
            let provider = parts[0];
            let key = parts[1];
            let model = format!("{}::{}", provider, name);
            return Ok(ClientConfig {
                model,
                api_key: Some(key.to_string())
            });
        }
    }

    // Legacy format: just provider name
    let model = match options {
        "openai" => format!("openai::{}", name),
        "gemini" => format!("gemini::{}", name),
        "cohere" => format!("cohere::{}", name),
        "anthropic" => format!("anthropic::{}", name),
        "ollama" => format!("ollama::{}", name),
        "groq" => format!("groq::{}", name),
        // If it already contains "::" assume it's a full model identifier
        s if s.contains("::") => s.to_string(),
        // Otherwise, assume it's a model name that should work with default provider
        _ => options.to_string(),
    };

    Ok(ClientConfig { model, api_key: None })
}

/// Legacy compatibility: Map old provider names to genai format
pub fn legacy_provider_to_model(provider: &str, model_name: &str) -> String {
    match provider {
        "openai" => format!("openai::{}", model_name),
        "nomic" => format!("openai::{}", model_name), // Nomic uses OpenAI-compatible API
        "cohere" => format!("cohere::{}", model_name),
        "jina" => format!("openai::{}", model_name), // Jina uses OpenAI-compatible API
        "mixedbread" => format!("openai::{}", model_name), // MixedBread uses OpenAI-compatible API
        "ollama" => format!("ollama::{}", model_name),
        "llamafile" => format!("ollama::{}", model_name), // Llamafile is Ollama-compatible
        _ => model_name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_client_options() {
        let config = parse_client_options("text-embedding-3-small", "openai").unwrap();
        assert_eq!(config.model, "openai::text-embedding-3-small");
        assert_eq!(config.api_key, None);

        let config = parse_client_options("embedding-001", "gemini").unwrap();
        assert_eq!(config.model, "gemini::embedding-001");
        assert_eq!(config.api_key, None);

        // Test passthrough for full model identifiers
        let config = parse_client_options("ignored", "openai::ada-002").unwrap();
        assert_eq!(config.model, "openai::ada-002");
        assert_eq!(config.api_key, None);
    }

    #[test]
    fn test_legacy_provider_mapping() {
        assert_eq!(
            legacy_provider_to_model("openai", "text-embedding-3-small"),
            "openai::text-embedding-3-small"
        );
        assert_eq!(
            legacy_provider_to_model("ollama", "nomic-embed-text"),
            "ollama::nomic-embed-text"
        );
    }
}