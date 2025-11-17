#!/usr/bin/env bash
# Prepare rust test binary and update launch.json program path
set -euo pipefail

if [ -z "${1-}" ]; then
  echo "Usage: $0 <test_name_prefix>"
  echo "Example: $0 entity_tests"
  exit 2
fi

TESTNAME=$1
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPTS_DIR="$ROOT_DIR/scripts"

echo "Building tests (no run) and preparing debug binary for: $TESTNAME"
pushd "$ROOT_DIR" > /dev/null
cargo test -p sea-core --no-run
"$SCRIPTS_DIR/find_and_link_test_binary.sh" "$TESTNAME"

# Update launch.json program entry to point to the stable symlink
python3 "$SCRIPTS_DIR/update_launch_program.py" --program "${ROOT_DIR}/target/debug/deps/sea_debug_test"
echo "Prepared debug program and updated launch.json"
popd > /dev/null
