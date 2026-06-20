#!/usr/bin/env bash
set -e

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "==> Building Python bindings..."

# Create virtual environment if it doesn't exist
if [ ! -d ".venv" ]; then
    echo "Creating virtual environment..."
    python3 -m venv .venv
fi

# Activate virtual environment
source .venv/bin/activate

# Check if maturin is available
if ! command -v maturin &> /dev/null; then
    echo "Installing maturin..."
    pip install maturin
fi

# Install pytest if not available
if ! python -c "import pytest" 2>/dev/null; then
    echo "Installing pytest..."
    pip install pytest
fi

# Build Python bindings in development mode.
# pyproject.toml lives in domainforge-python/ and references domainforge-core/Cargo.toml via a
# relative path, so maturin must be invoked from domainforge-python/.
echo "Building with maturin develop (pyproject in domainforge-python/)..."
cd domainforge-python && maturin develop

echo "✓ Python bindings built successfully"
echo "✓ Virtual environment activated at $ROOT_DIR/.venv"
