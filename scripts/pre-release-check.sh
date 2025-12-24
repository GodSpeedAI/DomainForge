#!/usr/bin/env bash
# pre-release-check.sh - Pre-flight validation for releases
# Usage: ./scripts/pre-release-check.sh [--dry-run]
#
# Validates the repository is ready for a release:
# - No uncommitted changes
# - On correct branch (main, dev, release/*)
# - All tests pass
# - Version files are in sync
# - CHANGELOG.md is up to date

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Flags
DRY_RUN=false
VERBOSE=false

# ============================================================================
# Helper Functions
# ============================================================================
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Pre-release validation checks for DomainForge.

OPTIONS:
    --dry-run       Show what would be checked without running tests
    --verbose       Show verbose output
    --skip-tests    Skip running test suites (faster, less thorough)
    -h, --help      Show this help message

CHECKS PERFORMED:
    1. Git status - no uncommitted changes
    2. Branch validation - must be main, dev, or release/*
    3. Latest changes pulled from remote
    4. All test suites pass (Rust, Python, TypeScript)
    5. Version consistency across Cargo.toml, pyproject.toml, package.json
    6. CHANGELOG.md has entry for current version

EXIT CODES:
    0   All checks passed
    1   One or more checks failed

EXAMPLES:
    $(basename "$0")              # Run full pre-release checks
    $(basename "$0") --dry-run    # Preview what would be checked
    $(basename "$0") --skip-tests # Skip running tests (faster)
EOF
}

# ============================================================================
# Parse Arguments
# ============================================================================
SKIP_TESTS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
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
# Checks
# ============================================================================
cd "$PROJECT_ROOT"

FAILED=0

echo ""
echo "=============================================="
echo "  DomainForge Pre-Release Checks"
echo "=============================================="
echo ""

if $DRY_RUN; then
    log_warn "DRY RUN MODE - Showing what would be checked"
    echo ""
fi

# Check 1: Git status
log_step "Checking git status for uncommitted changes..."
if $DRY_RUN; then
    log_info "Would run: git status --porcelain"
else
    if [[ -n $(git status --porcelain) ]]; then
        log_error "Uncommitted changes detected:"
        git status --short
        FAILED=1
    else
        log_success "No uncommitted changes"
    fi
fi

# Check 2: Branch validation
log_step "Validating current branch..."
if $DRY_RUN; then
    log_info "Would check if branch matches: main, dev, or release/*"
else
    BRANCH=$(git rev-parse --abbrev-ref HEAD)
    if [[ "$BRANCH" =~ ^(main|dev|release/.*)$ ]]; then
        log_success "On valid branch: $BRANCH"
    else
        log_warn "On branch '$BRANCH' - releases typically done from main, dev, or release/*"
        # Not a hard failure, just a warning
    fi
fi

# Check 3: Pull latest changes
log_step "Checking for latest changes from remote..."
if $DRY_RUN; then
    log_info "Would run: git fetch origin && git status -uno"
else
    git fetch origin --quiet 2>/dev/null || log_warn "Could not fetch from origin"
    LOCAL=$(git rev-parse HEAD)
    REMOTE=$(git rev-parse "@{u}" 2>/dev/null || echo "")
    if [[ -z "$REMOTE" ]]; then
        log_warn "No upstream branch set"
    elif [[ "$LOCAL" != "$REMOTE" ]]; then
        BEHIND=$(git rev-list --count HEAD..@{u} 2>/dev/null || echo "0")
        if [[ "$BEHIND" -gt 0 ]]; then
            log_warn "Branch is $BEHIND commits behind remote. Consider pulling."
        fi
    else
        log_success "Up to date with remote"
    fi
fi

# Check 4: Version consistency
log_step "Checking version consistency across files..."
if $DRY_RUN; then
    log_info "Would compare versions in:"
    log_info "  - sea-core/Cargo.toml"
    log_info "  - pyproject.toml"
    log_info "  - package.json"
else
    CARGO_VERSION=$(grep -m1 '^version = ' sea-core/Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    PYPROJECT_VERSION=$(grep -m1 '^version = ' pyproject.toml | sed 's/version = "\(.*\)"/\1/')
    PKG_VERSION=$(grep -m1 '"version"' package.json | sed 's/.*"\([0-9][^"]*\)".*/\1/')

    log_info "Cargo.toml:    $CARGO_VERSION"
    log_info "pyproject.toml: $PYPROJECT_VERSION"
    log_info "package.json:   $PKG_VERSION"

    if [[ "$CARGO_VERSION" == "$PYPROJECT_VERSION" && "$CARGO_VERSION" == "$PKG_VERSION" ]]; then
        log_success "All versions match: $CARGO_VERSION"
    else
        log_error "Version mismatch across files!"
        FAILED=1
    fi
fi

# Check 5: CHANGELOG.md has entry for current version
log_step "Checking CHANGELOG.md..."
if $DRY_RUN; then
    log_info "Would verify CHANGELOG.md contains entry for current version"
else
    if [[ -f "CHANGELOG.md" ]]; then
        # Get the first version entry
        FIRST_CHANGELOG_VERSION=$(grep -m1 '^\#\# \[' CHANGELOG.md | sed 's/.*\[\([0-9][^]]*\)\].*/\1/' || echo "")
        if [[ -n "$FIRST_CHANGELOG_VERSION" ]]; then
            if [[ "$FIRST_CHANGELOG_VERSION" == "$CARGO_VERSION" ]]; then
                log_success "CHANGELOG.md has entry for current version ($CARGO_VERSION)"
            else
                log_warn "CHANGELOG.md top entry ($FIRST_CHANGELOG_VERSION) doesn't match current version ($CARGO_VERSION)"
            fi
        else
            log_warn "Could not find version entry in CHANGELOG.md"
        fi
    else
        log_error "CHANGELOG.md not found"
        FAILED=1
    fi
fi

# Check 6: Run tests
if $SKIP_TESTS; then
    log_step "Skipping tests (--skip-tests flag)"
else
    log_step "Running test suites..."
    if $DRY_RUN; then
        log_info "Would run: just all-tests (Rust + Python + TypeScript)"
    else
        if command -v just &> /dev/null; then
            log_info "Running: just all-tests"
            if just all-tests; then
                log_success "All tests passed"
            else
                log_error "Tests failed"
                FAILED=1
            fi
        else
            log_warn "'just' not found, attempting individual test commands..."
            
            # Try Rust tests
            log_info "Running Rust tests..."
            if cargo test -p sea-core --features cli; then
                log_success "Rust tests passed"
            else
                log_error "Rust tests failed"
                FAILED=1
            fi
        fi
    fi
fi

# Check 7: Verify build works
log_step "Verifying release build..."
if $DRY_RUN; then
    log_info "Would run: cargo build -p sea-core --release --features cli"
else
    if cargo build -p sea-core --release --features cli 2>/dev/null; then
        log_success "Release build successful"
    else
        log_error "Release build failed"
        FAILED=1
    fi
fi

# ============================================================================
# Summary
# ============================================================================
echo ""
echo "=============================================="
if [[ $FAILED -eq 0 ]]; then
    log_success "All pre-release checks passed!"
    echo "=============================================="
    exit 0
else
    log_error "Some pre-release checks failed"
    echo "=============================================="
    exit 1
fi
