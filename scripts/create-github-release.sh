#!/usr/bin/env bash
# create-github-release.sh - Create GitHub release using gh CLI
# Usage: ./scripts/create-github-release.sh [VERSION] [OPTIONS]

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
DRAFT=false

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

Create a GitHub release using the GitHub CLI.

ARGUMENTS:
    VERSION         Version number (e.g., 0.7.0)
                    If not provided, reads from sea-core/Cargo.toml

OPTIONS:
    --dry-run       Show what would be done without creating release
    --draft         Create as draft release
    -h, --help      Show this help message

REQUIREMENTS:
    - GitHub CLI (gh) must be installed and authenticated
    - Tag must exist: v{VERSION}
    - RELEASE_NOTES.md should exist for release body

BEHAVIOR:
    - Creates release from existing tag
    - Uploads all artifacts from dist/ directory
    - Marks as pre-release if version contains alpha/beta/rc
    - Generates additional notes via --generate-notes

EXAMPLES:
    $(basename "$0")              # Create release for current version
    $(basename "$0") 0.7.0        # Create for specific version
    $(basename "$0") --dry-run    # Preview without creating
    $(basename "$0") --draft      # Create as draft
EOF
}

get_current_version() {
    grep -m1 '^version = ' "$PROJECT_ROOT/sea-core/Cargo.toml" | sed 's/version = "\(.*\)"/\1/'
}

is_prerelease() {
    local version="$1"
    [[ "$version" =~ (alpha|beta|rc|pre) ]]
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
        --draft)
            DRAFT=true
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

TAG_NAME="v$VERSION"
TAG_NAME="${TAG_NAME//vv/v}"

# ============================================================================
# Main
# ============================================================================
cd "$PROJECT_ROOT"

echo ""
echo "=============================================="
echo "  DomainForge GitHub Release"
echo "=============================================="
echo ""

if $DRY_RUN; then
    log_warn "DRY RUN MODE - No release will be created"
    echo ""
fi

# Check prerequisites
log_info "Checking prerequisites..."

# Check gh CLI
if ! command -v gh &>/dev/null; then
    log_error "GitHub CLI (gh) is not installed"
    log_info "Install with: https://cli.github.com/"
    exit 1
fi
log_success "GitHub CLI found"

# Check authentication
if ! gh auth status &>/dev/null; then
    log_error "GitHub CLI is not authenticated"
    log_info "Authenticate with: gh auth login"
    exit 1
fi
log_success "GitHub CLI authenticated"

# Check tag exists
if ! git rev-parse "$TAG_NAME" &>/dev/null; then
    log_error "Tag $TAG_NAME does not exist"
    log_info "Create with: git tag -a $TAG_NAME -m 'Release $TAG_NAME'"
    exit 1
fi
log_success "Tag $TAG_NAME exists"

log_info "Version: $VERSION"
log_info "Tag: $TAG_NAME"

# Build gh release command
GH_CMD="gh release create $TAG_NAME"
GH_CMD+=" --title \"Release $TAG_NAME\""

# Add release notes if available
if [[ -f "RELEASE_NOTES.md" ]]; then
    log_info "Using RELEASE_NOTES.md for release body"
    GH_CMD+=" --notes-file RELEASE_NOTES.md"
else
    log_info "No RELEASE_NOTES.md, using auto-generated notes"
    GH_CMD+=" --generate-notes"
fi

# Check for pre-release
if is_prerelease "$VERSION"; then
    log_info "Marking as pre-release (version contains alpha/beta/rc)"
    GH_CMD+=" --prerelease"
fi

# Draft flag
if $DRAFT; then
    log_info "Creating as draft release"
    GH_CMD+=" --draft"
fi

# Add artifacts
DIST_DIR="$PROJECT_ROOT/dist"
if [[ -d "$DIST_DIR" ]]; then
    ARTIFACT_COUNT=0
    for artifact in "$DIST_DIR"/*; do
        if [[ -f "$artifact" ]]; then
            GH_CMD+=" \"$artifact\""
            ARTIFACT_COUNT=$((ARTIFACT_COUNT + 1))
        fi
    done
    
    if [[ $ARTIFACT_COUNT -gt 0 ]]; then
        log_info "Will upload $ARTIFACT_COUNT artifacts from dist/"
    else
        log_warn "No artifacts found in dist/"
    fi
else
    log_warn "dist/ directory not found, no artifacts to upload"
fi

# Execute or preview
echo ""
log_info "Command:"
echo "  $GH_CMD"
echo ""

if $DRY_RUN; then
    log_info "Would execute the above command"
else
    log_info "Creating GitHub release..."
    
    # Build and execute the command properly
    RELEASE_CMD="gh release create $TAG_NAME --title \"Release $TAG_NAME\""
    
    if [[ -f "RELEASE_NOTES.md" ]]; then
        RELEASE_CMD+=" --notes-file RELEASE_NOTES.md"
    else
        RELEASE_CMD+=" --generate-notes"
    fi
    
    if is_prerelease "$VERSION"; then
        RELEASE_CMD+=" --prerelease"
    fi
    
    if $DRAFT; then
        RELEASE_CMD+=" --draft"
    fi
    
    # Add artifacts to the command
    if [[ -d "$DIST_DIR" ]]; then
        for artifact in "$DIST_DIR"/*; do
            if [[ -f "$artifact" ]]; then
                RELEASE_CMD+=" \"$artifact\""
            fi
        done
    fi
    
    # Execute using eval
    if eval "$RELEASE_CMD"; then
        log_success "GitHub release created successfully"
        
        # Show release URL
        RELEASE_URL=$(gh release view "$TAG_NAME" --json url -q '.url' 2>/dev/null || true)
        if [[ -n "$RELEASE_URL" ]]; then
            echo ""
            log_info "Release URL: $RELEASE_URL"
        fi
    else
        log_error "Failed to create GitHub release"
        exit 1
    fi
fi

echo ""
echo "=============================================="
log_success "GitHub release process complete"
echo "=============================================="
