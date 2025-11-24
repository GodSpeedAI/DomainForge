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
echo 'Entity "Factory" in manufacturing' > factory.sea
echo 'Resource "Widget" units' >> factory.sea
echo 'Flow "Widget" from "Factory" to "Factory" quantity 100' >> factory.sea

# Project to CALM
echo "Projecting to CALM..."
$SEA project --format calm factory.sea factory.json
cat factory.json

# Project to KG (Turtle)
echo "Projecting to Knowledge Graph (Turtle)..."
$SEA project --format kg factory.sea factory.ttl
cat factory.ttl

# Import back from KG (Turtle)
echo "Importing from Knowledge Graph..."
$SEA import --format kg factory.ttl

# Clean up
rm factory.sea factory.json factory.ttl
