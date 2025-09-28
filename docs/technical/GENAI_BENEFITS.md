# How GenAI Solves sqlite-rembed's Open Issues

## Issue #2: Rate Limiting Options

### The Challenge
Different providers have different rate limits, and coordinating these across multiple custom HTTP clients was complex. Some providers return rate limit information in headers (like OpenAI's `x-ratelimit-*` headers), while others don't.

### How GenAI Helps

#### 1. Automatic Retry with Exponential Backoff
GenAI includes built-in retry logic that automatically handles rate limiting:
```rust
// genai automatically retries with exponential backoff
client.embed(&model, text, None)
    .await  // Retries happen internally
```

This means:
- Transient 429 (Too Many Requests) errors are automatically retried
- Exponential backoff prevents hammering the API
- No manual retry logic needed

#### 2. Unified Error Handling
GenAI provides consistent error types across all providers:
```rust
match result {
    Err(e) if e.is_rate_limit() => {
        // Handle rate limit uniformly across providers
    }
    Err(e) => // Other errors
}
```

#### 3. Rate Limit Headers Access
GenAI can expose response metadata including rate limit headers:
```rust
let response = client.embed(&model, text, None).await?;
// Future: Access response.metadata() for rate limit info
```

### Future Improvements
With genai, we could implement:
- Smart request throttling based on header information
- Provider-specific rate limit tracking
- Automatic backoff when approaching limits

## Issue #3: Token/Request Usage Tracking

### The Challenge
Each provider reports token usage differently, making it difficult to track costs and usage across different APIs.

### How GenAI Helps

#### 1. Unified Usage Metrics
GenAI provides consistent token usage information across providers:
```rust
let response = client.embed_batch(&model, texts, None).await?;
// Access token usage
if let Some(usage) = response.usage() {
    let tokens_used = usage.total_tokens();
    let requests_made = 1;  // Track per request
}
```

#### 2. Batch Processing Reduces Tracking Complexity
With batch processing, tracking becomes simpler:
- 1 batch request = 1 API call (easy to count)
- Token usage is reported per batch
- Dramatic reduction in request count makes tracking easier

#### 3. Provider-Agnostic Metrics
GenAI normalizes metrics across providers:
```rust
pub struct Usage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}
```

### Implementation Ideas

#### Per-Client Usage Tracking
```sql
-- Could add a usage tracking table
CREATE TABLE rembed_usage (
    client_name TEXT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    requests INTEGER,
    tokens_used INTEGER,
    batch_size INTEGER
);

-- Track usage after each batch
INSERT INTO rembed_usage (client_name, requests, tokens_used, batch_size)
VALUES ('openai-fast', 1, 5000, 100);
```

#### Usage Statistics Function
```sql
-- Future: Add usage statistics function
SELECT rembed_usage_stats('openai-fast');
-- Returns: {"total_requests": 150, "total_tokens": 750000, "avg_batch_size": 50}
```

## Combined Benefits

The migration to genai provides a foundation for solving both issues:

1. **Unified Interface**: One library handles all provider quirks
2. **Consistent Metadata**: Rate limits and usage data in standard format
3. **Built-in Resilience**: Automatic retries reduce manual error handling
4. **Future-Proof**: New providers automatically get these benefits

## Code Example: Rate Limiting with Token Tracking

Here's how we could extend the current implementation:

```rust
// In genai_client.rs
pub struct EmbeddingClientWithTracking {
    client: Arc<GenAiClient>,
    model: String,
    usage: Arc<Mutex<UsageStats>>,
}

pub struct UsageStats {
    total_requests: u64,
    total_tokens: u64,
    rate_limit_hits: u64,
    last_rate_limit_reset: Option<Instant>,
}

impl EmbeddingClientWithTracking {
    pub fn embed_batch_with_tracking(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        let response = self.client.embed_batch(&self.model, texts, None).await?;

        // Track usage
        if let Some(usage) = response.usage() {
            let mut stats = self.usage.lock().unwrap();
            stats.total_requests += 1;
            stats.total_tokens += usage.total_tokens().unwrap_or(0) as u64;
        }

        // Check rate limit headers (when genai exposes them)
        if let Some(headers) = response.headers() {
            if let Some(remaining) = headers.get("x-ratelimit-remaining-requests") {
                // Implement smart throttling
            }
        }

        Ok(response.embeddings)
    }
}
```

## SQL Interface for Monitoring

```sql
-- Check current rate limit status
SELECT rembed_rate_limit_status('openai-fast');
-- Returns: {"remaining_requests": 4999, "reset_in": "12ms"}

-- Get usage statistics
SELECT rembed_usage_summary('openai-fast', 'today');
-- Returns: {"requests": 150, "tokens": 750000, "cost_estimate": "$0.15"}

-- Set rate limit configuration
INSERT INTO temp.rembed_rate_limits(client, max_rpm, max_tpm) VALUES
  ('openai-fast', 5000, 5000000);
```

## Conclusion

The genai migration provides:
1. **Immediate benefits**: Automatic retries partially address rate limiting
2. **Foundation for future**: Standardized interface for implementing full solutions
3. **Simplified implementation**: One place to add rate limiting/tracking logic
4. **Provider flexibility**: Works uniformly across all 10+ providers

While the full solutions for #2 and #3 aren't implemented yet, genai has transformed them from complex multi-provider challenges into straightforward feature additions.