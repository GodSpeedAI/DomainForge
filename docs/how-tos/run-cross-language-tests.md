# Run Cross-Language Tests

Goal: Execute and maintain Rust, Python, and TypeScript tests to ensure the DSL, bindings, and projections stay in sync.

## Prerequisites

- Rust toolchain (with `cargo`), Node.js 20+, and Python 3.10+ available on your machine.
- Install project dependencies:
  - `npm ci` to install JS deps.
  - `just python-setup` to build Python bindings in a virtualenv.
- Optional: install `just` (`cargo install just --locked`) to use the provided recipes.

## Steps (be concise)

1. **Run all suites sequentially**

   ```bash
   just all-tests
   ```

   - Executes Rust (`cargo test -p sea-core --features cli`), Python (`pytest`), and TypeScript (Vitest) in order.
   - Fails fast on the first failing command; rerun individual suites for debugging.

2. **Run Rust tests only**

   ```bash
   just rust-test
   ```

   - Includes CLI tests and parser/projection coverage under `sea-core/tests/`.
   - Use `cargo test -p sea-core --features "cli shacl"` if you need SHACL validation paths.

3. **Run Python tests only**

   ```bash
   just python-test
   ```

   - Rebuilds bindings via `maturin develop` in the virtualenv if needed.
   - To focus on a single file: `.venv/bin/pytest tests/test_golden_payment_flow.py -k payment`.

4. **Run TypeScript tests only**

   ```bash
   just ts-test
   ```

   - Invokes Vitest using the compiled napi module from `npm run build`.
   - Filter tests: `npm test -- --runInBand golden-payment-flow.test.ts`.

5. **Add a cross-language golden scenario**

   - Define the scenario in Rust tests under `sea-core/tests/` (e.g., roles/relations payment flow).
   - Mirror the scenario in `tests/` (Python) and `typescript-tests/` with equivalent assertions.
   - Keep DSL snippets identical to catch serialization and parsing differences.

6. **Debug mismatches**

   - If Python or TypeScript counts differ, log the graph contents:

     ```python
     for role in graph.all_roles():
         print(role.name, role.id)
     ```

     ```ts
     console.log(graph.allRelations().map(r => ({ name: r.name, predicate: r.predicate })));
     ```

   - Re-run the Rust test with `-- --nocapture` to see detailed parser output when comparing against bindings.

7. **Automate in CI**

   - Use `just ci-check` locally to mimic CI (formatting, clippy, and all tests).
   - In GitHub Actions, configure runners with Rust/Python/Node toolchains and call the same commands; set up caching for `target/`, `.venv/`, and `node_modules/` to shorten feedback loops.

## Adding Cross-Language Test Cases

- Place shared DSL fixtures under `examples/` or `tests/fixtures/` to avoid duplication.
- Prefer end-to-end assertions (parse → export → import) instead of unit-testing binding wrappers in isolation.
- Capture CALM/Turtle exports as golden files when the shape is stable; regenerate them intentionally after schema changes.

## Troubleshooting

- **Binding build failures**: Ensure `rustup default stable` and `npm run build` succeed before running tests; stale artifacts can cause symbol resolution errors.
- **Platform differences**: On Windows, run the Node tests in a shell that has `node-gyp` prerequisites; on macOS ARM, ensure a compatible Python version for `maturin` wheels.
- **Timeouts in Vitest**: Use `npm test -- --runInBand --testTimeout=60000` when running under constrained CI machines.

## Links

- Tutorials: [Python Binding Quickstart](../tutorials/python-binding-quickstart.md), [TypeScript Binding Quickstart](../tutorials/typescript-binding-quickstart.md)
- Reference: [CLI Commands](../reference/cli-commands.md), [Python API](../reference/python-api.md), [TypeScript API](../reference/typescript-api.md)

## Example: Payment Role Flow golden test

- DSL snippet lives in `tests/test_golden_payment_flow.py` and `typescript-tests/golden-payment-flow.test.ts`.
- Expectations:
  - `role_count()` returns 2 (`Payer`, `Payee`).
  - `relation_count()` returns 1 with predicate `pays`.
  - Flow quantity equals 10 and uses the resource `Money`.
- Use this as a template for new scenarios; keep IDs stable to avoid brittle UUID comparisons.

## Reporting and Maintaining Results

- Capture failing commands with full stdout/stderr in CI artifacts to compare Rust vs binding behavior.
- When bumping versions (e.g., to `0.1.0`), rerun the suites and update any snapshots to reflect metadata changes.
- Tag tests that rely on optional features (`shacl`, `wasm`) so they can be skipped on constrained environments.

## Readiness Checklist

- [ ] Rust, Python, and TypeScript suites green locally (`just all-tests`).
- [ ] Golden scenarios exercised in all languages with identical DSL inputs.
- [ ] New APIs added to bindings accompanied by parity tests.
- [ ] CI workflow caches warmed for `target/`, `.venv/`, and `node_modules/`.
- [ ] Release workflows (`release-pypi.yml`, `release-npm.yml`, `release-crates.yml`) reference the test commands.
