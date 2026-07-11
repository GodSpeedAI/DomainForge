#!/usr/bin/env bash
# prove-evidence collector (runs LAST).
#
# Merges every evidence/latest/fragments/*.json fragment written by the earlier
# prove-* gates into a single machine-readable evidence/latest/proof.json, then
# renders a human-readable evidence/latest/proof.md. Cross-checks the claim
# statuses parsed from PROOFS.md and sets overall_result. Exits nonzero if any
# fragment reports a failure or a required fragment is missing.
#
# Requires jq. Header conventions from scripts/verify/projection-targets/tla.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

command -v jq >/dev/null 2>&1 || { echo "evidence: jq is required (install jq)" >&2; exit 1; }

EVID_DIR="evidence/latest"
FRAG_DIR="$EVID_DIR/fragments"
mkdir -p "$FRAG_DIR"

PROOF_JSON="$EVID_DIR/proof.json"
PROOF_MD="$EVID_DIR/proof.md"
PROOFS_MD="PROOFS.md"

REQUIRED=(language canonical projections roundtrip drift contracts)
for r in "${REQUIRED[@]}"; do
  if [[ ! -f "$FRAG_DIR/$r.json" ]]; then
    echo "evidence: required fragment $r.json is missing (did prove-$r run?)" >&2
    exit 1
  fi
done

# --- metadata -------------------------------------------------------------
VERSION="$(sed -n 's/^version[[:space:]]*=[[:space:]]*"\(.*\)".*/\1/p' domainforge-core/Cargo.toml | head -1)"
COMMIT="$(git rev-parse HEAD 2>/dev/null || echo unknown)"
GENERATED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
if [[ -z "$(git status --porcelain 2>/dev/null)" ]]; then CLEAN=true; else CLEAN=false; fi

# --- claim_status from PROOFS.md -----------------------------------------
# PROOFS.md rows: | CLAIM-ID | ... | status | ...  (status is column 3).
CLAIM_STATUS_JSON="$(
  awk -F'|' '
    /^\|[[:space:]]*CLAIM-/ {
      id=$2; st=$4;
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", id);
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", st);
      printf "%s\t%s\n", id, st;
    }
  ' "$PROOFS_MD" | jq -R -s '
    split("\n") | map(select(length>0)) |
    map(split("\t")) | map({(.[0]): .[1]}) | add // {}
  '
)"

# --- merge fragments ------------------------------------------------------
MERGED="$(jq -s 'reduce .[] as $f ({}; . * $f)' "$FRAG_DIR"/*.json)"

# --- overall result -------------------------------------------------------
# Passed iff every fragment that carries a `passed` boolean is true, and the
# projection gates all passed.
overall="passed"
if ! jq -e '
  (.language.passed == true) and
  (.canonical_determinism.passed == true) and
  (.drift.passed == true) and
  (.contracts.passed == true) and
  (.roundtrip.calm.passed == true) and
  (.roundtrip.kg.passed == true) and
  ([.projection_gates[].passed] | all)
' <<<"$MERGED" >/dev/null; then
  overall="failed"
fi

# --- assemble proof.json --------------------------------------------------
jq -n \
  --arg sv "1.0.0" \
  --arg ver "${VERSION:-unknown}" \
  --arg commit "$COMMIT" \
  --arg gen "$GENERATED_AT" \
  --argjson clean "$CLEAN" \
  --argjson merged "$MERGED" \
  --argjson claims "$CLAIM_STATUS_JSON" \
  --arg overall "$overall" \
  '{
     schema_version: $sv,
     domainforge_version: $ver,
     commit: $commit,
     generated_at: $gen,
     clean_worktree: $clean
   }
   + $merged
   + { claim_status: $claims, overall_result: $overall }' \
  > "$PROOF_JSON"

# --- render proof.md ------------------------------------------------------
{
  echo "# DomainForge Proof Report"
  echo
  echo "- Generated: $GENERATED_AT"
  echo "- Version: ${VERSION:-unknown}"
  echo "- Commit: \`$COMMIT\`"
  echo "- Clean worktree: $CLEAN"
  echo "- Overall result: **$overall**"
  echo
  echo "## Language"
  jq -r '.language | "- fixtures parsed: \(.fixtures_parsed)\n- fixtures validated: \(.fixtures_validated)\n- negative fixtures rejected: \(.negative_fixtures_rejected)\n- format round-trip idempotent (subset): \(.format_idempotent)"' "$PROOF_JSON"
  echo
  echo "## Canonical determinism"
  jq -r '.canonical_determinism | "- projection: \(.projection)\n- runs compared: \(.runs_compared)\n- byte-identical: \(.byte_identical)\n- graph hash (single build): \(.graph_hash)\n- note: \(.note)"' "$PROOF_JSON"
  echo
  echo "## Projection gates"
  echo
  echo "| Target | Passed | Native validator | Validator |"
  echo "|---|---|---|---|"
  jq -r '.projection_gates | to_entries[] | "| \(.key) | \(.value.passed) | \(.value.native) | \(.value.validator) |"' "$PROOF_JSON"
  echo
  echo "## Round-trip"
  jq -r '.roundtrip | "- CALM: \(.calm.passed)\n- KG (counts e/r/f): \(.kg.passed) (\(.kg.entities)/\(.kg.resources)/\(.kg.flows))"' "$PROOF_JSON"
  echo
  echo "## Drift (pack diff)"
  jq -r '.drift | "- matching diff correct (no spurious drift): \(.matching_diff_correct)\n- drift diff correct (\(.drift_diff_entries) typed entries): \(.drift_diff_correct)\n- mutation: \(.mutation)"' "$PROOF_JSON"
  echo
  echo "## Claim status (from PROOFS.md)"
  echo
  echo "| Claim | Status |"
  echo "|---|---|"
  jq -r '.claim_status | to_entries[] | "| \(.key) | \(.value) |"' "$PROOF_JSON"
  echo
  echo "## Reproduction"
  echo
  echo '```bash'
  echo "just prove"
  echo '```'
  echo
  echo "Regenerates this entire \`evidence/latest/\` directory from the current worktree."
} > "$PROOF_MD"

echo "==> wrote $PROOF_JSON and $PROOF_MD (overall_result=$overall)"

if [[ "$overall" != "passed" ]]; then
  echo "evidence: overall_result is '$overall'" >&2
  exit 1
fi
echo "==> prove-evidence OK"
