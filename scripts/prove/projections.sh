#!/usr/bin/env bash
# prove-projections wrapper.
#
# Runs the authoritative all-target projection gate
# (scripts/verify/projection-targets/all.sh) UNCHANGED, then the projection
# contract parity check, and records a per-target evidence fragment derived
# from the contract files (status + validator + native flag). The gate is the
# source of truth for pass/fail; if all.sh exits nonzero this script aborts
# before writing a passing fragment.
#
# Writes evidence/latest/fragments/projections.json. Invoked by
# `just prove-projections`. Header conventions from tla.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

command -v jq >/dev/null 2>&1 || { echo "projections: jq is required" >&2; exit 1; }

FRAG_DIR="evidence/latest/fragments"
CONTRACT_DIR="scripts/verify/projection-targets/contracts"
mkdir -p "$FRAG_DIR"

echo "==> projection contract parity check"
bash scripts/prove/check-contracts.sh

echo "==> running all-target projection gate (all.sh)"
# all.sh is the single source of truth; do not duplicate its logic.
bash scripts/verify/projection-targets/all.sh
echo "==> all.sh passed"

# Detect whether the native TLA+ toolchain actually ran (Java + tla2tools.jar).
tla_native=false
if command -v java >/dev/null 2>&1; then
  if [[ -n "${TLA2TOOLS_JAR:-}" || -f "$REPO_ROOT/schemas/tla/tla2tools.jar" ]]; then
    tla_native=true
  fi
fi

# Build per-target projection_gates object from the gate scripts + contracts.
# Every gate that all.sh ran passed (all.sh is set -e), so passed=true here.
gates_json="{}"
for g in scripts/verify/projection-targets/*.sh; do
  name="$(basename "$g" .sh)"
  case "$name" in all|regenerate-fixtures) continue ;; esac
  contract="$CONTRACT_DIR/$name.yaml"
  validator="structural check only"
  native="false"
  if [[ -f "$contract" ]]; then
    validator="$(sed -n 's/^[[:space:]]*command:[[:space:]]*"\(.*\)"[[:space:]]*$/\1/p' "$contract" | head -1)"
    if grep -q 'native: true' "$contract"; then native="true"; fi
  fi
  # TLA+ native flag reflects whether the toolchain was actually present.
  if [[ "$name" == "tla" ]]; then native="$tla_native"; fi
  gates_json="$(jq \
    --arg n "$name" --arg v "$validator" --argjson nat "$native" \
    '. + {($n): {passed: true, validator: $v, native: $nat}}' <<<"$gates_json")"
done

jq -n --argjson gates "$gates_json" \
  '{projection_gates: $gates}' > "$FRAG_DIR/projections.json"

echo "==> prove-projections OK"
