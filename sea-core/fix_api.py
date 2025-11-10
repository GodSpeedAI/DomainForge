#!/usr/bin/env python3
"""
DEPRECATED: This script has been replaced by fix_api_v2.py

This regex-based migration script is deprecated and should not be used.
Use fix_api_v2.py instead, which provides:
- Robust AST-based transformations using tree-sitter
- Safe import insertion
- Automatic backup/rollback
- Idempotent operations
- Better handling of edge cases (nested calls, multiline args, comments, etc.)

Usage:
    python fix_api_v2.py --dir <directory> --transform all
    python fix_api_v2.py --file <file> --transform entity
    python fix_api_v2.py --file <file> --dry-run

For more information, see fix_api_v2.py --help
"""
import sys


def main():
    print("ERROR: fix_api.py is deprecated and has been replaced by fix_api_v2.py")
    print()
    print("Please use fix_api_v2.py instead:")
    print("  python fix_api_v2.py --dir <directory> --transform all")
    print("  python fix_api_v2.py --file <file> --transform entity")
    print("  python fix_api_v2.py --file <file> --dry-run")
    print()
    print("For more information: python fix_api_v2.py --help")
    sys.exit(1)


if __name__ == "__main__":
    main()
