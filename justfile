default := "ai-validate"
cargo := `if command -v cargo >/dev/null 2>&1; then printf 'cargo'; elif command -v devbox >/dev/null 2>&1; then printf 'devbox run -- cargo'; else printf 'cargo'; fi`

# Run all validation checks required for CI and AI agents
ai-validate:
    {{cargo}} test -p domainforge-core --features cli

# Test tasks - run per-language test suites
rust-test:
    @echo "Running Rust tests..."
    {{cargo}} test -p domainforge-core --features cli

# Test all CLI commands
cli-test:
    {{cargo}} test -p domainforge-core --test cli_tests --features cli

# Run CLI validation examples
cli-validate:
    cd domainforge-core/examples/cli && ./validate_example.sh

# Run CLI import/export workflow
cli-workflow:
    cd domainforge-core/examples/cli && ./import_export_workflow.sh

build-rust-tests:
    @echo "Build Rust tests without running them (to prepare debug binaries)"
    {{cargo}} test -p domainforge-core --features cli --no-run

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
    if command -v bun >/dev/null 2>&1; then \
        bun test; \
    else \
        npm run test:node --silent; \
    fi

all-tests:
    @echo "Running all tests (Rust, Python, TypeScript)..."
    just rust-test
    just python-test
    just ts-test
prepare-rust-debug:
    @echo "Prepare a Rust test binary for codelldb debugging (build & symlink)"
    if [ -z "${RUST_TEST_NAME-}" ]; then RUST_TEST_NAME="entity_tests"; fi
    echo "Using RUST_TEST_NAME=${RUST_TEST_NAME}"
    ./scripts/prepare_rust_debug.sh "${RUST_TEST_NAME}"
clear-rust-debug:
    @echo "Clear symlink created for rust debug binary"
    ./scripts/clear_debug_test.sh || true
python-setup:
    @echo "Setting up Python virtualenv and installing dev dependencies..."
    rm -rf .venv
    python -m venv .venv || python3 -m venv .venv || (echo "Python 3 is required; please install python."; exit 1)
    .venv/bin/python -m pip install --upgrade pip
    # Install runtime requirements if present
    if [ -f requirements.txt ]; then .venv/bin/python -m pip install -r requirements.txt || true; fi
    # Install development requirements if present
    if [ -f requirements-dev.txt ]; then .venv/bin/python -m pip install -r requirements-dev.txt || true; fi
    # Install dev/test tooling
    .venv/bin/python -m pip install -U pytest maturin
    # Build & install the Python bindings into the venv.
    # pyproject.toml lives in domainforge-python/ and declares manifest-path, features,
    # and module-name under [tool.maturin]. Do NOT pass --manifest-path on the CLI:
    # maturin resolves the pyproject from the manifest dir when it's given, which makes
    # it ignore [tool.maturin] (features/module-name) and fall back to cffi. Invoke from
    # domainforge-python/ so maturin reads the pyproject next to it.
    cd domainforge-python && ../.venv/bin/maturin develop --release || (cd domainforge-python && ../.venv/bin/maturin develop)

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
    # Install Git hooks if lefthook is available from local dependencies or PATH
    npx --yes lefthook install || true
    # Prepare a Python virtualenv and install dev dependencies if possible
    just python-setup || true

python-clean:
    @echo "Removing Python virtual environment (.venv)"
    if [ -d .venv ]; then rm -rf .venv || true; fi

audit:
    cargo audit
    cargo deny check

