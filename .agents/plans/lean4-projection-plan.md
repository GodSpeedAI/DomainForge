# Implementation Plan — Lean 4 Projection Target (`--format lean`)

**Created:** 2026-07-04

**Source of truth:** This document (research + repo inspection performed 2026-07-04). Lean 4 facts verified against official sources: lean-lang.org documentation, Lake docs (`github.com/leanprover/lean4/tree/master/src/lake`), and the official `leanprover/lean-action` CI action.

**Originating context:** Feature request on `main` — add Lean 4 as a first-class DomainForge projection so `.sea` models emit formally checkable artifacts: type definitions, domain predicates, policy invariants, theorem stubs, proof obligations, and CI-checkable proof files. The uncommitted `ai_learning` projection family on this branch is the architectural template to mirror.

**Status of the work today:** Nothing Lean-related exists. The projection architecture (`ai_learning` family: context → `emit(ctx, sink)` → deterministic file map) is proven and is exactly the shape this plan reuses. No IR changes are required for v1 (see "Semantic sufficiency" below).

---

## 0. How to use this plan (agent operating instructions)

- Execute tasks in order. Dependencies: Task 1→2 dependent (2 consumes the expression compiler); Task 2→3→4 dependent (CLI and bindings call the in-memory API); Tasks 5, 6 depend on 3; Task 7 (docs/CI) last. Task 1 and the fixture check in Task 5 are independently startable.
- **Every task ends with a verification gate.** Do not mark a task done until its gate exits 0.
- **CORE PRINCIPLE: One emitter, string-map out.** All Lean output is produced by `project_lean_in_memory(graph, opts) -> BTreeMap<String, String>` (relative path → content). CLI, Python, TypeScript, and WASM are thin wrappers over that one function — never re-implement generation in a wrapper or test. This mirrors `project_ai_learning_in_memory` (`domainforge-core/src/projection/ai_learning/mod.rs:394`).
- **CORE PRINCIPLE 2: Two-layer proof discipline.** The `DomainForge` Lean library must always compile with **zero `sorry`** (proofs auto-discharged by `decide`/`rfl`/`simp` only). Human/AI-strengthened proofs live in a separate `Obligations` library where `sorry` is expected. CI gates the first layer only. This is the competitive moat: "your domain invariants are machine-checked on every commit" only sells if the checked layer never needs a proof engineer.
- Match surrounding code style; the idioms to mirror live in `domainforge-core/src/projection/ai_learning/` (module layout, `emit(ctx, sink)` signature, `ArtifactSink`, determinism via `BTreeMap` + fixed `--created-at`), `domainforge-core/src/cli/project.rs:330` (`run_ai_learning`), and `domainforge-core/tests/ai_learning_tests.rs` (test conventions).
- Determinism is the spec: identical inputs (model + fixed `created_at`) must produce byte-identical Lean output. Iterate graphs in sorted order; never use HashMap iteration order or wall-clock time in content.

### Global verification gates (must stay green after EVERY task)

```bash
cargo test --features cli --workspace          # rust unit + integration tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
just ci-test-python                            # after Task 4 only
just ci-test-ts                                # after Task 4 only
```

Rebuild steps if you touched the binding surfaces (`src/python/`, `src/typescript/`, `src/wasm/`) before running the language gates:

```bash
just python-setup                              # rebuilds the pyo3 module
npm run build                                  # rebuilds the napi .node artifact
```

---

## Key facts already discovered (do not re-derive)

