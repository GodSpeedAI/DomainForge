#!/usr/bin/env bash
# generate-changelog.sh - Generate changelog entries from git commits
# Usage: ./scripts/generate-changelog.sh [VERSION] [OPTIONS]
#
# Parses conventional commits and updates CHANGELOG.md

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
Usage: $(basename "$0") [VERSION] [OPTIONS]

Generate changelog entries from git commits using conventional commit format.

ARGUMENTS:
    VERSION         Version number for the changelog entry (e.g., 0.7.0)
                    If not provided, reads from sea-core/Cargo.toml

OPTIONS:
    --dry-run       Show what would be generated without modifying files
    --no-commit     Update CHANGELOG.md but don't create git commit
    -h, --help      Show this help message

COMMIT CATEGORIES:
    feat:       -> 🎉 Added
    fix:        -> 🐛 Fixed
    docs:       -> 📚 Documentation
    refactor:   -> ✨ Changed
    perf:       -> ⚡ Performance
    test:       -> 🧪 Testing
    chore:      -> 🔧 Maintenance

EXAMPLES:
    $(basename "$0")                      # Generate for current version
    $(basename "$0") 0.7.0                # Generate for specific version
    $(basename "$0") --dry-run            # Preview without changes
EOF
}

get_last_tag() {
    git describe --tags --abbrev=0 2>/dev/null || echo ""
}

get_current_version() {
    grep -m1 '^version = ' "$PROJECT_ROOT/sea-core/Cargo.toml" | sed 's/version = "\(.*\)"/\1/'
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
            if [[ -z "$VERSION" ]]; then
                VERSION="$1"
            fi
            shift
            ;;
    esac
done

# Default to current version
if [[ -z "$VERSION" ]]; then
    VERSION=$(get_current_version)
fi

# ============================================================================
# Main
# ============================================================================
cd "$PROJECT_ROOT"

echo ""
echo "=============================================="
echo "  DomainForge Changelog Generator"
echo "=============================================="
echo ""

if $DRY_RUN; then
    log_warn "DRY RUN MODE - No files will be modified"
    echo ""
fi

log_info "Generating changelog for version: $VERSION"

# Get commit range
LAST_TAG=$(get_last_tag)
if [[ -n "$LAST_TAG" ]]; then
    log_info "Commits since tag: $LAST_TAG"
    COMMIT_RANGE="$LAST_TAG..HEAD"
else
    log_warn "No previous tags found, using all commits"
    COMMIT_RANGE=""
fi

# Parse commits into categories
declare -a ADDED=()
declare -a FIXED=()
declare -a CHANGED=()
declare -a DOCS=()
declare -a OTHER=()

while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    
    # Extract commit message (format: hash subject)
    subject="${line#* }"
    
    # Categorize by conventional commit prefix
    case "$subject" in
        feat:*|feat\(*) 
            ADDED+=("${subject#feat:} ")
            ADDED+=("${subject#feat(*)}")
            ;;
        fix:*|fix\(*)
            FIXED+=("${subject#fix:} ")
            ;;
        docs:*|docs\(*)
            DOCS+=("${subject#docs:} ")
            ;;
        refactor:*|perf:*|style:*)
            CHANGED+=("$subject")
            ;;
        chore:*|test:*|ci:*|build:*)
            # Skip maintenance commits from changelog
            ;;
        *)
            # Include non-conventional commits in other
            if [[ ! "$subject" =~ ^(Merge|chore|test|ci|build) ]]; then
                OTHER+=("$subject")
            fi
            ;;
    esac
done < <(git log $COMMIT_RANGE --oneline --no-merges 2>/dev/null || true)

# Generate changelog entry
DATE=$(date +%Y-%m-%d)
ENTRY="## [$VERSION] - $DATE"
ENTRY+="\n"

if [[ ${#ADDED[@]} -gt 0 ]]; then
    ENTRY+="\n### 🎉 Added\n"
    for item in "${ADDED[@]}"; do
        # Clean up the commit message
        clean_item=$(echo "$item" | sed 's/^[[:space:]]*//' | sed 's/[[:space:]]*$//')
        [[ -n "$clean_item" ]] && ENTRY+="- $clean_item\n"
    done
fi

if [[ ${#CHANGED[@]} -gt 0 ]]; then
    ENTRY+="\n### ✨ Changed\n"
    for item in "${CHANGED[@]}"; do
        clean_item=$(echo "$item" | sed 's/^[a-z]*:[[:space:]]*//')
        [[ -n "$clean_item" ]] && ENTRY+="- $clean_item\n"
    done
fi

if [[ ${#FIXED[@]} -gt 0 ]]; then
    ENTRY+="\n### 🐛 Fixed\n"
    for item in "${FIXED[@]}"; do
        clean_item=$(echo "$item" | sed 's/^[[:space:]]*//' | sed 's/[[:space:]]*$//')
        [[ -n "$clean_item" ]] && ENTRY+="- $clean_item\n"
    done
fi

if [[ ${#DOCS[@]} -gt 0 ]]; then
    ENTRY+="\n### 📚 Documentation\n"
    for item in "${DOCS[@]}"; do
        clean_item=$(echo "$item" | sed 's/^[[:space:]]*//' | sed 's/[[:space:]]*$//')
        [[ -n "$clean_item" ]] && ENTRY+="- $clean_item\n"
    done
fi

# If no categorized commits, add placeholder
if [[ ${#ADDED[@]} -eq 0 && ${#CHANGED[@]} -eq 0 && ${#FIXED[@]} -eq 0 && ${#DOCS[@]} -eq 0 ]]; then
    ENTRY+="\n### Added\n- (Add new features here)\n"
    ENTRY+="\n### Changed\n- (Add changes here)\n"
    ENTRY+="\n### Fixed\n- (Add bug fixes here)\n"
fi

ENTRY+="\n"

# Show preview
echo ""
log_info "Generated changelog entry:"
echo "---"
echo -e "$ENTRY"
echo "---"

# Update CHANGELOG.md
if $DRY_RUN; then
    log_info "Would insert above entry into CHANGELOG.md"
else
    if [[ -f "CHANGELOG.md" ]]; then
        # Find first version heading and insert before it
        FIRST_VERSION_LINE=$(grep -n "^## \[" CHANGELOG.md | head -1 | cut -d: -f1 || echo "")
        
        if [[ -n "$FIRST_VERSION_LINE" ]]; then
            head -n $((FIRST_VERSION_LINE - 1)) CHANGELOG.md > /tmp/changelog_new.md
            echo -e "$ENTRY" >> /tmp/changelog_new.md
            tail -n "+$FIRST_VERSION_LINE" CHANGELOG.md >> /tmp/changelog_new.md
            mv /tmp/changelog_new.md CHANGELOG.md
        else
            # No existing version entries, append
            echo -e "$ENTRY" >> CHANGELOG.md
        fi
        
        log_success "Updated CHANGELOG.md"
    else
        # Create new CHANGELOG.md
        echo "# Changelog" > CHANGELOG.md
        echo "" >> CHANGELOG.md
        echo -e "$ENTRY" >> CHANGELOG.md
        log_success "Created CHANGELOG.md"
    fi
    
    # Create git commit
    if ! $NO_COMMIT; then
        git add CHANGELOG.md
        git commit -m "docs: update changelog for v$VERSION" --quiet
        log_success "Created commit: docs: update changelog for v$VERSION"
    fi
fi

echo ""
echo "=============================================="
log_success "Changelog generation complete"
echo "=============================================="
