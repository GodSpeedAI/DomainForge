#!/usr/bin/env bash
# TLA+ projection gate.
#
# Projects the projection-cell fixture to a TLA+ spec (.tla) + TLC config
# (.cfg) and validates: module header, EXTENDS, CONSTANTS for entities+resources,
# a VARIABLES holder, Init, one Transfer<R> action per flow resource, a Next
# disjunction, Spec, and a TypeInvariant; plus a .cfg binding the CONSTANTS to
# model values and specifying Spec + TypeInvariant. Invoked by
# scripts/verify/projection-targets/all.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

FIXTURE="fixtures/projection_cell/basic/model.sea"
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

echo "==> project --format tla"
cargo run -q -p domainforge-core --features cli -- project --format tla \
  --created-at '2026-07-02T00:00:00+00:00' "$FIXTURE" "$OUT"

echo "==> validate *.tla + *.cfg"
TLA="$OUT/procurement.tla"
CFG="$OUT/procurement.cfg"
test -s "$TLA"
test -s "$CFG"
python3 - "$TLA" "$CFG" <<'PY'
import re, sys
tla = open(sys.argv[1]).read()
cfg = open(sys.argv[2]).read()

# --- .tla structural checks.
assert re.search(r"MODULE procurement\b", tla), "missing MODULE declaration"
assert "EXTENDS Naturals, Sequences, TLC" in tla, "missing EXTENDS"
assert "VARIABLES holder" in tla, "missing VARIABLES holder"
# CONSTANTS cover entities + resources.
for name in ("Buyer", "Supplier", "Approver", "PurchaseOrder", "Payment"):
    assert re.search(rf"\bCONSTANTS\b[^\n]*\b{name}\b", tla) or \
           re.search(rf"\b{name}\b.*\b{name}\b", tla[tla.find("CONSTANTS"):tla.find("\n", tla.find("CONSTANTS"))]) \
           or name in tla[tla.find("CONSTANTS"):tla.find("Entities")], f"constant {name} missing"
assert "Init ==" in tla, "missing Init"
assert "Spec == Init /\\ [][Next]_holder" in tla, "missing Spec"
assert "TypeInvariant ==" in tla, "missing TypeInvariant"
# One transfer action per flow resource (2 resources in fixture).
for r in ("PurchaseOrder", "Payment"):
    assert f"Transfer{r} ==" in tla, f"missing Transfer{r} action"
    assert re.search(rf"holder\[{r}\] = \w+", tla), f"missing holder[{r}] guard"
    assert re.search(rf"EXCEPT\s*!\[{r}\]\s*=", tla), f"missing EXCEPT ![{r}]"
# Next is a disjunction of the transfer actions (order-independent).
next_line = next((l for l in tla.splitlines() if l.startswith("Next == ")), None)
assert next_line is not None, "missing Next"
for r in ("PurchaseOrder", "Payment"):
    assert f"Transfer{r}" in next_line, f"Transfer{r} not in Next: {next_line}"
assert "\\/" in next_line, f"Next is not a disjunction: {next_line}"
# Module terminator.
assert "=====" in tla, "missing module terminator"

# --- .cfg structural checks.
assert re.search(r"^CONSTANTS\s*$", cfg, re.M), "cfg missing CONSTANTS"
for name in ("Buyer", "Supplier", "Approver", "PurchaseOrder", "Payment"):
    assert re.search(rf"^{name} = {name}\s*$", cfg, re.M), f"cfg missing constant binding {name}"
assert re.search(r"^SPECIFICATION Spec\s*$", cfg, re.M), "cfg missing SPECIFICATION"
assert re.search(r"^INVARIANT TypeInvariant\s*$", cfg, re.M), "cfg missing INVARIANT"
print("  tla+ OK (spec + cfg valid)")
PY

echo "==> tla gate OK"
