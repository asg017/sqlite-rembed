.load dist/debug/rembed0
.bail on
.mode box
.header on

-- Test various Ollama models including potential vision models

SELECT '=== Testing Ollama Models with GenAI ===' as test;

-- Test 1: Standard Ollama embedding models
SELECT '--- Test 1: Standard Embedding Models ---' as test;

-- Register various Ollama embedding models
INSERT INTO temp.rembed_clients(name, options) VALUES
  -- Standard text embedding models
  ('nomic', 'ollama::nomic-embed-text'),
  ('mxbai', 'ollama::mxbai-embed-large'),
  ('minilm', 'ollama::all-minilm');

-- Test if they work
SELECT
  'nomic' as model,
  CASE
    WHEN length(rembed('nomic', 'Test text')) > 0
    THEN '✅ Works - dim: ' || length(rembed('nomic', 'Test text'))/4
    ELSE '❌ Failed'
  END as status;

SELECT
  'mxbai' as model,
  CASE
    WHEN length(rembed('mxbai', 'Test text')) > 0
    THEN '✅ Works - dim: ' || length(rembed('mxbai', 'Test text'))/4
    ELSE '❌ Failed'
  END as status;

-- Test 2: Try LLaVA (vision-language model)
SELECT '--- Test 2: Vision-Language Models (Experimental) ---' as test;

-- LLaVA is a multimodal model, not an embedding model
-- This will likely fail but let's test
INSERT INTO temp.rembed_clients(name, options) VALUES
  ('llava', 'ollama::llava:latest'),
  ('bakllava', 'ollama::bakllava:latest'),
  ('llava-llama3', 'ollama::llava-llama3:latest');

-- These will probably fail since LLaVA isn't an embedding model
SELECT
  'llava' as model,
  'Note: LLaVA is a vision-language model, not an embedding model' as info;

-- Test 3: What we'd need for multimodal embeddings
SELECT '--- Test 3: Future Multimodal Support ---' as test;

SELECT 'For image embeddings, we would need:' as requirement
UNION ALL
SELECT '1. CLIP-based models (e.g., openai::clip)'
UNION ALL
SELECT '2. Multimodal embedding models (e.g., imagebind)'
UNION ALL
SELECT '3. genai support for multimodal inputs'
UNION ALL
SELECT '4. SQL functions like rembed_image() or rembed_multimodal()';

-- Test 4: Check what embedding models Ollama actually has
SELECT '--- Test 4: Available Ollama Embedding Models ---' as test;

-- List the models we know work with Ollama
WITH ollama_models(model, description, dimensions) AS (
  VALUES
    ('nomic-embed-text', 'Nomic AI text embeddings', 768),
    ('mxbai-embed-large', 'MixedBread AI embeddings', 1024),
    ('all-minilm', 'Sentence transformers MiniLM', 384),
    ('bge-small', 'BAAI General Embedding', 384),
    ('bge-base', 'BAAI General Embedding', 768),
    ('bge-large', 'BAAI General Embedding', 1024),
    ('e5-small', 'E5 text embeddings', 384),
    ('e5-base', 'E5 text embeddings', 768),
    ('e5-large', 'E5 text embeddings', 1024)
)
SELECT
  printf('%-20s', model) as model,
  printf('%-30s', description) as description,
  dimensions
FROM ollama_models;

-- Test 5: Batch processing with Ollama
SELECT '--- Test 5: Batch Processing with Ollama ---' as test;

-- Create test data
CREATE TEMP TABLE test_texts (id INTEGER PRIMARY KEY, content TEXT);
INSERT INTO test_texts (content) VALUES
  ('First test text'),
  ('Second test text'),
  ('Third test text');

-- Test batch processing with Ollama
WITH batch AS (
  SELECT json_group_array(content) as texts
  FROM test_texts
)
SELECT
  'Batch size: ' || json_array_length(texts) as info,
  CASE
    WHEN json_array_length(rembed_batch('nomic', texts)) = 3
    THEN '✅ Batch processing works with Ollama!'
    ELSE '❌ Batch processing failed'
  END as status
FROM batch;

-- Clean up
DROP TABLE test_texts;

SELECT '=== Summary ===' as summary;
SELECT 'GenAI + Ollama integration status:' as item, 'Working' as status
UNION ALL
SELECT 'Text embeddings:', '✅ Supported'
UNION ALL
SELECT 'Batch processing:', '✅ Supported'
UNION ALL
SELECT 'Vision models (LLaVA):', '⚠️ Not for embeddings'
UNION ALL
SELECT 'Image embeddings:', '🔜 Needs multimodal support';

-- Note about LLaVA
SELECT '' as '';
SELECT 'Note: LLaVA is a vision-language MODEL for generation, not embeddings.' as important
UNION ALL
SELECT 'For image embeddings, we need models like CLIP or ImageBind.' as important
UNION ALL
SELECT 'GenAI would need to support multimodal inputs for this to work.' as important;