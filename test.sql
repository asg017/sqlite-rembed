.load dist/debug/rembed0
.bail on
.mode box
.header on
.timer on
.echo on

INSERT INTO temp.rembed_clients(name, options) VALUES
  ('snowflake-arctic-embed:s', 'ollama');

create table articles as
  select column1 as headline
  from (VALUES
    ('Shohei Ohtani''s ex-interpreter pleads guilty to charges related to gambling and theft'),
    ('The jury has been selected in Hunter Biden''s gun trial'),
    ('Larry Allen, a Super Bowl champion and famed Dallas Cowboy, has died at age 52'),
    ('After saying Charlotte, a lone stingray, was pregnant, aquarium now says she''s sick'),
    ('An Epoch Times executive is facing money laundering charge'),
    ('Hassan Nasrallah’s killing transforms an already deadly regional conflict'),
    ('Who was Hassan Nasrallah, the Hezbollah leader killed by Israel?'),
    ('What is Hezbollah, the militia fighting Israel in Lebanon?'),
    ('Netanyahu defies calls for a cease-fire at the U.N., as Israel strikes Lebanon'),
    ('Death toll from Hurricane Helene mounts as aftermath assessment begins'),
    ('5 things to know from this week’s big report on cannabis'),
    ('VP debates may alter a close race’s dynamic even when they don''t predict the winner'),
    ('SpaceX launches ISS-bound crew that hopes to bring home 2 stuck astronauts'),
    ('Why the price of eggs is on the rise again'),
    ('A guide to your weekend viewing and reading'),
    ('At the border in Arizona, Harris lays out a plan to get tough on fentanyl'),
    ('A new kind of drug for schizophrenia promises fewer side effects'),
    ('Meet the astronauts preparing to travel farther from Earth than any human before'),
    ('‘SNL’ has always taken on politics. Here’s what works — and why'),
    ('Golden-age rappers make a digital-age leap — and survive'),
    ('Why Russia''s broadcaster RT turned to covertly funding American pro-Trump influencers'),
    ('Read the indictment: NYC Mayor Eric Adams charged with bribery, fraud, foreign donations'),
    ('Justice Department sues Alabama, claiming it purged voters too close to the election'),
    ('Exactly 66 years ago, another Hurricane Helene rocked the Carolinas'),
    ('A meteorologist in Atlanta rescued a woman from Helene floodwaters on camera')
  );

select * from articles;

.timer on
select headline, length(rembed('snowflake-arctic-embed:s', headline)) from articles;

select contents, length(embedding)
from rembed_batch(
  'snowflake-arctic-embed:s',
  (
    select json_group_array(
      json_object(
        'id', rowid,
      'contents', headline
      )
    ) from articles
  )
);
.exit


select *
from rembed_batch(
  'snowflake-arctic-embed:s',
  json('[
  {"id": 1, "contents": "alex garcia"},
  {"id": 1, "contents": "joe biden"},
  {"id": 1, "contents": "kamala harris"}
]'));


.exit


INSERT INTO temp.rembed_clients(name, options) VALUES
  ('text-embedding-3-small','openai'),
  ('jina-embeddings-v2-base-en','jina'),
  ('mixedbread-ai/mxbai-embed-large-v1','mixedbread'),
  ('nomic-embed-text-v1.5', 'nomic'),
  ('embed-english-v3.0', 'cohere'),
  ('snowflake-arctic-embed:s', 'ollama'),
  ('llamafile', 'llamafile'),
  (
    'mxbai-embed-large-v1-f16',
    rembed_client_options(
      'format', 'llamafile',
      --'url', 'http://mm1:8080/v1/embeddings'
      'url', 'http://mm1:8080/embedding'
    )
  );

select length(rembed('mixedbread-ai/mxbai-embed-large-v1', 'obama the person'));
.exit
select length(rembed('jina-embeddings-v2-base-en', 'obama the person'));

.exit

select length(rembed('text-embedding-3-small', 'obama the person'));
select length(rembed('llamafile', 'obama the person'));
select length(rembed('snowflake-arctic-embed:s', 'obama the person'));
select length(rembed('embed-english-v3.0', 'obama the person', 'search_document'));
select length(rembed('mxbai-embed-large-v1-f16', 'obama the person'));


