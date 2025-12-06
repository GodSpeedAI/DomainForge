# Onboarding Contributors

Welcome to the DomainForge team! This guide will get you set up and ready to contribute.

## 1. Development Environment

You need:

- **Rust**: Latest stable (`rustup update`)
- **Python**: 3.8+
- **Node.js**: 16+
- **Just**: Command runner (`cargo install just`)

## 2. Initial Setup

Run the setup script to install dependencies for all languages.

```bash
just setup
```

## 3. Codebase Tour

- **`sea-core/`**: The brain. Start here.
  - `grammar/`: The syntax definition.
  - `primitives/`: The data structures.
- **`tests/`**: Integration tests.
- **`docs/`**: You are here.

## 4. Running Tests

Before submitting a PR, ensure everything passes.

```bash
just all-tests
```

This runs:

1. `cargo test` (Core)
2. `pytest` (Python bindings)
3. `vitest` (TypeScript bindings)

## 5. Your First Issue

Look for issues labeled `good first issue`.

- **Documentation**: Fix typos or add examples.
- **Parser**: Add a small syntax feature.
- **CLI**: Improve output formatting.

## 6. PR Process

1. Fork the repo.
2. Create a feature branch.
3. Make changes.
4. Add tests! (We love tests).
5. Run `cargo fmt` and `cargo clippy`.
6. Submit PR.

## See Also

- [Architecture Overview](../explanations/architecture-overview.md)
- [Developer Commands](../../../justfile)
