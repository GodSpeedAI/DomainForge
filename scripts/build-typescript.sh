#!/usr/bin/env bash
set -e

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "==> Building TypeScript bindings..."

# package.json with the napi build config lives in domainforge-typescript/.
# The build script there uses --cargo-cwd ../domainforge-core to locate the Rust crate.
cd domainforge-typescript && npm run build

echo "✓ TypeScript bindings built successfully"