| Thing | Location |
| --- | --- |
| `ProjectFormat` enum (CLI target list; add `Lean` here) | `domainforge-core/src/cli/project.rs:84` |
| Format dispatch (`run_ai_learning` called for ai formats) | `domainforge-core/src/cli/project.rs:144` |
| `run_ai_learning` — the CLI wrapper shape to mirror | `domainforge-core/src/cli/project.rs:330` |
| Directory-output validation for multi-file projections | `domainforge-core/src/cli/project.rs:414` |
| `ArtifactSink` (rel-path → content writer; reuse it) | `domainforge-core/src/projection/ai_learning/mod.rs:371` |
| `project_ai_learning_in_memory` — the binding-surface pattern | `domainforge-core/src/projection/ai_learning/mod.rs:394` |
| Projection module registry (`pub mod` list) | `domainforge-core/src/projection/mod.rs:1` |
| `Policy` struct (name, namespace, expression, modality, kind, priority, rationale, tags) | `domainforge-core/src/policy/core.rs:30` |
| `PolicyModality` (Obligation / Prohibition / Permission) | `domainforge-core/src/policy/core.rs:12` |
| `Expression` AST (Literal, QuantityLiteral{Decimal,unit}, Time/IntervalLiteral, Variable, GroupBy, Binary, Unary, Cast, Quantifier, MemberAccess, Aggregation, AggregationComprehension) | `domainforge-core/src/policy/expression.rs:12` |
| `BinaryOp` (And/Or, comparisons, arithmetic, Contains/StartsWith/EndsWith/Matches, HasRole, Before/After/During) | `domainforge-core/src/policy/expression.rs:82` |
| `ThreeValuedBool` (policies evaluate tri-state) | `domainforge-core/src/policy/three_valued.rs:4` |
| `Policy::normalized_expression()` — canonical form for deterministic display | `domainforge-core/src/policy/core.rs:59` |
| `Graph::all_policies()` (also `all_entities`, `all_roles`, `all_resources`, `all_flows`, `all_relations` nearby) | `domainforge-core/src/graph/mod.rs:662` |
| `Entity` (id, name, namespace, version, attributes: `HashMap<String, serde_json::Value>`) | `domainforge-core/src/primitives/entity.rs:58` |
| `Resource` (id, name, unit, namespace, attributes) | `domainforge-core/src/primitives/resource.rs:72` |
| `Flow` (resource_id, from_id, to_id, quantity: `Decimal`, namespace) | `domainforge-core/src/primitives/flow.rs:36` |
| Binding wrappers to mirror (`export_ai_learning`) | `domainforge-core/src/python/graph.rs:334`, `domainforge-core/src/typescript/graph.rs:338`, `domainforge-core/src/wasm/graph.rs:401` |
| TS type declarations to extend | `domainforge-typescript/index.d.ts` |
| Test conventions (assert_cmd, fixed `--created-at`, fixture-driven) | `domainforge-core/tests/ai_learning_tests.rs:1` |
| Proving fixture (entities, roles, resources, flows, relations; policies enter via graph API / .sea `Policy` blocks — grammar supports the `Policy` keyword per `parser/lint.rs:23`) | `fixtures/ai_learning/manufacturing_quality/domain/model.sea` |
| CI job layout (lint, test-rust matrix, test-python, test-ts) | `.github/workflows/ci.yml:26` |
| Just gates | `justfile:163` (`ci-test-rust`), `:168` (`ci-test-python`), `:173` (`ci-test-ts`) |

### Lean 4 facts (researched; official sources)

| Fact | Consequence for this plan |
| --- | --- |
| A Lean project = `lean-toolchain` file (e.g. `leanprover/lean4:v4.21.0`) + `lakefile.toml`; `elan` fetches the pinned toolchain; `lake build` elaborates every module — **compiling IS proof-checking** | Emit both files; the CI gate is simply `lake build` |
| `lakefile.toml` supports multiple `[[lean_lib]]` entries and a `defaultTargets` list | `DomainForge` lib is the default target (sorry-free); `Obligations` lib is non-default and built explicitly |
| `sorry` elaborates with a **warning**, not an error (`declaration uses 'sorry'`) | The sorry-free gate = `lake build DomainForge` output must not contain `uses 'sorry'` (grep in CI script / test) |
| Lean core alone provides `Nat`, `Int`, `String`, `List`, `Array`, `structure`, `inductive`, `deriving DecidableEq, Repr, BEq`, `theorem`, `by decide`, `#guard` | **Zero external dependencies** — no Mathlib, no Batteries. `lake build` needs no network; hermetic + fast CI. Do not add Mathlib. |
| `by decide` discharges any `Decidable` proposition over the finite generated model; `#guard` gives compile-time boolean checks | Model-level invariants (finite domains from the graph) are auto-provable with no tactic expertise |
| `rust_decimal::Decimal` has no core-Lean analogue; `Rat` availability varies by toolchain layer | Represent quantities as scaled integers: `structure Quantity where value : Int; scale : Nat; unit : String` — exact, `DecidableEq`-derivable |
| Official CI action: `leanprover/lean-action@v1` (installs elan + toolchain, runs `lake build`, caches `.lake`) | Optional CI job is ~6 lines |
| Lean 3 syntax/tooling (`leanpkg`, `import data.*`) is dead; target Lean 4 only | Pin a current stable `v4.x` toolchain in one constant in the emitter |

