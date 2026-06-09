#!/usr/bin/env python3
"""Post-build patcher for napi-rs generated index.d.ts.

napi-rs auto-generates index.d.ts with two known issues that must be fixed
after each `napi build`:

1. `Self | null` — napi-rs generates `Self` for `Option<Self>` return types
   on class methods. TypeScript consumers need the concrete class name.
2. `function:` — Rust parameter names that are JS reserved words get emitted
   verbatim. We patch them to safe names.

This script is idempotent and safe to run multiple times.
"""

import re
import sys
from pathlib import Path

DECL_FILE = Path("index.d.ts")

# Patches: (pattern, replacement)
PATCHES = [
    # Fix `Self | null` → `NamespaceRegistry | null` in discover() return type
    (
        r"static discover\(path: string\): Self \| null",
        r"static discover(path: string): NamespaceRegistry | null",
    ),
    # Fix reserved-word `function:` param → `aggregateFunction:`
    (
        r"aggregationComprehension\(function:",
        r"aggregationComprehension(aggregateFunction:",
    ),
]


def patch_file(path: Path) -> int:
    if not path.exists():
        print(f"ERROR: {path} not found", file=sys.stderr)
        return 1

    original = path.read_text(encoding="utf-8")
    patched = original
    changes = 0

    for pattern, replacement in PATCHES:
        new_text = re.sub(pattern, replacement, patched)
        if new_text != patched:
            changes += 1
            patched = new_text

    if changes > 0:
        path.write_text(patched, encoding="utf-8")
        print(f"Patched {changes} issue(s) in {path}")
    else:
        print(f"No patches needed for {path}")

    return 0


if __name__ == "__main__":
    target = Path(sys.argv[1]) if len(sys.argv) > 1 else DECL_FILE
    sys.exit(patch_file(target))
