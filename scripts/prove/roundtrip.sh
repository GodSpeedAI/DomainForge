#!/usr/bin/env bash
# prove roundtrip evidence collector.
#
# Records the round-trip proofs into an evidence fragment. The authoritative
# CALM round-trip gate is scripts/verify/projection-targets/roundtrip-cell.sh
# (run by prove-projections via all.sh); this script re-runs it for a captured
# pass/fail and additionally proves a KG round-trip:
#   project --format kg -> import --format kg, comparing entity/resource/flow
#   counts against the source graph.
#
# Writes evidence/latest/fragments/roundtrip.json. Invoked by prove-drift's
# recipe (co-located with the drift proof). Header conventions from tla.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

CARGO_RUN=(cargo run -q -p domainforge-core --features cli --)
FRAG_DIR="evidence/latest/fragments"
mkdir -p "$FRAG_DIR"

FIXTURE="fixtures/projection_cell/basic/model.sea"
test -s "$FIXTURE" || { echo "roundtrip: missing $FIXTURE" >&2; exit 1; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo "==> CALM round-trip (authoritative gate)"
calm_passed=false
if bash scripts/verify/projection-targets/roundtrip-cell.sh >/dev/null 2>&1; then
  calm_passed=true
  echo "    CALM round-trip OK"
else
  echo "roundtrip: CALM round-trip gate FAILED" >&2
  exit 1
fi

echo "==> KG round-trip (project kg -> import kg, count comparison)"
# Source counts from parse.
src="$("${CARGO_RUN[@]}" parse "$FIXTURE" 2>/dev/null)"
src_e="$(printf '%s\n' "$src" | sed -n 's/^Entities: //p')"
src_r="$(printf '%s\n' "$src" | sed -n 's/^Resources: //p')"
src_f="$(printf '%s\n' "$src" | sed -n 's/^Flows: //p')"

"${CARGO_RUN[@]}" project --format kg "$FIXTURE" "$WORK/out.ttl" >/dev/null
imp="$("${CARGO_RUN[@]}" import --format kg "$WORK/out.ttl" 2>&1)"
# Line form: "Imported KG (Turtle) to Graph: entities=3 resources=2 flows=2"
imp_e="$(printf '%s\n' "$imp" | sed -n 's/.*entities=\([0-9]*\).*/\1/p')"
imp_r="$(printf '%s\n' "$imp" | sed -n 's/.*resources=\([0-9]*\).*/\1/p')"
imp_f="$(printf '%s\n' "$imp" | sed -n 's/.*flows=\([0-9]*\).*/\1/p')"

kg_passed=false
if [[ "$src_e" == "$imp_e" && "$src_r" == "$imp_r" && "$src_f" == "$imp_f" ]]; then
  kg_passed=true
  echo "    KG round-trip counts match (entities=$src_e resources=$src_r flows=$src_f) OK"
else
  echo "roundtrip: KG count mismatch src(e=$src_e r=$src_r f=$src_f) vs import(e=$imp_e r=$imp_r f=$imp_f)" >&2
  exit 1
fi

cat > "$FRAG_DIR/roundtrip.json" <<JSON
{
  "roundtrip": {
    "calm": {"passed": $calm_passed},
    "kg": {
      "passed": $kg_passed,
      "entities": ${src_e:-0},
      "resources": ${src_r:-0},
      "flows": ${src_f:-0}
    }
  }
}
JSON

echo "==> prove roundtrip OK"
