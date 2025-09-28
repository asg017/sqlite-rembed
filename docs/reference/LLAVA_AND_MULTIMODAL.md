# LLaVA and Multimodal Support in sqlite-rembed

## Understanding LLaVA vs Image Embeddings

### What is LLaVA?
LLaVA (Large Language and Vision Assistant) is a **vision-language generation model**, not an embedding model. It's designed to:
- Generate text descriptions from images
- Answer questions about images
- Perform visual reasoning tasks

### LLaVA is NOT for Embeddings
```sql
-- This WON'T work - LLaVA doesn't produce embeddings
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('llava', 'ollama::llava:latest');

SELECT rembed('llava', 'text');  -- ❌ Will fail
```

## Current Image Support in GenAI

According to the genai documentation, there IS limited image support for:
- **OpenAI** (GPT-4V)
- **Gemini Flash-2** (Multimodal)
- **Anthropic** (Claude Vision)

### How This Could Work for Embeddings

While these models primarily generate text from images, some providers offer image embedding capabilities:

#### OpenAI CLIP-style Embeddings
OpenAI's newer embedding models might support images:
```sql
-- Hypothetical future implementation
SELECT rembed_image('openai-clip', readfile('image.jpg'));
```

#### Google Gemini Multimodal Embeddings
Gemini has true multimodal embedding support:
```sql
-- Potential implementation with Gemini
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('gemini-multi', 'gemini::multimodal-embedding-001');

-- Could work for text + image embeddings
SELECT rembed_multimodal('gemini-multi',
  json_object('text', 'describe this', 'image', readfile('image.jpg')));
```

## What We Need for True Image Embeddings

### 1. Embedding Models (Not Generation Models)

| Model Type | Purpose | Examples |
|------------|---------|----------|
| **Generation Models** | Create text from images | LLaVA, GPT-4V, Claude Vision |
| **Embedding Models** | Create vectors from images | CLIP, ImageBind, Gemini Multimodal |

### 2. Proper Ollama Models for Embeddings

For Ollama, we need embedding-specific models:
```sql
-- Text embedding models that work TODAY
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('nomic', 'ollama::nomic-embed-text'),        -- ✅ Works
  ('mxbai', 'ollama::mxbai-embed-large'),       -- ✅ Works
  ('bge', 'ollama::bge-large'),                 -- ✅ Works
  ('e5', 'ollama::e5-large');                   -- ✅ Works

-- Vision models that DON'T work for embeddings
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('llava', 'ollama::llava'),                   -- ❌ Generation model
  ('bakllava', 'ollama::bakllava'),             -- ❌ Generation model
  ('llava-llama3', 'ollama::llava-llama3');     -- ❌ Generation model
```

## Implementation Path for Image Embeddings

### Step 1: Check GenAI's Current Capabilities
```rust
// Check if genai supports multimodal inputs
use genai::{Client, InputContent};

// Hypothetical API (needs verification)
let client = Client::default();
let input = InputContent::MultiModal {
    text: Some("describe this"),
    image: Some(image_bytes),
};
let embedding = client.embed("gemini::multimodal", input).await?;
```

### Step 2: Add SQL Functions for Images
```sql
-- New functions we'd need to add
CREATE FUNCTION rembed_image(client_name, image_blob) -> BLOB;
CREATE FUNCTION rembed_multimodal(client_name, json_input) -> BLOB;
```

### Step 3: Implement in lib.rs
```rust
pub fn rembed_image(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
    clients: &Rc<RefCell<HashMap<String, EmbeddingClient>>>,
) -> Result<()> {
    let client_name = api::value_text(&values[0])?;
    let image_blob = api::value_blob(&values[1])?;

    // Use genai's image capabilities
    let embedding = client.embed_image_sync(image_blob)?;

    api::result_blob(context, embedding.as_bytes());
    api::result_subtype(context, FLOAT32_VECTOR_SUBTYPE);
    Ok(())
}
```

## Available Ollama Embedding Models

Here are the Ollama models that ACTUALLY work for embeddings:

| Model | Dimensions | Use Case |
|-------|------------|----------|
| `nomic-embed-text` | 768 | General purpose |
| `mxbai-embed-large` | 1024 | High quality |
| `all-minilm` | 384 | Fast, lightweight |
| `bge-small` | 384 | Chinese + English |
| `bge-base` | 768 | Balanced |
| `bge-large` | 1024 | High quality |
| `e5-small` | 384 | Efficient |
| `e5-base` | 768 | Balanced |
| `e5-large` | 1024 | Best quality |

## Testing What Works Today

```sql
-- Load the extension
.load ./rembed0

-- Register working Ollama embedding models
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('ollama-nomic', 'ollama::nomic-embed-text'),
  ('ollama-e5', 'ollama::e5-large');

-- Test text embeddings (works today)
SELECT length(rembed('ollama-nomic', 'Hello world'));  -- ✅ Returns 768*4 bytes

-- Test batch processing (works today)
WITH batch AS (
  SELECT json_group_array(text) as texts
  FROM (VALUES ('text1'), ('text2'), ('text3'))
)
SELECT json_array_length(rembed_batch('ollama-nomic', texts));  -- ✅ Returns 3
```

## Conclusion

1. **LLaVA cannot be used for embeddings** - it's a generation model
2. **GenAI has limited image support** for OpenAI, Gemini, and Anthropic
3. **For true image embeddings**, we need:
   - CLIP-like models (not LLaVA)
   - GenAI multimodal input support
   - New SQL functions (`rembed_image`, `rembed_multimodal`)
4. **Ollama text embeddings work great** with models like nomic-embed-text
5. **Issue #7 (Image embeddings)** has a clear implementation path once genai adds full multimodal support

### Next Steps
1. Test genai's existing image capabilities with OpenAI/Gemini
2. Check if Gemini's multimodal embeddings work through genai
3. Consider adding CLIP model support through OpenAI or HuggingFace
4. Implement `rembed_image()` when genai has stable multimodal API