#!/usr/bin/env bash
set -e

echo "========================================"
echo "  DomainForge SEA DSL - Test Suite"
echo "========================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

FAILED=0

# Rust tests
echo "==> Running Rust tests..."
if cargo test --manifest-path sea-core/Cargo.toml; then
    echo -e "${GREEN}✓ Rust tests passed${NC}"
else
    echo -e "${RED}✗ Rust tests failed${NC}"
    FAILED=1
fi
echo ""

# Python tests
echo "==> Running Python tests..."
if [ -f "tests/test_primitives.py" ]; then
    # Build Python bindings if not already built
    if [ ! -d ".venv" ] || ! .venv/bin/python -c "import sea_dsl" 2>/dev/null; then
        echo "Building Python bindings..."
        bash scripts/build-python.sh
    fi

    # Activate virtual environment and run tests
    source .venv/bin/activate
    if pytest tests/ -v; then
        echo -e "${GREEN}✓ Python tests passed${NC}"
    else
        echo -e "${RED}✗ Python tests failed${NC}"
        FAILED=1
    fi
    deactivate
else
    echo -e "${YELLOW}⚠ No Python tests found${NC}"
fi
echo ""

# TypeScript tests
echo "==> Running TypeScript tests..."
if [ -f "vitest.config.ts" ]; then
    # Build TypeScript bindings if not already built
    if [ ! -f "sea-core.linux-x64-gnu.node" ] && [ ! -f "index.node" ]; then
        echo "Building TypeScript bindings..."
        bash scripts/build-typescript.sh
    fi

    if npm test; then
        echo -e "${GREEN}✓ TypeScript tests passed${NC}"
    else
        echo -e "${RED}✗ TypeScript tests failed${NC}"
        FAILED=1
    fi
else
    echo -e "${YELLOW}⚠ No TypeScript tests found${NC}"
fi
echo ""

echo "========================================"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
