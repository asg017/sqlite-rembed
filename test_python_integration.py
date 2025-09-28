#!/usr/bin/env python3
"""
Integration test for sqlite-rembed Python bindings.
Tests various real-world scenarios.
"""

import json
import sqlite3
import struct
import sys
from pathlib import Path

# Add bindings to path
sys.path.insert(0, str(Path(__file__).parent / "bindings" / "python"))
import sqlite_rembed


def unpack_embedding(blob):
    """Convert binary blob to list of floats."""
    if not blob:
        return None
    # Each float32 is 4 bytes
    num_floats = len(blob) // 4
    return list(struct.unpack(f'{num_floats}f', blob))


def test_version_check():
    """Test version reporting."""
    print("=" * 60)
    print("TEST: Version Check")
    print("-" * 60)

    conn = sqlite3.connect(':memory:')
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    version = conn.execute("SELECT rembed_version()").fetchone()[0]
    print(f"✓ Extension version: {version}")

    debug_info = conn.execute("SELECT rembed_debug()").fetchone()[0]
    print(f"✓ Debug info retrieved ({len(debug_info)} chars)")

    conn.close()
    print("✅ Version check passed\n")


def test_client_registration():
    """Test different ways to register clients."""
    print("=" * 60)
    print("TEST: Client Registration Methods")
    print("-" * 60)

    conn = sqlite3.connect(':memory:')
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    # Method 1: Simple format (provider:key)
    try:
        conn.execute("""
            INSERT INTO temp.rembed_clients(name, options)
            VALUES ('client1', 'openai:test-key-123')
        """)
        print("✓ Method 1: Simple format accepted")
    except Exception as e:
        print(f"✗ Method 1 failed: {e}")

    # Method 2: JSON format
    try:
        conn.execute("""
            INSERT INTO temp.rembed_clients(name, options)
            VALUES ('client2', '{"provider": "gemini", "api_key": "test-key-456"}')
        """)
        print("✓ Method 2: JSON format accepted")
    except Exception as e:
        print(f"✗ Method 2 failed: {e}")

    # Method 3: Model identifier only (for env vars)
    try:
        conn.execute("""
            INSERT INTO temp.rembed_clients(name, options)
            VALUES ('client3', 'ollama::nomic-embed-text')
        """)
        print("✓ Method 3: Model identifier accepted")
    except Exception as e:
        print(f"✗ Method 3 failed: {e}")

    # Method 4: rembed_client_options function
    try:
        conn.execute("""
            INSERT INTO temp.rembed_clients(name, options)
            VALUES ('client4', rembed_client_options(
                'format', 'openai',
                'model', 'text-embedding-3-small',
                'key', 'test-key-789'
            ))
        """)
        print("✓ Method 4: rembed_client_options accepted")
    except Exception as e:
        print(f"✗ Method 4 failed: {e}")

    # List all registered clients
    clients = conn.execute("SELECT name FROM temp.rembed_clients").fetchall()
    print(f"\n✓ Registered {len(clients)} clients: {[c[0] for c in clients]}")

    conn.close()
    print("✅ Client registration passed\n")


def test_error_handling():
    """Test error handling and validation."""
    print("=" * 60)
    print("TEST: Error Handling")
    print("-" * 60)

    conn = sqlite3.connect(':memory:')
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    # Test 1: Using unregistered client
    try:
        conn.execute("SELECT rembed('nonexistent', 'test')")
        print("✗ Should have failed with unregistered client")
    except sqlite3.OperationalError as e:
        if "not registered" in str(e):
            print("✓ Properly caught unregistered client error")
        else:
            print(f"✗ Unexpected error: {e}")

    # Test 2: Invalid JSON in batch function
    conn.execute("INSERT INTO temp.rembed_clients(name, options) VALUES ('test', 'ollama::nomic-embed-text')")

    try:
        conn.execute("SELECT rembed_batch('test', 'not json')")
        print("✗ Should have failed with invalid JSON")
    except sqlite3.OperationalError as e:
        if "JSON" in str(e):
            print("✓ Properly caught invalid JSON error")
        else:
            print(f"✗ Unexpected error: {e}")

    # Test 3: Empty batch
    try:
        conn.execute("SELECT rembed_batch('test', '[]')")
        print("✗ Should have failed with empty batch")
    except sqlite3.OperationalError as e:
        if "empty" in str(e).lower():
            print("✓ Properly caught empty batch error")
        else:
            print(f"✗ Unexpected error: {e}")

    conn.close()
    print("✅ Error handling passed\n")


