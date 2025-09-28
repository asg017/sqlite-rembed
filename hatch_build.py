"""
Hatchling build hook for sqlite-rembed.
This integrates the Rust build process with Python packaging.
"""

import subprocess
import sys
from pathlib import Path

from hatchling.builders.hooks.plugin.interface import BuildHookInterface


class RustExtensionBuildHook(BuildHookInterface):
    """Build hook to compile Rust extension during wheel building."""

    PLUGIN_NAME = "rust-extension"

    def initialize(self, version, build_data):
        """Initialize the build hook and compile the Rust extension."""
        print("Initializing Rust extension build...")

        # Run our build script
        result = subprocess.run(
            [sys.executable, "build.py"],
            capture_output=True,
            text=True,
        )

        if result.returncode != 0:
            print(f"Build failed:\n{result.stderr}", file=sys.stderr)
            raise RuntimeError("Failed to build Rust extension")

        print(result.stdout)

        # Ensure the extension is included in the wheel
        package_dir = Path("bindings/python/sqlite_rembed")
        for ext_file in package_dir.glob("rembed0.*"):
            if ext_file.suffix in [".so", ".dylib", ".dll"]:
                # Add to wheel artifacts
                rel_path = ext_file.relative_to("bindings/python")
                build_data["artifacts"].append(str(ext_file))
                print(f"Added artifact: {rel_path}")