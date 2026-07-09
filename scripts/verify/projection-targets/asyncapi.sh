#!/usr/bin/env bash
# AsyncAPI projection gate.
#
# Projects the projection-cell fixture to an AsyncAPI 2.6 document and
# validates that the JSON is well-formed and carries the required top-level
# fields (asyncapi version, info, channels). Invoked by
# scripts/verify/projection-targets/all.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

FIXTURE="fixtures/projection_cell/basic/model.sea"
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

echo "==> project --format asyncapi"
cargo run -q -p domainforge-core --features cli -- project --format asyncapi \
  --created-at '2026-07-02T00:00:00+00:00' "$FIXTURE" "$OUT"

echo "==> validate asyncapi.json"
test -s "$OUT/asyncapi.json"
python3 - "$OUT/asyncapi.json" <<'PY'
import json, sys
# The file has a leading `//` comment line; take the first line that opens an object.
text = sys.stdin.read() if False else open(sys.argv[1]).read()
obj_start = next((i for i, l in enumerate(text.splitlines()) if l.lstrip().startswith("{")), None)
assert obj_start is not None, "no JSON object found in asyncapi.json"
doc = json.loads("\n".join(text.splitlines()[obj_start:]))
assert doc["asyncapi"] == "2.6.0", f"bad asyncapi version: {doc['asyncapi']}"
assert "info" in doc and "version" in doc["info"], "missing info/version"
assert "channels" in doc and isinstance(doc["channels"], dict), "missing channels object"
assert len(doc["channels"]) > 0, "no channels emitted"
for key, ch in doc["channels"].items():
    msg = ch.get("publish", {}).get("message", {})
    assert msg.get("contentType") == "application/json", f"channel {key}: bad contentType"
    assert "payload" in msg, f"channel {key}: message missing payload"
print(f"  {len(doc['channels'])} AsyncAPI channels validated")
PY

echo "==> asyncapi gate OK"
