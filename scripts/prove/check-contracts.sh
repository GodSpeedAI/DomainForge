#!/usr/bin/env bash
# Projection-contract parity check.
#
# Enforces that every projection gate script has a conformance contract and
# vice-versa:
#   * every scripts/verify/projection-targets/<target>.sh (excluding the
#     orchestrator all.sh and regenerate-fixtures.sh) has a matching
#     contracts/<target>.yaml;
#   * every contracts/<target>.yaml either has a matching gate script OR is
#     explicitly marked `gate_optional: true` (contract-only targets calm/kg/
#     protobuf that ship without a per-target gate script);
#   * every contract declares status / preserves / prohibited_losses /
#     target_validator.native.
#
# Fails nonzero on any mismatch. Writes evidence/latest/fragments/contracts.json.
# Header conventions from scripts/verify/projection-targets/tla.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

GATE_DIR="scripts/verify/projection-targets"
CONTRACT_DIR="$GATE_DIR/contracts"
FRAG_DIR="evidence/latest/fragments"
mkdir -p "$FRAG_DIR"

test -d "$CONTRACT_DIR" || { echo "contracts: missing $CONTRACT_DIR" >&2; exit 1; }

fail=0

# 1. Every gate script needs a contract.
gate_targets=()
for g in "$GATE_DIR"/*.sh; do
  name="$(basename "$g" .sh)"
  case "$name" in
    all|regenerate-fixtures) continue ;;
  esac
  gate_targets+=("$name")
  if [[ ! -f "$CONTRACT_DIR/$name.yaml" ]]; then
    echo "contracts: gate script $name.sh has NO contract ($name.yaml)" >&2
    fail=1
  fi
done

# 2. Every contract needs a gate script unless gate_optional: true.
contract_count=0
native_count=0
for c in "$CONTRACT_DIR"/*.yaml; do
  name="$(basename "$c" .yaml)"
  contract_count=$((contract_count + 1))

  # Required fields.
  for field in "target:" "status:" "preserves:" "prohibited_losses:" "native:"; do
    if ! grep -q "$field" "$c"; then
      echo "contracts: $name.yaml missing required field '$field'" >&2
      fail=1
    fi
  done

  if grep -q 'native: true' "$c"; then
    native_count=$((native_count + 1))
  fi

  if [[ -f "$GATE_DIR/$name.sh" ]]; then
    continue
  fi
  if grep -q 'gate_optional: true' "$c"; then
    continue
  fi
  echo "contracts: contract $name.yaml has NO gate script ($name.sh) and is not gate_optional" >&2
  fail=1
done

if [[ $fail -ne 0 ]]; then
  echo "contracts: parity check FAILED" >&2
  exit 1
fi

cat > "$FRAG_DIR/contracts.json" <<JSON
{
  "contracts": {
    "passed": true,
    "gate_scripts": ${#gate_targets[@]},
    "contracts": $contract_count,
    "native_validators_declared": $native_count
  }
}
JSON

echo "==> contract parity OK (${#gate_targets[@]} gates, $contract_count contracts)"