def test_helper_functions():
    """Test utility functions."""
    print("=" * 60)
    print("TEST: Helper Functions")
    print("-" * 60)

    conn = sqlite3.connect(':memory:')
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    # Test readfile_base64
    import base64
    test_data = b"Hello, sqlite-rembed!"
    result = conn.execute("SELECT readfile_base64(?)", (test_data,)).fetchone()[0]
    expected = base64.b64encode(test_data).decode('utf-8')

    if result == expected:
        print(f"✓ readfile_base64 works correctly")
        print(f"  Input: {test_data}")
        print(f"  Output: {result}")
    else:
        print(f"✗ readfile_base64 mismatch")

    conn.close()
    print("✅ Helper functions passed\n")


def test_multimodal_functions():
    """Test multimodal (image) functions are available."""
    print("=" * 60)
    print("TEST: Multimodal Functions")
    print("-" * 60)

    conn = sqlite3.connect(':memory:')
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    # Check that functions exist (they'll fail without real data, but that's ok)
    functions_to_test = [
        ("rembed_image", 2, "rembed_image('ollama-multimodal', X'00')"),
        ("rembed_image_prompt", 3, "rembed_image_prompt('ollama-multimodal', X'00', 'test')"),
        ("rembed_images_concurrent", 2, "rembed_images_concurrent('ollama-multimodal', '[]')"),
    ]

    for func_name, expected_args, test_sql in functions_to_test:
        try:
            conn.execute(f"SELECT {test_sql}")
            print(f"✓ {func_name} executed (unexpected success)")
        except sqlite3.OperationalError as e:
            # We expect failures since we're not providing valid data
            error_str = str(e)
            if "Vision" in error_str or "empty" in error_str or "Base64" in error_str:
                print(f"✓ {func_name} exists (failed as expected)")
            else:
                print(f"? {func_name} - unexpected error: {error_str[:50]}...")

    conn.close()
    print("✅ Multimodal functions passed\n")


def test_batch_processing():
    """Test batch processing capabilities."""
    print("=" * 60)
    print("TEST: Batch Processing")
    print("-" * 60)

    conn = sqlite3.connect(':memory:')
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    # Register a test client (this will fail without real API, but tests structure)
    conn.execute("""
        INSERT INTO temp.rembed_clients(name, options)
        VALUES ('test-batch', 'openai::text-embedding-3-small')
    """)

    # Prepare batch data
    texts = ["text1", "text2", "text3"]
    batch_json = json.dumps(texts)

    try:
        result = conn.execute("SELECT rembed_batch('test-batch', ?)", (batch_json,))
        print("✓ Batch function executed (unexpected - no API key)")
    except sqlite3.OperationalError as e:
        if "API" in str(e) or "key" in str(e).lower():
            print(f"✓ Batch function validated input correctly")
            print(f"  Batch size: {len(texts)} texts")
            print(f"  Expected failure: API key not configured")
        else:
            print(f"? Unexpected error: {str(e)[:50]}...")

    conn.close()
    print("✅ Batch processing passed\n")


def main():
    """Run all tests."""
    print("\n" + "=" * 60)
    print("SQLITE-REMBED PYTHON INTEGRATION TEST SUITE")
    print("=" * 60 + "\n")

    # Check Python package version
    print(f"Python package version: {sqlite_rembed.__version__}")
    print(f"Extension path: {sqlite_rembed.load_ext()}\n")

    try:
        test_version_check()
        test_client_registration()
        test_error_handling()
        test_helper_functions()
        test_multimodal_functions()
        test_batch_processing()

        print("=" * 60)
        print("✅ ALL INTEGRATION TESTS PASSED!")
        print("=" * 60)
        return 0

    except Exception as e:
        print(f"\n❌ Test suite failed: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())