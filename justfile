default := "ai-validate"

# Run all validation checks required for CI and AI agents
ai-validate:
    cargo test -p sea-core --features cli

# Test tasks - run per-language test suites
rust-test:
    @echo "Running Rust tests..."
    cargo test -p sea-core --features cli

# Test all CLI commands
cli-test:
    cargo test -p sea-core --test cli_tests --features cli

# Run CLI validation examples
cli-validate:
    cd sea-core/examples/cli && ./validate_example.sh

# Run CLI import/export workflow
cli-workflow:
    cd sea-core/examples/cli && ./import_export_workflow.sh

build-rust-tests:
    @echo "Build Rust tests without running them (to prepare debug binaries)"
    cargo test -p sea-core --features cli --no-run

python-test:
    @echo "Running Python tests..."
    # Prefer the virtualenv python if it exists to avoid system site-packages and PEP 668 restrictions
    if [ -x ".venv/bin/python" ]; then \
        .venv/bin/python -m pytest -q; \
    else \
        python3 -m pytest -q || python -m pytest -q; \
    fi

ts-test:
    @echo "Running TypeScript tests (Vitest)..."
    if command -v bun >/dev/null 2>&1; then bun vitest run; else npm run test:node; fi

# Run TypeScript tests with Bun explicitly (faster)
bun-test:
    @echo "Running TypeScript tests with Bun..."
    bun vitest run

# Install dependencies with Bun (faster than npm)
bun-install:
    @echo "Installing dependencies with Bun..."
    bun install

all-tests:
    @echo "Running all tests (Rust, Python, TypeScript)..."
    just rust-test
    just python-test
    just ts-test
prepare-rust-debug:
    @echo "Prepare a Rust test binary for codelldb debugging (build & symlink)"
    if [ -z "$${RUST_TEST_NAME-}" ]; then RUST_TEST_NAME="entity_tests"; fi
    echo "Using RUST_TEST_NAME=$${RUST_TEST_NAME}"
    ./scripts/prepare_rust_debug.sh "$${RUST_TEST_NAME}"
clear-rust-debug:
    @echo "Clear symlink created for rust debug binary"
    ./scripts/clear_debug_test.sh || true
python-setup:
    @echo "Setting up Python virtualenv and installing dev dependencies..."
    python3 -m venv .venv || python -m venv .venv || (echo "Python 3 is required; please install python3."; exit 1)
    .venv/bin/python -m pip install --upgrade pip
    # Install runtime requirements if present
    if [ -f requirements.txt ]; then .venv/bin/python -m pip install -r requirements.txt || true; fi
    # Install development requirements if present
    if [ -f requirements-dev.txt ]; then .venv/bin/python -m pip install -r requirements-dev.txt || true; fi
    # Install dev/test tooling
    .venv/bin/python -m pip install -U pytest maturin
    # Build & install the Python bindings into the venv
    .venv/bin/maturin develop --manifest-path sea-core/Cargo.toml --release || .venv/bin/maturin develop --manifest-path sea-core/Cargo.toml

install-just:
    @echo "Checking if 'just' is already installed..."
    if command -v just >/dev/null 2>&1; then \
        echo "just is already installed: $(just --version)"; exit 0; \
    fi
    @echo "Attempting to install 'just' via OS package manager..."
    if command -v winget >/dev/null 2>&1; then \
        echo "Installing via winget..."; winget install --id Just -e || true; \
    fi;
    if command -v scoop >/dev/null 2>&1; then \
        echo "Installing via scoop..."; scoop install just || true; \
    fi;
    if command -v brew >/dev/null 2>&1; then \
        brew install just || true; \
    fi;
    if command -v apt-get >/dev/null 2>&1; then \
        sudo apt-get update && sudo apt-get install -y just || true; \
    fi;
    if command -v dnf >/dev/null 2>&1; then \
        sudo dnf install -y just || true; \
    fi;
    if command -v pacman >/dev/null 2>&1; then \
        sudo pacman -S --noconfirm just || true; \
    fi;
    @echo "Fallback: attempt to install via cargo..."
    cargo install just --locked || (echo "cargo install just failed; please install manually using your OS package manager (brew, apt, dnf, pacman) or visit https://github.com/casey/just#installation"; exit 1)

setup:
    @echo "Installing TypeScript dependencies and Python dev dependencies..."
    bun install || npm ci || npm install
    # Prepare a Python virtualenv and install dev dependencies if possible
    just python-setup || true

python-clean:
    @echo "Removing Python virtual environment (.venv)"
    if [ -d .venv ]; then rm -rf .venv || true; fi

