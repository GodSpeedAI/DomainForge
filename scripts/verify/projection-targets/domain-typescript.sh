#!/usr/bin/env bash
# domain-typescript projection gate.
#
# Projects the projection-cell fixture to a complete TypeScript DDD/CQRS package
# and validates:
#   1. Structural checks (file set, key interfaces/classes).
#   2. tsc --noEmit (strict) — the package typechecks with zero edits. Uses the
#      local typescript install if present, else `npx -y -p typescript tsc`; if
#      neither npx nor a network is available, warn+skip (CI always runs it).
#
# Invoked by scripts/verify/projection-targets/all.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

FIXTURE="fixtures/projection_cell/basic/model.sea"
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

echo "==> project --format domain-typescript"
cargo run -q -p domainforge-core --features cli -- project --format domain-typescript \
  --created-at '2026-07-02T00:00:00+00:00' "$FIXTURE" "$OUT"

echo "==> validate generated package (structural)"
test -s "$OUT/package.json"
test -s "$OUT/tsconfig.json"
test -s "$OUT/src/domain/aggregates.ts"
test -s "$OUT/src/ports/repositories.ts"
test -s "$OUT/src/domain/errors.ts"
test -s "$OUT/src/smoke.ts"
grep -q "interface PurchaseOrderRepository" "$OUT/src/ports/repositories.ts"
grep -q "class RequireApprovalViolation extends DomainError" "$OUT/src/domain/errors.ts"
! grep -q '"dependencies"' "$OUT/package.json"
echo "  structural OK"

# Pick a TypeScript compiler: local install > npx > skip.
TSC_BIN=""
if [ -x "$(command -v tsc)" ]; then
    TSC_BIN="tsc"
elif command -v npx >/dev/null 2>&1; then
    TSC_BIN="npx -y -p typescript tsc"
fi

if [ -n "$TSC_BIN" ]; then
    echo "==> tsc --noEmit (strict)"
    (cd "$OUT" && $TSC_BIN --noEmit -p .)
    echo "  tsc OK"
else
    echo "==> WARNING: tsc/npx not found — skipping strict typecheck (CI installs typescript)"
fi

echo "==> domain-typescript gate OK"
