# sqlite-rembed Documentation

Welcome to the sqlite-rembed documentation! This directory contains comprehensive guides, technical details, and reference materials for using and understanding sqlite-rembed.

## 📚 Documentation Structure

### 🎯 [User Guides](./guides/)
Practical guides for using sqlite-rembed features:

- **[API Key Configuration Guide](./guides/API_KEY_GUIDE.md)** - Four flexible methods to configure API keys
- **[Batch Processing Guide](./guides/BATCH_PROCESSING.md)** - Process thousands of texts with 100-1000x performance improvements
- **[Concurrent Processing Guide](./guides/CONCURRENT_PROCESSING.md)** - High-performance parallel image processing (2-6x faster)
- **[Hybrid Multimodal Implementation](./guides/HYBRID_MULTIMODAL_IMPLEMENTATION.md)** - Image embeddings using LLaVA → text → embedding approach

### 🔧 [Technical Documentation](./technical/)
Implementation details and migration information:

- **[GenAI Migration](./technical/GENAI_MIGRATION.md)** - Complete migration from custom HTTP clients to genai
- **[GenAI Benefits](./technical/GENAI_BENEFITS.md)** - Why genai transformed sqlite-rembed
- **[Migration Summary](./technical/MIGRATION_SUMMARY.md)** - Executive summary of the transformation
- **[Fork Update Summary](./technical/FORK_UPDATE_SUMMARY.md)** - Updates from rsp2k/rust-genai fork
- **[Fork Integration Complete](./technical/FORK_INTEGRATION_COMPLETE.md)** - Latest performance improvements integrated

### 📖 [Reference](./reference/)
Background information and issue tracking:

- **[Issues Resolved](./reference/ISSUES_RESOLVED.md)** - How genai migration addressed all open issues
- **[LLaVA and Multimodal](./reference/LLAVA_AND_MULTIMODAL.md)** - Understanding vision models vs embeddings

## 🚀 Quick Start

New to sqlite-rembed? Start here:

1. **Installation**: See the main [README](../README.md#installation)
2. **Basic Usage**: Configure API keys with the [API Key Guide](./guides/API_KEY_GUIDE.md)
3. **Performance**: Learn about [Batch Processing](./guides/BATCH_PROCESSING.md) for 100x improvements
4. **Advanced**: Explore [Concurrent Processing](./guides/CONCURRENT_PROCESSING.md) for maximum speed

## 📊 Feature Comparison

| Feature | Before Migration | After Migration | Documentation |
|---------|-----------------|-----------------|---------------|
| **Providers** | 7 hardcoded | 10+ automatic | [GenAI Benefits](./technical/GENAI_BENEFITS.md) |
| **Batch Processing** | Not supported | 100-1000x faster | [Batch Guide](./guides/BATCH_PROCESSING.md) |
| **Image Embeddings** | Not supported | Hybrid approach | [Multimodal Guide](./guides/HYBRID_MULTIMODAL_IMPLEMENTATION.md) |
| **Concurrent Processing** | Sequential only | 2-6x faster | [Concurrent Guide](./guides/CONCURRENT_PROCESSING.md) |
| **Code Complexity** | 795 lines | 160 lines | [Migration Summary](./technical/MIGRATION_SUMMARY.md) |

## 🎯 Common Use Cases

### Text Embeddings
```sql
-- Single text
SELECT rembed('openai', 'Hello world');

-- Batch processing (100x faster)
SELECT rembed_batch('openai', json_array('text1', 'text2', 'text3'));
```

### Image Embeddings
```sql
-- Single image
SELECT rembed_image('ollama-multimodal', readfile('image.jpg'));

-- Concurrent batch (4x faster)
SELECT rembed_images_concurrent('ollama-multimodal',
    json_array(readfile_base64(readfile('img1.jpg')), ...));
```

## 📈 Performance Benchmarks

| Processing Type | Method | Speed | Best For |
|----------------|--------|-------|----------|
| **Text Batch** | `rembed_batch()` | 100-1000x faster | Large text datasets |
| **Image Sequential** | `rembed_image()` | Baseline | Single images |
| **Image Concurrent** | `rembed_images_concurrent()` | 2-6x faster | Multiple images |

## 🔗 External Resources

- [sqlite-vec](https://github.com/asg017/sqlite-vec) - Vector similarity search
- [rust-genai](https://github.com/jeremychone/rust-genai) - Unified AI client library
- [rsp2k/rust-genai fork](https://github.com/rsp2k/rust-genai) - Multimodal enhancements

## 📝 Contributing

Documentation improvements are welcome! When adding new docs:

1. Place user-facing guides in `docs/guides/`
2. Put technical details in `docs/technical/`
3. Add reference materials to `docs/reference/`
4. Update this index with your new documentation

## 📜 License

This documentation is part of the sqlite-rembed project and follows the same license.