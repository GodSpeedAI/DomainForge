#!/usr/bin/env bash
# CloudEvents projection gate.
#
# Projects the projection-cell fixture to CloudEvents 1.0 and validates that
# every emitted line is a JSON object carrying the required CloudEvents
# attributes (specversion, id, source, type, time). H5: events.jsonl is now
# strict JSON Lines — no `#` comment header to skip.
#
# H3: the gate validates each event's `time` attribute is RFC 3339 (M6) and
# that every line parses as JSON without comment-stripping (H5).
#
# Invoked by scripts/verify/projection-targets/all.sh.
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
import json, re, sys

required = {"specversion", "id", "source", "type", "time"}
# RFC 3339 pattern (simplified — sufficient for validation gate).
rfc3339 = re.compile(
    r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$"
)

n = 0
with open(sys.argv[1]) as f:
    for line_no, line in enumerate(f, 1):
        line = line.strip()
        if not line:
            continue
        # H5: every non-empty line must parse as JSON — no comment lines.
        ev = json.loads(line)
        missing = required - ev.keys()
        assert not missing, f"line {line_no}: CloudEvent missing required attributes: {missing}"
        assert ev["specversion"] == "1.0", f"line {line_no}: bad specversion: {ev['specversion']}"
        assert ev["id"] and ev["source"] and ev["type"], f"line {line_no}: required attribute is empty"
        # M6: time must be RFC 3339.
        assert rfc3339.match(ev["time"]), f"line {line_no}: time is not RFC 3339: {ev['time']}"
        n += 1
assert n > 0, "no CloudEvents emitted"
print(f"  {n} CloudEvents validated (strict JSONL, RFC 3339 time)")
PY

echo "==> cloudevents gate OK"
