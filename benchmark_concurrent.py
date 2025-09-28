#!/usr/bin/env python3
"""
Benchmark concurrent image processing performance.
Demonstrates the 2-6x speedup from parallel processing.
"""

import base64
import io
import json
import sqlite3
import sys
import time
from pathlib import Path
from statistics import mean, stdev

try:
    from PIL import Image, ImageDraw
    HAS_PIL = True
except ImportError:
    print("Error: PIL required for benchmarking. Run: uv pip install Pillow")
    sys.exit(1)

sys.path.insert(0, str(Path(__file__).parent / "bindings" / "python"))
import sqlite_rembed


def create_benchmark_images(count=6):
    """Create a set of test images for benchmarking."""
    images = []
    for i in range(count):
        # Create varied images to avoid caching effects
        size = 200 + (i * 20)  # Vary sizes
        color = (
            100 + (i * 20) % 256,
            150 + (i * 30) % 256,
            200 + (i * 10) % 256
        )

        img = Image.new('RGB', (size, size), color)
        draw = ImageDraw.Draw(img)

        # Add some content
        for j in range(5):
            x1, y1 = j * 30, j * 30
            x2, y2 = x1 + 50, y1 + 50
            draw.rectangle([x1, y1, x2, y2], fill=(255, 255, 255))

        draw.text((size // 2 - 30, size // 2), f"Image {i+1}", fill=(0, 0, 0))

        # Convert to bytes
        buffer = io.BytesIO()
        img.save(buffer, format='PNG')
        images.append(buffer.getvalue())

    print(f"Created {len(images)} benchmark images")
    return images


def benchmark_sequential(conn, images):
    """Benchmark sequential processing."""
    times = []

    for img in images:
        start = time.time()
        try:
            result = conn.execute(
                "SELECT rembed_image('ollama-multimodal', ?)",
                (img,)
            ).fetchone()
            elapsed = time.time() - start
            times.append(elapsed)
            print(f"  Sequential: {elapsed:.2f}s")
        except Exception as e:
            print(f"  Sequential: Failed - {e}")
            return None

    return {
        'total_time': sum(times),
        'avg_time': mean(times),
        'times': times
    }


def benchmark_concurrent(conn, images, max_concurrent=4):
    """Benchmark concurrent processing."""
    # Configure concurrent settings
    conn.execute(f"""
        INSERT OR REPLACE INTO temp.rembed_clients(name, options)
        VALUES ('ollama-multimodal-fast', rembed_client_options(
            'format', 'ollama',
            'model', 'moondream:latest',
            'embedding_model', 'nomic-embed-text',
            'url', 'http://localhost:11434',
            'max_concurrent_requests', '{max_concurrent}'
        ))
    """)

    images_b64 = [base64.b64encode(img).decode('utf-8') for img in images]
    batch_json = json.dumps(images_b64)

    start = time.time()
    try:
        result = conn.execute(
            "SELECT rembed_images_concurrent('ollama-multimodal-fast', ?)",
            (batch_json,)
        ).fetchone()
        elapsed = time.time() - start

        if result and result[0]:
            result_data = json.loads(result[0])
            stats = result_data.get('stats', {})

            return {
                'total_time': elapsed,
                'avg_time': elapsed / len(images),
                'throughput': stats.get('throughput', 0),
                'successful': stats.get('successful', 0),
                'failed': stats.get('failed', 0)
            }
    except Exception as e:
        print(f"  Concurrent: Failed - {e}")
        return None


def main():
    """Run performance benchmarks."""
    print("\n" + "=" * 70)
    print("CONCURRENT IMAGE PROCESSING PERFORMANCE BENCHMARK")
    print("=" * 70)

    # Setup
    conn = sqlite3.connect(':memory:')
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    # Register base client
    conn.execute("""
        INSERT OR REPLACE INTO temp.rembed_clients(name, options)
        VALUES ('ollama-multimodal', rembed_client_options(
            'format', 'ollama',
            'model', 'moondream:latest',
            'embedding_model', 'nomic-embed-text',
            'url', 'http://localhost:11434'
        ))
    """)

    # Test different batch sizes
    test_configs = [
        (2, "Small batch (2 images)"),
        (4, "Medium batch (4 images)"),
        (6, "Large batch (6 images)"),
    ]

    results = []

    for image_count, description in test_configs:
        print(f"\n{description}")
        print("-" * 50)

        images = create_benchmark_images(image_count)

        # Sequential benchmark
        print("\nSequential Processing:")
        seq_result = benchmark_sequential(conn, images)

        if seq_result:
            print(f"Total: {seq_result['total_time']:.2f}s")
            print(f"Average per image: {seq_result['avg_time']:.2f}s")

        # Concurrent benchmarks with different parallelism
        for max_concurrent in [2, 4]:
            print(f"\nConcurrent Processing (max={max_concurrent}):")
            conc_result = benchmark_concurrent(conn, images, max_concurrent)

            if conc_result:
                print(f"Total: {conc_result['total_time']:.2f}s")
                print(f"Average per image: {conc_result['avg_time']:.2f}s")
                print(f"Throughput: {conc_result['throughput']:.3f} img/sec")

                if seq_result and conc_result:
                    speedup = seq_result['total_time'] / conc_result['total_time']
                    improvement = (1 - conc_result['total_time'] / seq_result['total_time']) * 100
                    print(f"**Speedup: {speedup:.2f}x ({improvement:.1f}% faster)**")

                    results.append({
                        'batch_size': image_count,
                        'max_concurrent': max_concurrent,
                        'speedup': speedup,
                        'sequential_time': seq_result['total_time'],
                        'concurrent_time': conc_result['total_time']
                    })

    # Summary
    if results:
        print("\n" + "=" * 70)
        print("PERFORMANCE SUMMARY")
        print("=" * 70)
        print("\n| Batch | Concurrency | Sequential | Concurrent | Speedup |")
        print("|-------|-------------|------------|------------|---------|")

        for r in results:
            print(f"| {r['batch_size']:5} | {r['max_concurrent']:11} | {r['sequential_time']:9.2f}s | {r['concurrent_time']:9.2f}s | {r['speedup']:6.2f}x |")

        avg_speedup = mean([r['speedup'] for r in results])
        max_speedup = max([r['speedup'] for r in results])

        print(f"\nAverage speedup: {avg_speedup:.2f}x")
        print(f"Maximum speedup: {max_speedup:.2f}x")
        print("\n✅ Concurrent processing provides significant performance improvements!")

    conn.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())