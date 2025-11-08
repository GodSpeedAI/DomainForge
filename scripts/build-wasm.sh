#!/bin/bash

# Build SEA Core WASM bindings
# This script builds the WASM module and prepares it for npm distribution

set -e

echo "🔨 Building SEA Core WASM bindings..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack is not installed"
    echo "Install it with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

# Check if wasm-opt is installed (optional but recommended)
if ! command -v wasm-opt &> /dev/null; then
    echo "⚠️  wasm-opt is not installed (optional but recommended for size optimization)"
    echo "Install it from: https://github.com/WebAssembly/binaryen"
fi

# Build the WASM package
echo "📦 Running wasm-pack build..."
cd sea-core
wasm-pack build --target web --release --out-dir ../pkg --features wasm

cd ..

# Optimize WASM binary if wasm-opt is available
if command -v wasm-opt &> /dev/null; then
    echo "🔧 Optimizing WASM binary with wasm-opt..."
    wasm-opt -Oz -o pkg/sea_core_bg_opt.wasm pkg/sea_core_bg.wasm
    mv pkg/sea_core_bg_opt.wasm pkg/sea_core_bg.wasm
fi

# Create gzipped version for size check
echo "📊 Creating gzipped version for size check..."
gzip -k -f pkg/sea_core_bg.wasm

# Check the size
WASM_SIZE=$(stat -f%z pkg/sea_core_bg.wasm 2>/dev/null || stat -c%s pkg/sea_core_bg.wasm 2>/dev/null)
GZIP_SIZE=$(stat -f%z pkg/sea_core_bg.wasm.gz 2>/dev/null || stat -c%s pkg/sea_core_bg.wasm.gz 2>/dev/null)
WASM_SIZE_KB=$((WASM_SIZE / 1024))
GZIP_SIZE_KB=$((GZIP_SIZE / 1024))

echo "✅ Build complete!"
echo ""
echo "📈 Bundle sizes:"
echo "   WASM (uncompressed): ${WASM_SIZE_KB} KB"
echo "   WASM (gzipped):      ${GZIP_SIZE_KB} KB"
echo ""

# Check if size target is met
if [ $GZIP_SIZE_KB -lt 500 ]; then
    echo "✅ Size target met: ${GZIP_SIZE_KB} KB < 500 KB"
else
    echo "⚠️  Warning: Size target exceeded: ${GZIP_SIZE_KB} KB >= 500 KB"
fi

echo ""
echo "📁 Package contents:"
ls -lh pkg/

echo ""
echo "🚀 To test the package:"
echo "   cd pkg"
echo "   npm link"
echo "   cd ../examples"
echo "   python3 -m http.server 8000"
echo "   # Open http://localhost:8000/browser.html in your browser"

echo ""
echo "📦 To publish to npm:"
echo "   cd pkg"
echo "   npm publish --access public"
