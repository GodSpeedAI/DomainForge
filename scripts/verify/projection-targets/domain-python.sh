#!/usr/bin/env bash
# domain-python projection gate.
#
# Projects the projection-cell fixture to a complete Python DDD/CQRS package
# and validates:
#   1. Structural checks (file set, key classes/methods).
#   2. python -m compileall — every emitted .py byte-compiles.
#   3. mypy --strict — the package typechecks with zero edits (if mypy on PATH;
#      CI installs it; otherwise warn+skip, the emitted code is still strict).
#   4. python -m unittest — the smoke test exercises commands/events/VOs.
#
# Invoked by scripts/verify/projection-targets/all.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

FIXTURE="fixtures/projection_cell/basic/model.sea"
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

echo "==> project --format domain-python"
cargo run -q -p domainforge-core --features cli -- project --format domain-python \
  --created-at '2026-07-02T00:00:00+00:00' "$FIXTURE" "$OUT"

echo "==> validate generated package (structural)"
PKG="$OUT/src/procurement_domain"
test -s "$OUT/pyproject.toml"
test -s "$PKG/domain/aggregates.py"
test -s "$PKG/ports/repositories.py"
test -s "$PKG/domain/errors.py"
test -s "$OUT/tests/test_domain_smoke.py"
grep -q "class PurchaseOrderRepository(ABC)" "$PKG/ports/repositories.py"
grep -q "def transfer_to_supplier" "$PKG/domain/aggregates.py"
grep -q "class RequireApprovalViolation(DomainError)" "$PKG/domain/errors.py"
grep -q "dependencies = \[\]" "$OUT/pyproject.toml"
echo "  structural OK"

echo "==> python -m compileall"
python3 -m compileall -q "$OUT/src" "$OUT/tests"

if command -v mypy >/dev/null 2>&1; then
    echo "==> mypy --strict"
    (cd "$OUT" && mypy --strict --explicit-package-bases src)
else
    echo "==> WARNING: mypy not found — skipping strict typecheck (CI installs it)"
fi

echo "==> python -m unittest (smoke test)"
(cd "$OUT" && PYTHONPATH=src python3 -m unittest discover -s tests -p 'test_*.py')

echo "==> domain-python gate OK"
