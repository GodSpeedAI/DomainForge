#!/usr/bin/env bash
set -o errexit
set -o nounset
set -o pipefail
# This script builds tests (cargo test --no-run), finds a test binary that includes
# the given test file name (or test binary stem), and creates a stable symlink
# at target/debug/deps/sea_debug_test which can be used by debuggers.

if [ "$#" -lt 1 ]; then
  echo "Usage: $0 <test-file-stem-or-binary-name>"
  echo "Example: $0 aggregation_eval_tests"
  exit 2
fi

TEST_NAME="$1"
CRATE_PATH="sea-core"
BIN_DIR="target/debug/deps"
ESCAPED_TEST_NAME="$(
  python3 -c 'import re,sys; print(re.escape(sys.argv[1]))' -- "$TEST_NAME"
)"

echo "Building test binaries for crate: ${CRATE_PATH}"
cargo test -p "${CRATE_PATH}" --no-run

if [ ! -d "${BIN_DIR}" ]; then
  echo "Error: ${BIN_DIR} not found; make sure build succeeded" >&2
  exit 1
fi

echo "Searching for test binary matching: ${TEST_NAME}"
# Attempt to match the test binary by stem (binary names are typically 'testname-<hash>')
FOUND_BIN=$(ls "${BIN_DIR}" 2>/dev/null | grep -E "^${ESCAPED_TEST_NAME}(-[0-9a-f]+)?(\\.exe)?$" | head -n 1 || true)

if [ -z "${FOUND_BIN}" ]; then
  # Not found by exact match; try contains
  FOUND_BIN=$(ls "${BIN_DIR}" 2>/dev/null | grep -E "${ESCAPED_TEST_NAME}" | head -n 1 || true)
fi

if [ -z "${FOUND_BIN}" ]; then
  echo "Could not locate test binary in ${BIN_DIR} for test name: ${TEST_NAME}" >&2
  echo "Available candidates:" >&2
  ls -1 "${BIN_DIR}" >&2
  exit 1
fi

SRC="${BIN_DIR}/${FOUND_BIN}"
DEST="${BIN_DIR}/sea_debug_test"

echo "Found binary: ${SRC}"
# Remove old link if present
if [ -e "${DEST}" ] || [ -L "${DEST}" ]; then
  rm -f "${DEST}"
fi

echo "Linking ${SRC} -> ${DEST}"
# Prefer symlink; fallback to copy on Windows/other filesystems
if ln -s "${SRC}" "${DEST}" 2>/dev/null; then
  echo "Created symlink: ${DEST} -> ${SRC}"
else
  cp "${SRC}" "${DEST}"
  chmod +x "${DEST}"
  echo "Copied binary to: ${DEST}"
fi

echo "Prepared debug binary: ${DEST}"
printf '%s\n' "${DEST}"
exit 0
