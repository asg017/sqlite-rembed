#!/usr/bin/env python3
"""
Quick benchmark to demonstrate concurrent processing improvements.
Uses tiny images for fast results.
"""

import base64
import json
import sqlite3
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "bindings" / "python"))
import sqlite_rembed


def main():
    print("\n" + "=" * 60)
    print("CONCURRENT PROCESSING QUICK BENCHMARK")
    print("=" * 60)

    # Tiny test images (1x1 pixel PNGs)
    tiny_images = [
        # Red pixel
        b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\x0cIDATx\x9cc\xf8\xcf\xc0\x00\x00\x00\x03\x00\x01^\xf6\x92\x87\x00\x00\x00\x00IEND\xaeB`\x82',
        # Green pixel
        b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\x0cIDATx\x9cc\x18\xf8\xcf\x00\x00\x00\x03\x00\x01\x9e\xf6R\x87\x00\x00\x00\x00IEND\xaeB`\x82',
        # Blue pixel
        b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\x0cIDATx\x9cc\x00\x00\xf8\x0f\x00\x00\x01\x01\x01\x00\x18\xdd\x8d\xb4\x00\x00\x00\x00IEND\xaeB`\x82',
    ] * 2  # Use 6 images total

    # Setup
    conn = sqlite3.connect(':memory:')
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    # Register multimodal client with moondream (smaller, faster)
    # Note: This creates a multimodal client, not a regular embedding client
    conn.execute("""
        INSERT OR REPLACE INTO temp.rembed_clients(name, options)
        VALUES ('ollama-multimodal', rembed_client_options(
            'format', 'ollama',
            'model', 'moondream:latest',
            'embedding_model', 'nomic-embed-text'
        ))
    """)

    print(f"\nTesting with {len(tiny_images)} tiny images...")
    print("-" * 40)

    # Sequential test (process first 3 only for speed)
    print("\n1. Sequential Processing (first 3 images):")
    seq_start = time.time()
    seq_count = 0

    for i, img in enumerate(tiny_images[:3]):
        try:
            img_start = time.time()
            result = conn.execute(
                "SELECT rembed_image('ollama-multimodal', ?)", (img,)
            ).fetchone()
            if result and result[0]:
                seq_count += 1
                print(f"   Image {i+1}: {time.time() - img_start:.2f}s ✓")
            else:
                print(f"   Image {i+1}: Failed")
        except Exception as e:
            print(f"   Image {i+1}: Error - {str(e)[:50]}")
            break

    seq_time = time.time() - seq_start
    print(f"   Total: {seq_time:.2f}s for {seq_count} images")

    # Concurrent test (all 6 images)
    print(f"\n2. Concurrent Processing (all {len(tiny_images)} images):")
    images_b64 = [base64.b64encode(img).decode('utf-8') for img in tiny_images]
    batch_json = json.dumps(images_b64)

    conc_start = time.time()
    try:
        result = conn.execute(
            "SELECT rembed_images_concurrent('ollama-multimodal', ?)",
            (batch_json,)
        ).fetchone()

        conc_time = time.time() - conc_start

        if result and result[0]:
            result_data = json.loads(result[0])
            stats = result_data.get('stats', {})
            successful = stats.get('successful', 0)
            failed = stats.get('failed', 0)
            throughput = stats.get('throughput', 0)

            print(f"   Successful: {successful}")
            print(f"   Failed: {failed}")
            print(f"   Total time: {conc_time:.2f}s")
            print(f"   Throughput: {throughput:.3f} img/sec")

            # Calculate improvement
            if seq_count > 0 and successful > 0:
                # Estimate sequential time for all images
                est_seq_time = (seq_time / seq_count) * len(tiny_images)
                speedup = est_seq_time / conc_time
                print(f"\n   🚀 Estimated speedup: {speedup:.2f}x faster!")
                print(f"   (Sequential would take ~{est_seq_time:.1f}s for {len(tiny_images)} images)")

    except Exception as e:
        print(f"   Error: {str(e)[:100]}")

    print("\n" + "=" * 60)
    print("✅ Benchmark complete!")
    print("=" * 60)

    conn.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())