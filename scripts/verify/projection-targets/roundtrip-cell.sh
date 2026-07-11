#!/usr/bin/env bash
# Roundtrip-cell gate.
#
# Verifies the projection-cell fixture round-trips through CALM: export the
# model to CALM JSON, re-import it to a Graph, and assert the representable
# primitives (entities, resources, flows) survive with their names intact.
# Unlike the per-format gates, this is not a one-way projection — it is the
# round-trip invariant for the projection cell. Invoked by
# scripts/verify/projection-targets/all.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

FIXTURE="fixtures/projection_cell/basic/model.sea"
test -s "$FIXTURE" || { echo "roundtrip-cell: fixture $FIXTURE missing" >&2; exit 1; }

echo "==> round-trip projection-cell fixture through CALM (export -> import)"
cargo test -q -p domainforge-core --features cli --test roundtrip_cell_tests

echo "==> roundtrip-cell gate OK"