smoke-test-npm:
    @echo "Running npm pack smoke test..."
    @output=$(cd domainforge-typescript && npm pack --dry-run 2>&1); \
    echo "$output"; \
    echo "$output" | grep -q "index\.js" || { echo "ERROR: index.js missing from pack"; exit 1; }; \
    echo "$output" | grep -q "index\.d\.ts" || { echo "ERROR: index.d.ts missing from pack"; exit 1; }; \
    echo "$output" | grep -q "domainforge-core.*\.node" || { echo "ERROR: native .node binary missing from pack"; exit 1; }; \
    echo "$output" | grep -q "README\.md" || { echo "ERROR: README.md missing from pack"; exit 1; }; \
    echo "$output" | grep -q "LICENSE" || { echo "ERROR: LICENSE missing from pack"; exit 1; }; \
    echo "npm pack smoke test passed"

smoke-test-wasm:
    @echo "Running WASM pkg smoke test..."
    test -d target/wasm-pkg || { echo "ERROR: WASM pkg not built. Run scripts/build-wasm.sh first"; exit 1; }
    for f in domainforge_core.js domainforge_core.d.ts domainforge_core_bg.wasm domainforge_core_bg.wasm.d.ts package.json; do \
        test -f target/wasm-pkg/$f || { echo "ERROR: missing $f"; exit 1; }; \
    done
    @echo "WASM smoke test passed"

enterprise-verify:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Enterprise Release Verification ==="
    echo "1/6: Format check..."
    cargo fmt --all --check
    echo "2/6: Clippy..."
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    echo "3/6: Rust tests..."
    cargo test --workspace --all-targets --all-features
    echo "4/6: Doctests..."
    cargo test -p domainforge-core --features cli --doc
    echo "5/6: All language tests..."
    just all-tests
    echo "6/6: Audit..."
    just audit
    echo "=== Enterprise Verification PASSED ==="

# ============================================================================
# Self-proving harness — `just prove`
# ============================================================================
# Runs the full proof suite from the current worktree and writes a machine- and
# human-readable evidence pack to evidence/latest/. Each sub-recipe is
# independently runnable and fails nonzero on any error. See PROOFS.md.

# Full proof suite: gate unit/integration tests, then every proof, then evidence.
prove: rust-test prove-language prove-canonical prove-projections prove-drift prove-evidence
    @echo "=== just prove: PASSED — see evidence/latest/proof.md ==="

# Parse + validate + format-stability on fixtures (+ negative fixtures rejected).
prove-language:
    bash scripts/prove/language.sh

# Determinism: two isolated RDF projections, byte compare.
prove-canonical:
    bash scripts/prove/canonical.sh

# Every projection-target gate (all.sh) + projection-contract parity.
prove-projections:
    bash scripts/prove/projections.sh

# Round-trip (CALM + KG) and declared-drift via pack diff.
prove-drift:
    bash scripts/prove/roundtrip.sh
    bash scripts/prove/drift.sh

# Merge fragments into evidence/latest/proof.json + proof.md (runs LAST).
prove-evidence:
    bash scripts/prove/collect-evidence.sh

# Cell-environment projection: byte-determinism + structural + native checks.
cell-verify:
    bash scripts/verify/cell.sh

# ============================================================================
# CI-specific recipes (used by GitHub Actions)
# ============================================================================

# Run Rust tests (CI variant)
ci-test-rust:
    @echo "Running Rust tests (CI)..."
    {{cargo}} test --verbose --workspace --features cli

# Run Python tests (CI variant)
ci-test-python:
    @echo "Running Python tests (CI)..."
    .venv/bin/python -m pytest tests/

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
ci-verify-package ARCHIVE_PATH BINARY_NAME="domainforge":
    @echo "Verifying packaged archive: {{ARCHIVE_PATH}}"
    python3 scripts/ci_tasks.py unpack-verify --archive "{{ARCHIVE_PATH}}" --binary-name "{{BINARY_NAME}}"

# Check packaged artifact size
ci-check-package-size ARCHIVE_PATH MAX_BYTES="73400320":
    @echo "Checking package size: {{ARCHIVE_PATH}}"
    python3 scripts/ci_tasks.py check-size --file "{{ARCHIVE_PATH}}" --max-bytes {{MAX_BYTES}} --label "Packaged artifact"
