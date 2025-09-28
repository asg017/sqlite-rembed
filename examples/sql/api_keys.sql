.load dist/debug/rembed0
.bail on
.mode box
.header on

-- Test various ways to set API keys through SQL

-- Method 1: Simple provider:key format
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('openai-with-key', 'openai:sk-test-key-12345');

-- Method 2: JSON format with key
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('gemini-with-key', '{"provider": "gemini", "api_key": "test-gemini-key-67890"}');

-- Method 3: Full model with JSON including key
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('custom-openai', '{"model": "openai::text-embedding-3-large", "key": "sk-custom-key-abcdef"}');

-- Method 4: Using rembed_client_options (existing method)
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('options-based',
   rembed_client_options(
     'format', 'openai',
     'model', 'text-embedding-ada-002',
     'key', 'sk-options-key-xyz789'
   )
  );

-- Method 5: For local models (no key needed)
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('ollama-local', 'ollama::nomic-embed-text');

-- Verify all clients were registered
SELECT name FROM temp.rembed_clients ORDER BY name;

-- Show debug info to confirm backend
SELECT rembed_version();

.exit