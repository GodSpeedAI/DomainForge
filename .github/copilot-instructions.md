# DomainForge - SEA DSL Copilot Instructions

Targeted guidance for AI coding agents working on DomainForge (Semantic Enterprise Architecture DSL).

## Architecture Overview

DomainForge is a **Rust-first, multi-language** semantic modeling system:

```
sea-core/                    # Canonical Rust implementation (authoritative)
├── grammar/sea.pest         # PEG grammar (Pest) - ALL syntax changes start here
├── src/
│   ├── primitives/          # Core domain types: Entity, Resource, Flow, Instance, Policy
│   ├── graph/mod.rs         # Graph store (uses IndexMap for deterministic iteration)
│   ├── parser/              # AST generation from grammar
│   ├── policy/              # Expression evaluation, three-valued logic, type inference
│   ├── calm/                # FINOS CALM export/import (architecture-as-code)
│   ├── kg.rs, kg_import.rs  # RDF/Turtle Knowledge Graph projections
│   ├── python/              # PyO3 bindings (wraps core types)
│   ├── typescript/          # napi-rs bindings (wraps core types)
│   └── wasm/                # wasm-bindgen bindings
└── tests/                   # Integration tests (60+ test files)

tests/                       # Python integration tests
typescript-tests/            # TypeScript/Vitest integration tests
```

**Key principle**: Rust core is canonical. Bindings wrap core types — never duplicate business logic.

## Developer Commands

```bash
# Essential commands (use just task runner)
just all-tests              # Run Rust + Python + TypeScript tests (recommended)
just rust-test              # Rust tests with CLI: cargo test -p sea-core --features cli
just python-test            # Python tests (uses .venv if present)
just ts-test                # TypeScript tests via Vitest

# Setup
just setup                  # Install TypeScript + Python dependencies
just python-setup           # Create .venv, install deps, build Python bindings via maturin

# Debugging
just prepare-rust-debug     # Symlink test binary for codelldb debugging

# Pre-PR checks
cargo clippy -- -D warnings && cargo fmt
```

## Change Workflow (Non-Negotiable)

When modifying **core data structures or primitives**, you MUST update ALL of:

1. Rust core (`sea-core/src/primitives/`, `graph/`, `policy/`) + tests
2. PyO3 bindings (`sea-core/src/python/`) + Python tests (`tests/test_*.py`)
3. napi-rs bindings (`sea-core/src/typescript/`) + TypeScript tests (`typescript-tests/`)
4. WASM bindings if applicable (`sea-core/src/wasm/`)

When modifying **parser/grammar**:

1. Update `sea-core/grammar/sea.pest` first
2. Update AST in `sea-core/src/parser/ast.rs`
3. Add parser tests in `sea-core/tests/parser_*.rs`
4. Update projections (CALM, KG) if the change affects export format

## Code Conventions

- **Deterministic iteration**: Use `IndexMap` (not `HashMap`) in `graph/mod.rs` for policy-relevant collections
- **IDs**: `ConceptId` from namespace+name; `Uuid::new_v4()` for unique identifiers
- **Quantities**: `rust_decimal::Decimal` for precise calculations
- **Units**: `Unit::new()` constructor; `sea_core::units::unit_from_string()` for parsing
- **Namespaces**: `namespace()` returns `&str`, defaults to `"default"`
- **Flows**: `Flow::new()` takes `ConceptId` for resource/from/to (IDs, not references)

## Testing Patterns

- **Cross-language parity**: Changes to core must pass all three test suites
- **Round-trip tests**: CALM export/import (`cargo test calm_round_trip`) validates serialization
- **Golden tests**: Check `sea-core/tests/` for expected output patterns
- **Policy evaluation**: Three-valued logic tests in `three_valued_quantifiers_tests.rs`

## Key References

| What                 | Where                                                                 |
| -------------------- | --------------------------------------------------------------------- |
| Grammar syntax       | `sea-core/grammar/sea.pest`                                           |
| Primitives           | `sea-core/src/primitives/` (Entity, Resource, Flow, Instance, Policy) |
| Graph operations     | `sea-core/src/graph/mod.rs`                                           |
| Policy expressions   | `sea-core/src/policy/expression.rs`                                   |
| CALM mapping spec    | `docs/reference/specs/calm-mapping.md`                                |
| Implementation plans | `docs/plans/` (phase roadmaps, feature plans)                         |
| DSL examples         | `examples/`, `sea-core/examples/`                                     |
| Error codes          | `docs/specs/error_codes.md`, `sea-core/src/validation_error.rs`       |

## Feature Flags

```bash
cargo build --features python      # Build Python bindings
cargo build --features typescript  # Build TypeScript bindings
cargo build --features wasm        # Build WASM bindings
cargo build --features cli         # Build CLI tools
```

## Common Mistakes (AI Agent Anti-Patterns)

**DO NOT:**

```rust
// ❌ WRONG: HashMap causes non-deterministic policy evaluation
use std::collections::HashMap;
let entities: HashMap<String, Entity> = ...;

// ✅ CORRECT: IndexMap preserves insertion order
use indexmap::IndexMap;
let entities: IndexMap<String, Entity> = ...;
```

```rust
// ❌ WRONG: Passing entity object to Flow
Flow::new(resource, from_entity, to_entity)

// ✅ CORRECT: Pass ConceptId (IDs, not references)
Flow::new(resource.concept_id(), from.concept_id(), to.concept_id())
```

```rust
// ❌ WRONG: Modifying parser without grammar
// Editing src/parser/ast.rs first

// ✅ CORRECT: Grammar-first workflow
// 1. sea-core/grammar/sea.pest
// 2. src/parser/ast.rs
// 3. tests/parser_*.rs
```

**Cross-binding rule**: If you touch `sea-core/src/primitives/*.rs`, you MUST also update:

- `sea-core/src/python/primitives.rs`
- `sea-core/src/typescript/primitives.rs`
- `sea-core/src/wasm/primitives.rs` (if public API)

Run `just all-tests` to verify all bindings remain in sync.

## Maintaining These Instructions

**When you discover a new anti-pattern** (a mistake that causes test failures or unexpected behavior):

1. Add it to the "Common Mistakes" section above with ❌/✅ examples
2. Keep examples minimal (3-5 lines max) — show the wrong way and right way
3. If the pattern is project-specific (not general Rust), it belongs here

**When project structure changes**:

- Update the Architecture Overview tree if directories are added/moved
- Update Key References table if key files are renamed
- Update Developer Commands if `justfile` recipes change

**Self-check**: Before submitting code, ask: "Did I make a mistake that future AI agents might repeat?" If yes, document it here.
