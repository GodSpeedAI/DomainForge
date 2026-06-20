# DomainForge - SEA DSL Agent Router

Primary routing and execution guidance for AI coding agents working on DomainForge (Semantic Enterprise Architecture DSL). Copilot-specific behavior belongs in `.github/copilot-instructions.md`; repository-wide execution policy and work-state management belong here.

## Normative Language

The key words MUST, MUST NOT, REQUIRED, SHOULD, SHOULD NOT, RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in RFC 2119.

## Instruction Routing

This file is the canonical router for agent behavior in this repository.

- `.agents/` is the active shared local state store for ongoing work and MUST be used for plans, specs, reports, lessons, and execution state.
- `.agents/` SHOULD remain gitignored unless a task explicitly requires committing agent working state.
- When `.agents/` or any required subfolder is missing, the agent MUST create it before relying on it.

## Shared State In `.agents/`

Agents MUST treat `.agents/` as the durable working memory for this repository.

Required structure:

- `.agents/plans/` for active or approved implementation plans
- `.agents/reports/` for investigation notes, reviews, and execution reports
- `.agents/specs/` for feature or architecture specs
- `.agents/lessons/` for generalizable lessons, recurring pitfalls, and workflow improvements
- `.agents/current_state.md` for the current phase of work and checklist-backed execution state
- `.agents/next_steps.md` for the next two or three concrete steps and expected outcomes

Minimum operating rules:

1. Before starting substantial work, read `.agents/current_state.md` and `.agents/next_steps.md` if they exist.
2. If the task is non-trivial, create or update a plan or spec in `.agents/plans/` or `.agents/specs/` before broad implementation.
3. While executing, update `.agents/current_state.md` as checkpoints are completed.
4. At handoff or completion, update `.agents/next_steps.md` so the next agent can continue without re-discovery.
5. When a reusable challenge, failure mode, or efficiency improvement is discovered, record it in `.agents/lessons/`.

Rules for `.agents/current_state.md`:

- It MUST be checklist-based.
- Every checked item MUST include implementation evidence.
- Documentation references alone are NOT sufficient evidence.
- Valid evidence includes modified or created file paths, added tests, command lines used for validation, and the resulting pass/fail outcome.

Rules for `.agents/next_steps.md`:

- Keep only the next two or three steps.
- Each step MUST include the expected outcome.
- Each expected outcome MUST contribute to the intended plan, spec, or project objective.

## Architecture Overview

```text
domainforge-core/                    # Canonical Rust implementation (authoritative)
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
just rust-test              # Rust tests with CLI: cargo test -p domainforge-core --features cli
just python-test            # Python tests (uses .venv if present)
just ts-test                # TypeScript tests via Vitest (uses Bun, falls back to npm)
just bun-test               # TypeScript tests via Bun explicitly (faster)

# Setup
just setup                  # Install TypeScript (Bun) + Python dependencies
just bun-install            # Install only TypeScript deps with Bun
just python-setup           # Create .venv, install deps, build Python bindings via maturin

# Debugging
just prepare-rust-debug     # Symlink test binary for codelldb debugging

# Pre-PR checks
cargo clippy -- -D warnings && cargo fmt
```

> **Note**: This project uses [Bun](https://bun.sh/) as the primary JavaScript runtime and package manager for TypeScript tooling. Node.js is available as a fallback (use `npm run test:node`).

## Change Workflow (Non-Negotiable)

When modifying **core data structures or primitives**, you MUST update ALL of:

1. Rust core (`domainforge-core/src/primitives/`, `graph/`, `policy/`) + tests
2. PyO3 bindings (`domainforge-core/src/python/`) + Python tests (`tests/test_*.py`)
3. napi-rs bindings (`domainforge-core/src/typescript/`) + TypeScript tests (`typescript-tests/`)
4. WASM bindings if applicable (`domainforge-core/src/wasm/`)

When modifying **parser/grammar**:

1. Update `domainforge-core/grammar/sea.pest` first
2. Update AST in `domainforge-core/src/parser/ast.rs`
3. Add parser tests in `domainforge-core/tests/parser_*.rs`
4. Update projections (CALM, KG) if the change affects export format

## Code Conventions

- **Deterministic iteration**: Use `IndexMap` (not `HashMap`) in `graph/mod.rs` for policy-relevant collections
- **IDs**: `ConceptId` from namespace+name; `Uuid::new_v4()` for unique identifiers
- **Quantities**: `rust_decimal::Decimal` for precise calculations
- **Units**: `Unit::new()` constructor; `domainforge_core::units::unit_from_string()` for parsing
- **Namespaces**: `namespace()` returns `&str`, defaults to `"default"`
- **Flows**: `Flow::new()` takes `ConceptId` for resource/from/to (IDs, not references)

## Testing Patterns

- **Cross-language parity**: Changes to core must pass all three test suites
- **Round-trip tests**: CALM export/import (`cargo test calm_round_trip`) validates serialization
- **Golden tests**: Check `domainforge-core/tests/` for expected output patterns
- **Policy evaluation**: Three-valued logic tests in `three_valued_quantifiers_tests.rs`

## Key References

| What                 | Where                                                                 |
| -------------------- | --------------------------------------------------------------------- |
| Grammar syntax       | `domainforge-core/grammar/sea.pest`                                           |
| Primitives           | `domainforge-core/src/primitives/` (Entity, Resource, Flow, Instance, Policy) |
| Graph operations     | `domainforge-core/src/graph/mod.rs`                                           |
| Policy expressions   | `domainforge-core/src/policy/expression.rs`                                   |
| CALM mapping spec    | `docs/reference/specs/calm-mapping.md`                                |
| Implementation plans | `docs/plans/` (phase roadmaps, feature plans)                         |
| DSL examples         | `examples/`, `domainforge-core/examples/`                                     |
| Error codes          | `docs/specs/error_codes.md`, `domainforge-core/src/validation_error.rs`       |

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
// 1. domainforge-core/grammar/sea.pest
// 2. src/parser/ast.rs
// 3. tests/parser_*.rs
```

**Cross-binding rule**: If you touch `domainforge-core/src/primitives/*.rs`, you MUST also update:

- `domainforge-core/src/python/primitives.rs`
- `domainforge-core/src/typescript/primitives.rs`
- `domainforge-core/src/wasm/primitives.rs` (if public API)

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

## General Guidance

- prioritize clarity and maintainability in code
- follow existing project conventions unless there's a strong reason to deviate
- ensure all changes are well-tested across all language bindings
- adhere strictly to the change workflow to maintain cross-language parity
- Ensure all changes are well-documented in the project's documentation and specs
- consult `.agents/current_state.md` and `.agents/next_steps.md` before duplicating discovery work
- leave behind enough state in `.agents/` for another agent to resume execution directly
