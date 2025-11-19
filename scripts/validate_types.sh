#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "Verifying Supabase type artifacts are up-to-date..."
CHANGED_FILES=$(cd "$ROOT_DIR" && git status --porcelain libs/shared/types libs/shared/types-py || true)

if [ -n "$CHANGED_FILES" ]; then
  echo "Type artifacts are stale. Run 'just gen-types-ts' and 'just gen-types-py' and commit the results." >&2
  echo "$CHANGED_FILES"
  exit 1
fi

echo "Type artifacts are up-to-date."
