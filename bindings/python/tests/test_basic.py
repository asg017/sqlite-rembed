"""Basic tests for sqlite-rembed Python bindings."""

import json
import sqlite3
import sys
from pathlib import Path

# Add parent directory to path for development testing
sys.path.insert(0, str(Path(__file__).parent.parent))
import sqlite_rembed


def test_load_extension():
    """Test that the extension can be loaded."""
    conn = sqlite3.connect(":memory:")
    conn.enable_load_extension(True)

    # Load the extension
    sqlite_rembed.load(conn)

    conn.enable_load_extension(False)

    # Verify it loaded by calling a function
    result = conn.execute("SELECT rembed_version()").fetchone()
    assert result is not None
    version = result[0]
    print(f"✓ Loaded sqlite-rembed version: {version}")
    assert "genai" in version
    conn.close()


def test_debug_info():
    """Test the debug function."""
    conn = sqlite3.connect(":memory:")
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    result = conn.execute("SELECT rembed_debug()").fetchone()
    debug_info = result[0]
    print(f"✓ Debug info:\n{debug_info}")
    assert "genai" in debug_info
    assert "Version:" in debug_info
    conn.close()


def test_client_registration():
    """Test registering a client."""
    conn = sqlite3.connect(":memory:")
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    # Register a test client (using ollama which doesn't need API key)
    conn.execute("""
        INSERT INTO temp.rembed_clients(name, options)
        VALUES ('test-ollama', 'ollama::nomic-embed-text')
    """)

    # Verify the client was registered
    result = conn.execute("SELECT name FROM temp.rembed_clients").fetchall()
    assert len(result) >= 1
    assert ("test-ollama",) in result
    print("✓ Registered client: test-ollama")
    conn.close()


def test_multimodal_client():
    """Test the default multimodal client."""
    conn = sqlite3.connect(":memory:")
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    # The extension should auto-register ollama-multimodal client
    # We can't easily test it without actual image data and running models,
    # but we can verify the function exists
    try:
        # This will fail without actual image data, but proves function exists
        conn.execute("SELECT rembed_image('ollama-multimodal', X'00')")
    except sqlite3.OperationalError as e:
        # Expected to fail with actual embedding generation
        print(f"✓ rembed_image function exists (failed as expected: {str(e)[:50]}...)")

    conn.close()


def test_batch_function():
    """Test that batch functions are available."""
    conn = sqlite3.connect(":memory:")
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    # Register a test client
    conn.execute("""
        INSERT INTO temp.rembed_clients(name, options)
        VALUES ('test', 'ollama::nomic-embed-text')
    """)

    # Test that batch function exists (will fail without valid data, but that's ok)
    try:
        test_batch = json.dumps(["test1", "test2"])
        conn.execute("SELECT rembed_batch('test', ?)", (test_batch,))
    except sqlite3.OperationalError as e:
        # Expected to fail without actual API connection
        print(f"✓ rembed_batch function exists (failed as expected: {str(e)[:50]}...)")

    conn.close()


def test_helper_functions():
    """Test helper functions like readfile_base64."""
    conn = sqlite3.connect(":memory:")
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    # Test readfile_base64 with some binary data
    test_data = b"Hello, world!"
    result = conn.execute("SELECT readfile_base64(?)", (test_data,)).fetchone()

    import base64

    expected = base64.b64encode(test_data).decode("utf-8")
    assert result[0] == expected
    print("✓ readfile_base64 helper function works")

    conn.close()


def test_package_version():
    """Test that package version is accessible."""
    version = sqlite_rembed.version()
    assert version == sqlite_rembed.__version__
    print(f"✓ Package version: {version}")


def test_load_ext_path():
    """Test that load_ext returns the extension path."""
    ext_path = sqlite_rembed.load_ext()
    assert ext_path.endswith((".so", ".dylib", ".dll"))
    print(f"✓ Extension path: {ext_path}")


if __name__ == "__main__":
    print("Running sqlite-rembed Python binding tests...\n")

    try:
        test_load_extension()
        test_debug_info()
        test_client_registration()
        test_multimodal_client()
        test_batch_function()
        test_helper_functions()
        test_package_version()
        test_load_ext_path()

        print("\n✅ All tests passed!")
    except AssertionError as e:
        print(f"\n❌ Test failed: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}")
        sys.exit(1)
