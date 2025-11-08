#!/usr/bin/env bash
set -e

echo "==> Building TypeScript bindings..."

# Use npm script which has correct napi build configuration
npm run build

echo "✓ TypeScript bindings built successfully"
