#!/usr/bin/env bash
# Alloy projection gate.
#
# Projects the projection-cell fixture to an Alloy model (.als) and validates:
# the file has a module declaration, abstract Principal/Resource sigs, an
# entity sig extending Principal for each SEA entity, a resource sig extending
# Resource for each SEA resource, a Flow sig with resource/from/to relations,
# one fact per declared flow, and a `run` command. Invoked by
# scripts/verify/projection-targets/all.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

FIXTURE="fixtures/projection_cell/basic/model.sea"
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

echo "==> project --format alloy"
cargo run -q -p domainforge-core --features cli -- project --format alloy \
  --created-at '2026-07-02T00:00:00+00:00' "$FIXTURE" "$OUT"

echo "==> validate *.als"
ALS="$OUT/procurement.als"
test -s "$ALS"
python3 - "$ALS" <<'PY'
import re, sys
text = open(sys.argv[1]).read()

assert re.search(r"^module procurement\b", text, re.M), "missing module declaration"
assert "abstract sig Principal {}" in text, "missing abstract Principal sig"
assert "abstract sig Resource {}" in text, "missing abstract Resource sig"
assert "sig Flow {" in text, "missing Flow sig"
assert re.search(r"resource:\s*one\s+Resource", text), "Flow.resource missing multiplicity"
assert re.search(r"from,\s*to:\s*one\s+Principal", text), "Flow.from/to missing multiplicity"

# Each entity appears in an `extends Principal` sig line.
for ent in ("Buyer", "Supplier", "Approver"):
    assert re.search(rf"\b{ent}\b[^{{]*extends Principal", text), f"entity {ent} not a Principal subtype"
# Each resource appears in an `extends Resource` sig line.
for res in ("PurchaseOrder", "Payment"):
    assert re.search(rf"\b{res}\b[^{{]*extends Resource", text), f"resource {res} not a Resource subtype"

# One `fact flow_...` per declared flow (fixture has 2).
fact_count = len(re.findall(r"^fact flow_\w+", text, re.M))
assert fact_count == 2, f"expected 2 flow facts, got {fact_count}"
# Each fact asserts some Flow with resource/from/to in the right subtypes.
for clause in ("f.resource in PurchaseOrder", "f.from in Buyer", "f.to in Supplier"):
    assert clause in text, f"flow fact missing clause: {clause}"
assert re.search(r"^run\s*\{\s*\}\s*for\s+\d+", text, re.M), "missing run command"
print(f"  alloy OK ({fact_count} flow fact(s))")
PY

echo "==> alloy gate OK"
