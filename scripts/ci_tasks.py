#!/usr/bin/env python3
"""
CI Tasks Script - Cross-platform CI automation helpers.

This script consolidates common CI tasks to reduce duplication across
GitHub Actions workflows and ensure consistency between local and CI environments.
"""

from __future__ import annotations

import argparse
import os
import platform
import shutil
import subprocess
import sys
import tarfile
import zipfile
from pathlib import Path
from typing import Optional


def check_size(file_path: str, max_bytes: int, label: str = "File") -> int:
    """
    Check if a file's size is within the specified limit.

    Args:
        file_path: Path to the file to check
        max_bytes: Maximum allowed size in bytes
        label: Human-readable label for error messages

    Returns:
        0 if size is within limit, 1 otherwise
    """
    path = Path(file_path)

    if not path.exists():
        print(f"::error::File not found: {file_path}", file=sys.stderr)
        return 1

    if not path.is_file():
        print(f"::error::Not a file: {file_path}", file=sys.stderr)
        return 1

    size = path.stat().st_size
    size_mb = size / (1024 * 1024)
    max_mb = max_bytes / (1024 * 1024)

    print(f"{label} size: {size:,} bytes ({size_mb:.2f} MB)")

    if size > max_bytes:
        print(
            f"::error::{label} exceeds {max_bytes:,} bytes ({max_mb:.2f} MB): "
            f"actual size is {size:,} bytes ({size_mb:.2f} MB)",
            file=sys.stderr,
        )
        return 1

    print(f"✓ {label} is within size limit ({max_mb:.2f} MB)")
    return 0


def package_archive(
    input_path: str,
    output_path: str,
    archive_format: Optional[str] = None,
    base_name: Optional[str] = None,
) -> int:
    """
    Package a file or directory into an archive.

    Args:
        input_path: Path to file or directory to package
        output_path: Output archive path
        archive_format: 'zip' or 'tar.gz' (auto-detected from output_path if not specified)
        base_name: Base name for the archived file (defaults to input filename)

    Returns:
        0 on success, 1 on failure
    """
    input_p = Path(input_path)
    output_p = Path(output_path)

    if not input_p.exists():
        print(f"::error::Input path not found: {input_path}", file=sys.stderr)
        return 1

    # Auto-detect format from extension
    if archive_format is None:
        if output_path.endswith(".zip"):
            archive_format = "zip"
        elif output_path.endswith(".tar.gz") or output_path.endswith(".tgz"):
            archive_format = "tar.gz"
        else:
            print(
                f"::error::Cannot determine archive format from: {output_path}",
                file=sys.stderr,
            )
            return 1

    # Determine the name to use inside the archive
    if base_name is None:
        base_name = input_p.name

    try:
        output_p.parent.mkdir(parents=True, exist_ok=True)

        if archive_format == "zip":
            with zipfile.ZipFile(output_p, "w", zipfile.ZIP_DEFLATED) as zf:
                if input_p.is_file():
                    zf.write(input_p, base_name)
                else:
                    for file in input_p.rglob("*"):
                        if file.is_file():
                            arcname = base_name / file.relative_to(input_p)
                            zf.write(file, arcname)
            print(f"✓ Created ZIP archive: {output_p}")

        elif archive_format == "tar.gz":
            with tarfile.open(output_p, "w:gz") as tf:
                if input_p.is_file():
                    tf.add(input_p, arcname=base_name)
                else:
                    tf.add(input_p, arcname=base_name)
            print(f"✓ Created tar.gz archive: {output_p}")

        else:
            print(
                f"::error::Unsupported archive format: {archive_format}",
                file=sys.stderr,
            )
            return 1

        return 0

    except Exception as e:
        print(f"::error::Failed to create archive: {e}", file=sys.stderr)
        return 1


def verify_cli_binary(binary_path: str, expected_output: Optional[str] = None) -> int:
    """
    Verify that a CLI binary can execute successfully.

    Args:
        binary_path: Path to the binary to test
        expected_output: Optional substring to check in output

    Returns:
        0 if binary runs successfully, 1 otherwise
    """
    path = Path(binary_path)

    if not path.exists():
        print(f"::error::Binary not found: {binary_path}", file=sys.stderr)
        return 1

    if not path.is_file():
        print(f"::error::Not a file: {binary_path}", file=sys.stderr)
        return 1

    # Make executable on Unix-like systems
    if platform.system() != "Windows":
        path.chmod(path.stat().st_mode | 0o111)

    # Set up environment for dynamic libraries
    env = os.environ.copy()
    bin_dir = str(path.parent.absolute())

    if platform.system() == "Darwin":
        dyld_path = env.get("DYLD_LIBRARY_PATH", "")
        env["DYLD_LIBRARY_PATH"] = f"{bin_dir}:{dyld_path}" if dyld_path else bin_dir
    elif platform.system() == "Linux":
        ld_path = env.get("LD_LIBRARY_PATH", "")
        env["LD_LIBRARY_PATH"] = f"{bin_dir}:{ld_path}" if ld_path else bin_dir
    elif platform.system() == "Windows":
        path_env = env.get("PATH", "")
        env["PATH"] = f"{bin_dir};{path_env}" if path_env else bin_dir

    try:
        print(f"Running: {binary_path} --version")
        result = subprocess.run(
            [str(path), "--version"],
            capture_output=True,
            text=True,
            env=env,
            timeout=10,
        )

        if result.returncode != 0:
            print(
                f"::error::Binary exited with code {result.returncode}", file=sys.stderr
            )
            print(f"stdout: {result.stdout}", file=sys.stderr)
            print(f"stderr: {result.stderr}", file=sys.stderr)
            return 1

        output = result.stdout + result.stderr
        print("✓ Binary executed successfully")
        print(f"Output: {output.strip()}")

        if expected_output and expected_output not in output:
            print(
                f"::error::Expected output '{expected_output}' not found in: {output}",
                file=sys.stderr,
            )
            return 1

        return 0

    except subprocess.TimeoutExpired:
        print("::error::Binary execution timed out", file=sys.stderr)
        return 1
    except Exception as e:
        print(f"::error::Failed to execute binary: {e}", file=sys.stderr)
        return 1


