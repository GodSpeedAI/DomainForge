#!/usr/bin/env bash
# Prepare rust test binary and update launch.json program path
set -euo pipefail

if [ -z "${1-}" ]; then
  echo "Usage: $0 <test_name_prefix>"
  echo "Example: $0 entity_tests"
  exit 2
fi

PYTHON_BIN="${PYTHON_BIN:-python3}"
if ! command -v "${PYTHON_BIN}" >/dev/null 2>&1; then
  if command -v python >/dev/null 2>&1; then
    PYTHON_BIN="python"
  else
    echo "python3 (or python) is required to update launch.json" >&2
    exit 1
  fi
fi

TESTNAME=$1
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPTS_DIR="$ROOT_DIR/scripts"

echo "Preparing debug binary for: $TESTNAME"
pushd "$ROOT_DIR" > /dev/null
log_file="$(mktemp)"
trap 'rm -f "$log_file"' EXIT
if ! "$SCRIPTS_DIR/find_and_link_test_binary.sh" "$TESTNAME" | tee "$log_file"; then
  echo "Error: failed to locate or link debug binary" >&2
  exit 1
fi
link_path="$(grep -v '^[[:space:]]*$' "$log_file" | tail -n 1)"
if [ -z "${link_path}" ]; then
  echo "Error: linking script did not report the binary path" >&2
  exit 1
fi
if [ ! -e "${link_path}" ]; then
  echo "Error: linked binary not found at reported path: ${link_path}" >&2
  exit 1
fi
if [ ! -f "${link_path}" ] && [ ! -L "${link_path}" ]; then
  echo "Error: reported path is not a file or symlink: ${link_path}" >&2
  exit 1
fi
trap - EXIT
rm -f "$log_file"

# Update launch.json program entry to point to the stable symlink
"${PYTHON_BIN}" "$SCRIPTS_DIR/update_launch_program.py" --program "${link_path}"
echo "Prepared debug program and updated launch.json"
popd > /dev/null
