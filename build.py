#!/usr/bin/env python3
"""
Build script for sqlite-rembed Rust extension.
This is called by the build backend (hatchling) during wheel creation.
"""

import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path


def get_platform_info():
    """Get platform-specific information for building."""
    system = platform.system()
    machine = platform.machine().lower()

    if system == "Linux":
        ext = "so"
        lib_prefix = "lib"
    elif system == "Darwin":
        ext = "dylib"
        lib_prefix = "lib"
    elif system == "Windows":
        ext = "dll"
        lib_prefix = ""
    else:
        raise RuntimeError(f"Unsupported platform: {system}")

    return {
        "system": system,
        "machine": machine,
        "ext": ext,
        "lib_prefix": lib_prefix,
        "rust_lib": f"{lib_prefix}sqlite_rembed.{ext}",
        "output_lib": f"rembed0.{ext}",
    }


def build_rust_extension(release=True):
    """Build the Rust extension using cargo."""
    print("Building Rust extension...")

    cmd = ["cargo", "build", "--verbose"]
    if release:
        cmd.append("--release")

    # Check if we're cross-compiling
    target = os.environ.get("CARGO_BUILD_TARGET")
    if target:
        cmd.extend(["--target", target])
        print(f"Cross-compiling for target: {target}")

    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"cargo build failed:\n{result.stderr}", file=sys.stderr)
        sys.exit(1)

    print("Rust extension built successfully")


def copy_extension_to_package():
    """Copy the built extension to the Python package directory."""
    platform_info = get_platform_info()

    # Determine source path
    target = os.environ.get("CARGO_BUILD_TARGET")
    if target:
        build_dir = Path("target") / target / "release"
    else:
        build_dir = Path("target") / "release"

    src_path = build_dir / platform_info["rust_lib"]

    # Destination path
    package_dir = Path("bindings") / "python" / "sqlite_rembed"
    package_dir.mkdir(parents=True, exist_ok=True)
    dst_path = package_dir / platform_info["output_lib"]

    # Copy the file
    if not src_path.exists():
        print(f"Error: Built library not found at {src_path}", file=sys.stderr)
        sys.exit(1)

    print(f"Copying {src_path} -> {dst_path}")
    shutil.copy2(src_path, dst_path)

    # Make executable on Unix-like systems
    if platform_info["system"] in ["Linux", "Darwin"]:
        os.chmod(dst_path, 0o755)

    return dst_path


def main():
    """Main build function."""
    # Check if we're in development mode
    is_dev = os.environ.get("SQLITE_REMBED_DEV", "").lower() in ["1", "true", "yes"]

    if is_dev:
        print("Building in development mode (debug build)")
        build_rust_extension(release=False)
    else:
        print("Building in release mode")
        build_rust_extension(release=True)

    # Copy to package
    output_path = copy_extension_to_package()
    print(f"✓ Extension available at: {output_path}")

    # Verify the extension can be loaded (basic sanity check)
    try:
        import sqlite3
        conn = sqlite3.connect(":memory:")
        conn.enable_load_extension(True)
        # Don't actually load it here, just verify the file exists
        if output_path.exists():
            print("✓ Extension file verified")
        conn.close()
    except Exception as e:
        print(f"Warning: Could not verify extension: {e}", file=sys.stderr)


if __name__ == "__main__":
    main()