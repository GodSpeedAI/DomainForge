#!/usr/bin/env bash
# Rego projection gate.
#
# Projects the flagship repair-agent fixture to an OPA/Rego bundle twice with a
# fixed --created-at and diff for byte-determinism, then runs the OPA toolchain
# (pinned 1.21.1) over the output:
#   - opa check --strict, opa fmt --fail, opa test (generated cases),
#     opa build --bundle.
# Finally runs the differential tests, which execute real OPA against generated
# inputs and require the decision document to equal DomainForge's own evaluator.
# OPA is required: the gate fails if it is missing or reports another version.
# Invoked by scripts/verify/projection-targets/all.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

FIXTURE="fixtures/cell_env/repair-agent/model.sea"
CREATED_AT="2026-07-11T00:00:00Z"
OPA_VERSION="1.21.1"
OUT1="$(mktemp -d)"
OUT2="$(mktemp -d)"
trap 'rm -rf "$OUT1" "$OUT2"' EXIT

echo "==> checking opa is on PATH (pinned $OPA_VERSION)"
if ! command -v opa >/dev/null 2>&1; then
  echo "FAIL: opa is not on PATH; this gate requires OPA $OPA_VERSION (no skip)" >&2
  exit 1
fi
opa_version="$(opa version | sed -n 's/^Version: //p')"
if [[ "$opa_version" != "$OPA_VERSION" ]]; then
  echo "FAIL: opa reports version '$opa_version'; this gate requires OPA $OPA_VERSION" >&2
  exit 1
fi
echo "OK: opa $opa_version"

echo "==> project --format rego (run 1)"
cargo run -q -p domainforge-core --features cli -- project --format rego \
  --created-at "$CREATED_AT" "$FIXTURE" "$OUT1"

echo "==> project --format rego (run 2)"
cargo run -q -p domainforge-core --features cli -- project --format rego \
  --created-at "$CREATED_AT" "$FIXTURE" "$OUT2"

echo "==> asserting byte-determinism"
diff -r "$OUT1" "$OUT2"
echo "OK: two runs are byte-identical"

echo "==> opa check --strict"
(cd "$OUT1" && opa check --strict --v1-compatible bundle tests)

echo "==> opa fmt --fail"
(cd "$OUT1" && opa fmt --v1-compatible --fail --list bundle)

echo "==> opa test"
(cd "$OUT1" && opa test --v1-compatible bundle tests)

echo "==> opa build --bundle"
(cd "$OUT1" && opa build --bundle bundle --output "$OUT1/bundle.tar.gz")
test -s "$OUT1/bundle.tar.gz"

echo "==> differential tests against real OPA"
cargo test -p domainforge-core --features cli --test rego_projection_tests -- --include-ignored

echo "==> rego gate OK"
