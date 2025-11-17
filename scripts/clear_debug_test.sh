#!/usr/bin/env bash
set -euo pipefail
DEST="target/debug/deps/sea_debug_test"
if [ -L "$DEST" ] || [ -f "$DEST" ]; then
  rm -f "$DEST"
  echo "Removed debug test link: $DEST"
else
  echo "No debug test link found: $DEST"
fi
exit 0