# ============================================================================
# CI-specific recipes (used by GitHub Actions)
# ============================================================================

# Run Rust tests (CI variant)
ci-test-rust:
    @echo "Running Rust tests (CI)..."
    cargo test --verbose --workspace --features cli

# Run Python tests (CI variant)
ci-test-python:
    @echo "Running Python tests (CI)..."
    # Prefer the virtualenv python if it exists to avoid system site-packages and PEP 668 restrictions
    if [ -x ".venv/bin/python" ]; then \
        .venv/bin/python -m pytest tests/; \
    else \
        python3 -m pytest tests/ || python -m pytest tests/; \
    fi

# Run TypeScript tests (CI variant, with Node.js fallback)
ci-test-ts:
    @echo "Running TypeScript tests (CI)..."
    if command -v bun >/dev/null 2>&1; then bun vitest run; else just ci-test-ts-node; fi

# Run TypeScript tests with Node.js fallback (for Windows CI)
ci-test-ts-node:
    @echo "Running TypeScript tests with Node.js..."
    npm run test:node

# Verify CLI binary can execute
ci-verify-binary BINARY_PATH:
    @echo "Verifying CLI binary: {{BINARY_PATH}}"
    python3 scripts/ci_tasks.py verify-cli --binary "{{BINARY_PATH}}"

# Check CLI binary size
ci-check-binary-size BINARY_PATH MAX_BYTES="52428800":
    @echo "Checking binary size: {{BINARY_PATH}}"
    python3 scripts/ci_tasks.py check-size --file "{{BINARY_PATH}}" --max-bytes {{MAX_BYTES}} --label "CLI binary"

# Package CLI binary for release
ci-package-binary BINARY_PATH OUTPUT_PATH:
    @echo "Packaging binary: {{BINARY_PATH}} -> {{OUTPUT_PATH}}"
    python3 scripts/ci_tasks.py package --input "{{BINARY_PATH}}" --output "{{OUTPUT_PATH}}"

# Verify packaged archive
ci-verify-package ARCHIVE_PATH BINARY_NAME="sea":
    @echo "Verifying packaged archive: {{ARCHIVE_PATH}}"
    python3 scripts/ci_tasks.py unpack-verify --archive "{{ARCHIVE_PATH}}" --binary-name "{{BINARY_NAME}}"

# Check packaged artifact size
ci-check-package-size ARCHIVE_PATH MAX_BYTES="73400320":
    @echo "Checking package size: {{ARCHIVE_PATH}}"
    python3 scripts/ci_tasks.py check-size --file "{{ARCHIVE_PATH}}" --max-bytes {{MAX_BYTES}} --label "Packaged artifact"

# Run full local CI pipeline (test, build release, check size, package)
ci-pipeline:
    @echo "🚀 Running full local CI pipeline..."
    just ci-test-rust
    just ci-test-python
    just ci-test-ts
    @echo "📦 Building release binary..."
    cargo build -p sea-core --release
    just ci-verify-binary target/release/sea
    just ci-check-binary-size target/release/sea
    just ci-package-binary target/release/sea sea.tar.gz
    just ci-verify-package sea.tar.gz sea
    just ci-check-package-size sea.tar.gz
    @echo "✅ CI pipeline passed!"

# Check debug binary size (with higher limit)
ci-check-debug-size BINARY_PATH="target/debug/sea" MAX_BYTES="104857600":
    @echo "Checking debug binary size (limit: 100MB)..."
    just ci-check-binary-size {{BINARY_PATH}} {{MAX_BYTES}}

# ============================================================================
# Merge Automation Recipes
# ============================================================================

# Run CI-equivalent checks (format, lint, tests)
ci-check:
    @echo "🔍 Running CI-equivalent checks..."
    cargo fmt --all -- --check
    cargo clippy -p sea-core -- -D warnings
    just all-tests
    @echo "✅ All CI checks passed"

# Pre-merge validation (run before merging any feature branch)
pre-merge:
    @echo "🔍 Running pre-merge checks..."
    cargo fmt --all -- --check
    cargo clippy -p sea-core -- -D warnings
    just all-tests
    @echo "✅ All pre-merge checks passed"

