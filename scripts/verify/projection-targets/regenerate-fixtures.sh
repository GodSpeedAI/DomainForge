#!/usr/bin/env bash
# Regenerate committed/generated projection-cell fixtures.
#
# Deliberate, reviewed fixture regeneration entrypoint (per
# docs/reference/generated-artifacts-policy.md). Currently a no-op until
# committed generated examples are added; per-target renderers regenerate
# into temp directories during their gates rather than committing output.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

echo "==> regenerate-fixtures: nothing to regenerate yet (per-target gates use temp output)"
