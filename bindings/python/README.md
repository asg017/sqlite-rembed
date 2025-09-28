# sqlite-rembed Python Package

Generate text and image embeddings from remote APIs inside SQLite.

A SQLite extension that provides embedding generation from 10+ AI providers including OpenAI, Gemini, Anthropic, Ollama, and more. Powered by the [rust-genai](https://github.com/rsp2k/rust-genai) fork with multimodal support.

## Installation

```bash
pip install sqlite-rembed
```

## Quick Start

```python
import sqlite3
import sqlite_rembed

# Load the extension
conn = sqlite3.connect(':memory:')
conn.enable_load_extension(True)
sqlite_rembed.load(conn)
conn.enable_load_extension(False)

# Configure API clients
conn.execute("""
    INSERT INTO temp.rembed_clients(name, options) VALUES
        ('openai', 'openai:YOUR_OPENAI_KEY'),
        ('gemini', 'gemini:YOUR_GEMINI_KEY'),
        ('ollama', 'ollama::nomic-embed-text')  -- Local, no key needed
""")

# Generate embeddings
result = conn.execute("SELECT rembed('openai', 'Hello, world!')").fetchone()
embedding = result[0]  # Binary blob containing float32 array
```

## Features

### Text Embeddings

```python
# Single embedding
embedding = conn.execute(
    "SELECT rembed('openai', 'Your text here')"
).fetchone()[0]

# Batch processing (100-1000x faster for multiple texts)
import json

texts = ["text1", "text2", "text3", "text4", "text5"]
batch_json = json.dumps(texts)

embeddings_json = conn.execute(
    "SELECT rembed_batch('openai', ?)", (batch_json,)
).fetchone()[0]

# Parse results
import base64
embeddings = json.loads(embeddings_json)
for encoded in embeddings:
    embedding = base64.b64decode(encoded)
    # Use embedding (float32 array)
```

### Image Embeddings (Hybrid Multimodal)

```python
# Process image using LLaVA → text → embedding approach
with open('image.jpg', 'rb') as f:
    image_data = f.read()

embedding = conn.execute(
    "SELECT rembed_image('ollama-multimodal', ?)", (image_data,)
).fetchone()[0]

# Concurrent batch processing (2-6x faster)
images = [img1_bytes, img2_bytes, img3_bytes]
images_b64 = [base64.b64encode(img).decode() for img in images]
batch_json = json.dumps(images_b64)

result_json = conn.execute(
    "SELECT rembed_images_concurrent('ollama-multimodal', ?)", (batch_json,)
).fetchone()[0]

result = json.loads(result_json)
embeddings = [base64.b64decode(e) for e in result['embeddings']]
print(f"Processed {result['stats']['successful']} images at {result['stats']['throughput']} img/sec")
```

## Supported Providers

All providers from the [rust-genai](https://github.com/rsp2k/rust-genai) library:

- **OpenAI** - `openai::text-embedding-3-small`
- **Gemini** - `gemini::text-embedding-004`
- **Anthropic** - `anthropic::voyage-3`
- **Ollama** - `ollama::nomic-embed-text` (local, free)
- **Groq** - `groq::llama-3.3-70b`
- **Cohere** - `cohere::embed-english-v3.0`
- **Mistral** - `mistral::mistral-embed`
- And more...

## API Key Configuration

Four ways to configure API keys:

### 1. Simple Format
```python
conn.execute("""
    INSERT INTO temp.rembed_clients(name, options)
    VALUES ('my-client', 'openai:sk-...')
""")
```

### 2. JSON Format
```python
conn.execute("""
    INSERT INTO temp.rembed_clients(name, options)
    VALUES ('my-client', '{"provider": "openai", "api_key": "sk-..."}')
""")
```

### 3. Environment Variables
```python
import os
os.environ['OPENAI_API_KEY'] = 'sk-...'

conn.execute("""
    INSERT INTO temp.rembed_clients(name, options)
    VALUES ('my-client', 'openai::text-embedding-3-small')
""")
```

### 4. rembed_client_options Function
```python
conn.execute("""
    INSERT INTO temp.rembed_clients(name, options)
    VALUES ('my-client', rembed_client_options(
        'format', 'openai',
        'model', 'text-embedding-3-large',
        'key', 'sk-...'
    ))
""")
```

## Integration with sqlite-vec

sqlite-rembed works seamlessly with [sqlite-vec](https://github.com/asg017/sqlite-vec) for vector similarity search:

```python
import sqlite3
import sqlite_vec
import sqlite_rembed

# Load both extensions
conn = sqlite3.connect(':memory:')
conn.enable_load_extension(True)
sqlite_vec.load(conn)
sqlite_rembed.load(conn)
conn.enable_load_extension(False)

# Configure embedding client
conn.execute("""
    INSERT INTO temp.rembed_clients(name, options)
    VALUES ('openai', 'openai:YOUR_KEY')
""")

# Create vector table
conn.execute("""
    CREATE VIRTUAL TABLE vec_items USING vec0(
        embedding float[1536]
    )
""")

# Store embeddings
texts = ["apple", "banana", "cherry", "date", "elderberry"]
for text in texts:
    embedding = conn.execute(
        "SELECT rembed('openai', ?)", (text,)
    ).fetchone()[0]
    conn.execute(
        "INSERT INTO vec_items(embedding) VALUES (?)",
        (embedding,)
    )

# Semantic search
query = "fruit that's red"
query_embedding = conn.execute(
    "SELECT rembed('openai', ?)", (query,)
).fetchone()[0]

results = conn.execute("""
    SELECT rowid, distance
    FROM vec_items
    WHERE embedding MATCH ?
    ORDER BY distance
    LIMIT 3
""", (query_embedding,)).fetchall()

for rowid, distance in results:
    print(f"Match {rowid}: distance={distance:.4f}")
```

## Advanced Features

### Helper Functions

```python
# Base64 encode files for image processing
encoded = conn.execute(
    "SELECT readfile_base64(?)", (image_bytes,)
).fetchone()[0]
```

### Performance Configuration

The multimodal client uses optimized defaults:
- Max concurrent requests: 4
- Request timeout: 30 seconds
- Batch size: 10

### Error Handling

```python
try:
    embedding = conn.execute(
        "SELECT rembed('openai', 'text')"
    ).fetchone()[0]
except sqlite3.OperationalError as e:
    if "not registered" in str(e):
        print("Client not configured")
    elif "API" in str(e):
        print("API error occurred")
    else:
        raise
```

## Testing

Run the included tests:

```bash
cd bindings/python
python tests/test_basic.py
```

## Documentation

- [Main Documentation](https://github.com/asg017/sqlite-rembed/tree/main/docs)
- [API Reference](https://github.com/asg017/sqlite-rembed/tree/main/docs/guides)
- [Examples](https://github.com/asg017/sqlite-rembed/tree/main/examples)

## License

MIT OR Apache-2.0

## Credits

Built on:
- [rust-genai](https://github.com/rsp2k/rust-genai) - Unified AI client library with multimodal support
- [sqlite-loadable-rs](https://github.com/asg017/sqlite-loadable-rs) - Framework for SQLite extensions in Rust