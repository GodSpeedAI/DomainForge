#!/usr/bin/env bash
# release.sh - Master release orchestration script for DomainForge
# Usage: ./scripts/release.sh [major|minor|patch|X.Y.Z] [OPTIONS]
#
# Orchestrates the complete release process:
# 1. Pre-release checks
# 2. Version bump
# 3. Changelog generation
# 4. Release notes generation
# 5. Git tag creation
# 6. Build artifacts
# 7. Push and create GitHub release

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
CYAN='\033[0;36m'
NC='\033[0m'

# Flags
DRY_RUN=false
SKIP_TESTS=false
SKIP_BUILD=false
AUTO_CONFIRM=false

# ============================================================================
# Helper Functions
# ============================================================================
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[✓]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[✗]${NC} $1"; }
log_step() { echo -e "${CYAN}[STEP]${NC} $1"; }

banner() {
    echo ""
    echo -e "${CYAN}=============================================="
    echo "  🚀 DomainForge Release Process"
    echo "==============================================${NC}"
    echo ""
}

usage() {
    cat <<EOF
Usage: $(basename "$0") [VERSION_TYPE] [OPTIONS]

Complete release orchestration for DomainForge.

VERSION_TYPE:
    major           Bump major version (X.0.0)
    minor           Bump minor version (x.Y.0)
    patch           Bump patch version (x.y.Z) [default]
    X.Y.Z           Set explicit version (e.g., 1.2.3)

OPTIONS:
    --dry-run       Run through all steps without making changes
    --skip-tests    Skip running test suites (faster, less safe)
    --skip-build    Skip building release artifacts
    --yes, -y       Auto-confirm push and release creation
    -h, --help      Show this help message

RELEASE PROCESS:
    1. Run pre-release checks (git status, tests, version sync)
    2. Bump version in all package files
    3. Generate changelog from commits
    4. Generate release notes for GitHub
    5. Commit version changes
    6. Create annotated git tag
    7. Build release artifacts (CLI, Python, WASM)
    8. Push to remote (with confirmation)
    9. Create GitHub release with artifacts

EXAMPLES:
    $(basename "$0")                  # Patch release (0.6.2 -> 0.6.3)
    $(basename "$0") minor            # Minor release (0.6.2 -> 0.7.0)
    $(basename "$0") major            # Major release (0.6.2 -> 1.0.0)
    $(basename "$0") --dry-run        # Preview full release process
    $(basename "$0") patch --yes      # Patch release with auto-confirm

REQUIREMENTS:
    - git
    - cargo (Rust toolchain)
    - just (optional, for tests)
    - maturin (for Python wheels)
    - wasm-pack (for WASM bundle)
    - gh (GitHub CLI, for release creation)

ROLLBACK:
    If the release fails after version bump:
        git reset --hard HEAD~1
        git tag -d v{VERSION}
EOF
}

confirm() {
    local prompt="$1"
    local default="${2:-n}"
    
    if $AUTO_CONFIRM; then
        return 0
    fi
    
    if $DRY_RUN; then
        log_info "Would prompt: $prompt"
        return 0
    fi
    
    local answer
    read -r -p "$prompt [y/N] " answer
    [[ "$answer" =~ ^[Yy]$ ]]
}

run_script() {
    local script="$1"
    shift
    local args=("$@")
    
    if $DRY_RUN; then
        args+=("--dry-run")
    fi
    
    if [[ -x "$SCRIPT_DIR/$script" ]]; then
        "$SCRIPT_DIR/$script" "${args[@]}"
    else
        log_error "Script not found or not executable: $script"
        exit 1
    fi
}

# ============================================================================
# Parse Arguments
# ============================================================================
VERSION_TYPE="patch"

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --yes|-y)
            AUTO_CONFIRM=true
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
            VERSION_TYPE="$1"
            shift
            ;;
    esac
done

# ============================================================================
# Main
# ============================================================================
cd "$PROJECT_ROOT"

banner

if $DRY_RUN; then
    log_warn "DRY RUN MODE - No changes will be made"
    echo ""
fi

log_info "Release type: $VERSION_TYPE"
echo ""

# ============================================================================
# Step 1: Pre-release checks
# ============================================================================
log_step "Step 1/9: Running pre-release checks..."
echo ""

PRE_CHECK_ARGS=()
if $SKIP_TESTS; then
    PRE_CHECK_ARGS+=("--skip-tests")
fi
if $DRY_RUN; then
    PRE_CHECK_ARGS+=("--dry-run")
fi

if ! "$SCRIPT_DIR/pre-release-check.sh" "${PRE_CHECK_ARGS[@]}"; then
    log_error "Pre-release checks failed. Fix issues before releasing."
    exit 1
