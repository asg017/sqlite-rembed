# sqlite-rembed Examples

This directory contains practical examples demonstrating various features and use cases of sqlite-rembed.

## 📂 Directory Structure

- **[sql/](./sql/)** - SQL examples for direct SQLite usage
- **[rust/](./rust/)** - Rust code examples for programmatic usage

## 🎯 SQL Examples

### Basic Usage
- **[basic_usage.sql](./sql/basic_usage.sql)** - Fundamental operations and setup
- **[basic.sql](./sql/basic.sql)** - Basic functionality tests

### Provider-Specific
- **[genai.sql](./sql/genai.sql)** - GenAI backend examples
- **[ollama_models.sql](./sql/ollama_models.sql)** - Ollama model testing
- **[llava.rs](./rust/llava.rs)** - LLaVA multimodal examples

### Features
- **[api_keys.sql](./sql/api_keys.sql)** - API key configuration examples
- **[batch.sql](./sql/batch.sql)** - Batch processing demonstrations

## 🚀 Quick Start Examples

### 1. Basic Text Embedding
```sql
-- Load extension
.load ./rembed0

-- Configure client
INSERT INTO temp.rembed_clients(name, options) VALUES
    ('openai', 'openai:YOUR_API_KEY');

-- Generate embedding
SELECT length(rembed('openai', 'Hello, world!'));
```

### 2. Batch Processing
```sql
-- Process multiple texts in one API call
WITH texts AS (
    SELECT json_array('text1', 'text2', 'text3') as batch
)
SELECT rembed_batch('openai', batch) FROM texts;
```

### 3. Image Embeddings
```sql
-- Process image with hybrid approach
SELECT rembed_image('ollama-multimodal', readfile('photo.jpg'));

-- Concurrent batch processing (4x faster)
SELECT rembed_images_concurrent('ollama-multimodal',
    json_array(
        readfile_base64(readfile('img1.jpg')),
        readfile_base64(readfile('img2.jpg'))
    ));
```

## 🏃 Running Examples

### SQL Examples
```bash
# Run a specific example
sqlite3 :memory: '.read examples/sql/basic_usage.sql'

# With the extension loaded
sqlite3 :memory: '.load dist/debug/rembed0' '.read examples/sql/test_batch.sql'
```

### Rust Examples
```bash
# Run Rust example
cd examples/rust
cargo run --example test_llava
```

## 📊 Performance Examples

### Sequential vs Concurrent
```sql
-- Sequential (baseline)
SELECT rembed_image('model', readfile('image.jpg'))
FROM images;

-- Concurrent (4x faster)
SELECT rembed_images_concurrent('model',
    json_group_array(readfile_base64(readfile(path)))
) FROM images;
```

### Batch Processing Impact
```sql
-- Individual calls (slow: 100 API calls)
SELECT rembed('model', text) FROM documents LIMIT 100;

-- Batch processing (fast: 1 API call)
WITH batch AS (
    SELECT json_group_array(text) as texts FROM documents LIMIT 100
)
SELECT rembed_batch('model', texts) FROM batch;
```

## 🔧 Configuration Examples

### Environment Variables
```bash
export OPENAI_API_KEY="sk-..."
export GEMINI_API_KEY="AIza..."
export OLLAMA_HOST="http://localhost:11434"
```

### SQL Configuration
```sql
-- Method 1: Simple format
INSERT INTO temp.rembed_clients(name, options) VALUES
    ('client1', 'openai:sk-...');

-- Method 2: JSON format
INSERT INTO temp.rembed_clients(name, options) VALUES
    ('client2', '{"provider": "gemini", "api_key": "AIza..."}');

-- Method 3: Function format
INSERT INTO temp.rembed_clients(name, options) VALUES
    ('client3', rembed_client_options(
        'format', 'openai',
        'model', 'text-embedding-3-large',
        'key', 'sk-...'
    ));
```

## 📈 Benchmarking

Run performance comparisons:

```bash
# Compare sequential vs concurrent
sqlite3 :memory: '.load ./rembed0' '.read examples/sql/benchmark_concurrent.sql'

# Test batch processing performance
sqlite3 :memory: '.load ./rembed0' '.read examples/sql/benchmark_batch.sql'
```

## 🤝 Contributing Examples

When adding new examples:

1. Use descriptive filenames (e.g., `multimodal_search.sql`)
2. Include comments explaining what the example demonstrates
3. Add error handling where appropriate
4. Update this README with your example

## 📝 Notes

- Examples assume the extension is built and available at `./rembed0`
- Replace API keys with your actual keys before running
- Some examples require external services (Ollama, OpenAI, etc.)
- Check the main [documentation](../docs/) for detailed guides