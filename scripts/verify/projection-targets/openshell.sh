#!/usr/bin/env bash
# OpenShell projection gate.
#
# Projects the flagship repair-agent fixture to an OpenShell v0.1.3 policy
# (policy.yaml, sandbox.json, mapping-report.json, openshell.lock) and validates:
#   1. Byte-determinism: two runs without overrides, and two runs with
#      --overrides, are each identical (diff -r).
#   2. Containment: the real openshell-prover 0.1.3 reports the projected policy
#      as "within_boundary" against itself.
#   3. Differential/structural tests (cargo test --include-ignored), including
#      the prover-backed weakened-variant checks.
# openshell-prover 0.1.3 is required: the gate fails if it is missing or
# reports another version. Invoked by scripts/verify/projection-targets/all.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

FIXTURE="fixtures/cell_env/repair-agent/model.sea"
OVERRIDES="fixtures/cell_env/repair-agent/domainforge.cell.toml"
CREATED_AT="2026-07-11T00:00:00Z"
PROVER_VERSION="openshell-prover 0.1.3"
OUT1="$(mktemp -d)"
OUT2="$(mktemp -d)"
OUT3="$(mktemp -d)"
OUT4="$(mktemp -d)"
trap 'rm -rf "$OUT1" "$OUT2" "$OUT3" "$OUT4"' EXIT

echo "==> checking openshell-prover is on PATH (pinned 0.1.3)"
if ! command -v openshell-prover >/dev/null 2>&1; then
  echo "FAIL: openshell-prover is not on PATH; this gate requires '$PROVER_VERSION' (no skip)" >&2
  exit 1
fi
prover_version="$(openshell-prover --version || true)"
if [[ "$prover_version" != "$PROVER_VERSION" ]]; then
  echo "FAIL: openshell-prover reports '$prover_version'; this gate requires '$PROVER_VERSION'" >&2
  exit 1
fi
echo "OK: $prover_version"

echo "==> project --format openshell (run 1, no overrides)"
cargo run -q -p domainforge-core --features cli -- project --format openshell \
  --created-at "$CREATED_AT" "$FIXTURE" "$OUT1"

echo "==> project --format openshell (run 2, no overrides)"
cargo run -q -p domainforge-core --features cli -- project --format openshell \
  --created-at "$CREATED_AT" "$FIXTURE" "$OUT2"

echo "==> asserting byte-determinism (no overrides)"
diff -r "$OUT1" "$OUT2"
echo "OK: two runs are byte-identical"

echo "==> project --format openshell (run 3, with overrides)"
cargo run -q -p domainforge-core --features cli -- project --format openshell \
  --overrides "$OVERRIDES" --created-at "$CREATED_AT" "$FIXTURE" "$OUT3"

echo "==> project --format openshell (run 4, with overrides)"
cargo run -q -p domainforge-core --features cli -- project --format openshell \
  --overrides "$OVERRIDES" --created-at "$CREATED_AT" "$FIXTURE" "$OUT4"

echo "==> asserting byte-determinism (with overrides)"
diff -r "$OUT3" "$OUT4"
echo "OK: two override runs are byte-identical"

echo "==> openshell-prover check (containment against the projected boundary)"
for dir in "$OUT1" "$OUT3"; do
  report="$(cd "$dir" && openshell-prover check policy.yaml --boundary policy.yaml --output json || true)"
  printf '%s' "$report" | python3 -c '
import json, sys
doc = json.load(sys.stdin)
result = doc.get("result")
if result != "within_boundary":
    sys.exit(f"FAIL: openshell-prover result is {result!r}, expected \"within_boundary\"")
print(f"OK: openshell-prover result is within_boundary ({sys.argv[1]})")
' "$dir"
done

echo "==> differential and structural tests"
cargo test -p domainforge-core --features cli --test openshell_projection_tests -- --include-ignored

echo "==> openshell gate OK"