### Semantic sufficiency (question 5 answered — no IR changes for v1)

The graph already carries enough to generate genuinely useful Lean: closed finite domains (entities/roles/resources as inductives with `DecidableEq`), concrete facts (flows with exact quantities, relations as a decidable relation table), and policies with a full typed expression AST including quantifiers, deontic modality, and priority. Known gaps, all deferrable:

1. **Untyped attributes** — `attributes: HashMap<String, serde_json::Value>` has no declared schema, so entity attributes can't become typed Lean fields. v1 omits attributes from generated structures; policies referencing `MemberAccess` on attributes fall into the "obligation stub" path. *Future IR addition (small): typed attribute declarations in `.sea`.*
2. **Three-valued logic** — Lean `Prop` is two-valued. v1 emits an explicit `inductive TBool | true | false | unknown` mirroring `ThreeValuedBool`, and compiles policy expressions into `TBool`-valued functions with a `theorem …_decided : eval p = TBool.true := by decide` only where the expression is fully groundable from the model. Honest, and it preserves the resolver semantics.
3. **Instance-level data** — quantifiers over runtime instances can't be proven at generation time; they become theorem stubs in `Obligations`, parameterized over a `World` structure. That is a feature (the proof obligation IS the deliverable), not a gap.

### Competitive-advantage framing (drives the design choices below)

The outcome sought is not "a Lean pretty-printer"; it is **"DomainForge models are formally verified in CI with zero proof-engineering cost."** Three design consequences: (a) zero-dependency Lean packages so `lake build` is hermetic and fast; (b) every emitted theorem in the default lib is auto-discharged (`decide`) so the green check is unconditional; (c) the `Obligations` layer turns unprovable-at-gentime properties into a visible, versioned proof backlog — the upgrade path for customers (and AI agents) to deepen verification over time. Consistency checks that no other DSL projection offers (e.g. "no policy is simultaneously an Obligation and a Prohibition on the same groundable condition") are emitted as theorems, not docs.

---

## Task 1 — Lean expression compiler (`policy::Expression` → Lean source)

**Goal:** A pure function that compiles the supported subset of `Expression` into a Lean term string, and classifies every expression as `Groundable` (fully evaluable over the generated model → goes in `Policies.lean` with a `decide` proof) or `Deferred(reason)` (→ theorem stub in `Obligations.lean`).

**Why this shape:** Classification-at-compile keeps the sorry-free invariant structural: nothing that could fail `decide` ever lands in the default lib. A single compiler function keeps expression→Lean logic out of the emitters.

### Steps

1. Create `domainforge-core/src/projection/lean/mod.rs` and `domainforge-core/src/projection/lean/expr.rs`; register `pub mod lean;` in `domainforge-core/src/projection/mod.rs`.
2. In `expr.rs`, add `pub enum LoweredExpr { Groundable(String), Deferred { lean_prop: String, reason: String } }` and `pub fn lower(expr: &Expression, ctx: &NameTable) -> LoweredExpr`. Supported → Groundable: `Literal` (bool/int/string), `QuantityLiteral` (scaled-int `Quantity`), `Variable` bound by a model-level `Quantifier` over a known finite collection, `Binary` on And/Or/comparisons/arithmetic, `Unary`, `HasRole` (lookup in generated relation table). Everything else (`GroupBy`, `Aggregation*`, `MemberAccess` on attributes, temporal ops, `Cast`, `Matches`) → Deferred with a human-readable reason.
3. Add `pub fn lean_ident(name: &str) -> String` (sanitize `.sea` names — spaces/quotes → valid Lean identifiers, dedup via suffix; deterministic). Keep a `NameTable` built once from the graph in sorted order.
4. Unit tests in the same file (`#[cfg(test)]`): each supported node kind round-trips to the expected Lean string; each unsupported kind defers with a reason; identifier sanitization is deterministic and collision-free.

### Gate

```bash
cargo test --workspace lean::expr
```

**Done when:** all node kinds in `Expression` (`policy/expression.rs:12`) are handled (no `_ => todo!()`); flipping a supported op to unsupported in a test makes the test fail (teeth-check: assert exact Lean strings, not just "is ok").

