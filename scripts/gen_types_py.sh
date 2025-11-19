#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIR="$ROOT_DIR/libs/shared/types-py/src"
SCHEMA_PATH="${SUPABASE_JSON_SCHEMA:-$ROOT_DIR/schemas/sea-registry.schema.json}"
PYTHON_BIN="${PYTHON_BIN:-}"

if [ -z "$PYTHON_BIN" ]; then
  if [ -x "$ROOT_DIR/.venv/bin/python" ]; then
    PYTHON_BIN="$ROOT_DIR/.venv/bin/python"
  elif command -v python3 >/dev/null 2>&1; then
    PYTHON_BIN="$(command -v python3)"
  elif command -v python >/dev/null 2>&1; then
    PYTHON_BIN="$(command -v python)"
  else
    echo "Python is required to generate Pydantic models." >&2
    exit 1
  fi
fi

if [ ! -f "$SCHEMA_PATH" ]; then
  cat >&2 <<MSG
Could not find a JSON schema to convert into Pydantic models.
Expected path: $SCHEMA_PATH
Provide SUPABASE_JSON_SCHEMA to point at a schema file exported from Supabase
(e.g. via supabase db dump) and rerun the command.
MSG
  exit 1
fi

mkdir -p "$OUTPUT_DIR"
TARGET_FILE="$OUTPUT_DIR/database_types.py"

set +e
"$PYTHON_BIN" -m datamodel_code_generator \
  --input "$SCHEMA_PATH" \
  --input-file-type jsonschema \
  --output "$TARGET_FILE" \
  --target-python-version 3.11 \
  --use-double-quotes \
  --disable-appending-item-suffix \
  --strict-nullable
STATUS=$?
set -e

if [ $STATUS -ne 0 ]; then
  cat >&2 <<MSG
Failed to run datamodel-code-generator. Ensure it is installed in your environment
(pip install datamodel-code-generator or include it in your virtualenv).
MSG
  exit $STATUS
fi

echo "Generated Python domain types at $TARGET_FILE"