def unpack_and_verify(archive_path: str, binary_name: str) -> int:
    """
    Unpack an archive and verify the binary inside can execute.

    Args:
        archive_path: Path to the archive
        binary_name: Name of the binary to verify

    Returns:
        0 on success, 1 on failure
    """
    archive_p = Path(archive_path)

    if not archive_p.exists():
        print(f"::error::Archive not found: {archive_path}", file=sys.stderr)
        return 1

    # Create temp directory for unpacking
    temp_dir = Path("tmp_verify_unpack")
    temp_dir.mkdir(exist_ok=True)

    try:
        # Unpack
        if archive_path.endswith(".zip"):
            with zipfile.ZipFile(archive_p, "r") as zf:
                zf.extractall(temp_dir)
        elif archive_path.endswith(".tar.gz") or archive_path.endswith(".tgz"):
            with tarfile.open(archive_p, "r:gz") as tf:
                tf.extractall(temp_dir)
        else:
            print(
                f"::error::Unsupported archive format: {archive_path}", file=sys.stderr
            )
            return 1

        print(f"✓ Unpacked archive to {temp_dir}")

        # Find the binary
        binary_path = None
        for file in temp_dir.rglob(binary_name):
            if file.is_file():
                binary_path = file
                break

        # Also check with .exe extension on Windows
        if binary_path is None and platform.system() == "Windows":
            for file in temp_dir.rglob(f"{binary_name}.exe"):
                if file.is_file():
                    binary_path = file
                    break

        if binary_path is None:
            print(
                f"::error::Binary '{binary_name}' not found in archive", file=sys.stderr
            )
            print("Archive contents:", file=sys.stderr)
            for file in temp_dir.rglob("*"):
                print(f"  {file.relative_to(temp_dir)}", file=sys.stderr)
            return 1

        print(f"Found binary: {binary_path}")

        # Verify it runs
        return verify_cli_binary(str(binary_path))

    finally:
        # Cleanup
        if temp_dir.exists():
            shutil.rmtree(temp_dir, ignore_errors=True)


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="CI tasks automation script",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    subparsers = parser.add_subparsers(dest="command", help="Command to run")

    # check-size command
    size_parser = subparsers.add_parser("check-size", help="Check file size")
    size_parser.add_argument("--file", required=True, help="File to check")
    size_parser.add_argument(
        "--max-bytes", type=int, required=True, help="Maximum size in bytes"
    )
    size_parser.add_argument(
        "--label", default="File", help="Label for output messages"
    )

    # package command
    package_parser = subparsers.add_parser("package", help="Package files into archive")
    package_parser.add_argument(
        "--input", required=True, help="Input file or directory"
    )
    package_parser.add_argument("--output", required=True, help="Output archive path")
    package_parser.add_argument(
        "--format", choices=["zip", "tar.gz"], help="Archive format"
    )
    package_parser.add_argument("--base-name", help="Base name for archived content")

    # verify-cli command
    verify_parser = subparsers.add_parser(
        "verify-cli", help="Verify CLI binary executes"
    )
    verify_parser.add_argument("--binary", required=True, help="Path to binary")
    verify_parser.add_argument("--expected-output", help="Expected substring in output")

    # unpack-verify command
    unpack_parser = subparsers.add_parser(
        "unpack-verify", help="Unpack archive and verify binary"
    )
    unpack_parser.add_argument("--archive", required=True, help="Archive path")
    unpack_parser.add_argument(
        "--binary-name", required=True, help="Binary name to verify"
    )

    args = parser.parse_args()

    if args.command == "check-size":
        return check_size(args.file, args.max_bytes, args.label)
    elif args.command == "package":
        return package_archive(args.input, args.output, args.format, args.base_name)
    elif args.command == "verify-cli":
        return verify_cli_binary(args.binary, args.expected_output)
    elif args.command == "unpack-verify":
        return unpack_and_verify(args.archive, args.binary_name)
    else:
        parser.print_help()
        return 1


if __name__ == "__main__":
    sys.exit(main())
