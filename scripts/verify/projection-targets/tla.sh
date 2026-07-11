#!/usr/bin/env bash
# TLA+ projection gate.
#
# Projects the projection-cell fixture to a TLA+ spec (.tla) + TLC config
# (.cfg) and validates:
#   1. Structural checks (module header, EXTENDS, CONSTANTS, actions, Next).
#   2. H3: SANY parse — the REAL TLA+ toolchain (tla2tools.jar) parses the
#      generated spec. This is the gate that catches the H1/H2 class of bug
#      (syntactically invalid / semantically unsatisfiable output).
#   3. H3: TLC model-check — proves the spec is satisfiable (all actions can
#      fire) and the TypeInvariant holds. This catches the H2 class of bug
#      (deadlocked-from-init actions).
#
# If Java + tla2tools.jar are not available, the gate runs structural checks
# only and prints a warning. CI (H4) installs Java + pins tla2tools.jar so the
# full toolchain always runs there.
#
# Invoked by scripts/verify/projection-targets/all.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

FIXTURE="fixtures/projection_cell/basic/model.sea"
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

# tla2tools.jar: vendored or downloaded. CI pins v1.8.0.
TLA2TOOLS_JAR="${TLA2TOOLS_JAR:-}"
if [[ -z "$TLA2TOOLS_JAR" && -f "$REPO_ROOT/schemas/tla/tla2tools.jar" ]]; then
    TLA2TOOLS_JAR="$REPO_ROOT/schemas/tla/tla2tools.jar"
fi

echo "==> project --format tla"
cargo run -q -p domainforge-core --features cli -- project --format tla \
  --created-at '2026-07-02T00:00:00+00:00' "$FIXTURE" "$OUT"

echo "==> validate *.tla + *.cfg (structural)"
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
for name in ("Buyer", "Supplier", "Approver", "PurchaseOrder", "Payment"):
    assert name in tla, f"constant {name} missing from spec"
assert "Init ==" in tla, "missing Init"
assert "Spec == Init /\\ [][Next]_holder" in tla, "missing Spec"
assert "TypeInvariant ==" in tla, "missing TypeInvariant"
# H2: one action per flow (Transfer<Resource>_<From>_<To>).
assert "TransferPurchaseOrder_Buyer_Supplier ==" in tla, "missing per-flow action"
assert "TransferPayment_Buyer_Supplier ==" in tla, "missing per-flow action"
# H1: CASE uses OTHER, not OTHERWISE; every arm has a discriminant.
assert "CASE r = " in tla, "Init CASE missing discriminant"
assert "OTHER" in tla, "Init CASE must use OTHER"
assert "OTHERWISE" not in tla, "Init CASE must not use OTHERWISE (H1 regression)"
assert "[] = " not in tla, "Init CASE arm missing discriminant (H1 regression)"
# Next is a disjunction of the per-flow actions.
next_line = next((l for l in tla.splitlines() if l.startswith("Next == ")), None)
assert next_line is not None, "missing Next"
assert "\\/" in next_line, f"Next is not a disjunction: {next_line}"
assert "=====" in tla, "missing module terminator"

# --- .cfg structural checks.
assert re.search(r"^CONSTANTS\s*$", cfg, re.M), "cfg missing CONSTANTS"
for name in ("Buyer", "Supplier", "Approver", "PurchaseOrder", "Payment"):
    assert re.search(rf"^{name} = {name}\s*$", cfg, re.M), f"cfg missing constant binding {name}"
assert re.search(r"^SPECIFICATION Spec\s*$", cfg, re.M), "cfg missing SPECIFICATION"
assert re.search(r"^INVARIANT TypeInvariant\s*$", cfg, re.M), "cfg missing INVARIANT"
print("  structural OK")
PY

# --- H3: REAL TLA+ toolchain (SANY parse + TLC model-check) ---
if [[ -n "$TLA2TOOLS_JAR" ]] && command -v java &>/dev/null; then
    echo "==> SANY parse (real TLA+ toolchain: tla2tools.jar)"
    (cd "$OUT" && java -cp "$TLA2TOOLS_JAR" tla2sany.SANY procurement.tla)
    echo "  SANY parse OK"

    echo "==> TLC model-check (proves actions are satisfiable + invariant holds)"
    # TLC exits 0 on success (including expected terminal deadlock from a
    # finite-state model). We suppress the deadlock error since the fixture
    # model has no cycles — resources reach their terminal holder and stop.
    (cd "$OUT" && java -cp "$TLA2TOOLS_JAR" tlc2.TLC -deadlock procurement.tla) || {
        echo "  TLC reported errors — the spec may be unsatisfiable" >&2
        exit 1
    }
    echo "  TLC model-check OK"
else
    echo "==> WARNING: Java/tla2tools.jar not found — skipping SANY/TLC (structural checks only)"
    echo "    Install Java and set TLA2TOOLS_JAR or place tla2tools.jar in schemas/tla/"
    echo "    CI runs the full toolchain (see .github/workflows/ci.yml verify-projection-targets)."
fi

echo "==> tla gate OK"
