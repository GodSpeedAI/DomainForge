#!/usr/bin/env bash
# generate-release-notes.sh - Generate GitHub release notes
# Usage: ./scripts/generate-release-notes.sh [VERSION] [OPTIONS]
#
# Extracts content from CHANGELOG.md and generates RELEASE_NOTES.md

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

Generate GitHub release notes from CHANGELOG.md.

ARGUMENTS:
    VERSION         Version number (e.g., 0.7.0)
                    If not provided, reads from sea-core/Cargo.toml

OPTIONS:
    --dry-run       Show what would be generated without creating file
    -h, --help      Show this help message

OUTPUT:
    Creates RELEASE_NOTES.md with:
    - Version header
    - What's Changed section (from CHANGELOG)
    - Breaking Changes (if any BREAKING CHANGE: footers found)
    - Contributors list

EXAMPLES:
    $(basename "$0")              # Generate for current version
    $(basename "$0") 0.7.0        # Generate for specific version
    $(basename "$0") --dry-run    # Preview without creating file
EOF
}

get_current_version() {
    grep -m1 '^version = ' "$PROJECT_ROOT/sea-core/Cargo.toml" | sed 's/version = "\(.*\)"/\1/'
}

get_last_tag() {
    git describe --tags --abbrev=0 2>/dev/null || echo ""
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

# ============================================================================
# Main
# ============================================================================
cd "$PROJECT_ROOT"

echo ""
echo "=============================================="
echo "  DomainForge Release Notes Generator"
echo "=============================================="
echo ""

if $DRY_RUN; then
    log_warn "DRY RUN MODE - No files will be created"
    echo ""
fi

log_info "Generating release notes for version: $VERSION"

DATE=$(date +%Y-%m-%d)

# Start building release notes
NOTES="# Release v$VERSION ($DATE)\n\n"

# Extract changelog section for this version
if [[ -f "CHANGELOG.md" ]]; then
    log_info "Extracting changes from CHANGELOG.md..."
    
    # Find the section for this version
    CHANGELOG_SECTION=$(awk -v version="$VERSION" '
        /^## \[/ {
            if (found) exit;
            if ($0 ~ "\\[" version "\\]") found=1;
        }
        found { print }
    ' CHANGELOG.md | tail -n +2)
    
    if [[ -n "$CHANGELOG_SECTION" ]]; then
        NOTES+="## What's Changed\n\n"
        NOTES+="$CHANGELOG_SECTION\n"
    else
        log_warn "No changelog section found for version $VERSION"
        NOTES+="## What's Changed\n\n"
        NOTES+="See [CHANGELOG.md](./CHANGELOG.md) for details.\n\n"
    fi
else
    log_warn "CHANGELOG.md not found"
    NOTES+="## What's Changed\n\n"
    NOTES+="See commit history for details.\n\n"
fi

# Check for breaking changes in commits
LAST_TAG=$(get_last_tag)
if [[ -n "$LAST_TAG" ]]; then
    BREAKING_CHANGES=$(git log "$LAST_TAG..HEAD" --grep="BREAKING CHANGE" --pretty=format:"- %s" 2>/dev/null || true)
    
    if [[ -n "$BREAKING_CHANGES" ]]; then
        NOTES+="## ⚠️ Breaking Changes\n\n"
        NOTES+="$BREAKING_CHANGES\n\n"
    fi
fi

# Get contributors
log_info "Collecting contributors..."
if [[ -n "$LAST_TAG" ]]; then
    CONTRIBUTORS=$(git log "$LAST_TAG..HEAD" --format="%an" 2>/dev/null | sort -u | head -20 || true)
else
    CONTRIBUTORS=$(git log --format="%an" -20 2>/dev/null | sort -u | head -10 || true)
fi

if [[ -n "$CONTRIBUTORS" ]]; then
    NOTES+="## Contributors\n\n"
    while IFS= read -r contributor; do
        [[ -n "$contributor" ]] && NOTES+="- @$contributor\n"
    done <<< "$CONTRIBUTORS"
    NOTES+="\n"
fi

# Add links
NOTES+="## Links\n\n"
NOTES+="- [Full Changelog](./CHANGELOG.md)\n"
NOTES+="- [Documentation](./docs/)\n"

if [[ -n "$LAST_TAG" ]]; then
    NOTES+="- [Compare with previous version](https://github.com/GodSpeedAI/DomainForge/compare/$LAST_TAG...v$VERSION)\n"
fi

# Preview or write
echo ""
log_info "Generated release notes:"
echo "---"
echo -e "$NOTES"
echo "---"

if $DRY_RUN; then
    log_info "Would write above content to RELEASE_NOTES.md"
else
    echo -e "$NOTES" > RELEASE_NOTES.md
    log_success "Created RELEASE_NOTES.md"
fi

echo ""
echo "=============================================="
log_success "Release notes generation complete"
echo "=============================================="
