# DomainForge - SEA DSL Copilot Instructions (Concise)

Quick, targeted guidance for AI coding agents working on DomainForge.

1. Canonical source

- The authoritative implementation is `sea-core/` (Rust). All language bindings (Python, TypeScript, WASM) wrap `sea_core` types — do not duplicate business logic in bindings.

2. Quick commands (use `just` where available)

- Run all tests (recommended): `just all-tests`
- Rust tests, CLI included: `just rust-test` (executes `cargo test -p sea-core --features cli`)
- Python tests: `just python-test` (uses `.venv` if present)
- TypeScript tests: `just ts-test`
- Prepare debug binary for codelldb: `just prepare-rust-debug`
- Prepare dev env: `just python-setup` and `npm ci` (or `npm install`)

3. Feature flags & builds

- Rust features: `--features python`, `--features typescript`, `--features wasm`.
- Python builds commonly use `maturin develop --features python` (see `just python-setup`).
- TypeScript builds depend on `npm run build` (calls cargo with typescript features).

4. Editing and change rules (non-negotiable)

- Rust core is canonical. When you change core data structures, you MUST update:
  - the Rust APIs and tests
  - the PyO3 bindings under `sea-core/src/python/` and Python tests
  - the napi-rs bindings under `sea-core/src/typescript/` and TypeScript tests
  - WASM bindings if relevant (`sea-core/src/wasm/`)

5. Parser/grammar changes

- Update `sea-core/grammar/sea.pest`, update `parser` AST in `sea-core/src/parser` and add tests under `tests/` and `sea-core/tests/`. Follow multi-stage tests for new syntax.

6. Conventions & common pitfalls

- Deterministic iteration via `IndexMap` in `sea-core/src/graph/mod.rs` (do not switch to HashMap for policy-relevant collections).
- Use `Uuid::new_v4()` for IDs; use `rust_decimal::Decimal` for quantities for precise calculations.
- `namespace()` returns `&str` (default "default"). Use `Unit::new()` and `sea_core::units::unit_from_string` for unit handling.
- Use `Flow::new` with `ConceptId` for resource/from/to values (IDs rather than references).

7. Testing policy & integration

- Cross-language parity is critical: run Rust, Python, TypeScript tests after core changes. Use `just all-tests`.
- CALM export/import round-trip tests exist and should pass after changes to the graph or primitives: check `sea-core/tests/*` and `cargo test calm_round_trip`.

8. Debugging & CI

- Use `just prepare-rust-debug` to create a symlinked test binary for codelldb; attach debugger to that binary when stepping through Rust tests.
- Run `cargo clippy -- -D warnings` and `cargo fmt` pre-PR.

9. Where to look for more details

- Architecture & phase plans: `docs/plans/` and `docs/specs/`.
- Parser grammar and examples: `sea-core/grammar/sea.pest` and `examples/`.
- Key code: primitives -> `sea-core/src/primitives/`, graph -> `sea-core/src/graph/`, policy -> `sea-core/src/policy/`, bindings -> `sea-core/src/python|typescript|wasm/`.

10. Communication & PR expectations

- Small, testable changes. Update bindings & tests for language parity. Include ADR updates when making architectural changes. Run the `just` tasks and CI checks locally; provide a brief test summary in PR.


# DomainForge - SEA DSL Copilot Instructions (Concise)

Quick, targeted guidance for AI coding agents working on DomainForge.

1. Canonical source

- The authoritative implementation is `sea-core/` (Rust). All language bindings (Python, TypeScript, WASM) wrap `sea_core` types — do not duplicate business logic in bindings.

2. Quick commands (use `just` where available)

- Run all tests (recommended): `just all-tests`
- Rust tests, CLI included: `just rust-test` (executes `cargo test -p sea-core --features cli`)
- Python tests: `just python-test` (uses `.venv` if present)
- TypeScript tests: `just ts-test`
- Prepare debug binary for codelldb: `just prepare-rust-debug`
- Prepare dev env: `just python-setup` and `npm ci` (or `npm install`)

3. Feature flags & builds

- Rust features: `--features python`, `--features typescript`, `--features wasm`.
- Python builds commonly use `maturin develop --features python` (see `just python-setup`).
- TypeScript builds depend on `npm run build` (calls cargo with typescript features).

4. Editing and change rules (non-negotiable)

- Rust core is canonical. When you change core data structures, you MUST update:
  - the Rust APIs and tests
  - the PyO3 bindings under `sea-core/src/python/` and Python tests
  - the napi-rs bindings under `sea-core/src/typescript/` and TypeScript tests
  - WASM bindings if relevant (`sea-core/src/wasm/`)

5. Parser/grammar changes

- Update `sea-core/grammar/sea.pest`, update `parser` AST in `sea-core/src/parser` and add tests under `tests/` and `sea-core/tests/`. Follow multi-stage tests for new syntax.

6. Conventions & common pitfalls

- Deterministic iteration via `IndexMap` in `sea-core/src/graph/mod.rs` (do not switch to HashMap for policy-relevant collections).
- Use `Uuid::new_v4()` for IDs; use `rust_decimal::Decimal` for quantities for precise calculations.
- `namespace()` returns `&str` (default "default"). Use `Unit::new()` and `sea_core::units::unit_from_string` for unit handling.
- Use `Flow::new` with `ConceptId` for resource/from/to values (IDs rather than references).

7. Testing policy & integration

- Cross-language parity is critical: run Rust, Python, TypeScript tests after core changes. Use `just all-tests`.
- CALM export/import round-trip tests exist and should pass after changes to the graph or primitives: check `sea-core/tests/*` and `cargo test calm_round_trip`.

8. Debugging & CI

- Use `just prepare-rust-debug` to create a symlinked test binary for codelldb; attach debugger to that binary when stepping through Rust tests.
- Run `cargo clippy -- -D warnings` and `cargo fmt` pre-PR.

9. Where to look for more details

- Architecture & phase plans: `docs/plans/` and `docs/specs/`.
- Parser grammar and examples: `sea-core/grammar/sea.pest` and `examples/`.
- Key code: primitives -> `sea-core/src/primitives/`, graph -> `sea-core/src/graph/`, policy -> `sea-core/src/policy/`, bindings -> `sea-core/src/python|typescript|wasm/`.

10. Communication & PR expectations

- Small, testable changes. Update bindings & tests for language parity. Include ADR updates when making architectural changes. Run the `just` tasks and CI checks locally; provide a brief test summary in PR.
