# API Key Configuration Guide

With the new genai backend, sqlite-rembed offers multiple flexible ways to configure API keys directly through SQL, eliminating the need to set environment variables.

## 🔑 API Key Configuration Methods

### Method 1: Simple Provider:Key Format
The easiest way - just use `provider:your-api-key`:

```sql
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('my-openai', 'openai:sk-proj-abc123...'),
  ('my-gemini', 'gemini:AIza...'),
  ('my-groq', 'groq:gsk_abc123...');
```

### Method 2: JSON Configuration
More explicit with JSON format:

```sql
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('my-client', '{"provider": "openai", "api_key": "sk-proj-abc123..."}');

-- Or specify the full model
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('my-client', '{"model": "openai::text-embedding-3-large", "key": "sk-proj-abc123..."}');
```

### Method 3: Using rembed_client_options
The most flexible approach:

```sql
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('my-client',
   rembed_client_options(
     'format', 'openai',
     'model', 'text-embedding-3-small',
     'key', 'sk-proj-abc123...'
   )
  );
```

### Method 4: Environment Variables (Still Supported)
For production deployments, you can still use environment variables:

```bash
export OPENAI_API_KEY="sk-proj-abc123..."
export GEMINI_API_KEY="AIza..."
```

Then register without keys in SQL:
```sql
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('my-openai', 'openai::text-embedding-3-small');
```

## 🎯 Complete Examples

### OpenAI with API Key
```sql
-- Simple format
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('openai-embed', 'openai:sk-proj-your-key-here');

-- JSON format
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('openai-embed', '{"provider": "openai", "api_key": "sk-proj-your-key-here"}');

-- Use it
SELECT rembed('openai-embed', 'Hello, world!');
```

### Multiple Providers with Keys
```sql
INSERT INTO temp.rembed_clients(name, options) VALUES
  -- OpenAI
  ('gpt-small', 'openai:sk-proj-abc123'),
  ('gpt-large', '{"model": "openai::text-embedding-3-large", "key": "sk-proj-abc123"}'),

  -- Gemini
  ('gemini', 'gemini:AIzaSy...'),

  -- Anthropic
  ('claude', '{"provider": "anthropic", "api_key": "sk-ant-..."}'),

  -- Local models (no key needed)
  ('local-llama', 'ollama::llama2'),
  ('local-nomic', 'ollama::nomic-embed-text');
```

### Dynamic Key Management
```sql
-- Create a table to store API keys
CREATE TABLE api_keys (
  provider TEXT PRIMARY KEY,
  key TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Store keys securely
INSERT INTO api_keys (provider, key) VALUES
  ('openai', 'sk-proj-...'),
  ('gemini', 'AIza...');

-- Register clients using stored keys
INSERT INTO temp.rembed_clients(name, options)
SELECT
  provider || '-client',
  provider || ':' || key
FROM api_keys;
```

## 🔒 Security Considerations

### Development vs Production

**Development** - API keys in SQL are convenient:
```sql
-- Quick testing with inline keys
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('test', 'openai:sk-test-key');
```

**Production** - Use environment variables:
```bash
# Set in environment
export OPENAI_API_KEY="sk-proj-production-key"
```

```sql
-- Reference without exposing key
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('prod', 'openai::text-embedding-3-small');
```

### Best Practices

1. **Never commit API keys** to version control
2. **Use environment variables** in production
3. **Rotate keys regularly**
4. **Use restricted keys** when possible (limited scope/permissions)
5. **Store keys encrypted** if persisting in database

## 🎨 Provider-Specific Formats

| Provider | Simple Format | Environment Variable |
|----------|--------------|---------------------|
| OpenAI | `openai:sk-proj-...` | `OPENAI_API_KEY` |
| Gemini | `gemini:AIza...` | `GEMINI_API_KEY` |
| Anthropic | `anthropic:sk-ant-...` | `ANTHROPIC_API_KEY` |
| Groq | `groq:gsk_...` | `GROQ_API_KEY` |
| Cohere | `cohere:co-...` | `CO_API_KEY` |
| DeepSeek | `deepseek:sk-...` | `DEEPSEEK_API_KEY` |
| Mistral | `mistral:...` | `MISTRAL_API_KEY` |
| Ollama | `ollama::model` | None (local) |

## 🚀 Quick Start

```sql
-- Load the extension
.load ./rembed0

-- Register OpenAI with inline key (development)
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('embedder', 'openai:sk-proj-your-key-here');

-- Generate embeddings
SELECT length(rembed('embedder', 'Hello, world!'));

-- Register multiple providers
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('fast', 'openai:sk-proj-key1'),
  ('accurate', '{"model": "openai::text-embedding-3-large", "key": "sk-proj-key1"}'),
  ('free', 'ollama::nomic-embed-text');

-- Use different models
SELECT rembed('fast', 'Quick embedding');
SELECT rembed('accurate', 'Precise embedding');
SELECT rembed('free', 'Local embedding');
```

## 🎭 Migration from Environment Variables

If you're currently using environment variables and want to switch to SQL-based keys:

```sql
-- Before (requires OPENAI_API_KEY env var)
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('my-client', 'openai');

-- After (self-contained)
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('my-client', 'openai:sk-proj-your-key-here');
```

Both methods continue to work, giving you flexibility in deployment!