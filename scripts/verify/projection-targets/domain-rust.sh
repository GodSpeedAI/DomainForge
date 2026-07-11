#!/usr/bin/env bash
# domain-rust projection gate.
#
# Projects the projection-cell fixture to a complete zero-dependency Rust
# DDD/CQRS crate and validates:
#   1. Structural checks (file set, key traits/variants).
#   2. cargo check --offline (zero-dep crate builds offline).
#   3. cargo test (runs the smoke test: constructs commands/events/VOs,
#      asserts the pattern VO rejects empty input).
#
# Invoked by scripts/verify/projection-targets/all.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

# Resolve the real cargo binary (bypass per-directory mise shims: the generated
# crate lives in a temp dir with no mise config). Prepend the toolchain bin dir
# to PATH so cargo's internal rustc/rustdoc calls also resolve directly.
CARGO_BIN="$(rustup which cargo 2>/dev/null || command -v cargo)"
TOOLCHAIN_BIN="$(dirname "$CARGO_BIN")"
export PATH="$TOOLCHAIN_BIN:$PATH"

FIXTURE="fixtures/projection_cell/basic/model.sea"
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

echo "==> project --format domain-rust"
cargo run -q -p domainforge-core --features cli -- project --format domain-rust \
  --created-at '2026-07-02T00:00:00+00:00' "$FIXTURE" "$OUT"

echo "==> validate generated crate (structural)"
test -s "$OUT/Cargo.toml"
test -s "$OUT/src/lib.rs"
test -s "$OUT/src/aggregates.rs"
test -s "$OUT/src/ports.rs"
test -s "$OUT/tests/smoke.rs"
grep -q "pub trait PurchaseOrderRepository" "$OUT/src/ports.rs"
grep -q "RequireApprovalViolation" "$OUT/src/errors.rs"
grep -q "\[workspace\]" "$OUT/Cargo.toml"
echo "  structural OK"

echo "==> cargo check --offline (zero-dep crate)"
(cd "$OUT" && cargo check --offline)

echo "==> cargo test (smoke test)"
(cd "$OUT" && cargo test --offline)

echo "==> domain-rust gate OK"
