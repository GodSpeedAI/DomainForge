#!/bin/sh
# Functional acceptance test for an installed or extracted domainforge binary.
#
# It proves more than --version: validation accepts a valid model and rejects an
# invalid one, AST/JSON parsing is machine-readable, a code projection
# materializes inspectable files, and the RDF projection is byte-deterministic
# when given a fixed created-at timestamp.
#
# Usage: packaging/smoke.sh <domainforge> <valid.sea> <invalid.sea> <version>
set -eu

if [ "$#" -ne 4 ]; then
    echo "usage: $0 <domainforge> <valid.sea> <invalid.sea> <version>" >&2
    exit 2
fi

cli=$1
valid=$2
invalid=$3
expected_version=$4

if [ ! -x "$cli" ]; then
    echo "error: not executable: $cli" >&2
    exit 1
fi
for fixture in "$valid" "$invalid"; do
    if [ ! -f "$fixture" ]; then
        echo "error: fixture not found: $fixture" >&2
        exit 1
    fi
done

python=''
for candidate in python3 python; do
    if command -v "$candidate" >/dev/null 2>&1; then
        python=$candidate
        break
    fi
done
if [ -z "$python" ]; then
    echo "error: python3 is required for the JSON and projection checks" >&2
    exit 1
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

version_output=$("$cli" --version)
[ "$version_output" = "domainforge $expected_version" ] \
    || fail "version output '$version_output' != 'domainforge $expected_version'"
echo "smoke: version -> $version_output"

"$cli" validate --format human --no-color "$valid" > "$work/validate.out" 2>&1 \
    || fail "valid model was rejected"
grep -q 'Validation succeeded' "$work/validate.out" \
    || fail "valid model output did not report success"
echo "smoke: valid model accepted"

set +e
"$cli" validate --format human --no-color "$invalid" > "$work/invalid.out" 2>&1
invalid_rc=$?
set -e
[ "$invalid_rc" -ne 0 ] || fail "invalid model was accepted"
[ -s "$work/invalid.out" ] || fail "invalid model produced no diagnostic"
echo "smoke: invalid model rejected (exit $invalid_rc)"

"$cli" parse --ast --format json "$valid" > "$work/ast.json"
"$python" - "$work/ast.json" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    document = json.load(handle)
declarations = document.get("declarations")
if not isinstance(declarations, list) or not declarations:
    raise SystemExit("AST has no declarations")
print(f"smoke: AST JSON parsed ({len(declarations)} declarations)")
PY

"$cli" project --format domain-python "$valid" "$work/python" > "$work/project.out"
[ -s "$work/project.out" ] || fail "domain-python projection printed no summary"
[ -f "$work/python/pyproject.toml" ] || fail "domain-python projection omitted pyproject.toml"
[ -f "$work/python/README.md" ] || fail "domain-python projection omitted README.md"
[ -n "$(find "$work/python/src" -name entities.py -type f 2>/dev/null)" ] \
    || fail "domain-python projection omitted entities.py"
[ -n "$(find "$work/python/tests" -name 'test_*.py' -type f 2>/dev/null)" ] \
    || fail "domain-python projection omitted tests"
"$python" -m compileall -q "$work/python"
echo "smoke: domain-python projection materialized and compiles"

set +e
"$cli" project --format domain-python "$invalid" "$work/bad" > "$work/bad.out" 2>&1
bad_rc=$?
set -e
[ "$bad_rc" -ne 0 ] || fail "projection accepted an invalid model"
echo "smoke: invalid projection rejected (exit $bad_rc)"

"$cli" project --format rdf --created-at 2026-01-01T00:00:00Z "$valid" "$work/rdf-a" >/dev/null
"$cli" project --format rdf --created-at 2026-01-01T00:00:00Z "$valid" "$work/rdf-b" >/dev/null
[ -f "$work/rdf-a/model.ttl" ] || fail "RDF projection omitted model.ttl"
diff -r "$work/rdf-a" "$work/rdf-b" >/dev/null \
    || fail "RDF projection is not deterministic for a fixed created-at"
echo "smoke: RDF projection is byte-deterministic"

echo "smoke: PASS"
