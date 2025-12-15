# Contributing to DomainForge

Thank you for your interest in contributing to DomainForge! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Building the Project](#building-the-project)
- [Running Tests](#running-tests)
- [Code Style](#code-style)
- [Pull Request Process](#pull-request-process)
- [Documentation](#documentation)
- [Reporting Issues](#reporting-issues)

## Code of Conduct

This project follows a standard code of conduct. Please be respectful and constructive in all interactions.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/DomainForge.git
   cd DomainForge
   ```
3. **Add the upstream remote**:
   ```bash
   git remote add upstream https://github.com/GodSpeedAI/DomainForge.git
   ```

## Development Setup

### Prerequisites

- **Rust** 1.75+ (install via [rustup](https://rustup.rs/))
- **Python** 3.8+ with pip
- **Node.js** 18+ or **Bun** 1.1+
- **just** command runner: `cargo install just`

### Optional Dependencies

- **maturin**: For Python bindings (`pip install maturin`)
- **wasm-pack**: For WASM builds (`cargo install wasm-pack`)
- **buf**: For protobuf linting ([buf.build](https://buf.build/docs/installation))

### Environment Setup

```bash
# Install Rust toolchain
rustup default stable
rustup component add rustfmt clippy

# Install just (task runner)
cargo install just

# Install Python development dependencies
pip install maturin pytest

# Install Node.js/Bun dependencies
bun install  # or: npm install
```

## Building the Project

### Rust Core

```bash
# Build the library
cargo build -p sea-core

# Build with CLI feature
cargo build -p sea-core --features cli

# Build in release mode
cargo build -p sea-core --release --features cli
```

### Python Bindings

```bash
# Development build (editable)
maturin develop

# Release build
maturin build --release
```

### TypeScript Bindings

```bash
# Build TypeScript bindings
bun run build
# or: npm run build
```

### WASM

```bash
# Build WASM package
cd sea-core
wasm-pack build --target web --release --features wasm
```

### Using Just

The project uses `just` as a task runner:

```bash
just --list            # Show all available tasks
just build             # Build all components
just test              # Run all tests
just fmt               # Format code
just lint              # Run linters
```

## Running Tests

### All Tests

```bash
just all-tests
```

### Per-Language Tests

```bash
# Rust tests
just ci-test-rust
# or: cargo test -p sea-core --all-features

# Python tests
just ci-test-python
# or: pytest

# TypeScript tests
just ci-test-ts
# or: bun test
```

### Specific Test

```bash
# Run a specific Rust test
cargo test -p sea-core test_name

# Run tests with output
cargo test -p sea-core -- --nocapture
```

## Code Style

### Rust

- Follow standard Rust conventions
- Use `cargo fmt` before committing
- Ensure `cargo clippy -- -D warnings` passes
- Add doc comments for public APIs

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings
```

### Documentation

- Use `//!` for module-level docs
- Use `///` for function/struct docs
- Include examples in doc comments

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

Examples:

```
feat(protobuf): add gRPC service generation from Flow patterns
fix(parser): resolve keyword ambiguity in Resource declarations
docs(how-tos): add export-to-protobuf guide
```

## Pull Request Process

1. **Create a feature branch** from `dev`:

   ```bash
   git checkout dev
   git pull upstream dev
   git checkout -b feature/my-feature
   ```

2. **Make your changes** with clear, focused commits

3. **Ensure all tests pass**:

   ```bash
   just all-tests
   ```

4. **Format and lint**:

   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   ```

5. **Push your branch**:

   ```bash
   git push origin feature/my-feature
   ```

6. **Open a Pull Request** against the `dev` branch

7. **Address review feedback** promptly

### PR Checklist

- [ ] Tests pass locally (`just all-tests`)
- [ ] Code is formatted (`cargo fmt --all --check`)
- [ ] Clippy passes (`cargo clippy -- -D warnings`)
- [ ] Documentation updated if needed
- [ ] CHANGELOG.md updated for user-facing changes
- [ ] Commit messages follow conventional commits

## Documentation

### Types of Documentation

| Location             | Purpose                      |
| -------------------- | ---------------------------- |
| `docs/tutorials/`    | Step-by-step learning guides |
| `docs/how-tos/`      | Task-focused guides          |
| `docs/reference/`    | API and CLI documentation    |
| `docs/explanations/` | Conceptual explanations      |
| `docs/specs/`        | ADRs, PRDs, and SDSs         |
| `docs/playbooks/`    | Operational procedures       |

### Documentation Guidelines

1. **For new features**: Add a how-to guide and update the relevant reference docs
2. **For API changes**: Update `primitives-api.md` and binding-specific docs
3. **For CLI changes**: Update `cli-commands.md`
4. **For architecture decisions**: Create an ADR in `docs/specs/`

### Building Documentation

```bash
# Generate Rust docs
cargo doc --no-deps --open

# Check documentation links
markdown-link-check docs/**/*.md
```

## Reporting Issues

### Bug Reports

Include:

- DomainForge version (`sea --version`)
- OS and version
- Minimal reproduction steps
- Expected vs actual behavior
- Relevant error messages

### Feature Requests

Include:

- Use case description
- Proposed solution (if any)
- Alternatives considered

## Project Structure

```
DomainForge/
├── sea-core/              # Rust core library
│   ├── src/
│   │   ├── bin/           # CLI binary
│   │   ├── cli/           # CLI command implementations
│   │   ├── parser/        # SEA DSL parser
│   │   ├── primitives/    # Core domain types
│   │   ├── projection/    # Export engines (protobuf, etc.)
│   │   ├── policy/        # Policy evaluation engine
│   │   ├── python/        # PyO3 bindings
│   │   ├── typescript/    # napi-rs bindings
│   │   └── wasm/          # wasm-bindgen bindings
│   ├── grammar/           # Pest grammar files
│   └── tests/             # Integration tests
├── docs/                  # Documentation
├── examples/              # Example models
└── scripts/               # Build and CI scripts
```

## Getting Help

- **Issues**: For bugs and feature requests
- **Discussions**: For questions and ideas
- **Pull Requests**: For code contributions

Thank you for contributing to DomainForge! 🎉
