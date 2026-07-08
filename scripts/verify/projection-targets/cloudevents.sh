#!/usr/bin/env bash
# CloudEvents projection gate.
#
# Projects the projection-cell fixture to CloudEvents 1.0 and validates that
# every emitted line is a JSON object carrying the required CloudEvents
# attributes (specversion, id, source, type). Invoked by
# scripts/verify/projection-targets/all.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

FIXTURE="fixtures/projection_cell/basic/model.sea"
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

echo "==> project --format cloudevents"
cargo run -q -p domainforge-core --features cli -- project --format cloudevents \
  --created-at '2026-07-02T00:00:00+00:00' "$FIXTURE" "$OUT"

echo "==> validate events.jsonl"
test -s "$OUT/events.jsonl"
python3 - "$OUT/events.jsonl" <<'PY'
import json, sys
required = {"specversion", "id", "source", "type"}
n = 0
with open(sys.argv[1]) as f:
    for line in f:
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        ev = json.loads(line)
        missing = required - ev.keys()
        assert not missing, f"CloudEvent missing required attributes: {missing}"
        assert ev["specversion"] == "1.0", f"bad specversion: {ev['specversion']}"
        assert ev["id"] and ev["source"] and ev["type"], "required attribute is empty"
        n += 1
assert n > 0, "no CloudEvents emitted"
print(f"  {n} CloudEvents validated")
PY

echo "==> cloudevents gate OK"
