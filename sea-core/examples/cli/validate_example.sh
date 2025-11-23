#!/bin/bash
set -e

# Build the CLI
cargo build --bin sea --features cli

# Path to the binary
SEA=../../../target/debug/sea

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
