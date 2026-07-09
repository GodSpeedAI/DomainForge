#!/usr/bin/env bash
# Devbox projection gate.
#
# Projects the projection-cell fixture to a devbox.json and validates that the
# JSON is well-formed (skipping the leading `//` comment) and carries the
# required devbox top-level fields plus the DomainForge manifest env vars
# (namespace + non-empty entities/resources). Invoked by
# scripts/verify/projection-targets/all.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

FIXTURE="fixtures/projection_cell/basic/model.sea"
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

echo "==> project --format devbox"
cargo run -q -p domainforge-core --features cli -- project --format devbox \
  --created-at '2026-07-02T00:00:00+00:00' "$FIXTURE" "$OUT"

echo "==> validate devbox.json"
test -s "$OUT/devbox.json"
python3 - "$OUT/devbox.json" <<'PY'
import json, sys
# The file has leading `//` comment lines; take the first line that opens an object.
text = open(sys.argv[1]).read()
obj_start = next((i for i, l in enumerate(text.splitlines()) if l.lstrip().startswith("{")), None)
assert obj_start is not None, "no JSON object found in devbox.json"
doc = json.loads("\n".join(text.splitlines()[obj_start:]))

# Required devbox top-level fields (per jetify devbox.json reference).
assert "packages" in doc, "missing packages"
assert isinstance(doc["packages"], list), "packages must be a list"
assert "env" in doc and isinstance(doc["env"], dict), "missing/invalid env"
assert "shell" in doc and isinstance(doc["shell"], dict), "missing/invalid shell"

# DomainForge manifest env vars.
env = doc["env"]
assert env.get("DOMAINFORGE_NAMESPACE") == "procurement", f"bad namespace: {env.get('DOMAINFORGE_NAMESPACE')}"
assert env.get("DOMAINFORGE_MODEL", "").endswith("model.sea"), f"bad model: {env.get('DOMAINFORGE_MODEL')}"
ent = env.get("DOMAINFORGE_ENTITIES", "")
assert ent and "Buyer" in ent and "Supplier" in ent, f"entities missing/empty: {ent}"
res = env.get("DOMAINFORGE_RESOURCES", "")
assert res and "PurchaseOrder" in res, f"resources missing/empty: {res}"
flows = env.get("DOMAINFORGE_FLOWS", "")
assert flows and "PurchaseOrder:Buyer->Supplier" in flows, f"flows missing/empty: {flows}"

# init_hook must be a string or array of strings; scripts is a map.
hook = doc["shell"].get("init_hook")
assert hook is not None, "missing shell.init_hook"
assert isinstance(hook, (str, list)), "init_hook must be string or array"
assert isinstance(doc["shell"].get("scripts", {}), dict), "scripts must be a map"
print(f"  devbox.json OK ({len(env)} env vars, {len(doc['packages'])} packages)")
PY

echo "==> devbox gate OK"
