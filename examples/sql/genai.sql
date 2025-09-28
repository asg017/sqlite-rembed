.load dist/debug/rembed0
.bail on
.mode box
.header on

-- Test version to confirm genai backend
SELECT rembed_version();
SELECT rembed_debug();

-- Test legacy compatibility - old style registration
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('test-ollama', 'ollama');

-- Test new style with model identifier
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('test-openai', 'openai::text-embedding-3-small');

-- Verify clients were registered
SELECT name FROM temp.rembed_clients;

-- Test using rembed_client_options for more complex setup
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('test-custom',
   rembed_client_options(
     'format', 'openai',
     'model', 'text-embedding-3-large'
   )
  );

SELECT name FROM temp.rembed_clients;

.exit