# Dry run of merge workflow (preview without changes)
merge-to-main-dry branch:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "🔍 DRY RUN: Merge workflow for '{{branch}}'"
    echo ""
    echo "Would perform the following steps:"
    echo "  1. git fetch --all --prune"
    echo "  2. git checkout {{branch}}"
    echo "  3. git merge origin/main --no-edit"
    echo "  4. Run: cargo fmt --all -- --check"
    echo "  5. Run: cargo clippy -p sea-core -- -D warnings"
    echo "  6. Run: just all-tests"
    echo "  7. git checkout main"
    echo "  8. git pull --ff-only origin main"
    echo "  9. git merge --no-ff {{branch}} -m 'Merge {{branch}} into main'"
    echo " 10. Run: just all-tests"
    echo " 11. git push origin main"
    echo " 12. git branch -d {{branch}}"
    echo " 13. git push origin --delete {{branch}}"
    echo ""
    echo "Current branch status:"
    git branch -a | grep -E "({{branch}}|main)" || echo "Branch not found locally"
    echo ""
    echo "To execute, run: just merge-to-main {{branch}}"

# Full merge workflow: validate, merge to main, cleanup
merge-to-main branch:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "🚀 Starting merge workflow for '{{branch}}'"
    echo ""

    echo "📥 Step 1/7: Fetching latest changes..."
    git fetch --all --prune

    echo "📋 Step 2/7: Validating branch '{{branch}}' exists..."
    if ! git show-ref --verify --quiet refs/heads/{{branch}}; then
        echo "❌ Branch '{{branch}}' not found locally"
        echo "   Available branches:"
        git branch | head -20
        exit 1
    fi

    echo "🔀 Step 3/7: Checking out and updating feature branch..."
    git checkout {{branch}}
    git merge origin/main --no-edit || { echo "❌ Merge conflict with main. Resolve manually."; exit 1; }

    echo "🧪 Step 4/7: Running pre-merge checks on feature branch..."
    cargo fmt --all -- --check || { echo "❌ Format check failed"; exit 1; }
    cargo clippy -p sea-core -- -D warnings || { echo "❌ Clippy check failed"; exit 1; }
    just all-tests || { echo "❌ Tests failed on feature branch"; exit 1; }

    echo "🔀 Step 5/7: Merging to main..."
    git checkout main
    git pull --ff-only origin main || { echo "❌ Failed to update main"; exit 1; }
    git merge --no-ff {{branch}} -m "Merge {{branch}} into main" || { echo "❌ Merge failed"; exit 1; }

    echo "🧪 Step 6/7: Running post-merge validation on main..."
    just all-tests || { echo "❌ Tests failed after merge. Consider reverting."; exit 1; }

    echo "⬆️ Step 7/7: Pushing and cleaning up..."
    git push origin main || { echo "❌ Failed to push main"; exit 1; }
    git branch -d {{branch}} || echo "⚠️ Could not delete local branch"
    git push origin --delete {{branch}} || echo "⚠️ Remote branch already deleted or doesn't exist"

    echo ""
    echo "✅ Successfully merged '{{branch}}' into main!"
    echo "   - All checks passed"
    echo "   - Branch deleted locally and remotely"
    echo "   - Main pushed to origin"

# ============================================================================
# Local Release Preparation (mirrors .github/workflows/prepare-release.yml)
# ============================================================================

