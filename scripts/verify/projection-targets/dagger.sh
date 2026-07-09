#!/usr/bin/env bash
# Dagger projection gate.
#
# Projects the projection-cell fixture to a minimal Dagger Python module and
# validates that dagger.json is schema-shaped (name+engineVersion required,
# sdk=python), main.py declares one @dagger.function per flow on a namespaced
# class, and pyproject.toml carries the build backend. Invoked by
# scripts/verify/projection-targets/all.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

FIXTURE="fixtures/projection_cell/basic/model.sea"
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

echo "==> project --format dagger"
cargo run -q -p domainforge-core --features cli -- project --format dagger \
  --created-at '2026-07-02T00:00:00+00:00' "$FIXTURE" "$OUT"

echo "==> validate dagger module"
test -s "$OUT/dagger.json"
test -s "$OUT/main.py"
test -s "$OUT/pyproject.toml"
python3 - "$OUT/dagger.json" "$OUT/main.py" "$OUT/pyproject.toml" <<'PY'
import json, re, sys

manifest_p, src_p, pyproject_p = sys.argv[1], sys.argv[2], sys.argv[3]

# --- dagger.json: skip leading // comment, parse, validate schema-required fields.
text = open(manifest_p).read()
obj_start = next((i for i, l in enumerate(text.splitlines()) if l.lstrip().startswith("{")), None)
assert obj_start is not None, "no JSON object in dagger.json"
doc = json.loads("\n".join(text.splitlines()[obj_start:]))
for req in ("name", "engineVersion"):
    assert req in doc and doc[req], f"dagger.json missing required field: {req}"
assert doc["name"] == "procurement", f"bad module name: {doc.get('name')}"
assert re.match(r"^v\d+\.\d+\.\d+", doc["engineVersion"]), f"bad engineVersion: {doc['engineVersion']}"
assert doc.get("sdk") == "python", f"bad sdk: {doc.get('sdk')}"
assert doc.get("source") == ".", f"bad source: {doc.get('source')}"

# --- main.py: namespaced object_type class, >=1 function per flow, valid identifier fn names.
src = open(src_p).read()
assert "@dagger.object_type" in src, "missing object_type"
assert re.search(r"class\s+\w+\s*:", src), "no class declaration"
class_name = doc["name"]
expected = "".join(w.capitalize() for w in re.split(r"\W+", class_name) if w)
assert f"class {expected}:" in src, f"class name not PascalCase of namespace: expected {expected}"
fn_count = len(re.findall(r"@dagger\.function", src))
assert fn_count >= 1, "no @dagger.function emitted"
# Each function def must use a valid Python identifier.
for m in re.finditer(r"def\s+(run_[A-Za-z0-9_]+)\s*\(", src):
    name = m.group(1)
    assert re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", name), f"invalid fn identifier: {name}"

# --- pyproject.toml: build backend + project name.
pp = open(pyproject_p).read()
assert 'build-backend = "hatchling.build"' in pp, "missing hatchling build backend"
assert f'name = "{class_name}"' in pp, "pyproject name != module name"

print(f"  dagger module OK ({fn_count} function(s), class {expected}, engine {doc['engineVersion']})")
PY

echo "==> dagger gate OK"
