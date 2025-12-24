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
    npm test --silent

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
    npm ci || npm install
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
    .venv/bin/pytest tests/

# Run TypeScript tests (CI variant)
ci-test-ts:
    @echo "Running TypeScript tests (CI)..."
    npm test

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
