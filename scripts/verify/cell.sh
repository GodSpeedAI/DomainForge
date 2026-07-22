#!/usr/bin/env bash
# Verify the cell-environment projection: project the flagship repair-agent
# fixture twice with a fixed --created-at and diff for byte-determinism,
# then run whichever native toolchain checks are actually available on
# PATH. A skipped native check is reported as skipped, never as passed.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

FIXTURE="fixtures/cell_env/repair-agent/model.sea"
OVERRIDES="fixtures/cell_env/repair-agent/domainforge.cell.toml"
CREATED_AT="2026-07-11T00:00:00Z"
OUT1="$(mktemp -d)"
OUT2="$(mktemp -d)"
trap 'rm -rf "$OUT1" "$OUT2"' EXIT

BIN=(cargo run --quiet --features cli --bin domainforge --)

echo "==> projecting $FIXTURE (run 1)"
"${BIN[@]}" project --format cell --overrides "$OVERRIDES" \
  --created-at "$CREATED_AT" "$FIXTURE" "$OUT1"

echo "==> projecting $FIXTURE (run 2)"
"${BIN[@]}" project --format cell --overrides "$OVERRIDES" \
  --created-at "$CREATED_AT" "$FIXTURE" "$OUT2"

echo "==> asserting byte-determinism"
diff -r "$OUT1" "$OUT2"
echo "OK: two runs are byte-identical"

echo "==> structural checks"
python3 -c "
import json, sys
for f in ['cell.lock', 'devbox/devbox.json', 'mise/projection-manifest.json',
          'sandbox/network.json', 'authority/credential-contracts.json',
          'evidence/environment-proof-contract.json']:
    json.load(open('$OUT1/' + f))
print('OK: every JSON artifact parses')
"

echo "==> native toolchain checks (skipped if the tool is not on PATH)"
if command -v devbox >/dev/null 2>&1; then
  # `devbox generate` alone prints usage (it requires a subcommand); `direnv`
  # is the side-effect-free-enough one that still fails on an invalid
  # devbox.json, which is what we're actually checking.
  if ! (cd "$OUT1/devbox" && devbox generate direnv); then
    echo "FAIL: devbox generate direnv failed (devbox.json rejected by devbox)" >&2
    exit 1
  fi
  echo "OK: devbox generate direnv (devbox.json is valid)"
else
  echo "SKIPPED: devbox generate direnv (devbox not on PATH) — exact command: (cd $OUT1/devbox && devbox generate direnv)"
fi

if command -v mise >/dev/null 2>&1; then
  mise trust "$OUT1/mise/mise.toml" >/dev/null 2>&1 || true
  if ! (cd "$OUT1/mise" && mise config); then
    echo "FAIL: mise config failed (mise.toml rejected by mise)" >&2
    exit 1
  fi
  echo "OK: mise config"
else
  echo "SKIPPED: mise config (mise not on PATH) — exact command: (cd $OUT1/mise && mise trust mise.toml && mise config)"
fi

if command -v uv >/dev/null 2>&1; then
  echo "SKIPPED (informational): uv sync --frozen would run against fixtures/cell_env/repair-agent/pyproject.toml in a real environment"
else
  echo "SKIPPED: uv sync --frozen (uv not on PATH)"
fi

echo "==> cell.sh: PASSED"
