#!/usr/bin/env bash
# prove-drift gate.
#
# Proves DomainForge detects declared-model drift through the typed semantic
# diff (`pack diff`), and does NOT report spurious drift when nothing changed:
#
#   MATCHING case: `pack diff A A'` on two INDEPENDENT rebuilds of the same
#     unmutated source reports an EMPTY typed diff and identical
#     meaning_fingerprints — the semantic-identity invariant, proven across
#     rebuilds now that Flow concept ids are content-addressable
#     (domainforge-core/src/primitives/flow.rs, CLAIM-PACK-FINGERPRINT proven).
#   DRIFT case: `pack diff A B`, where B renames a declared entity, reports a
#     NON-EMPTY typed diff containing a `breaking` classification, and the two
#     packs' meaning_fingerprints differ.
#
# Writes evidence/latest/fragments/drift.json. Invoked by `just prove-drift`.
# Header conventions from scripts/verify/projection-targets/tla.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

command -v jq >/dev/null 2>&1 || { echo "drift: jq is required" >&2; exit 1; }

CARGO_RUN=(cargo run -q -p domainforge-core --features cli --)
FRAG_DIR="evidence/latest/fragments"
mkdir -p "$FRAG_DIR"

FIXTURE="fixtures/projection_cell/basic/model.sea"
test -s "$FIXTURE" || { echo "drift: missing $FIXTURE" >&2; exit 1; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

cp "$FIXTURE" "$WORK/a.sea"
cp "$FIXTURE" "$WORK/a_prime.sea"
cp "$FIXTURE" "$WORK/b.sea"

# Semantic mutation: rename a declared entity actually present in the fixture.
grep -q 'Entity "Approver"' "$WORK/b.sea" || {
  echo "drift: expected entity 'Approver' not found in fixture; update the mutation" >&2
  exit 1
}
sed -i 's/Entity "Approver"/Entity "Reviewer"/' "$WORK/b.sea"

build() {
  "${CARGO_RUN[@]}" pack build --source "$1" --org acme --domain procurement \
    --version 1.0.0 --meaning-version 1.0.0 --out "$2" >/dev/null 2>&1
}

echo "==> pack build A + A' (independent rebuilds, unmutated) + B (Approver -> Reviewer)"
build "$WORK/a.sea" "$WORK/A.json"
build "$WORK/a_prime.sea" "$WORK/A_PRIME.json"
build "$WORK/b.sea" "$WORK/B.json"

FP_A="$(jq -r '.meaning_fingerprint' "$WORK/A.json")"
FP_A_PRIME="$(jq -r '.meaning_fingerprint' "$WORK/A_PRIME.json")"
FP_B="$(jq -r '.meaning_fingerprint' "$WORK/B.json")"

if [[ "$FP_A" != "$FP_A_PRIME" ]]; then
  echo "drift: two independent rebuilds of the same source produced different fingerprints ($FP_A vs $FP_A_PRIME)" >&2
  exit 1
fi

# NOTE: `pack diff` exits NONZERO when it detects breaking changes (that is a
# feature — it lets CI gate on drift). We therefore capture its exit code with
# `|| true` and assert on the JSON content, not the exit status.
echo "==> MATCHING case: pack diff A A' (independent rebuilds) must be empty"
"${CARGO_RUN[@]}" pack diff "$WORK/A.json" "$WORK/A_PRIME.json" --format json 2>/dev/null > "$WORK/diff_match.json" || true
MATCH_ENTRIES="$(jq -r '.entries | length' "$WORK/diff_match.json")"
matching_correct=false
if [[ "$MATCH_ENTRIES" == "0" ]]; then
  matching_correct=true
  echo "    matching diff empty (A vs independent rebuild A') OK"
else
  echo "drift: matching diff (A vs A') reported $MATCH_ENTRIES entries; expected 0" >&2
  jq -c '.entries' "$WORK/diff_match.json" >&2
  exit 1
fi

echo "==> DRIFT case: pack diff A B must be non-empty + breaking + fingerprints differ"
"${CARGO_RUN[@]}" pack diff "$WORK/A.json" "$WORK/B.json" --format json 2>/dev/null > "$WORK/diff_drift.json" || true
test -s "$WORK/diff_drift.json" || { echo "drift: pack diff produced no JSON output" >&2; exit 1; }
DRIFT_ENTRIES="$(jq -r '.entries | length' "$WORK/diff_drift.json")"
HAS_BREAKING="$(jq -r '[.entries[].classification] | any(. == "breaking")' "$WORK/diff_drift.json")"
drift_correct=false
if [[ "$DRIFT_ENTRIES" -gt 0 && "$HAS_BREAKING" == "true" && "$FP_A" != "$FP_B" ]]; then
  drift_correct=true
  echo "    drift diff non-empty ($DRIFT_ENTRIES entries, breaking present), fingerprints differ OK"
else
  echo "drift: expected non-empty breaking diff with differing fingerprints" >&2
  echo "    entries=$DRIFT_ENTRIES breaking=$HAS_BREAKING fp_equal=$([[ "$FP_A" == "$FP_B" ]] && echo yes || echo no)" >&2
  exit 1
fi

cat > "$FRAG_DIR/drift.json" <<JSON
{
  "drift": {
    "matching_diff_correct": $matching_correct,
    "drift_diff_correct": $drift_correct,
    "matching_diff_entries": $MATCH_ENTRIES,
    "drift_diff_entries": $DRIFT_ENTRIES,
    "mutation": "rename entity Approver -> Reviewer",
    "passed": true
  }
}
JSON

echo "==> prove-drift OK"
