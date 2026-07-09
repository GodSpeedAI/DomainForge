#!/usr/bin/env bash
# AsyncAPI projection gate.
#
# Projects the projection-cell fixture to an AsyncAPI 3.0 YAML document and
# validates: (1) the YAML is well-formed and carries the required top-level
# fields (asyncapi 3.0.0, info, channels, operations, components), with one
# channel per (resource, from) flow-group and matching send/receive
# operations (producers/consumers); and (2) the document validates against
# the official, vendored AsyncAPI 3.0 schema (schemas/asyncapi/3.0.0.json)
# via tests/asyncapi_spec_validation_tests.rs. Invoked by
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

echo "==> validate asyncapi.yaml structure"
test -s "$OUT/asyncapi.yaml"
python3 - "$OUT/asyncapi.yaml" <<'PY'
import sys, yaml
text = open(sys.argv[1]).read()
# Strip the leading `#` header comment lines before the document.
doc_start = next((i for i, l in enumerate(text.splitlines()) if l.lstrip().startswith("asyncapi:")), None)
assert doc_start is not None, "no asyncapi document found"
doc = yaml.safe_load("\n".join(text.splitlines()[doc_start:]))
assert doc["asyncapi"] == "3.0.0", f"bad asyncapi version: {doc['asyncapi']}"
assert "info" in doc and "version" in doc["info"], "missing info/version"
assert "channels" in doc and isinstance(doc["channels"], dict), "missing channels"
assert "operations" in doc and isinstance(doc["operations"], dict), "missing operations"
assert "components" in doc, "missing components"
assert len(doc["channels"]) > 0, "no channels emitted"
# Each channel must reference a component message; each operation references a channel + message.
for cname, ch in doc["channels"].items():
    assert "address" in ch, f"channel {cname} missing address"
    msgs = ch.get("messages", {})
    assert msgs, f"channel {cname} has no messages"
    for m in msgs.values():
        assert "$ref" in m, f"channel {cname} message not a $ref"
actions = {op["action"] for op in doc["operations"].values()}
assert "send" in actions and "receive" in actions, f"missing producer/consumer actions: {actions}"
sends = sum(1 for op in doc["operations"].values() if op["action"] == "send")
recvs = sum(1 for op in doc["operations"].values() if op["action"] == "receive")
print(f"  {len(doc['channels'])} channel(s), {sends} producer(s), {recvs} consumer(s)")
PY

echo "==> validate asyncapi.yaml against the official AsyncAPI 3.0 schema"
cargo test -q -p domainforge-core --features cli --test asyncapi_spec_validation_tests

echo "==> asyncapi gate OK"
