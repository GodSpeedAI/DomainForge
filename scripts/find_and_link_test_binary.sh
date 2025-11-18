#!/usr/bin/env bash
set -o errexit
set -o nounset
set -o pipefail
# Build the requested test binary, locate the hashed executable, and ensure
# a stable path (`target/debug/deps/sea_debug_test`) exists for codelldb.

if [ "$#" -lt 1 ]; then
  echo "Usage: $0 <test-file-stem-or-binary-name>"
  echo "Example: $0 aggregation_eval_tests"
  exit 2
fi

PYTHON_BIN="${PYTHON_BIN:-python3}"
if ! command -v "${PYTHON_BIN}" >/dev/null 2>&1; then
  if command -v python >/dev/null 2>&1; then
    PYTHON_BIN="python"
  else
    echo "python3 (or python) is required to locate the test binary" >&2
    exit 1
  fi
fi

TEST_NAME="$1"
CRATE_PATH="sea-core"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPTS_DIR="${ROOT_DIR}/scripts"
STABLE_DEST_DIR="${ROOT_DIR}/target/debug/deps"
DEST="${STABLE_DEST_DIR}/sea_debug_test"

echo "Building test binaries for crate: ${CRATE_PATH}"
pushd "${ROOT_DIR}" >/dev/null
cargo test -p "${CRATE_PATH}" --no-run

echo "Locating compiled binary for: ${TEST_NAME}"
FOUND_BIN="$(
  "${PYTHON_BIN}" "${SCRIPTS_DIR}/resolve_rust_binary.py" \
    --workspace "${ROOT_DIR}" \
    --name "${TEST_NAME}" \
    --profile debug \
    --match-mode prefix \
    --fallback-mode contains \
    --deps-subdir deps \
    --require-executable
)"
popd >/dev/null

if [ -z "${FOUND_BIN}" ]; then
  echo "Error: resolver did not return a binary path" >&2
  exit 1
fi

echo "Found binary: ${FOUND_BIN}"
mkdir -p "${STABLE_DEST_DIR}"
if [ -e "${DEST}" ] || [ -L "${DEST}" ]; then
  rm -f "${DEST}"
fi

echo "Linking ${FOUND_BIN} -> ${DEST}"
if ln -s "${FOUND_BIN}" "${DEST}" 2>/dev/null; then
  echo "Created symlink: ${DEST} -> ${FOUND_BIN}"
else
  cp "${FOUND_BIN}" "${DEST}"
  chmod +x "${DEST}"
  echo "Copied binary to: ${DEST}"
fi

echo "Prepared debug binary: ${DEST}"
printf '%s\n' "${DEST}"
exit 0
