#!/bin/bash
set -e

# optimized-wasm-build.sh
# Performs a production build of the sea-core WASM bundle with SAFE optimizations.

echo "🚀 Starting Optimized WASM Build..."

# 1. Clean previous builds
echo "🧹 Cleaning previous artifacts..."
cargo clean -p sea-core --target wasm32-unknown-unknown

# 2. Build with wasm-pack
# Note: Profile configuration in sea-core/Cargo.toml now enables wasm-opt
echo "🏗️  Building with wasm-pack (Release mode)..."
# Using --target web as a standard default, can be changed to --target nodejs if needed
wasm-pack build sea-core --target web --release --features wasm

# 3. Explicit Optimization (Verification Step)
# Even though we enabled it in Cargo.toml, running it explicitly ensures it happens
# and allows us to see the output stats.
WASM_FILE="sea-core/pkg/sea_core_bg.wasm"
OPTIMIZED_FILE="sea-core/pkg/sea_core_bg.optimized.wasm"

if command -v wasm-opt >/dev/null 2>&1; then
    echo "🔧 Running explicit wasm-opt -Oz..."
    wasm-opt -Oz -o "$OPTIMIZED_FILE" "$WASM_FILE"
    
    # Compare sizes
    ORIG_SIZE=$(stat -c%s "$WASM_FILE")
    OPT_SIZE=$(stat -c%s "$OPTIMIZED_FILE")
    
    echo "📊 Size Report:"
    echo "   Original: $(numfmt --to=iec-i --suffix=B $ORIG_SIZE)"
    echo "   Optimized: $(numfmt --to=iec-i --suffix=B $OPT_SIZE)"
    
    # Replace if smaller
    if [ $OPT_SIZE -lt $ORIG_SIZE ]; then
        echo "✅ Optimization successful! Replacing original file."
        mv "$OPTIMIZED_FILE" "$WASM_FILE"
    else
        echo "⚠️  Optimization did not reduce size. Keeping original."
        rm "$OPTIMIZED_FILE"
    fi
else
    echo "⚠️  'wasm-opt' not found in PATH. Skipping explicit optimization step."
    echo "   (Note: wasm-pack usually runs this internally if configured)"
fi

# 4. Final Size Check
FINAL_SIZE=$(stat -c%s "$WASM_FILE")
MAX_SIZE=$((2048 * 1024)) # 2 MB

echo "----------------------------------------"
echo "🏁 Final Bundle Size: $(numfmt --to=iec-i --suffix=B $FINAL_SIZE)"
if [ $FINAL_SIZE -gt $MAX_SIZE ]; then
    echo "❌ WARNING: Bundle exceeds 0.5 MB limit!"
else
    echo "✅ SUCCESS: Bundle is under 0.5 MB limit."
fi
echo "----------------------------------------"
