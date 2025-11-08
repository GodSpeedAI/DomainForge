#!/usr/bin/env bash
set -e

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

# Build Python bindings in development mode
echo "Building with maturin develop..."
maturin develop --features python --manifest-path sea-core/Cargo.toml

echo "✓ Python bindings built successfully"
echo "✓ Virtual environment activated at .venv"
