default := "ai-validate"

# Run all validation checks required for CI and AI agents
ai-validate:
    cargo test -p sea-core

# Test tasks - run per-language test suites
rust-test:
    @echo "Running Rust tests..."
    cargo test -p sea-core

build-rust-tests:
    @echo "Build Rust tests without running them (to prepare debug binaries)"
    cargo test -p sea-core --no-run

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
