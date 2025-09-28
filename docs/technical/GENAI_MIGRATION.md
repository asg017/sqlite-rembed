# Migration to GenAI Crate

## Benefits of Using GenAI

### Current Implementation Problems
1. **600+ lines of duplicate code** - Each provider has nearly identical HTTP handling
2. **Manual HTTP management** - Timeout, retry, error handling all custom-built
3. **Parser bugs** - MixedbreadClient using wrong parser (JinaClient's)
4. **Maintenance burden** - Adding new providers requires 100+ lines of boilerplate
5. **No batch support** - Current implementation makes individual HTTP requests
6. **Limited error handling** - No automatic retries or rate limiting

### GenAI Solution

With genai crate (0.4.0-alpha.4), the entire `clients.rs` file can be replaced with ~100 lines:

```rust
// Before: 600+ lines for 7 providers
pub struct OpenAiClient { /* fields */ }
impl OpenAiClient {
    pub fn infer_single(&self, input: &str) -> Result<Vec<f32>> {
        // 50+ lines of HTTP handling and parsing
    }
}
// Repeat for each provider...

// After: One unified client
pub struct GenAIClient {
    client: Arc<genai::Client>,
    model: String,
}

impl GenAIClient {
    pub async fn infer_single(&self, input: &str) -> Result<Vec<f32>> {
        self.client
            .embed(&self.model, input, None)
            .await
            .map(/* simple conversion */)
    }
}
```

## Migration Steps

### 1. Update Cargo.toml
```toml
[dependencies]
genai = "0.4.0-alpha.4"
tokio = { version = "1", features = ["rt", "macros"] }
# Remove ureq - no longer needed
```

### 2. Update Client Registration

Current SQL:
```sql
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('text-embedding-3-small', 'openai');
```

New SQL (with provider namespacing):
```sql
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('text-embedding-3-small', 'openai::text-embedding-3-small');
```

### 3. Async Considerations

SQLite extensions are synchronous, but genai is async. Options:

**Option A: Block on async** (Simple)
```rust
pub fn rembed(...) -> Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    let embedding = runtime.block_on(client.infer_single(input))?;
    // ...
}
```

**Option B: Background thread pool** (Better performance)
```rust
// Use a shared tokio runtime across all calls
lazy_static! {
    static ref RUNTIME: tokio::runtime::Runtime =
        tokio::runtime::Runtime::new().unwrap();
}
```

## Feature Comparison

| Feature | Current Implementation | With GenAI |
|---------|----------------------|------------|
| Lines of Code | 600+ | ~100 |
| Providers | 7 hardcoded | 10+ with automatic detection |
| Batch Support | ❌ None | ✅ Native `embed_batch()` |
| Retry Logic | ❌ None | ✅ Built-in with backoff |
| Rate Limiting | ❌ None | ✅ Provider-aware limits |
| Timeout | ✅ Basic (30s) | ✅ Configurable per-provider |
| New Provider | 100+ lines | 0 lines (automatic) |
| Response Parsing | Manual for each | Unified interface |
| Error Messages | Basic | Rich, provider-specific |
| Token Usage | ❌ None | ✅ Tracked automatically |

## Code Quality Improvements

### Before
- 7 separate client implementations
- 3 different response parsing patterns
- Bug-prone (wrong parser references)
- Duplicate HTTP error handling

### After
- Single unified client
- Provider detection from model names
- Automatic response handling
- Centralized error management

## Performance Benefits

1. **Batch Processing**: Send multiple texts in one request
2. **Connection Pooling**: Reuse HTTP connections
3. **Automatic Retries**: Handle transient failures gracefully
4. **Concurrent Requests**: Process multiple embeddings in parallel

## Backward Compatibility

To maintain compatibility, we can:
1. Keep the same SQL interface
2. Map old provider names to new model format
3. Support both sync and async internally

## Recommendation

**Strongly recommend migrating to genai** because:
1. Reduces codebase by 80%
2. Fixes all current bugs automatically
3. Adds batch support (major performance win)
4. Future-proof (new providers work automatically)
5. Better error handling and observability

The only downside is adding async runtime overhead, but this is negligible compared to network latency for API calls.

## Example Implementation

See `src/clients_genai.rs` for a complete proof of concept showing how simple the implementation becomes with genai.