fi
echo ""

# ============================================================================
# Step 2: Bump version
# ============================================================================
log_step "Step 2/9: Bumping version..."
echo ""

BUMP_ARGS=("$VERSION_TYPE" "--no-commit")
if $DRY_RUN; then
    BUMP_ARGS+=("--dry-run")
fi

# Capture new version from bump script
NEW_VERSION=$("$SCRIPT_DIR/bump-version.sh" "${BUMP_ARGS[@]}" | tail -1)
log_success "New version: $NEW_VERSION"
echo ""

# ============================================================================
# Step 3: Generate changelog
# ============================================================================
log_step "Step 3/9: Generating changelog..."
echo ""

CHANGELOG_ARGS=("$NEW_VERSION" "--no-commit")
if $DRY_RUN; then
    CHANGELOG_ARGS+=("--dry-run")
fi

"$SCRIPT_DIR/generate-changelog.sh" "${CHANGELOG_ARGS[@]}"
echo ""

# ============================================================================
# Step 4: Generate release notes
# ============================================================================
log_step "Step 4/9: Generating release notes..."
echo ""

NOTES_ARGS=("$NEW_VERSION")
if $DRY_RUN; then
    NOTES_ARGS+=("--dry-run")
fi

"$SCRIPT_DIR/generate-release-notes.sh" "${NOTES_ARGS[@]}"
echo ""

# ============================================================================
# Step 5: Commit version changes
# ============================================================================
log_step "Step 5/9: Committing changes..."
echo ""

if $DRY_RUN; then
    log_info "Would commit: chore: release v$NEW_VERSION"
else
    git add -A
    git commit -m "chore: release v$NEW_VERSION" || log_warn "Nothing to commit or commit failed"
    log_success "Created commit: chore: release v$NEW_VERSION"
fi
echo ""

# ============================================================================
# Step 6: Create git tag
# ============================================================================
log_step "Step 6/9: Creating git tag..."
echo ""

TAG_ARGS=("$NEW_VERSION")
if $DRY_RUN; then
    TAG_ARGS+=("--dry-run")
fi

"$SCRIPT_DIR/create-tag.sh" "${TAG_ARGS[@]}"
echo ""

# ============================================================================
# Step 7: Build artifacts
# ============================================================================
if ! $SKIP_BUILD; then
    log_step "Step 7/9: Building release artifacts..."
    echo ""
    
    BUILD_ARGS=()
    if $DRY_RUN; then
        BUILD_ARGS+=("--dry-run")
    fi
    
    "$SCRIPT_DIR/build-release.sh" "${BUILD_ARGS[@]}"
    echo ""
else
    log_step "Step 7/9: Skipping build (--skip-build)"
    echo ""
fi

# ============================================================================
# Step 8: Push to remote
# ============================================================================
log_step "Step 8/9: Preparing to push..."
echo ""

BRANCH=$(git rev-parse --abbrev-ref HEAD)
TAG_NAME="v$NEW_VERSION"

log_warn "Ready to push the following:"
echo "  - Branch: $BRANCH"
echo "  - Tag: $TAG_NAME"
echo ""

if $DRY_RUN; then
    log_info "Would push: git push origin $BRANCH"
    log_info "Would push: git push origin $TAG_NAME"
else
    if confirm "Push to remote and create GitHub release?"; then
        log_info "Pushing branch..."
        git push origin "$BRANCH"
        log_success "Pushed branch: $BRANCH"
        
        log_info "Pushing tag..."
        git push origin "$TAG_NAME"
        log_success "Pushed tag: $TAG_NAME"
        echo ""
        
        # ============================================================================
        # Step 9: Create GitHub release
        # ============================================================================
        log_step "Step 9/9: Creating GitHub release..."
        echo ""
        
        GH_ARGS=("$NEW_VERSION")
        "$SCRIPT_DIR/create-github-release.sh" "${GH_ARGS[@]}"
        
        echo ""
        echo -e "${GREEN}=============================================="
        echo "  ✨ Release v$NEW_VERSION completed!"
        echo "==============================================${NC}"
        echo ""
        log_success "All release steps completed successfully"
        echo ""
        log_info "Next steps:"
        echo "  1. Verify GitHub release: https://github.com/GodSpeedAI/DomainForge/releases"
        echo "  2. Wait for CI to publish to npm, PyPI, and crates.io"
        echo "  3. Announce the release"
    else
        echo ""
        log_warn "Release cancelled. To undo local changes:"
        echo "    git reset --hard HEAD~1"
        echo "    git tag -d $TAG_NAME"
        exit 1
    fi
fi

echo ""
log_success "Release process complete"
