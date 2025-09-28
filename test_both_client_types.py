#!/usr/bin/env python3
"""
Test both regular and multimodal client registration comprehensively.
"""

import sqlite3
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "bindings" / "python"))
import sqlite_rembed


def test_comprehensive():
    """Test all client registration methods."""
    print("\n" + "=" * 60)
    print("COMPREHENSIVE CLIENT REGISTRATION TEST")
    print("=" * 60)

    conn = sqlite3.connect(':memory:')
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    results = []

    # Test 1: Regular client via rembed_client_options
    print("\n1. Regular client via rembed_client_options()...")
    try:
        conn.execute("""
            INSERT INTO temp.rembed_clients(name, options)
            VALUES ('regular-opts', rembed_client_options(
                'format', 'openai',
                'model', 'text-embedding-3-small',
                'key', 'test-key'
            ))
        """)

        # Try to use it
        try:
            conn.execute("SELECT rembed('regular-opts', 'test')")
            print("✓ Regular client via options: WORKS")
            results.append(("regular-opts", True))
        except sqlite3.OperationalError as e:
            if "not registered" in str(e):
                print("✗ Regular client via options: NOT FOUND")
                results.append(("regular-opts", False))
            else:
                print(f"✓ Regular client via options: Found (API error: {str(e)[:30]}...)")
                results.append(("regular-opts", True))
    except Exception as e:
        print(f"✗ Failed to register: {e}")
        results.append(("regular-opts", False))

    # Test 2: Regular client via simple text format
    print("\n2. Regular client via simple text format...")
    try:
        conn.execute("""
            INSERT INTO temp.rembed_clients(name, options)
            VALUES ('regular-text', 'openai:test-key-123')
        """)

        try:
            conn.execute("SELECT rembed('regular-text', 'test')")
            print("✓ Regular client via text: WORKS")
            results.append(("regular-text", True))
        except sqlite3.OperationalError as e:
            if "not registered" in str(e):
                print("✗ Regular client via text: NOT FOUND")
                results.append(("regular-text", False))
            else:
                print(f"✓ Regular client via text: Found (API error: {str(e)[:30]}...)")
                results.append(("regular-text", True))
    except Exception as e:
        print(f"✗ Failed to register: {e}")
        results.append(("regular-text", False))

    # Test 3: Regular client via JSON format
    print("\n3. Regular client via JSON format...")
    try:
        conn.execute("""
            INSERT INTO temp.rembed_clients(name, options)
            VALUES ('regular-json', '{"provider": "openai", "model": "text-embedding-3-small", "api_key": "test-key"}')
        """)

        try:
            conn.execute("SELECT rembed('regular-json', 'test')")
            print("✓ Regular client via JSON: WORKS")
            results.append(("regular-json", True))
        except sqlite3.OperationalError as e:
            if "not registered" in str(e):
                print("✗ Regular client via JSON: NOT FOUND")
                results.append(("regular-json", False))
            else:
                print(f"✓ Regular client via JSON: Found (API error: {str(e)[:30]}...)")
                results.append(("regular-json", True))
    except Exception as e:
        print(f"✗ Failed to register: {e}")
        results.append(("regular-json", False))

    # Test 4: Multimodal client
    print("\n4. Multimodal client via rembed_client_options()...")
    try:
        conn.execute("""
            INSERT INTO temp.rembed_clients(name, options)
            VALUES ('multi-opts', rembed_client_options(
                'format', 'ollama',
                'model', 'llava:7b',
                'embedding_model', 'nomic-embed-text'
            ))
        """)

        # Tiny test image
        test_img = b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde'

        try:
            conn.execute("SELECT rembed_image('multi-opts', ?)", (test_img,))
            print("✓ Multimodal client: WORKS")
            results.append(("multi-opts", True))
        except sqlite3.OperationalError as e:
            if "not registered" in str(e):
                print("✗ Multimodal client: NOT FOUND")
                results.append(("multi-opts", False))
            else:
                print(f"✓ Multimodal client: Found (Other error: {str(e)[:30]}...)")
                results.append(("multi-opts", True))
    except Exception as e:
        print(f"✗ Failed to register: {e}")
        results.append(("multi-opts", False))

    # Summary
    print("\n" + "=" * 60)
    print("SUMMARY")
    print("-" * 60)

    # Show all registered clients
    all_clients = conn.execute("SELECT name FROM temp.rembed_clients").fetchall()
    print(f"Clients in virtual table: {[c[0] for c in all_clients]}")

    print("\nRegistration Results:")
    for name, success in results:
        status = "✓ WORKS" if success else "✗ BROKEN"
        print(f"  {name}: {status}")

    working = sum(1 for _, success in results if success)
    total = len(results)
    print(f"\nTotal: {working}/{total} working")

    return working == total


def main():
    """Run the test."""
    all_working = test_comprehensive()

    print("\n" + "=" * 60)
    if all_working:
        print("✅ ALL CLIENT REGISTRATIONS WORKING!")
    else:
        print("⚠️  SOME CLIENT REGISTRATIONS HAVE ISSUES")
        print("\nThe virtual table INSERT with text options works,")
        print("but rembed_client_options() pointer passing may have issues.")
    print("=" * 60)

    return 0 if all_working else 1


if __name__ == "__main__":
    sys.exit(main())