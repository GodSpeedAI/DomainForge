#!/usr/bin/env bash
# All-target projection verification entrypoint.
#
# Runs the DomainForge baseline plus every per-target projection gate that
# has landed. Each per-target script is appended below as its target ships.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

FIXTURE="fixtures/projection_cell/basic/model.sea"

echo "==> domainforge baseline"
bash scripts/verify/domainforge-baseline.sh

echo "==> validate projection-cell fixture"
cargo run -q -p domainforge-core --features cli -- validate "$FIXTURE"

# Per-target gates (appended as each target lands):
  bash scripts/verify/projection-targets/cloudevents.sh
  bash scripts/verify/projection-targets/asyncapi.sh
  bash scripts/verify/projection-targets/devbox.sh
#   bash scripts/verify/projection-targets/dagger.sh
#   bash scripts/verify/projection-targets/cedar.sh
#   bash scripts/verify/projection-targets/gauge.sh
#   bash scripts/verify/projection-targets/alloy.sh
#   bash scripts/verify/projection-targets/tla.sh
#   bash scripts/verify/projection-targets/roundtrip-cell.sh

echo "==> all projection-target gates OK"
