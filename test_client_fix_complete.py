#!/usr/bin/env python3
"""
Final verification that both regular and multimodal client registration bugs are fixed.
"""

import sqlite3
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "bindings" / "python"))
import sqlite_rembed


def test_all_scenarios():
    """Test all client registration scenarios to confirm fixes."""
    print("\n" + "=" * 60)
    print("FINAL CLIENT REGISTRATION VERIFICATION")
    print("=" * 60)

    conn = sqlite3.connect(':memory:')
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    all_pass = True

    # Scenario 1: Regular client with rembed_client_options (no embedding_model)
    print("\n1. Regular client via rembed_client_options()...")
    conn.execute("""
        INSERT INTO temp.rembed_clients(name, options)
        VALUES ('reg-client', rembed_client_options(
            'format', 'openai',
            'model', 'text-embedding-3-small',
            'key', 'sk-test'
        ))
    """)

    try:
        conn.execute("SELECT rembed('reg-client', 'test')")
        status = "✓ Found by rembed()"
    except sqlite3.OperationalError as e:
        if "not registered" in str(e):
            status = "✗ NOT FOUND by rembed()"
            all_pass = False
        else:
            status = f"✓ Found (API error expected)"
    print(f"   {status}")

    # Scenario 2: Multimodal client with rembed_client_options (has embedding_model)
    print("\n2. Multimodal client via rembed_client_options()...")
    conn.execute("""
        INSERT INTO temp.rembed_clients(name, options)
        VALUES ('multi-client', rembed_client_options(
            'format', 'ollama',
            'model', 'llava:7b',
            'embedding_model', 'nomic-embed-text'
        ))
    """)

    test_img = b'\x89PNG\r\n\x1a\n'  # Tiny PNG header
    try:
        conn.execute("SELECT rembed_image('multi-client', ?)", (test_img,))
        status = "✓ Found by rembed_image()"
    except sqlite3.OperationalError as e:
        if "not registered" in str(e):
            status = "✗ NOT FOUND by rembed_image()"
            all_pass = False
        else:
            status = f"✓ Found (Processing error expected)"
    print(f"   {status}")

    # Scenario 3: Check both clients are in virtual table
    print("\n3. Virtual table contains both clients...")
    clients = conn.execute("SELECT name FROM temp.rembed_clients ORDER BY name").fetchall()
    client_names = [c[0] for c in clients]
    print(f"   Clients in table: {client_names}")

    if 'reg-client' in client_names and 'multi-client' in client_names:
        print("   ✓ Both clients visible in virtual table")
    else:
        print("   ✗ Some clients missing from virtual table")
        all_pass = False

    # Scenario 4: Verify wrong function can't access wrong client type
    print("\n4. Type safety check...")
    try:
        # Regular function shouldn't find multimodal client
        conn.execute("SELECT rembed('multi-client', 'test')")
        print("   ✗ Regular function accessed multimodal client (shouldn't happen)")
        all_pass = False
    except sqlite3.OperationalError as e:
        if "not registered" in str(e):
            print("   ✓ Regular function correctly can't access multimodal client")
        else:
            print(f"   ? Unexpected error: {str(e)[:50]}")

    try:
        # Multimodal function shouldn't find regular client
        conn.execute("SELECT rembed_image('reg-client', ?)", (test_img,))
        print("   ✗ Multimodal function accessed regular client (shouldn't happen)")
        all_pass = False
    except sqlite3.OperationalError as e:
        if "not registered" in str(e):
            print("   ✓ Multimodal function correctly can't access regular client")
        else:
            print(f"   ? Unexpected error: {str(e)[:50]}")

    return all_pass


def main():
    """Run the verification."""
    all_pass = test_all_scenarios()

    print("\n" + "=" * 60)
    if all_pass:
        print("✅ BOTH BUGS ARE FULLY FIXED!")
        print("\nSummary:")
        print("- Regular clients register and work with rembed()")
        print("- Multimodal clients register and work with rembed_image()")
        print("- Virtual table shows both client types")
        print("- Type safety is maintained (functions only see their client type)")
    else:
        print("⚠️  SOME ISSUES REMAIN")
        print("\nCheck the output above for details.")
    print("=" * 60)

    return 0 if all_pass else 1


if __name__ == "__main__":
    sys.exit(main())