**Redesign trigger:** if `Quantifier` collections turn out not to be resolvable to graph-level finite sets at lowering time, move quantifier handling entirely to the Deferred path for v1 — do not build an evaluator.

---

## Task 2 — Emitter: `project_lean_in_memory` producing a complete Lake package

**Goal:** `project_lean_in_memory(graph: &Graph, model_ref: &str, created_at: Option<String>) -> Result<BTreeMap<String, String>, String>` returns a byte-deterministic, self-contained Lean 4 package.

**Why this shape:** String-map-out is the proven binding surface (`ai_learning/mod.rs:394`); it makes the CLI, all three bindings, and tests trivial wrappers.

### Steps

1. In `projection/lean/mod.rs`, emit this file map (all content generated from sorted graph iteration; header comment in each file carries `model_ref` + `created_at` + model hash, mirroring ai_learning provenance):
   - `lean-toolchain` — pinned constant, e.g. `leanprover/lean4:v4.21.0` (single `const` in the module; document the bump procedure in the file header).
   - `lakefile.toml` — package `domainforge`, `[[lean_lib]] name = "DomainForge"` (default target), `[[lean_lib]] name = "Obligations"` (non-default).
   - `DomainForge.lean` — imports all `DomainForge/*` modules.
   - `DomainForge/Types.lean` — `inductive Entity | …` / `Role` / `Resource` from graph names (`deriving DecidableEq, Repr`), `structure Quantity where value : Int; scale : Nat; unit : String deriving DecidableEq, Repr`, `inductive TBool | true | false | unknown deriving DecidableEq, Repr`.
   - `DomainForge/Model.lean` — `def flows : List Flow := [...]` (structure with resource/from/to/quantity), `def relations : List Rel := [...]`; `Flow`/`Rel` structures with `DecidableEq`.
   - `DomainForge/Policies.lean` — per policy: a `def policy_<name> : Prop := <groundable lean>` plus `theorem policy_<name>_holds : policy_<name> := by decide` **only** for `Groundable` policies whose model-evaluation is true; groundable-but-false policies emit `theorem policy_<name>_violated : ¬ policy_<name> := by decide` (a failing invariant is information, not a build failure — the *theorem states the truth*). Also emit cross-policy consistency theorems where both sides are groundable (e.g. prohibition/permission pairs on the same subject cannot both ground true).
   - `Obligations/Stubs.lean` — for each `Deferred` policy: a documented `theorem obligation_<name> : <lean_prop> := by sorry` with the deferral reason and `rationale`/`tags` from `Policy` (`policy/core.rs:30`) as doc comments.
   - `README.md` — how to check (`lake build`), what the two libraries mean, how to strengthen obligations.
2. Empty-graph and no-policy graphs must still emit a valid, buildable package (empty inductives are illegal in `decide`-land — emit a `deriving`-free placeholder or skip the inductive and its dependents when a domain is empty; test this).
3. Wire an `emit(ctx, sink)`-style adapter reusing `ai_learning::ArtifactSink` (`ai_learning/mod.rs:371`) so directory writing is shared, not duplicated.
4. Unit tests: snapshot the full file map for a small in-code graph (entities + one flow + one groundable policy + one deferred policy); assert determinism (two calls, identical maps).

### Gate

```bash
cargo test --workspace projection::lean
```

**Done when:** the snapshot test pins every emitted file; changing any generator line breaks the snapshot (teeth); empty-graph emission produces a non-panicking valid map.

**Redesign trigger:** if reusing `ai_learning::ArtifactSink` forces an awkward dependency direction, move `ArtifactSink` up to `projection/mod.rs` in a separate mechanical commit — do not fork it.

---

## Task 3 — CLI: `--format lean`

**Goal:** `domainforge project --format lean --created-at <ts> model.sea outdir/` writes the package; output path must be a directory.

### Steps

1. Add `Lean` to `ProjectFormat` (`cli/project.rs:84`, doc comment `/// Lean 4 formal verification package (directory output)`).
2. Add `fn run_lean(args: &ProjectArgs, graph: &Graph) -> Result<()>` mirroring `run_ai_learning` (`cli/project.rs:330`): validate directory output (reuse the check at `cli/project.rs:414`), call `project_lean_in_memory`, write via `ArtifactSink::Dir`. Dispatch it at `cli/project.rs:144`. No `--recipe` needed; reject `--recipe` with a clear error for `lean`.
3. Respect the existing `--created-at` flag for deterministic output (same contract as ai formats).

