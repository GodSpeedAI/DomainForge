#!/usr/bin/env bash
# build-release.sh - Build release artifacts for current platform
# Usage: ./scripts/build-release.sh [OPTIONS]

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Flags
DRY_RUN=false
SKIP_CLI=false
SKIP_PYTHON=false
SKIP_WASM=false

# ============================================================================
# Helper Functions
# ============================================================================
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[✓]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[✗]${NC} $1"; }

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Build release artifacts for the current platform.

OPTIONS:
    --dry-run       Show what would be built without building
    --skip-cli      Skip CLI binary build
    --skip-python   Skip Python wheel build
    --skip-wasm     Skip WASM bundle build
    -h, --help      Show this help message

ARTIFACTS:
    - CLI binary (sea or sea.exe)
    - Python wheel (.whl)
    - WASM bundle (pkg/)
    - SHA256 checksums

OUTPUT:
    All artifacts placed in dist/ directory

EXAMPLES:
    $(basename "$0")                  # Build all artifacts
    $(basename "$0") --dry-run        # Preview build commands
    $(basename "$0") --skip-wasm      # Build CLI and Python only
EOF
}

get_current_version() {
    grep -m1 '^version = ' "$PROJECT_ROOT/sea-core/Cargo.toml" | sed 's/version = "\(.*\)"/\1/'
}

detect_platform() {
    case "$(uname -s)" in
        Linux*)     echo "linux" ;;
        Darwin*)    echo "darwin" ;;
        CYGWIN*|MINGW*|MSYS*) echo "windows" ;;
        *)          echo "unknown" ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   echo "x86_64" ;;
        aarch64|arm64)  echo "aarch64" ;;
        *)              echo "unknown" ;;
    esac
}

# ============================================================================
# Parse Arguments
# ============================================================================
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --skip-cli)
            SKIP_CLI=true
            shift
            ;;
        --skip-python)
            SKIP_PYTHON=true
            shift
            ;;
        --skip-wasm)
            SKIP_WASM=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# ============================================================================
# Main
# ============================================================================
cd "$PROJECT_ROOT"

echo ""
echo "=============================================="
echo "  DomainForge Release Build"
echo "=============================================="
echo ""

if $DRY_RUN; then
    log_warn "DRY RUN MODE - No builds will be executed"
    echo ""
fi

VERSION=$(get_current_version)
PLATFORM=$(detect_platform)
ARCH=$(detect_arch)

log_info "Version:  $VERSION"
log_info "Platform: $PLATFORM"
log_info "Arch:     $ARCH"
echo ""

# Clean/create dist directory
DIST_DIR="$PROJECT_ROOT/dist"
if $DRY_RUN; then
    log_info "Would clean/create: $DIST_DIR"
else
    rm -rf "$DIST_DIR"
    mkdir -p "$DIST_DIR"
    log_success "Created dist directory"
fi

# Build CLI
if ! $SKIP_CLI; then
    log_info "Building CLI binary..."
    if $DRY_RUN; then
        log_info "Would run: cargo build -p sea-core --release --features cli"
    else
        cargo build -p sea-core --release --features cli
        
        # Find and copy binary
        if [[ "$PLATFORM" == "windows" ]]; then
            CLI_BIN="target/release/sea.exe"
            CLI_OUT="$DIST_DIR/sea-$VERSION-$PLATFORM-$ARCH.exe"
        else
            CLI_BIN="target/release/sea"
            CLI_OUT="$DIST_DIR/sea-$VERSION-$PLATFORM-$ARCH"
        fi
        
        if [[ -f "$CLI_BIN" ]]; then
            cp "$CLI_BIN" "$CLI_OUT"
            chmod +x "$CLI_OUT"
            log_success "Built CLI: $CLI_OUT"
            
            # Create compressed archive
            ARCHIVE_NAME="sea-$VERSION-$PLATFORM-$ARCH.tar.gz"
            if [[ "$PLATFORM" == "windows" ]]; then
                ARCHIVE_NAME="sea-$VERSION-$PLATFORM-$ARCH.zip"
            fi
            
            cd "$DIST_DIR"
            if [[ "$PLATFORM" == "windows" ]]; then
                # Use zip on Windows
                if command -v zip &>/dev/null; then
                    zip "$ARCHIVE_NAME" "$(basename "$CLI_OUT")"
                else
                    log_warn "zip not found, skipping archive"
                fi
            else
                tar czf "$ARCHIVE_NAME" "$(basename "$CLI_OUT")"
            fi
            cd "$PROJECT_ROOT"
            log_success "Created archive: $ARCHIVE_NAME"
        else
            log_error "CLI binary not found at $CLI_BIN"
        fi
    fi
else
    log_info "Skipping CLI build (--skip-cli)"
fi

# Build Python wheel
if ! $SKIP_PYTHON; then
    log_info "Building Python wheel..."
    if $DRY_RUN; then
        log_info "Would run: maturin build --release --features python --out dist"
    else
        if command -v maturin &>/dev/null; then
            maturin build --release --features python --out "$DIST_DIR"
            log_success "Built Python wheel in dist/"
        else
            log_warn "maturin not found, skipping Python wheel"
            log_info "Install with: pip install maturin"
        fi
    fi
else
    log_info "Skipping Python build (--skip-python)"
fi

# Build WASM
if ! $SKIP_WASM; then
    log_info "Building WASM bundle..."
    if $DRY_RUN; then
        log_info "Would run: wasm-pack build --target web --release --features wasm"
    else
        if command -v wasm-pack &>/dev/null; then
            cd sea-core
            wasm-pack build --target web --release --features wasm
            
            # Package WASM artifacts
            WASM_ARCHIVE="$DIST_DIR/sea-core-wasm-$VERSION.tar.gz"
            cd pkg
            tar czf "$WASM_ARCHIVE" *
            cd "$PROJECT_ROOT"
            log_success "Built WASM: $WASM_ARCHIVE"
        else
            log_warn "wasm-pack not found, skipping WASM build"
            log_info "Install with: cargo install wasm-pack"
        fi
    fi
else
    log_info "Skipping WASM build (--skip-wasm)"
fi

# Generate checksums
if ! $DRY_RUN && [[ -d "$DIST_DIR" ]]; then
    log_info "Generating checksums..."
    cd "$DIST_DIR"
    
    # Generate SHA256 checksums
    CHECKSUM_FILE="SHA256SUMS.txt"
    rm -f "$CHECKSUM_FILE"
    
    for file in *; do
        if [[ -f "$file" && "$file" != "$CHECKSUM_FILE" ]]; then
            if command -v sha256sum &>/dev/null; then
                sha256sum "$file" >> "$CHECKSUM_FILE"
            elif command -v shasum &>/dev/null; then
                shasum -a 256 "$file" >> "$CHECKSUM_FILE"
            fi
        fi
    done
    
    cd "$PROJECT_ROOT"
    
    if [[ -f "$DIST_DIR/$CHECKSUM_FILE" ]]; then
        log_success "Generated checksums: $CHECKSUM_FILE"
    fi
fi

# Summary
echo ""
echo "=============================================="
log_success "Build complete"
echo ""
if ! $DRY_RUN; then
    log_info "Artifacts in $DIST_DIR:"
    ls -lh "$DIST_DIR" 2>/dev/null || true
fi
echo "=============================================="
