#!/usr/bin/env bash
# bump-version.sh - Semantic version bumping for DomainForge
# Usage: ./scripts/bump-version.sh [major|minor|patch|X.Y.Z] [OPTIONS]
#
# Updates version in:
# - sea-core/Cargo.toml
# - pyproject.toml  
# - package.json

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
NO_COMMIT=false

# ============================================================================
# Helper Functions
# ============================================================================
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[✓]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[✗]${NC} $1"; }

usage() {
    cat <<EOF
Usage: $(basename "$0") [VERSION_TYPE] [OPTIONS]

Bump version across all DomainForge package files.

VERSION_TYPE:
    major           Bump major version (X.0.0)
    minor           Bump minor version (x.Y.0)
    patch           Bump patch version (x.y.Z)
    X.Y.Z           Set explicit version (e.g., 1.2.3)
    X.Y.Z-suffix    Set version with pre-release (e.g., 1.2.3-alpha)

OPTIONS:
    --dry-run       Show what would be changed without modifying files
    --no-commit     Update files but don't create git commit
    -h, --help      Show this help message

FILES UPDATED:
    - sea-core/Cargo.toml
    - pyproject.toml
    - package.json

EXAMPLES:
    $(basename "$0") patch                    # 0.6.2 -> 0.6.3
    $(basename "$0") minor                    # 0.6.2 -> 0.7.0
    $(basename "$0") major                    # 0.6.2 -> 1.0.0
    $(basename "$0") 2.0.0                    # Set explicit version
    $(basename "$0") 1.0.0-beta.1             # Pre-release version
    $(basename "$0") patch --dry-run          # Preview changes
EOF
}

get_current_version() {
    grep -m1 '^version = ' "$PROJECT_ROOT/sea-core/Cargo.toml" | sed 's/version = "\(.*\)"/\1/'
}

calculate_new_version() {
    local current="$1"
    local bump_type="$2"
    
    # Handle explicit version
    if [[ "$bump_type" =~ ^[0-9]+\.[0-9]+\.[0-9]+ ]]; then
        echo "$bump_type"
        return
    fi
    
    # Parse current version (strip pre-release suffix for calculation)
    local base_version="${current%-*}"
    IFS='.' read -r major minor patch <<< "$base_version"
    
    case "$bump_type" in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        patch)
            patch=$((patch + 1))
            ;;
        *)
            log_error "Invalid bump type: $bump_type"
            exit 1
            ;;
    esac
    
    echo "${major}.${minor}.${patch}"
}

update_cargo_toml() {
    local current="$1"
    local new="$2"
    
    if $DRY_RUN; then
        log_info "Would update sea-core/Cargo.toml: $current -> $new"
    else
        sed -i "s/^version = \"$current\"/version = \"$new\"/" "$PROJECT_ROOT/sea-core/Cargo.toml"
        log_success "Updated sea-core/Cargo.toml"
    fi
}

update_pyproject_toml() {
    local current="$1"
    local new="$2"
    
    if $DRY_RUN; then
        log_info "Would update pyproject.toml: $current -> $new"
    else
        sed -i "s/^version = \"$current\"/version = \"$new\"/" "$PROJECT_ROOT/pyproject.toml"
        log_success "Updated pyproject.toml"
    fi
}

update_package_json() {
    local new="$1"
    
    if $DRY_RUN; then
        log_info "Would update package.json to: $new"
    else
        # Use jq to update version (avoids npm dependency issues)
        cd "$PROJECT_ROOT"
        jq --arg v "$new" '.version = $v' package.json > package.json.tmp && mv package.json.tmp package.json
        log_success "Updated package.json"
    fi
}

# ============================================================================
# Parse Arguments
# ============================================================================
VERSION_TYPE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --no-commit)
            NO_COMMIT=true
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
            if [[ -z "$VERSION_TYPE" ]]; then
                VERSION_TYPE="$1"
            else
                log_error "Too many arguments"
                usage
                exit 1
            fi
            shift
            ;;
    esac
done

if [[ -z "$VERSION_TYPE" ]]; then
    log_error "Version type required"
    usage
    exit 1
fi

# ============================================================================
# Main
# ============================================================================
cd "$PROJECT_ROOT"

echo ""
echo "=============================================="
echo "  DomainForge Version Bump"
echo "=============================================="
echo ""

if $DRY_RUN; then
    log_warn "DRY RUN MODE - No files will be modified"
    echo ""
fi

# Get current version
CURRENT_VERSION=$(get_current_version)
log_info "Current version: $CURRENT_VERSION"

# Calculate new version
NEW_VERSION=$(calculate_new_version "$CURRENT_VERSION" "$VERSION_TYPE")
log_info "New version:     $NEW_VERSION"
echo ""

# Validate new version
if [[ "$CURRENT_VERSION" == "$NEW_VERSION" ]]; then
    log_warn "New version is same as current version"
fi

# Update files
log_info "Updating version files..."
update_cargo_toml "$CURRENT_VERSION" "$NEW_VERSION"
update_pyproject_toml "$CURRENT_VERSION" "$NEW_VERSION"
update_package_json "$NEW_VERSION"

# Create git commit
if ! $DRY_RUN && ! $NO_COMMIT; then
    echo ""
    log_info "Creating git commit..."
    git add sea-core/Cargo.toml pyproject.toml package.json
    git commit -m "chore: bump version to v$NEW_VERSION" --quiet
    log_success "Created commit: chore: bump version to v$NEW_VERSION"
fi

echo ""
echo "=============================================="
log_success "Version bumped to $NEW_VERSION"
echo "=============================================="

# Output just the version for use in scripts
if $DRY_RUN; then
    echo ""
    echo "# Output (for script chaining):"
fi
echo "$NEW_VERSION"