### Gate

```bash
cargo test --features cli --workspace
target/debug/domainforge project --format lean --created-at 2026-07-02T00:00:00+00:00 \
  fixtures/ai_learning/manufacturing_quality/domain/model.sea /tmp/lean_out \
  && test -f /tmp/lean_out/lakefile.toml && test -f /tmp/lean_out/DomainForge/Types.lean
```

**Done when:** command exits 0 and the package files exist; passing a file path (not a dir) as output exits non-zero with the directory error (teeth).

**Redesign trigger:** none plausible.

---

## Task 4 — Bindings: Python, TypeScript, WASM

**Goal:** `graph.export_lean(model_ref, created_at?)` returns the path→content map in all three bindings, exactly like `export_ai_learning`.

### Steps

1. Mirror `export_ai_learning` at `python/graph.rs:334`, `typescript/graph.rs:338`, `wasm/graph.rs:401` — each is a ~10-line wrapper over `project_lean_in_memory`.
2. Add the method signature to `domainforge-typescript/index.d.ts` next to the existing `exportAiLearning` declaration.
3. Rebuild bindings (`just python-setup`, `npm run build`) before running language gates.

### Gate

```bash
just ci-test-python
just ci-test-ts
```

**Done when:** a Python test and a TS test (added in Task 5) each call `export_lean` and assert `lakefile.toml` and one theorem line are present in the returned map.

**Redesign trigger:** none plausible.

---## Task 5 — Tests: Rust integration + Python/TS smoke + optional real `lake build` e2e

**Goal:** The fixture-driven test suite pins the Lean output shape and the sorry-free invariant, following `ai_learning_tests.rs` conventions.

**Why this shape:** `fixtures/ai_learning/manufacturing_quality` is already the single proving fixture for projections; reuse it rather than inventing a new one. If it carries no `Policy` blocks, extend **the fixture** with 2–3 `.sea` policies (one groundable, one deferred) in the same commit as the tests that read them — the fixture is the spec.

### Steps

1. `domainforge-core/tests/lean_projection_tests.rs` (`#![cfg(feature = "cli")]`, `assert_cmd`, fixed `FIXED_TS` like `ai_learning_tests.rs:13`): project the fixture, assert file set, assert determinism (two runs byte-identical), assert `DomainForge/` files contain **no** `sorry` while `Obligations/` contains at least one, assert one exact `theorem … := by decide` line.
2. Add 2–3 `Policy` blocks to `fixtures/ai_learning/manufacturing_quality/domain/model.sea` if absent (verify against parser tests at `parser/mod.rs:139` for syntax). Regenerate any ai_learning expected fixtures **only** if the model change affects them — run the full suite to find out; keep regeneration in the same commit with justification.
3. Python smoke test in `tests/test_ai_learning.py`'s sibling style (`tests/test_lean.py`) and TS test `typescript-tests/lean.test.ts`, each ~15 lines against `export_lean`.
4. Gated end-to-end proof check: `#[test] #[ignore]` (or env-gated `LEAN_E2E=1`) test that runs `lake build` in the projected directory when `lake` is on PATH — skip otherwise. This is the only test needing a Lean toolchain locally.

### Gate

```bash
cargo test --features cli --workspace
just ci-test-python && just ci-test-ts
```

**Done when:** deleting the `by decide` proof from the generator makes the Rust integration test fail (teeth); adding a `sorry` to a `DomainForge/` template fails the no-sorry assertion.

**Redesign trigger:** if extending the shared fixture with policies breaks many ai_learning golden files, create `fixtures/lean/basic/domain/model.sea` instead (copy of the fixture + policies) and point the Lean tests there — do not churn ai_learning fixtures.

---

## Task 6 — CI: Lean proof-check job (optional but the point of the feature)

**Goal:** CI proves the emitted package actually checks: project the fixture, run `lake build DomainForge`, fail on any `uses 'sorry'` in its output.

### Steps

1. Add a `verify-lean` job to `.github/workflows/ci.yml` (after `test-rust`): build the CLI, project the fixture to a temp dir, then `leanprover/lean-action@v1` with `lake-package-directory` pointed at the output (or plain elan install + `lake build`), then `! grep -R "uses 'sorry'" build.log` for the default lib.
2. Keep it `ubuntu-latest` only, non-matrix — it verifies generation, not platforms. Cache via the action's built-in `.lake` caching (near-noop here since there are zero deps).

