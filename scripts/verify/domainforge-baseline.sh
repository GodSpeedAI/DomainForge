#!/usr/bin/env bash
# DomainForge baseline verification.
#
# Wraps the existing just/CI recipes rather than re-implementing them, so
# there remains exactly one source of truth for "baseline green". Invoked by
# scripts/verify/projection-targets/all.sh and by every per-target gate.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

echo "==> cargo fmt --all --check"
cargo fmt --all --check

echo "==> cargo clippy --all-targets --all-features -- -D warnings"
cargo clippy --all-targets --all-features -- -D warnings

echo "==> just ci-test-rust (cargo test --verbose --workspace --features cli)"
if command -v just >/dev/null 2>&1; then
  just ci-test-rust
else
  cargo test --verbose --workspace --features cli
fi

echo "==> baseline OK"
