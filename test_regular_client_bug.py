#!/usr/bin/env python3
"""
Test to confirm the regular embedding client registration bug.
"""

import sqlite3
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "bindings" / "python"))
import sqlite_rembed


def test_regular_client_bug():
    """Test that regular embedding clients have registration issues."""
    print("\n" + "=" * 60)
    print("TESTING REGULAR EMBEDDING CLIENT REGISTRATION BUG")
    print("=" * 60)

    conn = sqlite3.connect(':memory:')
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    # Test 1: Register a regular embedding client
    print("\n1. Registering regular embedding client...")
    try:
        conn.execute("""
            INSERT INTO temp.rembed_clients(name, options)
            VALUES ('test-openai', rembed_client_options(
                'format', 'openai',
                'model', 'text-embedding-3-small',
                'key', 'test-key-123'
            ))
        """)
        print("✓ Client registered in virtual table")
    except Exception as e:
        print(f"✗ Failed to register: {e}")
        return False

    # Test 2: Check if client appears in virtual table
    print("\n2. Checking virtual table...")
    clients = conn.execute("SELECT name FROM temp.rembed_clients").fetchall()
    client_names = [c[0] for c in clients]
    print(f"✓ Clients in table: {client_names}")

    # Test 3: Try to use the client with rembed()
    print("\n3. Testing if rembed() can find the client...")
    try:
        result = conn.execute(
            "SELECT rembed('test-openai', 'Hello world')"
        ).fetchone()

        print("✓ Client found and working!")
        return True

    except sqlite3.OperationalError as e:
        error_msg = str(e)
        if "not registered" in error_msg:
            print(f"✗ BUG CONFIRMED: {error_msg}")
            print("\nThe client is in the virtual table but rembed() can't find it!")
            return False
        else:
            # Other errors (like API key issues) are OK
            print(f"✓ Client was found (API error expected: {error_msg[:50]}...)")
            return True

    # Test 4: Try with simple text options format
    print("\n4. Testing simple text format...")
    try:
        conn.execute("""
            INSERT INTO temp.rembed_clients(name, options)
            VALUES ('test-simple', 'openai:test-key-456')
        """)

        result = conn.execute(
            "SELECT rembed('test-simple', 'Test')"
        ).fetchone()

        print("✓ Simple format works!")

    except sqlite3.OperationalError as e:
        if "not registered" in str(e):
            print(f"✗ Simple format also broken: {str(e)[:50]}...")
        else:
            print(f"✓ Client found (other error: {str(e)[:30]}...)")

    return False


def main():
    """Run the test."""
    has_bug = not test_regular_client_bug()

    print("\n" + "=" * 60)
    if has_bug:
        print("❌ REGULAR CLIENT REGISTRATION BUG CONFIRMED!")
        print("\nRegular embedding clients registered via rembed_client_options()")
        print("are not accessible to rembed() function.")
        print("\nThis needs the same fix as multimodal clients:")
        print("- Detect regular clients properly")
        print("- Store them in the correct HashMap")
    else:
        print("✅ Regular client registration works!")
    print("=" * 60)

    return 0 if not has_bug else 1


if __name__ == "__main__":
    sys.exit(main())