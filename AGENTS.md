# AGENTS.md

Durable operating contract for DomainForge agents. Keep this file project-specific, behavior-changing, and earned; task state belongs in `.agents/`, detailed domain rules in scoped specs/docs, and Copilot-specific behavior in `.github/copilot-instructions.md`.

DomainForge is a Semantic Enterprise Architecture DSL. The Rust core is canonical; Python, TypeScript, and WASM bindings expose core behavior and must not duplicate business logic.

## 1. Scope, Precedence, and Instruction Topology

Before substantial work:

1. Read this file.
2. Read `.agents/current_state.md` and `.agents/next_steps.md` when present.
3. Read the governing plan/spec under `.agents/`.
4. Inspect affected code, tests, configuration, and bindings.
5. Load additional documentation only when needed.

Instruction precedence:

1. Runtime/system safety and user constraints.
2. Nearest applicable scoped `AGENTS.md` (`domainforge-core/AGENTS.md`, `.agents/AGENTS.md`).
3. This root `AGENTS.md`.
4. Governing specs/plans and repository instructions.
5. Current implementation and tests.
6. General engineering defaults.

Instructions live at the narrowest scope that completely governs them:
* **`domainforge-core/` (`domainforge-core/AGENTS.md`)**: Governs the canonical Rust core library, CLI binary, Pest grammar, AST/parser, semantic kernel, projections, and FFI binding implementations (`src/python/`, `src/typescript/`, `src/wasm/`).
* **`.agents/` (`.agents/AGENTS.md`)**: Governs durable agent working memory, task state, handoff contracts (`current_state.md`, `next_steps.md`), and memory ledgers (`OBSERVED_DEBT.md`, `lessons/`).

Trust current executable behavior and tests over stale prose; surface conflicts instead of resolving them silently.

## 2. Investigation and Retrieval

Investigate before asking. Use deterministic tools and repository utilities to settle mechanically answerable questions; do not spend reasoning effort inferring facts those tools can establish directly.

Use the cheapest tool that can settle the question:

* `graft map` for first-pass repository orientation.
* `graft ask "<question>" --source` for ranked architectural/behavioral context with source spans.
* `graft callers <symbol>` (`--direction out`, `--depth N`) for call-graph/blast-radius questions.
* `graft skeleton <file>` for signatures/spans without whole-file reads.
* `graft grep "<literal>"` for exhaustive literal matches across indexed files.
* `zvec_grep_search` or `zg` for semantic/conceptual discovery when wording/location is unknown.
* `rg --files` for inventory and `rg` for known paths, symbols, identifiers, literals, config keys, errors, or regexes.
* `rust-analyzer` for Rust structural/semantic questions before grep-and-recompile loops.

Use Graft/zvec to narrow, then verify anchors with `rg` and read only relevant source/test/spec/history ranges. Ranked semantic results are not exhaustive. If Graft truncates a span, open that exact range before finalizing.

## 3. Universal Change Boundaries

Always:

* Make the smallest effective change that fully satisfies the requested outcome; do not deliver MVP-like, partial, placeholder, or knowingly incomplete work unless explicitly requested.
* Inspect existing patterns and nearby tests before adding new ones.
* Keep diffs task-bounded and preserve unrelated worktree changes.
* Run verification proportional to the changed surface.
* Update `.agents/` when consequential state, evidence, lessons, or next actions change.

Ask before:

* adding/upgrading dependencies;
* changing public API semantics, persisted formats, architecture boundaries, CI, deployment, or generated interfaces;
* deleting files or substantially widening task scope.

Never:

* duplicate Rust business logic in bindings;
* hand-edit generated artifacts merely to make checks pass;
* fabricate passing tests, benchmark results, or completion evidence;
* weaken assertions, remove failing tests, or update goldens merely because implementation differs.

## 4. Cross-Surface Invariants

For core primitive/data-structure changes, update every affected public surface:

1. Rust core (`domainforge-core`) + Rust tests (`domainforge-core/tests/`).
2. PyO3 bindings (`domainforge-core/src/python/`) + Python tests (`tests/`).
3. napi-rs bindings (`domainforge-core/src/typescript/`) + TypeScript tests (`typescript-tests/`).
4. WASM bindings (`domainforge-core/src/wasm/`) + WASM tests (`domainforge-core/tests/wasm_tests.rs`).

Generated artifacts are projections, not competing sources of truth.

## 5. Repository Topology & Surface Routing

Primary surfaces:

* `domainforge-core/` — canonical Rust core library, CLI, and FFI bindings (`domainforge-core/AGENTS.md`).
* `domainforge-python/` — Python package distribution (`pyproject.toml`, maturin). Integration tests in `tests/`.
* `domainforge-typescript/` — TypeScript package distribution (`package.json`, napi-rs). Integration tests in `typescript-tests/`.
* `docs/` — architecture, ADRs, projection status, error codes.
* `.agents/` — durable agent working memory (`.agents/AGENTS.md`).

Do not create competing layouts or place new root files when an established location owns the concern.

## 6. Commands and Verification

Use `just` as the canonical repository CLI; prefer existing recipes over equivalent raw commands.

Core commands:
* `just rust-test` — Rust core test suite (`domainforge-core`).
* `just python-test` — Python test suite (`tests/`).
* `just ts-test` — TypeScript test suite (`typescript-tests/`).
* `just all-tests` — all language test suites (Rust, Python, TypeScript).
* `just enterprise-verify` — full enterprise release gate (fmt, clippy, tests, doctests, all-tests, audit).
* `just prove` — full self-proving evidence harness (`PROOFS.md`).

Run the narrowest command that can settle the current claim, then broaden with blast radius:

* Rust-only localized behavior -> focused Rust test (`cargo test -p domainforge-core --features cli --test <name>`).
* Primitive/core API -> Rust + affected bindings/tests.
* Parser/grammar -> parser tests + affected projections/bindings + ADR check.
* Cross-binding/public-core change -> `just all-tests`.
* Pre-PR Rust quality -> `cargo clippy --workspace --all-targets --all-features -- -D warnings` + `cargo fmt --all --check`.

A passing Rust suite does not prove binding parity. A passing binding suite does not prove all language surfaces. Verification must match the claim.

Bun is the primary TypeScript runtime/package manager; Node/npm is fallback only where repository tooling explicitly supports it.

## 7. State, Handoff, and Completion

Follow `.agents/AGENTS.md` for handoff state requirements (`current_state.md`, `next_steps.md`, `OBSERVED_DEBT.md`).

Before declaring completion:

1. Inspect the final diff.
2. Run proof proportional to every claim and affected surface.
3. Confirm no test, invariant, or canonical source was weakened/bypassed.
4. Update `.agents/current_state.md` with actual implementation evidence.
5. Update `.agents/next_steps.md` so another agent can resume without rediscovery.
6. Record reusable failures in `.agents/lessons/`.
7. Run `graft build` after code changes that affect indexing.
8. Leave unresolved debt or uncertainty explicit in `.agents/OBSERVED_DEBT.md`.

Hard cap for this file: fewer than 150 lines and no more than 32 KiB.
