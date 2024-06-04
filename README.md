# `sqlite-rembed`

A SQLite extension for generating text embedding from remote sources (llamafile, OpenAI, Cohere, etc.).

Work in progress!

| Client     | API Reference                                                              |
| ---------- | -------------------------------------------------------------------------- |
| OpenAI     | https://platform.openai.com/docs/guides/embeddings                         |
| Nomic      | https://docs.nomic.ai/reference/endpoints/nomic-embed-text                 |
| Cohere     | https://docs.cohere.com/reference/embed                                    |
| Jina       | https://api.jina.ai/redoc#tag/embeddings                                   |
| MixedBread | https://www.mixedbread.ai/api-reference#quick-start-guide                  |
| llamafile  | https://github.com/Mozilla-Ocho/llamafile                                  |
| Ollama     | https://github.com/ollama/ollama/blob/main/docs/api.md#generate-embeddings |

TODO

- [ ] Support Google AI API https://ai.google.dev/api/rest/v1beta/models/embedText
- [ ] Support text-embeddings-inference https://github.com/huggingface/text-embeddings-inference
- [ ] image embeddings support
- [ ] batch support
- [ ] extra params (X-Client-Name headers, truncation_strategy, input_type, etc.)
