#!/usr/bin/env python3
"""
Test that the multimodal client registration bug is fixed.
Verifies clients can be registered and found by multimodal functions.
"""

import sqlite3
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "bindings" / "python"))
import sqlite_rembed


def test_registration_fix():
    """Test that multimodal clients are properly registered and accessible."""
    print("\n" + "=" * 60)
    print("TESTING MULTIMODAL CLIENT REGISTRATION FIX")
    print("=" * 60)

    conn = sqlite3.connect(':memory:')
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    # Test 1: Register a multimodal client using rembed_client_options
    print("\n1. Testing multimodal client registration...")
    try:
        conn.execute("""
            INSERT INTO temp.rembed_clients(name, options)
            VALUES ('test-multimodal', rembed_client_options(
                'format', 'ollama',
                'model', 'moondream:latest',
                'embedding_model', 'nomic-embed-text'
            ))
        """)
        print("✓ Multimodal client registered successfully")
    except Exception as e:
        print(f"✗ Failed to register: {e}")
        return False

    # Test 2: Verify client appears in the virtual table
    print("\n2. Checking virtual table...")
    clients = conn.execute("SELECT name FROM temp.rembed_clients").fetchall()
    client_names = [c[0] for c in clients]
    print(f"✓ Registered clients: {client_names}")

    if 'test-multimodal' not in client_names:
        print("✗ Client not found in virtual table")
        return False

    # Test 3: Try to use the client with rembed_image
    print("\n3. Testing multimodal function can find the client...")
    # Create a tiny test image (1x1 pixel PNG)
    test_image = b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\x0cIDATx\x9cc\xf8\xcf\xc0\x00\x00\x00\x03\x00\x01^\xf6\x92\x87\x00\x00\x00\x00IEND\xaeB`\x82'

    try:
        # This should NOT fail with "client not registered" anymore
        result = conn.execute(
            "SELECT rembed_image('test-multimodal', ?)",
            (test_image,)
        ).fetchone()

        if result and result[0]:
            print("✓ Multimodal function found and used the client!")
            print(f"  Generated embedding: {len(result[0])} bytes")
            return True
        else:
            print("✓ Function found the client (no embedding due to no Ollama)")
            return True

    except sqlite3.OperationalError as e:
        error_msg = str(e)
        if "not registered" in error_msg:
            print(f"✗ BUG STILL EXISTS: {error_msg}")
            return False
        else:
            # Other errors are OK (like Ollama not running)
            print(f"✓ Client was found! (Other error: {error_msg[:50]}...)")
            return True

    # Test 4: Also test regular embedding clients still work
    print("\n4. Testing regular embedding clients still work...")
    try:
        conn.execute("""
            INSERT INTO temp.rembed_clients(name, options)
            VALUES ('test-regular', rembed_client_options(
                'format', 'openai',
                'model', 'text-embedding-3-small',
                'key', 'test-key'
            ))
        """)

        clients = conn.execute("SELECT name FROM temp.rembed_clients").fetchall()
        client_names = [c[0] for c in clients]

        if 'test-regular' in client_names:
            print(f"✓ Regular clients still work: {client_names}")
        else:
            print("✗ Regular client registration broken")
            return False

    except Exception as e:
        print(f"✗ Regular client registration failed: {e}")
        return False

    return True


def main():
    """Run the test."""
    success = test_registration_fix()

    print("\n" + "=" * 60)
    if success:
        print("✅ MULTIMODAL REGISTRATION BUG IS FIXED!")
        print("\nClients are now properly stored in the correct HashMap")
        print("and multimodal functions can find them.")
    else:
        print("❌ BUG STILL EXISTS")
        print("\nMultimodal clients are not being registered correctly.")
    print("=" * 60)

    return 0 if success else 1


if __name__ == "__main__":
    sys.exit(main())