# Preview release changes (shows diff + generates replay script, no file modifications)
release-preview bump="patch" prerelease="":
    #!/usr/bin/env bash
    set -euo pipefail
    
    echo "🔍 RELEASE PREVIEW: {{bump}} bump"
    echo "=================================="
    echo ""
    
    # Get current version
    CURRENT=$(cargo metadata --manifest-path sea-core/Cargo.toml --format-version 1 | jq -r '.packages[] | select(.name == "sea-core") | .version')
    echo "📌 Current version: $CURRENT"
    
    # Parse and bump
    IFS='.' read -r MAJOR MINOR PATCH <<< "${CURRENT%-*}"
    case "{{bump}}" in
        major) MAJOR=$((MAJOR + 1)); MINOR=0; PATCH=0 ;;
        minor) MINOR=$((MINOR + 1)); PATCH=0 ;;
        patch) PATCH=$((PATCH + 1)) ;;
        *) echo "Unknown bump type: {{bump}}"; exit 1 ;;
    esac
    
    NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
    if [ -n "{{prerelease}}" ]; then
        NEW_VERSION="${NEW_VERSION}-{{prerelease}}"
    fi
    echo "📌 New version: $NEW_VERSION"
    echo ""
    
    # Create temp directory for simulation
    TMPDIR=$(mktemp -d)
    trap "rm -rf $TMPDIR" EXIT
    
    # Copy files to temp for diffing
    cp sea-core/Cargo.toml "$TMPDIR/Cargo.toml.orig"
    cp pyproject.toml "$TMPDIR/pyproject.toml.orig"
    cp package.json "$TMPDIR/package.json.orig"
    cp CHANGELOG.md "$TMPDIR/CHANGELOG.md.orig" 2>/dev/null || echo "# Changelog" > "$TMPDIR/CHANGELOG.md.orig"
    
    # Generate modified versions
    sed "s/^version = \"$CURRENT\"/version = \"$NEW_VERSION\"/" sea-core/Cargo.toml > "$TMPDIR/Cargo.toml.new"
    sed "s/^version = \"$CURRENT\"/version = \"$NEW_VERSION\"/" pyproject.toml > "$TMPDIR/pyproject.toml.new"
    jq ".version = \"$NEW_VERSION\"" package.json > "$TMPDIR/package.json.new"
    
    # Generate changelog entry
    DATE=$(date +%Y-%m-%d)
    H="###"
    ENTRY=$(printf '%s\n' \
        "## [$NEW_VERSION] - $DATE" \
        "" \
        "$H Added" \
        "- (Add new features here)" \
        "" \
        "$H Changed" \
        "- (Add changes here)" \
        "" \
        "$H Fixed" \
        "- (Add bug fixes here)" \
        "")
    
    # Insert changelog entry
    if grep -q "^## \[" "$TMPDIR/CHANGELOG.md.orig"; then
        FIRST_LINE=$(grep -n "^## \[" "$TMPDIR/CHANGELOG.md.orig" | head -1 | cut -d: -f1)
        head -n $((FIRST_LINE - 1)) "$TMPDIR/CHANGELOG.md.orig" > "$TMPDIR/CHANGELOG.md.new"
        echo "$ENTRY" >> "$TMPDIR/CHANGELOG.md.new"
        tail -n "+$FIRST_LINE" "$TMPDIR/CHANGELOG.md.orig" >> "$TMPDIR/CHANGELOG.md.new"
    else
        cat "$TMPDIR/CHANGELOG.md.orig" > "$TMPDIR/CHANGELOG.md.new"
        echo "" >> "$TMPDIR/CHANGELOG.md.new"
        echo "$ENTRY" >> "$TMPDIR/CHANGELOG.md.new"
    fi
    
    # Show diffs
    echo "📝 DIFF PREVIEW"
    echo "==============="
    echo ""
    echo "--- sea-core/Cargo.toml ---"
    diff -u "$TMPDIR/Cargo.toml.orig" "$TMPDIR/Cargo.toml.new" || true
    echo ""
    echo "--- pyproject.toml ---"
    diff -u "$TMPDIR/pyproject.toml.orig" "$TMPDIR/pyproject.toml.new" || true
    echo ""
    echo "--- package.json ---"
    diff -u "$TMPDIR/package.json.orig" "$TMPDIR/package.json.new" || true
    echo ""
    echo "--- CHANGELOG.md (first 40 lines) ---"
    diff -u "$TMPDIR/CHANGELOG.md.orig" "$TMPDIR/CHANGELOG.md.new" | head -50 || true
    echo ""
    
    # Generate replay script using printf (heredocs don't work well in justfile)
    SCRIPT_PATH="release-$NEW_VERSION.sh"
    {
        echo '#!/usr/bin/env bash'
        echo '# Auto-generated release script'
        echo '# Generated by: just release-preview'
        echo 'set -euo pipefail'
        echo ''
        echo "NEW_VERSION=\"$NEW_VERSION\""
        echo "CURRENT_VERSION=\"$CURRENT\""
        echo ''
        echo 'echo "🚀 Applying release $NEW_VERSION..."'
        echo ''
        echo '# Update Cargo.toml'
        echo 'sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" sea-core/Cargo.toml'
        echo 'echo "✓ Updated sea-core/Cargo.toml"'
        echo ''
        echo '# Update pyproject.toml'
        echo 'sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" pyproject.toml'
        echo 'echo "✓ Updated pyproject.toml"'
        echo ''
        echo '# Update package.json'
        echo 'npm version "$NEW_VERSION" --no-git-tag-version --allow-same-version'
        echo 'echo "✓ Updated package.json"'
        echo ''
        echo '# Update CHANGELOG.md'
        echo 'just changelog-entry "$NEW_VERSION"'
        echo 'echo "✓ Updated CHANGELOG.md"'
        echo ''
        echo 'echo ""'
        echo 'echo "✅ Release $NEW_VERSION prepared!"'
        echo 'echo "   Review changes: git diff"'
        echo 'echo "   Commit: git add -A && git commit -m '"'"'chore: bump version to $NEW_VERSION'"'"'"'
    } > "$SCRIPT_PATH"
    
    chmod +x "$SCRIPT_PATH"
    
    echo ""
    echo "📜 GENERATED REPLAY SCRIPT"
    echo "=========================="
    echo "Script saved to: $SCRIPT_PATH"
    echo ""
    echo "To apply these changes, run:"
    echo "  ./$SCRIPT_PATH"
    echo ""
    echo "Or to execute directly:"
    echo "  just prepare-release {{bump}} {{prerelease}}"

