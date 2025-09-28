#!/usr/bin/env python3
"""
Test multimodal image embedding functionality with sqlite-rembed.
Creates test images and processes them through the hybrid LLaVA pipeline.
"""

import base64
import io
import json
import sqlite3
import sys
import time
from pathlib import Path

# Try to use PIL for image generation, fall back to simple test data if not available
try:
    from PIL import Image, ImageDraw, ImageFont
    HAS_PIL = True
except ImportError:
    print("Note: PIL not installed. Using pre-generated test images.")
    HAS_PIL = False

# Add bindings to path for development
sys.path.insert(0, str(Path(__file__).parent / "bindings" / "python"))
import sqlite_rembed


def create_test_images():
    """Create simple test images with text labels."""
    images = []

    if HAS_PIL:
        # Generate images with PIL
        colors = [
            ("red", (255, 100, 100)),
            ("green", (100, 255, 100)),
            ("blue", (100, 100, 255)),
            ("yellow", (255, 255, 100)),
            ("purple", (200, 100, 200)),
        ]

        for i, (color_name, rgb) in enumerate(colors, 1):
            # Create a simple image with colored background and text
            img = Image.new('RGB', (200, 200), rgb)
            draw = ImageDraw.Draw(img)

            # Draw some shapes
            draw.rectangle([50, 50, 150, 150], fill=(255, 255, 255))
            draw.text((70, 90), f"Image {i}\n{color_name}", fill=(0, 0, 0))

            # Convert to bytes
            buffer = io.BytesIO()
            img.save(buffer, format='PNG')
            images.append(buffer.getvalue())

        print(f"✓ Created {len(images)} test images with PIL")
    else:
        # Use tiny 1x1 pixel images as fallback
        # These are valid PNG files with single colored pixels
        tiny_pngs = [
            # Red pixel
            b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\x0cIDATx\x9cc\xf8\xcf\xc0\x00\x00\x00\x03\x00\x01^\xf6\x92\x87\x00\x00\x00\x00IEND\xaeB`\x82',
            # Green pixel
            b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\x0cIDATx\x9cc\x18\xf8\xcf\x00\x00\x00\x03\x00\x01\x9e\xf6R\x87\x00\x00\x00\x00IEND\xaeB`\x82',
            # Blue pixel
            b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\x0cIDATx\x9cc\x00\x00\xf8\x0f\x00\x00\x01\x01\x01\x00\x18\xdd\x8d\xb4\x00\x00\x00\x00IEND\xaeB`\x82',
        ]
        images = tiny_pngs[:3]
        print(f"✓ Using {len(images)} tiny test PNG images")

    return images


def test_basic_image_embedding(conn, images):
    """Test basic single image embedding."""
    print("\n" + "=" * 60)
    print("TEST: Basic Image Embedding")
    print("-" * 60)

    # Register the multimodal client if not already done
    # Using moondream for better stability (1B params vs 7B for llava)
    conn.execute("""
        INSERT OR REPLACE INTO temp.rembed_clients(name, options)
        VALUES ('ollama-multimodal', rembed_client_options(
            'format', 'ollama',
            'model', 'moondream:latest',
            'embedding_model', 'nomic-embed-text',
            'url', 'http://localhost:11434'
        ))
    """)

    # Test single image
    image_data = images[0]

    try:
        # Try to process the image
        print(f"Processing image (size: {len(image_data)} bytes)...")

        result = conn.execute(
            "SELECT rembed_image('ollama-multimodal', ?)",
            (image_data,)
        ).fetchone()

        if result and result[0]:
            embedding = result[0]
            print(f"✓ Generated embedding with {len(embedding)} bytes")

            # Check it's a valid float array (should be 768 dimensions for nomic)
            import struct
            num_floats = len(embedding) // 4
            floats = struct.unpack(f'{num_floats}f', embedding)
            print(f"✓ Embedding has {num_floats} dimensions")
            print(f"✓ Sample values: [{floats[0]:.4f}, {floats[1]:.4f}, {floats[2]:.4f}, ...]")
            return True
        else:
            print("✗ No embedding returned")
            return False

    except sqlite3.OperationalError as e:
        print(f"✗ Image embedding failed: {e}")
        return False


def test_batch_image_processing(conn, images):
    """Test batch processing of multiple images."""
    print("\n" + "=" * 60)
    print("TEST: Batch Image Processing")
    print("-" * 60)

    # Encode images as base64 for JSON transport
    images_b64 = [base64.b64encode(img).decode('utf-8') for img in images[:3]]
    batch_json = json.dumps(images_b64)

    try:
        print(f"Processing batch of {len(images_b64)} images...")
        start_time = time.time()

        result = conn.execute(
            "SELECT rembed_images_concurrent('ollama-multimodal', ?)",
            (batch_json,)
        ).fetchone()

        elapsed = time.time() - start_time

        if result and result[0]:
            result_data = json.loads(result[0])

            if 'embeddings' in result_data:
                embeddings = result_data['embeddings']
                stats = result_data.get('stats', {})

                print(f"✓ Processed {len(embeddings)} images in {elapsed:.2f}s")
                print(f"✓ Successful: {stats.get('successful', 'N/A')}")
                print(f"✓ Failed: {stats.get('failed', 'N/A')}")
                print(f"✓ Throughput: {stats.get('throughput', 'N/A')} img/sec")

                # Verify embeddings
                for i, emb_b64 in enumerate(embeddings):
                    if emb_b64:
                        emb = base64.b64decode(emb_b64)
                        print(f"  - Image {i+1}: {len(emb)} bytes")

                return True
            else:
                print(f"✗ Unexpected result format: {result_data}")
                return False
        else:
            print("✗ No result returned")
            return False

    except sqlite3.OperationalError as e:
        print(f"✗ Batch processing failed: {e}")
        return False


def test_image_with_prompt(conn, images):
    """Test image embedding with custom text prompt."""
    print("\n" + "=" * 60)
    print("TEST: Image with Custom Prompt")
    print("-" * 60)

    image_data = images[0]
    prompt = "Describe the colors and shapes in this image"

    try:
        print(f"Processing image with prompt: '{prompt}'")

        result = conn.execute(
            "SELECT rembed_image_prompt('ollama-multimodal', ?, ?)",
            (image_data, prompt)
        ).fetchone()

        if result and result[0]:
            embedding = result[0]
            print(f"✓ Generated embedding with custom prompt")
            print(f"✓ Embedding size: {len(embedding)} bytes")
            return True
        else:
            print("✗ No embedding returned")
            return False

    except sqlite3.OperationalError as e:
        print(f"✗ Image with prompt failed: {e}")
        return False


def test_performance_comparison(conn, images):
    """Compare sequential vs concurrent processing performance."""
    print("\n" + "=" * 60)
    print("TEST: Performance Comparison")
    print("-" * 60)

    if len(images) < 2:
        print("⚠ Need at least 2 images for performance comparison")
        return False

    test_images = images[:2]  # Use just 2 images for quick test

    # Sequential processing (one by one)
    print("\nSequential processing:")
    start_time = time.time()
    sequential_results = []

    for i, img in enumerate(test_images):
        try:
            result = conn.execute(
                "SELECT rembed_image('ollama-multimodal', ?)",
                (img,)
            ).fetchone()
            if result and result[0]:
                sequential_results.append(result[0])
                print(f"  - Image {i+1}: ✓")
            else:
                print(f"  - Image {i+1}: ✗")
        except Exception as e:
            print(f"  - Image {i+1}: ✗ ({e})")

    sequential_time = time.time() - start_time
    print(f"Sequential time: {sequential_time:.2f}s")

    # Concurrent processing
    print("\nConcurrent processing:")
    images_b64 = [base64.b64encode(img).decode('utf-8') for img in test_images]
    batch_json = json.dumps(images_b64)

    start_time = time.time()
    try:
        result = conn.execute(
            "SELECT rembed_images_concurrent('ollama-multimodal', ?)",
            (batch_json,)
        ).fetchone()

        concurrent_time = time.time() - start_time

        if result and result[0]:
            result_data = json.loads(result[0])
            concurrent_count = len(result_data.get('embeddings', []))
            print(f"  - Processed {concurrent_count} images concurrently")

        print(f"Concurrent time: {concurrent_time:.2f}s")

        if sequential_time > 0:
            speedup = sequential_time / concurrent_time
            print(f"\n✓ Speedup: {speedup:.2f}x faster with concurrent processing")

        return True

    except Exception as e:
        print(f"✗ Concurrent processing failed: {e}")
        return False


def main():
    """Run all multimodal tests."""
    print("\n" + "=" * 60)
    print("SQLITE-REMBED MULTIMODAL IMAGE TESTING")
    print("=" * 60)

    # Check if Ollama is accessible
    try:
        import urllib.request
        response = urllib.request.urlopen('http://localhost:11434/api/tags', timeout=2)
        if response.status != 200:
            print("⚠ Warning: Ollama may not be running properly")
    except Exception as e:
        print(f"⚠ Warning: Cannot connect to Ollama at localhost:11434")
        print(f"  Error: {e}")
        print("\nPlease ensure Ollama is running with:")
        print("  - LLaVA model: ollama pull llava")
        print("  - Embedding model: ollama pull nomic-embed-text")
        return 1

    # Create test images
    images = create_test_images()

    # Set up database
    conn = sqlite3.connect(':memory:')
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    # Get version info
    version = conn.execute("SELECT rembed_version()").fetchone()[0]
    print(f"\nExtension version: {version}")

    # Run tests
    tests_passed = 0
    tests_total = 4

    if test_basic_image_embedding(conn, images):
        tests_passed += 1

    if test_batch_image_processing(conn, images):
        tests_passed += 1

    if test_image_with_prompt(conn, images):
        tests_passed += 1

    if test_performance_comparison(conn, images):
        tests_passed += 1

    # Summary
    print("\n" + "=" * 60)
    if tests_passed == tests_total:
        print(f"✅ ALL {tests_total} MULTIMODAL TESTS PASSED!")
    else:
        print(f"⚠ {tests_passed}/{tests_total} tests passed")
    print("=" * 60)

    conn.close()
    return 0 if tests_passed == tests_total else 1


if __name__ == "__main__":
    sys.exit(main())