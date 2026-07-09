#!/usr/bin/env bash
# Gauge projection gate.
#
# Projects the projection-cell fixture to a Gauge .spec and validates: the
# file is a single namespaced spec beginning with an H1 spec heading,
# containing one H2 scenario per flow, each scenario carrying Given/When/Then
# steps with quoted params, and no raw '<'/'>' (reserved by Gauge) in step
# text. Invoked by scripts/verify/projection-targets/all.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

FIXTURE="fixtures/projection_cell/basic/model.sea"
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

echo "==> project --format gauge"
cargo run -q -p domainforge-core --features cli -- project --format gauge \
  --created-at '2026-07-02T00:00:00+00:00' "$FIXTURE" "$OUT"

echo "==> validate *.spec"
SPEC="$OUT/procurement.spec"
test -s "$SPEC"
python3 - "$SPEC" <<'PY'
import re, sys
text = open(sys.argv[1]).read()
lines = text.splitlines()

# Single spec heading (H1).
h1 = [l for l in lines if l.startswith("# ")]
assert len(h1) == 1, f"expected exactly one H1 spec heading, got {len(h1)}"
assert h1[0].startswith("# procurement flows"), f"bad spec heading: {h1[0]}"

# >=1 H2 scenario; one per flow (the fixture has 2 flows).
h2 = [l for l in lines if l.startswith("## ")]
assert len(h2) >= 1, "no scenarios emitted"
assert all("Issue" in s and "from" in s and "to" in s for s in h2), \
    f"scenario heading not in 'Issue <r> from <f> to <t>' form: {h2}"

# Steps: lines starting with '* ', params quoted, no raw '<'/'>'.
steps = [l for l in lines if l.startswith("* ")]
assert len(steps) >= 3 * len(h2), f"too few steps ({len(steps)}) for {len(h2)} scenario(s)"
for st in steps:
    assert '"' in st, f"step has no quoted param: {st}"
    assert '<' not in st and '>' not in st, f"reserved angle bracket in step: {st}"

# Each scenario should have Given/When/Then beats.
assert any("Given a" in s for s in steps), "no Given step"
assert any('" issues "' in s for s in steps), "no When (issues) step"
assert any("should hold" in s for s in steps), "no Then (should hold) step"
print(f"  gauge OK ({len(h2)} scenario(s), {len(steps)} step(s))")
PY

echo "==> gauge gate OK"