# Actually prepare a release locally (modifies files, no git commits)
prepare-release bump="patch" prerelease="":
    #!/usr/bin/env bash
    set -euo pipefail
    echo "🚀 Preparing release ({{bump}})..."
    
    # Get current version
    CURRENT=$(cargo metadata --manifest-path sea-core/Cargo.toml --format-version 1 | jq -r '.packages[] | select(.name == "sea-core") | .version')
    echo "Current version: $CURRENT"
    
    # Parse and bump
    IFS='.' read -r MAJOR MINOR PATCH <<< "${CURRENT%-*}"
    case "{{bump}}" in
        major) MAJOR=$((MAJOR + 1)); MINOR=0; PATCH=0 ;;
        minor) MINOR=$((MINOR + 1)); PATCH=0 ;;
        patch) PATCH=$((PATCH + 1)) ;;
        *) echo "Unknown bump type: {{bump}}"; exit 1 ;;
    esac
    
    NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
    if [ -n "{{prerelease}}" ]; then
        NEW_VERSION="${NEW_VERSION}-{{prerelease}}"
    fi
    echo "New version: $NEW_VERSION"
    
    # Update Cargo.toml
    echo "📝 Updating sea-core/Cargo.toml..."
    sed -i "s/^version = \"$CURRENT\"/version = \"$NEW_VERSION\"/" sea-core/Cargo.toml
    
    # Update pyproject.toml
    echo "📝 Updating pyproject.toml..."
    sed -i "s/^version = \"$CURRENT\"/version = \"$NEW_VERSION\"/" pyproject.toml
    
    # Update package.json
    echo "📝 Updating package.json..."
    npm version "$NEW_VERSION" --no-git-tag-version --allow-same-version
    
    # Update CHANGELOG.md
    echo "📝 Updating CHANGELOG.md..."
    just changelog-entry "$NEW_VERSION"
    
    echo ""
    echo "✅ Files updated for v$NEW_VERSION"
    echo "   Review changes: git diff"
    echo "   Commit: git add -A && git commit -m 'chore: bump version to $NEW_VERSION'"

# Add a changelog entry for a version
changelog-entry version:
    #!/usr/bin/env bash
    set -euo pipefail
    DATE=$(date +%Y-%m-%d)
    H="###"
    ENTRY=$(printf '%s\n' \
        "## [{{version}}] - ${DATE}" \
        "" \
        "${H} Added" \
        "- (Add new features here)" \
        "" \
        "${H} Changed" \
        "- (Add changes here)" \
        "" \
        "${H} Fixed" \
        "- (Add bug fixes here)" \
        "")
    
    if [ -f CHANGELOG.md ]; then
        FIRST_VERSION_LINE=$(grep -n "^## \[" CHANGELOG.md | head -1 | cut -d: -f1 || echo "")
        if [ -n "$FIRST_VERSION_LINE" ]; then
            head -n $((FIRST_VERSION_LINE - 1)) CHANGELOG.md > /tmp/changelog_new.md
            echo "$ENTRY" >> /tmp/changelog_new.md
            tail -n "+$FIRST_VERSION_LINE" CHANGELOG.md >> /tmp/changelog_new.md
        else
            cat CHANGELOG.md > /tmp/changelog_new.md
            echo "" >> /tmp/changelog_new.md
            echo "$ENTRY" >> /tmp/changelog_new.md
        fi
        mv /tmp/changelog_new.md CHANGELOG.md
    else
        echo "# Changelog" > CHANGELOG.md
        echo "" >> CHANGELOG.md
        echo "$ENTRY" >> CHANGELOG.md
    fi
    echo "Added changelog entry for {{version}}"

# Test the changelog entry logic (safe - restores original file)
test-changelog-logic:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Testing changelog entry logic..."
    cp CHANGELOG.md /tmp/changelog_backup.md 2>/dev/null || echo "# Changelog" > /tmp/changelog_backup.md
    just changelog-entry "0.0.0-test"
    echo "--- Result (first 30 lines) ---"
    head -30 CHANGELOG.md
    echo "--- Restoring original ---"
    mv /tmp/changelog_backup.md CHANGELOG.md
    echo "✅ Test complete (original restored)"

