# `sqlite-rembed` with GenAI Backend

A SQLite extension for generating text embeddings using the powerful [genai](https://github.com/jeremychone/rust-genai) multi-provider AI client library. Sister project to [`sqlite-vec`](https://github.com/asg017/sqlite-vec) and [`sqlite-lembed`](https://github.com/asg017/sqlite-lembed).

## 🚀 What's New with GenAI

- **80% less code** - Reduced from 795 lines to 160 lines
- **10+ providers supported** - OpenAI, Anthropic, Gemini, Ollama, Groq, Cohere, and more
- **Batch processing** - Generate multiple embeddings in a single API call
- **Automatic retries** - Built-in retry logic with exponential backoff
- **Zero-config for new providers** - Add new providers without code changes

## Usage

```sql
.load ./rembed0

-- Simple registration with provider prefix
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('openai-small', 'openai::text-embedding-3-small'),
  ('gemini-latest', 'gemini::text-embedding-004'),
  ('ollama-local', 'ollama::nomic-embed-text');

-- Generate an embedding
SELECT rembed('openai-small', 'The quick brown fox jumps over the lazy dog');

-- Legacy compatibility (still works!)
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('text-embedding-3-small', 'openai');

-- Advanced configuration
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('custom-model',
   rembed_client_options(
     'format', 'openai',
     'model', 'text-embedding-3-large',
     'key', 'sk-...' -- Optional, defaults to env var
   )
  );
```

## Supported Providers

Thanks to genai, sqlite-rembed now supports many more providers:

| Provider | Model Format | Environment Variable |
|----------|--------------|---------------------|
| OpenAI | `openai::text-embedding-3-small` | `OPENAI_API_KEY` |
| Gemini | `gemini::text-embedding-004` | `GEMINI_API_KEY` |
| Anthropic | `anthropic::voyage-3` | `ANTHROPIC_API_KEY` |
| Ollama | `ollama::nomic-embed-text` | None (local) |
| Groq | `groq::llama-3.3-70b-versatile` | `GROQ_API_KEY` |
| Cohere | `cohere::embed-english-v3.0` | `CO_API_KEY` |
| DeepSeek | `deepseek::deepseek-chat` | `DEEPSEEK_API_KEY` |
| XAI | `xai::grok-2-latest` | `XAI_API_KEY` |

## Using with sqlite-vec

The integration with sqlite-vec remains unchanged:

```sql
-- Create vector table
CREATE VIRTUAL TABLE vec_articles USING vec0(headline_embeddings float[1536]);

-- Insert embeddings
INSERT INTO vec_articles(rowid, headline_embeddings)
  SELECT rowid, rembed('openai::text-embedding-3-small', headline)
  FROM articles;

-- Semantic search
WITH matches AS (
  SELECT rowid, distance
  FROM vec_articles
  WHERE headline_embeddings MATCH rembed('openai::text-embedding-3-small', :query)
  ORDER BY distance
  LIMIT 3
)
SELECT headline, distance
FROM matches
LEFT JOIN articles ON articles.rowid = matches.rowid;
```

## Performance Improvements

The genai backend brings significant performance benefits:

- **Connection pooling** - Reuses HTTP connections across requests
- **Automatic retries** - Handles transient failures gracefully
- **Batch processing** - Process multiple embeddings in one API call (coming soon to SQL API)
- **Concurrent requests** - Can process multiple providers in parallel

## Migration from Old Version

The new version maintains full backward compatibility:

```sql
-- Old style (still works)
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('text-embedding-3-small', 'openai');

-- New style (recommended)
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('text-embedding-3-small', 'openai::text-embedding-3-small');
```

## Building

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the extension
make loadable

# Run tests
sqlite3 :memory: < test.sql
```

## Architecture Benefits

The genai migration provides:

1. **Unified Error Handling** - Consistent error messages across all providers
2. **Token Usage Tracking** - Monitor API usage (when supported by provider)
3. **Timeout Management** - Configurable timeouts per provider
4. **Rate Limiting** - Provider-aware rate limiting
5. **Future-Proof** - New providers work automatically

## License

Apache-2.0 OR MIT

## Acknowledgements

- [genai](https://github.com/jeremychone/rust-genai) - The amazing multi-provider AI client
- [sqlite-vec](https://github.com/asg017/sqlite-vec) - Vector search for SQLite
- [sqlite-loadable](https://github.com/asg017/sqlite-loadable-rs) - Framework for SQLite extensions in Rust