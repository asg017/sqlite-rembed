// Test script to check if genai supports LLaVA through Ollama
// This would test multimodal capabilities for issue #7

use genai::Client;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing LLaVA with genai through Ollama...\n");

    // Create genai client
    let client = Client::default();

    // Test 1: Check if we can use LLaVA for text generation
    println!("Test 1: LLaVA text generation");
    let model = "ollama::llava:latest";

    match client.gen(model, "What is machine learning?", None).await {
        Ok(response) => {
            println!("✅ LLaVA text works: {}", response.text());
        }
        Err(e) => {
            println!("❌ LLaVA text failed: {}", e);
        }
    }

    // Test 2: Check if embeddings work with LLaVA
    // Note: LLaVA is primarily a vision-language model, not an embedding model
    println!("\nTest 2: LLaVA embeddings (likely to fail - wrong model type)");
    match client.embed(model, "Test text", None).await {
        Ok(response) => {
            if let Some(embedding) = response.first_embedding() {
                println!("✅ LLaVA embedding works! Dimension: {}", embedding.vector().len());
            }
        }
        Err(e) => {
            println!("❌ LLaVA embeddings failed (expected): {}", e);
        }
    }

    // Test 3: Try a proper Ollama embedding model
    println!("\nTest 3: Ollama embedding models");
    let embedding_models = vec![
        "ollama::nomic-embed-text",
        "ollama::mxbai-embed-large",
        "ollama::all-minilm",
    ];

    for model in embedding_models {
        print!("Testing {}: ", model);
        match client.embed(model, "Test embedding", None).await {
            Ok(response) => {
                if let Some(embedding) = response.first_embedding() {
                    println!("✅ Dimension: {}", embedding.vector().len());
                }
            }
            Err(e) => {
                println!("❌ Failed: {}", e);
            }
        }
    }

    // Test 4: Check multimodal with image (if genai supports it)
    println!("\nTest 4: Multimodal capabilities (experimental)");

    // This is hypothetical - genai might not have this API yet
    // But this is what we'd want for image embeddings
    /*
    let image_bytes = std::fs::read("test_image.jpg")?;
    let image_base64 = base64::encode(&image_bytes);

    let multimodal_input = json!({
        "text": "Describe this image",
        "image": image_base64
    });

    match client.gen(model, multimodal_input, None).await {
        Ok(response) => {
            println!("✅ Multimodal works: {}", response.text());
        }
        Err(e) => {
            println!("❌ Multimodal failed: {}", e);
        }
    }
    */

    println!("\nNote: Full multimodal support would require genai API extensions");

    Ok(())
}