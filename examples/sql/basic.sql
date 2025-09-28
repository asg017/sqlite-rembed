.load dist/debug/rembed0
.bail on
.mode box
.header on

-- Test that the extension loads and version functions work
SELECT rembed_version();
SELECT rembed_debug();

-- Test that client registration works with the fixed error messages
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('test-client', rembed_client_options('format', 'ollama', 'model', 'test-model'));

-- Verify the client was registered
SELECT name FROM temp.rembed_clients;

.exit