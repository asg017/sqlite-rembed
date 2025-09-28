#!/usr/bin/env python3
"""
Test the user's specific test case for client registration.
"""

import sqlite3
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "bindings" / "python"))
import sqlite_rembed


def test_embedding_client_registration():
    """Test that should pass when bug is fixed"""
    conn = sqlite3.connect(':memory:')
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    # Register client
    conn.execute("""
        INSERT INTO temp.rembed_clients(name, options)
        VALUES ('test', 'mock::text')
    """)

    # This should not raise an error
    try:
        result = conn.execute("SELECT rembed('test', 'hello')").fetchone()
        if result is not None:
            print("✅ Bug is fixed! Result returned.")
        else:
            print("✅ Bug is fixed! (null result but no 'not registered' error)")
        return True
    except sqlite3.OperationalError as e:
        if "not registered" in str(e):
            print(f"❌ Bug still exists: {e}")
            return False
        else:
            # Other errors (like unsupported provider) are OK
            print(f"✅ Client found! (Provider error as expected: {str(e)[:60]}...)")
            return True


def test_various_formats():
    """Test various client option formats."""
    conn = sqlite3.connect(':memory:')
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    test_cases = [
        ('mock-simple', 'mock::text'),
        ('mock-with-key', 'mock:test-key-123'),
        ('unknown-provider', 'unknown::model'),
        ('custom-format', 'custom:key:with:colons'),
    ]

    results = []
    for name, options in test_cases:
        print(f"\nTesting: {name} with options '{options}'")

        # Register
        conn.execute(f"""
            INSERT INTO temp.rembed_clients(name, options)
            VALUES ('{name}', '{options}')
        """)

        # Try to use
        try:
            conn.execute(f"SELECT rembed('{name}', 'test')")
            print(f"  ✓ Found (would work with real provider)")
            results.append(True)
        except sqlite3.OperationalError as e:
            if "not registered" in str(e):
                print(f"  ✗ NOT FOUND - Bug exists!")
                results.append(False)
            else:
                print(f"  ✓ Found (error: {str(e)[:40]}...)")
                results.append(True)

    return all(results)


def main():
    """Run all tests."""
    print("=" * 60)
    print("USER'S SPECIFIC TEST CASE")
    print("=" * 60)

    user_test_passes = test_embedding_client_registration()

    print("\n" + "=" * 60)
    print("TESTING VARIOUS FORMATS")
    print("=" * 60)

    various_formats_pass = test_various_formats()

    print("\n" + "=" * 60)
    if user_test_passes and various_formats_pass:
        print("✅ ALL TESTS PASS - BUG IS FIXED!")
    else:
        print("❌ SOME TESTS FAIL - BUG MAY STILL EXIST")
    print("=" * 60)

    return 0 if (user_test_passes and various_formats_pass) else 1


if __name__ == "__main__":
    sys.exit(main())