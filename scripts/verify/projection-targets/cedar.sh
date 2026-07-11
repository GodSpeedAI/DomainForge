#!/usr/bin/env bash
# Cedar projection gate.
#
# Projects the projection-cell fixture to a Cedar JSON schema + policy set and
# validates:
#   - schema has the namespaced entityTypes + >=1 Action with
#     principalTypes/resourceTypes populated (H5: strict JSON, no comment header).
#   - policies.cedar carries one `permit` per action (H6: scoped to the flow's
#     source entity type + resource type, not unconstrained).
# Invoked by scripts/verify/projection-targets/all.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

FIXTURE="fixtures/projection_cell/basic/model.sea"
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

echo "==> project --format cedar"
cargo run -q -p domainforge-core --features cli -- project --format cedar \
  --created-at '2026-07-02T00:00:00+00:00' "$FIXTURE" "$OUT"

echo "==> validate schema.cedarschema.json + policies.cedar"
test -s "$OUT/schema.cedarschema.json"
test -s "$OUT/policies.cedar"
python3 - "$OUT/schema.cedarschema.json" "$OUT/policies.cedar" <<'PY'
import json, re, sys

schema_p, policies_p = sys.argv[1], sys.argv[2]

# --- schema: H5 — strict JSON (no comment header to skip).
doc = json.loads(open(schema_p).read())
assert "procurement" in doc, "namespace missing from schema"
ns = doc["procurement"]
assert "entityTypes" in ns and isinstance(ns["entityTypes"], dict), "missing entityTypes"
assert "actions" in ns and isinstance(ns["actions"], dict), "missing actions"
assert len(ns["entityTypes"]) > 0, "no entity types declared"
for t in ("Buyer", "Supplier", "Approver", "PurchaseOrder", "Payment"):
    assert t in ns["entityTypes"], f"entity type {t} missing"
assert len(ns["actions"]) > 0, "no actions emitted"
for aname, action in ns["actions"].items():
    assert aname.startswith("Issue"), f"action {aname} not named Issue<resource>"
    at = action["appliesTo"]
    assert at.get("principalTypes"), f"action {aname} has no principalTypes"
    assert at.get("resourceTypes"), f"action {aname} has no resourceTypes"
    assert "to" in action.get("annotations", {}), f"action {aname} missing 'to' annotation"

# --- policies: one permit per action, FQ action EIDs, H6 scoped permits.
policies = open(policies_p).read()
permit_count = len(re.findall(r"\bpermit\s*\(", policies))
assert permit_count == len(ns["actions"]), \
    f"permit count {permit_count} != action count {len(ns['actions'])}"
for aname in ns["actions"]:
    eid = f'procurement::Action::"{aname}"'
    assert eid in policies, f"action EID {eid} not referenced in policies"
# H6: each permit must scope principal and resource by type (not unconstrained).
assert "principal is procurement::" in policies, \
    "permits must scope principal to entity type (H6)"
assert "resource is procurement::" in policies, \
    "permits must scope resource to resource type (H6)"
print(f"  cedar OK ({len(ns['entityTypes'])} entity types, {len(ns['actions'])} action(s), {permit_count} scoped permit(s))")
PY

echo "==> cedar gate OK"
