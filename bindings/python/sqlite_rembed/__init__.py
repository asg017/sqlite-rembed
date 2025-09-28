"""
sqlite-rembed: Generate text and image embeddings from remote APIs inside SQLite

A SQLite extension that provides embedding generation from 10+ AI providers including
OpenAI, Gemini, Anthropic, Ollama, and more.

Usage:
    import sqlite3
    import sqlite_rembed

    # Load the extension
    conn = sqlite3.connect(':memory:')
    conn.enable_load_extension(True)
    sqlite_rembed.load(conn)
    conn.enable_load_extension(False)

    # Configure API clients
    conn.execute('''
        INSERT INTO temp.rembed_clients(name, options) VALUES
            ('openai', 'openai:YOUR_API_KEY'),
            ('ollama', 'ollama::nomic-embed-text')
    ''')

    # Generate embeddings
    result = conn.execute("SELECT rembed('openai', 'Hello world')").fetchone()
"""

import platform
import sqlite3
from pathlib import Path
from typing import Optional

__version__ = "0.0.1a9"


def _find_extension() -> str:
    """Find the appropriate extension file for the current platform."""

    # Determine file extension based on OS
    system = platform.system()
    machine = platform.machine().lower()

    if system == "Linux":
        ext_name = "rembed0.so"
    elif system == "Darwin":  # macOS
        ext_name = "rembed0.dylib"
    elif system == "Windows":
        ext_name = "rembed0.dll"
    else:
        raise RuntimeError(f"Unsupported platform: {system}")

    # Look for the extension in the package directory
    package_dir = Path(__file__).parent
    ext_path = package_dir / ext_name

    if not ext_path.exists():
        # Try platform-specific subdirectory (for multi-platform wheels)
        platform_dir = f"{system.lower()}-{machine}"
        ext_path = package_dir / platform_dir / ext_name

        if not ext_path.exists():
            raise FileNotFoundError(
                f"Could not find {ext_name} for {system} {machine}. "
                f"Please ensure you have the correct platform-specific wheel installed."
            )

    return str(ext_path)


def load(conn: sqlite3.Connection, path: Optional[str] = None) -> None:
    """
    Load the sqlite-rembed extension into a SQLite connection.

    Args:
        conn: An open SQLite database connection
        path: Optional path to the extension file. If not provided,
              will attempt to find the bundled extension automatically.

    Example:
        >>> import sqlite3
        >>> import sqlite_rembed
        >>> conn = sqlite3.connect(':memory:')
        >>> conn.enable_load_extension(True)
        >>> sqlite_rembed.load(conn)
        >>> conn.enable_load_extension(False)
        >>> version = conn.execute("SELECT rembed_version()").fetchone()[0]
        >>> print(f"Loaded sqlite-rembed {version}")
    """
    if path is None:
        path = _find_extension()

    try:
        conn.load_extension(path)
    except sqlite3.OperationalError as e:
        if "not authorized" in str(e):
            raise RuntimeError(
                "Cannot load extension. Please call conn.enable_load_extension(True) first."
            ) from e
        raise


def load_ext(path: Optional[str] = None) -> str:
    """
    Return the path to the sqlite-rembed extension file.

    This is useful if you need to load the extension using a different method
    or want to know where the extension file is located.

    Args:
        path: Optional path to the extension file. If not provided,
              will attempt to find the bundled extension automatically.

    Returns:
        The full path to the extension file.

    Example:
        >>> import sqlite_rembed
        >>> ext_path = sqlite_rembed.load_ext()
        >>> print(f"Extension located at: {ext_path}")
    """
    if path is None:
        path = _find_extension()
    return path


# Convenience function for version checking
def version() -> str:
    """Return the version of the Python package."""
    return __version__
