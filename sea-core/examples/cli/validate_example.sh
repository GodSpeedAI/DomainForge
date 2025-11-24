#!/bin/bash
set -e

# Build the CLI
cargo build --bin sea --features cli

# Path to the binary (compute absolute path from script directory)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SEA="$SCRIPT_DIR/../../../target/debug/sea"

# Validate that the binary exists and is executable
if [ ! -x "$SEA" ]; then
    echo "Error: SEA binary not found or not executable at: $SEA" >&2
    echo "Please ensure 'cargo build --bin sea --features cli' completed successfully." >&2
    exit 1
fi

# Create a sample SEA file
echo 'Entity "Warehouse" in logistics' > sample.sea

# Validate it
echo "Validating sample.sea..."
$SEA validate sample.sea

# Validate with JSON output
echo "Validating with JSON output..."
$SEA validate --format json sample.sea

# Clean up
rm sample.sea
