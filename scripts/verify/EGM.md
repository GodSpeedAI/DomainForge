# Verify Scripts

## Overview
Fixture-driven end-to-end verification entry points: a baseline wrapper that
delegates to the authoritative `just`/CI recipes, a cell-environment projection
gate, and the per-projection-target gate suite under `projection-targets/`.

## Interface
- `[domainforge-baseline.sh:set -euo pipefail]` — baseline gate; runs
  `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D
  warnings`, and `just ci-test-rust` (falling back to `cargo test --verbose
  --workspace --features cli` when `just` is absent), then prints `baseline OK`.
- `[cell.sh:BIN]` — cell-environment gate; projects
  `fixtures/cell_env/repair-agent/model.sea` with `--format cell --overrides
  fixtures/cell_env/repair-agent/domainforge.cell.toml` twice at a pinned
  `--created-at`, `diff -r`s the outputs, then runs optional native checks.
- Child contracts: `./projection-targets/EGM.md` — per-target projection gates and
  the all-target orchestrator.

## Invariants
- `[domainforge-baseline.sh:just ci-test-rust]` — the baseline deliberately wraps
  the existing recipes rather than re-implementing them, so there is exactly one
  source of truth for "baseline green".
- `[domainforge-baseline.sh:cargo test --verbose --workspace --features cli]` —
  `just` is optional: the script runs `just ci-test-rust` when `just` is on
  `PATH`, and otherwise falls back to the equivalent `cargo test --verbose
  --workspace --features cli` in the `else` branch.
- `[cell.sh:diff -r]` — two runs at the same `--created-at` and the same
  `--overrides` file must be byte-identical, otherwise the gate fails.
- `[cell.sh:json.load]` — every required cell artifact (`cell.lock`,
  `devbox/devbox.json`, `mise/projection-manifest.json`, `sandbox/network.json`,
  `authority/credential-contracts.json`,
  `evidence/environment-proof-contract.json`) must parse as JSON.
- `[cell.sh:SKIPPED]` — a missing native tool (`devbox`, `mise`, `uv`) is reported
  as `SKIPPED` with the exact command that would have run; it is never reported as
  passed.
- `[cell.sh:devbox generate direnv]` — a `devbox`/`mise` rejection of the emitted
  config is a hard failure (`exit 1`), not a skip.

## Side Effects
- `[domainforge-baseline.sh:cargo]` — invokes `cargo fmt`, `cargo clippy`, and a
  full workspace test build, populating `target/`.
- `[cell.sh:OUT1]` — creates two `mktemp -d` output directories and removes both
  via `trap 'rm -rf "$OUT1" "$OUT2"' EXIT`.
- `[cell.sh:mise trust]` — runs `mise trust "$OUT1/mise/mise.toml"` (errors
  suppressed) and `mise config` in the emitted directory.
- `[cell.sh:devbox generate direnv]` — writes a direnv shim inside
  `$OUT1/devbox` when `devbox` is on `PATH`.

## Verification
- Run: `bash scripts/verify/domainforge-baseline.sh`
- Run: `bash scripts/verify/cell.sh` (also `just cell-verify`)
- Run: `bash scripts/verify/projection-targets/all.sh`
