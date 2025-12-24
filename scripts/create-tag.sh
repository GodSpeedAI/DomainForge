#!/usr/bin/env bash
# create-tag.sh - Create annotated git tag for release
# Usage: ./scripts/create-tag.sh [VERSION] [OPTIONS]

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
FORCE=false

# ============================================================================
# Helper Functions
# ============================================================================
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[✓]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[✗]${NC} $1"; }

usage() {
    cat <<EOF
Usage: $(basename "$0") [VERSION] [OPTIONS]

Create an annotated git tag for a release.

ARGUMENTS:
    VERSION         Version number (e.g., 0.7.0 or v0.7.0)
                    If not provided, reads from sea-core/Cargo.toml

OPTIONS:
    --dry-run       Show what would be done without creating tag
    --force         Overwrite existing tag
    -h, --help      Show this help message

BEHAVIOR:
    - Creates annotated tag with format: v{VERSION}
    - Uses RELEASE_NOTES.md as tag message (if exists)
    - Uses GPG signing if configured
    - Does NOT push the tag

EXAMPLES:
    $(basename "$0")              # Tag with current version
    $(basename "$0") 0.7.0        # Tag specific version
    $(basename "$0") --dry-run    # Preview without creating
    $(basename "$0") --force      # Overwrite existing tag
EOF
}

get_current_version() {
    grep -m1 '^version = ' "$PROJECT_ROOT/sea-core/Cargo.toml" | sed 's/version = "\(.*\)"/\1/'
}

has_gpg_signing() {
    git config --get user.signingkey &>/dev/null
}

# ============================================================================
# Parse Arguments
# ============================================================================
VERSION=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --force)
            FORCE=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        -*)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
        *)
            if [[ -z "$VERSION" ]]; then
                VERSION="$1"
            fi
            shift
            ;;
    esac
done

if [[ -z "$VERSION" ]]; then
    VERSION=$(get_current_version)
fi

# Ensure version has v prefix for tag
TAG_NAME="v$VERSION"
TAG_NAME="${TAG_NAME//vv/v}"  # Remove double v if already prefixed

# ============================================================================
# Main
# ============================================================================
cd "$PROJECT_ROOT"

echo ""
echo "=============================================="
echo "  DomainForge Git Tagging"
echo "=============================================="
echo ""

if $DRY_RUN; then
    log_warn "DRY RUN MODE - No tag will be created"
    echo ""
fi

log_info "Tag name: $TAG_NAME"

# Check if tag already exists
if git rev-parse "$TAG_NAME" &>/dev/null; then
    if $FORCE; then
        log_warn "Tag $TAG_NAME exists, will be overwritten (--force)"
        if ! $DRY_RUN; then
            git tag -d "$TAG_NAME" 2>/dev/null || true
        fi
    else
        log_error "Tag $TAG_NAME already exists. Use --force to overwrite."
        exit 1
    fi
fi

# Determine tag message source
if [[ -f "RELEASE_NOTES.md" ]]; then
    log_info "Using RELEASE_NOTES.md for tag message"
    TAG_MSG_FILE="RELEASE_NOTES.md"
else
    log_info "RELEASE_NOTES.md not found, using default message"
    TAG_MSG_FILE=""
fi

# Check for GPG signing
USE_GPG=false
if has_gpg_signing; then
    log_info "GPG signing key configured"
    USE_GPG=true
else
    log_info "No GPG signing key configured (unsigned tag)"
fi

# Create the tag
if $DRY_RUN; then
    log_info "Would create tag: $TAG_NAME"
    if $USE_GPG; then
        log_info "Would use: git tag -s $TAG_NAME"
    else
        log_info "Would use: git tag -a $TAG_NAME"
    fi
    if [[ -n "$TAG_MSG_FILE" ]]; then
        log_info "Would use message from: $TAG_MSG_FILE"
    fi
else
    if $USE_GPG; then
        if [[ -n "$TAG_MSG_FILE" ]]; then
            git tag -s "$TAG_NAME" -F "$TAG_MSG_FILE"
        else
            git tag -s "$TAG_NAME" -m "Release $TAG_NAME"
        fi
    else
        if [[ -n "$TAG_MSG_FILE" ]]; then
            git tag -a "$TAG_NAME" -F "$TAG_MSG_FILE"
        else
            git tag -a "$TAG_NAME" -m "Release $TAG_NAME"
        fi
    fi
    
    log_success "Created tag: $TAG_NAME"
    
    # Verify tag
    if git rev-parse "$TAG_NAME" &>/dev/null; then
        log_success "Tag verified successfully"
        git show "$TAG_NAME" --quiet --format="Commit: %H%nDate: %ci%nTagger: %an <%ae>"
    else
        log_error "Tag verification failed"
        exit 1
    fi
fi

echo ""
echo "=============================================="
log_success "Tagging complete"
echo ""
log_info "To push the tag:"
echo "    git push origin $TAG_NAME"
echo "=============================================="
