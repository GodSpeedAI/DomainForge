#!/bin/bash

set -e

echo "Building SEA Core WASM bindings..."

if ! command -v wasm-pack &> /dev/null; then
    echo "ERROR: wasm-pack is not installed"
    echo "Install it with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

if ! command -v wasm-opt &> /dev/null; then
    echo "WARNING: wasm-opt is not installed (optional but recommended for size optimization)"
    echo "Install it from: https://github.com/WebAssembly/binaryen"
fi

PKG_DIR="target/wasm-pkg"
VERSION=$(grep '^version =' sea-core/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')

if [ -z "$VERSION" ]; then
    echo "ERROR: Failed to extract version from sea-core/Cargo.toml: VERSION is empty" >&2
    exit 1
fi

echo "Building wasm-pack (version=${VERSION})..."
cd sea-core
wasm-pack build --target web --release --out-dir ../${PKG_DIR} --features wasm
cd ..

if command -v wasm-opt &> /dev/null; then
    echo "Optimizing WASM binary with wasm-opt..."
    wasm-opt -Oz -o ${PKG_DIR}/sea_core_bg_opt.wasm ${PKG_DIR}/sea_core_bg.wasm
    mv ${PKG_DIR}/sea_core_bg_opt.wasm ${PKG_DIR}/sea_core_bg.wasm
fi

echo "Patching package.json (name=@domainforge/sea-wasm, version=${VERSION})..."
cd "${PKG_DIR}"
if command -v python3 &> /dev/null; then
    python3 -c "
import json, sys
with open('package.json', 'r') as f:
    pkg = json.load(f)
pkg['name'] = '@domainforge/sea-wasm'
pkg['version'] = '${VERSION}'
pkg['description'] = 'WebAssembly bindings for SEA DSL - Semantic Enterprise Architecture'
pkg['author'] = 'DomainForge Contributors'
pkg['license'] = 'Apache-2.0'
pkg['repository'] = {'type': 'git', 'url': 'https://github.com/GodSpeedAI/DomainForge.git'}
pkg['homepage'] = 'https://github.com/GodSpeedAI/DomainForge'
pkg['keywords'] = ['wasm', 'domainforge', 'sea-dsl', 'enterprise-architecture']
pkg['files'] = ['sea_core.js', 'sea_core.d.ts', 'sea_core_bg.wasm', 'sea_core_bg.wasm.d.ts']
with open('package.json', 'w') as f:
    json.dump(pkg, f, indent=2)
    f.write('\n')
"
elif command -v node &> /dev/null; then
    node -e "
const fs = require('fs');
const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));
pkg.name = '@domainforge/sea-wasm';
pkg.version = '${VERSION}';
pkg.description = 'WebAssembly bindings for SEA DSL - Semantic Enterprise Architecture';
pkg.author = 'DomainForge Contributors';
pkg.license = 'Apache-2.0';
pkg.repository = {type: 'git', url: 'https://github.com/GodSpeedAI/DomainForge.git'};
pkg.homepage = 'https://github.com/GodSpeedAI/DomainForge';
pkg.keywords = ['wasm', 'domainforge', 'sea-dsl', 'enterprise-architecture'];
pkg.files = ['sea_core.js', 'sea_core.d.ts', 'sea_core_bg.wasm', 'sea_core_bg.wasm.d.ts'];
fs.writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');
"
else
    echo "WARNING: Could not patch package.json (need python3 or node)"
fi
cd ..

echo "Creating gzipped version for size check..."
gzip -k -f ${PKG_DIR}/sea_core_bg.wasm

WASM_SIZE=$(stat -c%s ${PKG_DIR}/sea_core_bg.wasm 2>/dev/null || stat -f%z ${PKG_DIR}/sea_core_bg.wasm 2>/dev/null)
GZIP_SIZE=$(stat -c%s ${PKG_DIR}/sea_core_bg.wasm.gz 2>/dev/null || stat -f%z ${PKG_DIR}/sea_core_bg.wasm.gz 2>/dev/null)
WASM_SIZE_KB=$((WASM_SIZE / 1024))
GZIP_SIZE_KB=$((GZIP_SIZE / 1024))

echo "Build complete!"
echo ""
echo "Bundle sizes:"
echo "   WASM (uncompressed): ${WASM_SIZE_KB} KB"
echo "   WASM (gzipped):      ${GZIP_SIZE_KB} KB"
echo ""

if [ $GZIP_SIZE_KB -lt 500 ]; then
    echo "Size target met: ${GZIP_SIZE_KB} KB < 500 KB"
else
    echo "WARNING: Size target exceeded: ${GZIP_SIZE_KB} KB >= 500 KB"
fi

echo ""
echo "Package contents:"
ls -lh ${PKG_DIR}/

echo ""
echo "To test the package:"
echo "   cd ${PKG_DIR}"
echo "   npm link"
echo "   cd ../examples"
echo "   python3 -m http.server 8000"
echo "   # Open http://localhost:8000/browser.html in your browser"

echo ""
echo "To publish to npm:"
echo "   cd ${PKG_DIR}"
echo "   npm publish --access public"
