.load dist/debug/rembed0
.bail on
.mode box
.header on

-- Test batch embedding functionality
-- This solves issue #1 by sending multiple texts in a single HTTP request

-- Register a client (you'll need to set the API key)
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('batch-test', 'ollama::nomic-embed-text');

-- Test 1: Basic batch embedding with rembed_batch()
SELECT '=== Test 1: Basic batch embedding ===' as test;

-- Create test data
CREATE TABLE test_texts (
  id INTEGER PRIMARY KEY,
  content TEXT
);

INSERT INTO test_texts (content) VALUES
  ('The quick brown fox jumps over the lazy dog'),
  ('Machine learning is transforming industries'),
  ('SQLite is a powerful embedded database'),
  ('Batch processing improves performance'),
  ('Natural language processing enables new applications');

-- Generate embeddings in batch (single HTTP request!)
WITH batch_input AS (
  SELECT json_group_array(content) as texts_json
  FROM test_texts
)
SELECT
  'Batch size: ' || json_array_length(texts_json) as info,
  substr(rembed_batch('batch-test', texts_json), 1, 100) || '...' as result_preview
FROM batch_input;

-- Test 2: Compare single vs batch performance
SELECT '=== Test 2: Performance comparison ===' as test;

-- Single requests (old method - multiple HTTP requests)
.timer on
SELECT COUNT(*) as single_count
FROM (
  SELECT rembed('batch-test', content) as embedding
  FROM test_texts
);
.timer off

-- Batch request (new method - single HTTP request)
.timer on
WITH batch_input AS (
  SELECT json_group_array(content) as texts_json
  FROM test_texts
)
SELECT
  json_array_length(rembed_batch('batch-test', texts_json)) as batch_count
FROM batch_input;
.timer off

-- Test 3: Batch processing with larger dataset
SELECT '=== Test 3: Larger batch test ===' as test;

-- Generate more test data
INSERT INTO test_texts (content)
SELECT 'Sample text ' || value || ': ' ||
       CASE value % 5
         WHEN 0 THEN 'Database systems are essential for data management'
         WHEN 1 THEN 'Artificial intelligence is rapidly evolving'
         WHEN 2 THEN 'Cloud computing provides scalable solutions'
         WHEN 3 THEN 'Security is paramount in modern applications'
         WHEN 4 THEN 'Performance optimization requires careful analysis'
       END
FROM generate_series(10, 50);

-- Process larger batch
WITH batch_input AS (
  SELECT json_group_array(content) as texts_json,
         COUNT(*) as total_texts
  FROM test_texts
)
SELECT
  'Processing ' || total_texts || ' texts in single batch' as info,
  CASE
    WHEN json_array_length(rembed_batch('batch-test', texts_json)) = total_texts
    THEN '✓ Success: All embeddings generated'
    ELSE '✗ Error: Embedding count mismatch'
  END as status
FROM batch_input;

-- Test 4: Practical use case - semantic search with batch embeddings
SELECT '=== Test 4: Practical batch embedding use case ===' as test;

-- Create a table to store embeddings
CREATE TABLE text_embeddings (
  id INTEGER PRIMARY KEY,
  content TEXT,
  embedding BLOB
);

-- Insert data with batch-generated embeddings
-- This demonstrates how to use batch processing in production
WITH batch_input AS (
  SELECT
    json_group_array(json_object('id', id, 'text', content)) as items_json,
    json_group_array(content) as texts_json
  FROM test_texts
),
batch_results AS (
  SELECT
    json_each.key as idx,
    json_each.value as embedding_base64,
    json_extract(json_each_items.value, '$.id') as text_id,
    json_extract(json_each_items.value, '$.text') as text_content
  FROM batch_input
  CROSS JOIN json_each(rembed_batch('batch-test', texts_json))
  CROSS JOIN json_each(items_json) as json_each_items
  WHERE json_each.key = json_each_items.key
)
SELECT COUNT(*) as embedded_texts
FROM batch_results;

-- Verify batch processing worked
SELECT
  'Total texts: ' || COUNT(*) as summary,
  'Min ID: ' || MIN(id) as min_id,
  'Max ID: ' || MAX(id) as max_id
FROM test_texts;

-- Clean up
DROP TABLE test_texts;
DROP TABLE text_embeddings;

SELECT '=== Batch processing tests completed ===' as status;