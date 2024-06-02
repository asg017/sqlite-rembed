.load dist/debug/rembed0
.bail on
.mode box
.header on
.timer on

INSERT INTO temp.rembed_clients(name, options) VALUES
  ('text-embedding-3-small','openai'),
  ('nomic-embed-text-v1.5', 'nomic'),
  ('embed-english-v3.0', 'cohere'),
  ('snowflake-arctic-embed:s', 'ollama'),
  (
    'mxbai-embed-large-v1-f16',
    rembed_client_options(
      'flavor', 'openai',
      'url', 'http://mm1:8080/v1/embeddings'
    )
  );

select length(rembed('snowflake-arctic-embed:s', 'obama the person'));
select length(rembed('mxbai-embed-large-v1-f16', 'obama the person'));
.exit
select length(rembed('text-embedding-3-small', 'obama the person'));