### Gate

```bash
# local rehearsal of the CI job body:
target/release/domainforge project --format lean --created-at 2026-07-02T00:00:00+00:00 \
  fixtures/ai_learning/manufacturing_quality/domain/model.sea /tmp/lean_ci \
  && (cd /tmp/lean_ci && lake build DomainForge 2>&1 | tee build.log) \
  && ! grep -q "uses 'sorry'" /tmp/lean_ci/build.log
```

**Done when:** the job is green on a PR; injecting `sorry` into a `DomainForge/` template turns it red (teeth — do this once on the PR branch and revert).

**Redesign trigger:** if the pinned toolchain download is flaky/slow in CI, keep the job but mark it non-required initially; do not weaken the local `#[ignore]` e2e test.

---

## Task 7 — Docs

**Goal:** `docs/lean-projections.md` documents the target: what is generated, the two-layer proof model, the groundable/deferred classification table, how to run `lake build`, how to strengthen obligations, and the explicit non-goals (no Mathlib, no Lean 3, attributes untyped in v1).

### Steps

1. Write `docs/lean-projections.md` mirroring the structure of `docs/ai-learning-projections.md`.
2. Add the `lean` format to the CLI `--format` docs / README section where the other formats are listed.
3. Status claims only after their gates pass (no "CI-verified" claim before Task 6 is green).

### Gate

```bash
grep -q "lean" docs/lean-projections.md && cargo test --features cli --workspace
```

**Done when:** a reader can go from `.sea` file to green `lake build` using only the doc.

**Redesign trigger:** none plausible.

---

## Final acceptance checklist (whole plan)

**Status: COMPLETE (2026-07-05).** All gates verified locally, including a real
`lake build` with Lean 4.15.0 (sorry-free default lib; teeth-check confirmed a
false proof fails the build). The `verify-lean` CI job awaits its first PR run.

- [x] Expression compiler handles every `Expression` variant (grounded or deferred-with-reason); exact-string tests have teeth _(Task 1)_
- [x] `project_lean_in_memory` emits a deterministic, self-contained, zero-dependency Lake package; snapshot test pins it _(Task 2)_
- [x] `--format lean` works end-to-end on the fixture; non-directory output rejected _(Task 3)_
- [x] `export_lean` available and tested in Python, TS, WASM; `index.d.ts` updated _(Task 4)_
- [x] Integration tests enforce: no `sorry` in `DomainForge/`, ≥1 `sorry` stub in `Obligations/`, byte-determinism _(Task 5)_
- [x] CI `verify-lean` job runs `lake build` on projected output and rejects `sorry` in the default lib _(Task 6; green locally, first CI run pending)_
- [x] `docs/lean-projections.md` reflects reality — no status ahead of a passing gate _(Task 7)_
- [x] `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all -- --check` exit 0.
- [x] All global gates green; Python/npm binding artifacts rebuilt.

## Guardrails (do not violate)

- **No Mathlib, no Batteries/Std dependency** in generated packages — Lean core only. The hermetic, dependency-free `lake build` is a deliberate product property, not an implementation shortcut.
- **The default `DomainForge` lib must never contain `sorry`** — anything unprovable-by-`decide` goes to `Obligations`. Do not "fix" a failing `decide` by weakening it to `sorry` in the default lib.
- Do NOT add new IR/parser surface for v1 (no typed attributes, no new `.sea` keywords) — the plan proves Lean emission over existing semantics; IR extensions are a separate future plan.
- Do NOT touch the existing `TargetFormat`/`ProjectionContract` mapping machinery (`projection/registry.rs`) — the Lean target is CLI-format-driven like ai_learning; user-declared `projection … target lean` contracts are out of scope.
- Determinism: no wall-clock, no HashMap iteration order, no random IDs in emitted content; `--created-at` fixes all timestamps (mirror `FIXED_TS` convention, `ai_learning_tests.rs:13`).
- Keep fixture edits (Task 5 step 2) in the same commit as the tests they justify; if ai_learning goldens would churn, use the fallback fixture instead.
- Toolchain pin lives in exactly one Rust constant; bumping it is a one-line PR plus a green `verify-lean` run.
