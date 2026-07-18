# Implementation Plan — Domain-Code Projection Targets (Python / TypeScript / Rust)

**Created:** 2026-07-10

**Source of truth:** `docs/how-tos/write-in-sea.md` (SEA element semantics), `docs/projection-families.md` + ADR-011 (family kernel shape), `docs/projection-target-implementation-status.md` (conventions: CLI-only, shared fixture, verify gates).

**Originating context:** Branch `agent/projection-targets`, after the eight event/authority/verification/activation targets landed. User request: add three **code** projection targets that generate a complete, ready-to-use DDD/CQRS domain layer — folders and files, all domain code generated, compiling out of the box — in Python, TypeScript, and Rust, covering everything **up to the port / abstract base** (no infrastructure adapters).

**Status of the work today:** Nothing exists for these targets. The kernel shape they must follow is fully established (17 existing targets). `projection/flows.rs` and `projection/ids.rs` are the blessed shared helpers. One gap blocks the CQRS mapping: flow annotations (`@cqrs { "kind": ... }`) never reach the `Graph` (Task 1 fixes this).

---

## 0. How to use this plan (agent operating instructions)

- Execute tasks in the order given. Dependencies: Task 1 → Task 2 (IR reads flow annotations); Task 2 → Tasks 3, 4, 5 (renderers consume the IR); Tasks 3, 4, 5 are **independent of each other**; Task 6 last (docs/CI only after gates pass).
- **Every task ends with a verification gate.** Do not mark a task done until its gate command exits 0.
- **CORE PRINCIPLE: One Domain IR, three renderers.** All DDD semantics — what becomes an aggregate, a command's name, an event's name, an error's name, a port's methods — are decided exactly once, in `projection/domain/ir.rs`. Renderers translate IR → language syntax and NOTHING else. A renderer never calls `graph.*`, never re-derives a name, never invents a construct the IR doesn't carry.
- Match surrounding code style; the idioms to mirror live in `domainforge-core/src/projection/dspy/` (ir.rs + render.rs + mod.rs split, Python emission), `domainforge-core/src/projection/cloudevents/mod.rs` (emit signature, in-memory binding surface, deterministic ids), and `scripts/verify/projection-targets/tla.sh` (gate script with real-toolchain + structural fallback).
- ADR-011 §2 is law: all identifiers go through `projection/ids.rs` (`slug`, `ident`, `sanitize_filename`, `element_id`, `NameRegistrar`) — no private sanitizers. All flow resolution goes through `projection/flows.rs::collect_flows` (loud dangling-ref policy) and `model_namespace` (single-namespace rule).

### Global verification gates (must stay green after EVERY task)

```bash
cargo test --features cli --manifest-path domainforge-core/Cargo.toml
cargo clippy --features cli --manifest-path domainforge-core/Cargo.toml -- -D warnings
cargo fmt --manifest-path domainforge-core/Cargo.toml -- --check
```

---

## Key facts already discovered (do not re-derive)

| Thing | Location |
| --- | --- |
| Family kernel shape: one IR, one renderer, `emit(graph, model_ref, created_at, &mut ArtifactSink) -> Result<Vec<String>, String>`, `project_<x>_in_memory`, fixture gate, CI `verify-<family>` job | `docs/projection-families.md`; example: `domainforge-core/src/projection/cloudevents/mod.rs:28-60` |
| `ArtifactSink` (Dir / Memory variants) | `domainforge-core/src/projection/sink.rs` |
| Blessed identifier helpers: `slug`, `ident`, `sanitize_filename`, `element_id`, `content_hash`, `NameRegistrar` (collision → `_<hash8>` suffix) | `domainforge-core/src/projection/ids.rs` |
| Blessed flow resolution: `collect_flows` returns `Vec<ResolvedFlow{resource,from,to,quantity,namespace}>` sorted by (resource,from,to,quantity); errors on dangling refs. `model_namespace` errors on multi-namespace, defaults `"default"` | `domainforge-core/src/projection/flows.rs:34-103` |
| CLI registration: `ProjectFormat` enum at `cli/project.rs:94-152` (e.g. `#[value(name = "cloudevents")]` at :131); dispatch match arms at :228-250; per-target `run_<x>` fns from :485 (pattern: reject `--recipe`, require/create output dir, `ArtifactSink::Dir`, call `emit`, print summary) | `domainforge-core/src/cli/project.rs` |
| Projection submodule registration | `domainforge-core/src/projection/mod.rs` |
| **Flow annotations are parsed into the AST but DROPPED at graph build** — third pass destructures `AstNode::Flow { .. }` ignoring `annotations` | `domainforge-core/src/parser/ast.rs:2469-2501` |
| `primitives::Flow` already has `attributes: HashMap<String, Value>` + `set_attribute`/`get_attribute`/`attributes` — currently unused by the parser | `domainforge-core/src/primitives/flow.rs:36,99-108` |
| Shared proving fixture (2 entities-as-actors + 1 approver, 2 resources, 2 flows, roles, pattern, relation, instance, policy, metric) | `fixtures/projection_cell/basic/model.sea` |
| Gate scripts live in `scripts/verify/projection-targets/<target>.sh`, invoked by `all.sh`; pattern: build fixture projection into `mktemp -d`, structural asserts via inline python, real toolchain if available, warn+skip otherwise (CI always runs full toolchain) | `scripts/verify/projection-targets/tla.sh` |
| CI verify jobs: `verify-lean` :343, `verify-dspy` :696, `verify-projection-targets` referenced by tla.sh | `.github/workflows/ci.yml` |
| Bindings convention: projection export is **CLI-only**; no Python/TS/WASM binding work needed | `docs/projection-target-implementation-status.md` §Bindings |
| Graph accessors available: `all_entities()`, `all_resources()`, `all_flows()`; entities/resources expose `id()`, `name()`, `namespace()`; policies/metrics/roles/patterns/instances reachable via graph (see how cedar/asyncapi read policies: `projection/cedar/mod.rs`, `projection/asyncapi/mod.rs`) | `domainforge-core/src/graph/mod.rs` |

---

## The mapping (normative — the IR encodes exactly this)

This is the SEA → DDD semantic mapping. It is language-neutral; every renderer expresses the same IR. Some domain constructs are minted from **combinations** of SEA elements (marked ⊕).

| SEA element | Domain construct |
| --- | --- |
| `@namespace` | Package/module name: `slug(namespace)` (Python package `<ns>_domain`, npm package `<ns>-domain`, crate `<ns>_domain`) |
| `Entity` | DDD **Entity** class: identity field `id` + `name`; attributes from any `instance of` that entity become typed optional fields (string-typed in v1) |
| `Resource` | ⊕ Three constructs: (a) **Aggregate root** — the stateful thing with lifecycle whose state transitions are the flows that move it; (b) **Repository port** (abstract base) `<Resource>Repository` with `get(id)`, `save(aggregate)` — resource is semantically isomorphic to repository; (c) the resource's unit → **Quantity value object** `Quantity { amount: Decimal-like, unit: str }` (one shared VO, unit carried as value) |
| `Flow` (default, no `@cqrs`) | ⊕ Flow + its Resource + endpoints mint THREE constructs: (a) **Command** `Transfer<Resource>From<From>To<To>` (payload: aggregate id, quantity; all name parts through `ident()`, registered in one `NameRegistrar`); (b) **Event** `<Resource>TransferredFrom<From>To<To>` (past-tense fact, payload mirrors command + occurred_at); (c) an **aggregate method** on the Resource's aggregate root — `transfer_to_<slug(to)>(...)` — that checks policy guards and returns the event (command → aggregate → event, the CQRS write path) |
| `Flow` with `@cqrs { "kind": "command" }` | Command + aggregate method only (no event) |
| `Flow` with `@cqrs { "kind": "event" }` | Event only (an externally-observed fact; no command, no method) |
| `Policy` | **Domain error** `<Ident(policy_name)>Violation` extending the package's `DomainError` base + a guard hook: each aggregate method calls `check_<slug(policy)>()` (v1 body: no-op returning None/Ok — the *expression* is not compiled to code; the `@rationale` string becomes the docstring so the human fills the predicate). One error type per policy regardless of how many methods guard it |
| `Metric` | **Query** (CQRS read side): method `get_<slug(metric)>()` on a `<Ns>ReadModel` port (abstract base), returning the Quantity VO; `@unit`/`@threshold` go in the docstring |
| `Pattern` | **Value object** `<Ident(pattern)>` wrapping a string, constructor validates against the regex and raises/returns `Invalid<Ident(pattern)>` domain error |
| `Role` | Enum `Role { <Ident(role)>, ... }`; commands carry an `issued_by: Role` field |
| `Relation` | Doc-comment on the Role enum listing `subject —predicate→ object` (v1; typed association is backlog) |
| `instance` | Example construction in the generated package's smoke test (see per-language layout) |
| Dependency injection | **Container**: one composition-root type holding every port (all repository ports, `EventBus`, `CommandBus`, `<Ns>ReadModel`) via constructor injection; no framework, no registry |
| Event/message bus | Two ports in `ports/bus`: `EventBus.publish(event: DomainEvent)` and `CommandBus.dispatch(command: Command)` — abstract only |

Determinism rules: all lists sorted (flows already sorted by `collect_flows`; sort entities/resources/policies/metrics/roles/patterns by name); every generated file's identifier set minted through one `NameRegistrar` per emission; no timestamps in code files (README may carry `created_at` like other targets); byte-identical output run-to-run.

---

## Task 1 — Plumb flow annotations into the Graph (parser · blocking)

**Goal:** `@cqrs { "kind": "command" }` on a Flow is readable from `Graph` flows and from `ResolvedFlow`, proven by a parser test.

**Why this shape:** The AST already carries `annotations` on `AstNode::Flow` and `primitives::Flow` already has an `attributes` map with `set_attribute` — the fix is a copy at graph-build time, not a new data model.

### Steps

1. In the third pass at `domainforge-core/src/parser/ast.rs:2469-2501`, stop discarding annotations: destructure `annotations` from `AstNode::Flow`, and after `Flow::new_with_namespace(...)` copy each `(key, value)` via `flow.set_attribute(key, value)` before `graph.add_flow(flow)`. Match however the AST stores annotation values (inspect `AstNode::Flow` definition in the same file / `parser/ast_schema.rs`) — convert to `serde_json::Value` as `primitives/flow.rs:99` expects.
2. Extend `ResolvedFlow` in `domainforge-core/src/projection/flows.rs:22-28` with `pub annotations: std::collections::BTreeMap<String, serde_json::Value>` populated from `f.attributes()` (BTreeMap for deterministic order). Update the struct's construction at :58-64.
3. Fix any compile fallout in the eight existing targets (they construct/pattern-match `ResolvedFlow` only via `collect_flows`, so fallout should be nil; tests in flows.rs construct via parse — add the field where needed).
4. Add a unit test in `flows.rs` tests: parse a model with `Flow "R" @cqrs { "kind": "command" } from "A" to "B"` and assert `flows[0].annotations["cqrs"]["kind"] == "command"`.

### Gate

```bash
cargo test --features cli --manifest-path domainforge-core/Cargo.toml flows
cargo test --features cli --manifest-path domainforge-core/Cargo.toml
```

**Done when:** The new annotation test passes, and deliberately reverting the `set_attribute` copy makes it fail (teeth-check). Full test suite still green.

**Redesign trigger:** If AST annotation values are not representable as `serde_json::Value` (e.g. a custom enum), add a small `fn annotation_to_json` in `parser/ast.rs` rather than changing `primitives::Flow`.

---

## Task 2 — Domain IR: `projection/domain/ir.rs` (kernel · blocking)

**Goal:** `DomainIr::from_graph(&Graph) -> Result<DomainIr, String>` produces the complete language-neutral model of the mapping table, proven by unit tests against the shared fixture.

**Why this shape:** CORE PRINCIPLE — semantics decided once. The dspy family (`projection/dspy/ir.rs` + `render.rs`) is the structural template: IR structs are plain data (String names already sanitized, Vec fields already sorted), renderers are pure functions IR → text.

### Steps

1. Create `domainforge-core/src/projection/domain/mod.rs` and `domainforge-core/src/projection/domain/ir.rs`; register `pub mod domain;` in `domainforge-core/src/projection/mod.rs`.
2. In `ir.rs`, define plain-data structs mirroring the mapping table. Suggested shape:
   `DomainIr { namespace: String, entities: Vec<EntityIr>, aggregates: Vec<AggregateIr>, commands: Vec<CommandIr>, events: Vec<EventIr>, errors: Vec<ErrorIr>, queries: Vec<QueryIr>, value_objects: Vec<ValueObjectIr>, roles: Vec<String>, relations_doc: Vec<String>, instances: Vec<InstanceIr> }`
   where `AggregateIr { name, resource_name, unit, methods: Vec<MethodIr> }`, `MethodIr { name, command: Option<String>, event: Option<String>, guard_errors: Vec<String>, to_entity, from_entity, quantity }`, `CommandIr { name, aggregate, payload fields... }`, `EventIr { name, ... }`, `ErrorIr { name, rationale: Option<String>, policy_name }`, `QueryIr { name, metric_name, unit: Option<String>, threshold: Option<String>, doc }`, `ValueObjectIr { name, regex, error_name }`, `InstanceIr { name, entity, fields: Vec<(String,String)> }`. All names pre-sanitized here via `ids::ident`/`ids::slug` + one `NameRegistrar`.
3. Implement `from_graph`: use `flows::model_namespace` and `flows::collect_flows` (Task 1's annotations drive the `@cqrs` kind switch: absent/other → command+event+method; `"command"` → no event; `"event"` → event only). Read entities/resources/roles/policies/metrics/patterns/instances from the graph the same way `cedar/mod.rs` and `asyncapi/mod.rs` do (copy their accessor idioms). Sort every collection by name. A Resource with no flows still yields an aggregate + repository (empty methods).
4. Unit tests in `ir.rs` parsing `fixtures/projection_cell/basic/model.sea` (read via `std::fs`, like existing target tests): assert 3 entities; 2 aggregates (`PurchaseOrder`, `Payment`); 2 commands + 2 events with exact names (`TransferPurchaseOrderFromBuyerToSupplier`, `PurchaseOrderTransferredFromBuyerToSupplier`, and the Payment pair); 1 error (`RequireApprovalViolation`) with rationale; 1 query (`get_order_count`); 1 value object (`OrderNumber` with the regex); 2 roles; determinism (`from_graph` twice → equal). Plus one inline-source test for each `@cqrs` kind.

### Gate

```bash
cargo test --features cli --manifest-path domainforge-core/Cargo.toml projection::domain
```

**Done when:** All IR tests pass; changing a fixture-derived expected name in the test makes it fail (teeth-check).

**Redesign trigger:** If graph accessors for patterns/instances/metrics don't exist or return insufficient data, check how `asyncapi`/`cedar` read them first; if genuinely absent, drop that mapping row to a doc-comment in the generated README (do NOT add new graph API in this plan) and note it in the status doc.

---

## Task 3 — Python renderer + CLI + gate (`--format domain-python`)

**Goal:** `domainforge project model.sea --format domain-python --output <dir>` writes a complete Python package that `python -m compileall` and `mypy --strict` accept as-is.

**Why this shape:** Ready-to-use means the emitted package must typecheck with zero edits. Abstract bases use `abc.ABC` + `@abstractmethod`; dataclasses for commands/events/VOs (frozen); no third-party runtime deps (stdlib only) so the package installs anywhere.

### Steps

1. Create `domainforge-core/src/projection/domain/python.rs` with `pub fn emit(graph, model_ref, created_at, &mut ArtifactSink) -> Result<Vec<String>, String>` and `project_domain_python_in_memory` (mirror `cloudevents/mod.rs:28-75` signatures). `emit` = `DomainIr::from_graph` + pure render functions.
2. Emit exactly this layout (root-relative paths into the sink; package dir `src/<slug(ns)>_domain/`):
   - `pyproject.toml` (name `<ns>-domain`, `requires-python >= "3.10"`, no deps, setuptools src-layout)
   - `README.md` (provenance: model_ref, created_at, the mapping table summary)
   - `src/<ns>_domain/__init__.py` (re-export public names)
   - `.../domain/__init__.py`, `value_objects.py` (Quantity + pattern VOs), `entities.py`, `errors.py` (`DomainError(Exception)` base + policy errors + pattern errors), `commands.py` (frozen dataclasses, `issued_by: Role`), `events.py` (frozen dataclasses + `DomainEvent` base, `occurred_at: str`), `aggregates.py` (one class per aggregate; each MethodIr → method with guard calls returning the event), `queries.py` (nothing if read model is a port — put query dataclasses here only if payload exists; else omit file)
   - `.../ports/__init__.py`, `repositories.py` (one ABC per aggregate: `get`, `save`), `bus.py` (`EventBus`, `CommandBus` ABCs), `read_model.py` (`<Ns>ReadModel` ABC with one method per QueryIr)
   - `.../container.py` (frozen dataclass `Container` holding every port)
   - `tests/test_domain_smoke.py` (constructs each command/event/VO, exercises instance examples, asserts pattern VO rejects a non-matching string)
3. Wire CLI: add `#[value(name = "domain-python")] DomainPython` to `ProjectFormat` (`cli/project.rs:94-152`) with doc-comment `/// Code operator: Python DDD/CQRS domain layer — complete package up to the ports (directory output)`; add match arm ~:250 and `run_domain_python` following the `run_cloudevents` pattern at :485 (reject `--recipe`, dir-only output).
4. Rust unit tests in `python.rs`: in-memory emit of the fixture → assert file set, assert key content (`class PurchaseOrderRepository(ABC)`, `def transfer_to_supplier`, `class RequireApprovalViolation(DomainError)`), determinism (two emits byte-equal).
5. Gate script `scripts/verify/projection-targets/domain-python.sh` (copy tla.sh skeleton): project fixture to `mktemp -d`; structural asserts via inline python; then `python3 -m compileall -q src tests`; then `mypy --strict src` if mypy on PATH else warn+skip (CI installs it); then run the smoke test `python3 -m unittest discover tests` (write the smoke test with `unittest`, not pytest, so no install needed). Register in `scripts/verify/projection-targets/all.sh`.

### Gate

```bash
cargo test --features cli --manifest-path domainforge-core/Cargo.toml projection::domain::python
bash scripts/verify/projection-targets/domain-python.sh
```

**Done when:** Gate exits 0 with compileall + smoke test actually running locally; deliberately emitting a syntax error (e.g. drop a `:`) makes the script fail (teeth-check).

**Redesign trigger:** If `mypy --strict` fights generated ABCs, downgrade the script to `mypy` default strictness and note it in the script comment — do not weaken the emitted code.

---

## Task 4 — TypeScript renderer + CLI + gate (`--format domain-typescript`)

**Goal:** Same model → a complete TypeScript package that `tsc --noEmit` accepts as-is.

**Why this shape:** Interfaces are the natural port form; abstract classes only for aggregates (state + methods). Zero runtime deps; `strict: true` tsconfig proves the shapes.

### Steps

1. Create `domainforge-core/src/projection/domain/typescript.rs`: `emit` + `project_domain_typescript_in_memory` (same signatures as Task 3, same IR input).
2. Layout: `package.json` (name `<ns>-domain`, `"private": true`, devDependency `typescript`, script `"check": "tsc --noEmit"`), `tsconfig.json` (`strict`, `noEmit`-compatible, `target: "ES2020"`, `module: "ES2020"`), `README.md`, `src/index.ts`, `src/domain/valueObjects.ts`, `entities.ts`, `errors.ts` (class `DomainError extends Error` + subclasses), `commands.ts` (readonly interfaces + `type Command = union`), `events.ts` (readonly interfaces + `type DomainEvent = union`), `aggregates.ts` (classes; each method validates guards, returns the event object), `src/ports/repositories.ts` (one `interface <X>Repository { get(id): Promise<X | null>; save(a): Promise<void> }` per aggregate), `src/ports/bus.ts` (`EventBus { publish(e: DomainEvent): Promise<void> }`, `CommandBus { dispatch(c: Command): Promise<void> }`), `src/ports/readModel.ts`, `src/container.ts` (`interface Container` + `createContainer(ports): Container`), `src/smoke.ts` (compile-time exercise of every construct — instance examples as consts; runtime assert via a tiny `function assert()`).
3. Wire CLI: `#[value(name = "domain-typescript")] DomainTypescript`, match arm, `run_domain_typescript` (same pattern as Task 3 step 3).
4. Rust unit tests in `typescript.rs`: file set, key content (`interface PurchaseOrderRepository`, `class RequireApprovalViolation extends DomainError`), determinism.
5. Gate script `scripts/verify/projection-targets/domain-typescript.sh`: project fixture; structural asserts; then if `npx` available: `npm install --no-audit --no-fund typescript` in the temp dir (or `npx -y -p typescript tsc --noEmit -p .`) and run `tsc --noEmit`; else warn+skip. Register in `all.sh`.

### Gate

```bash
cargo test --features cli --manifest-path domainforge-core/Cargo.toml projection::domain::typescript
bash scripts/verify/projection-targets/domain-typescript.sh
```

**Done when:** `tsc --noEmit` passes on the generated package; deliberately removing a type import makes it fail (teeth-check).

**Redesign trigger:** none plausible.

---

## Task 5 — Rust renderer + CLI + gate (`--format domain-rust`)

**Goal:** Same model → a complete Rust crate that `cargo check` accepts as-is.

**Why this shape:** Ports are traits (object-safe: `&self` methods, no generics); commands/events are structs with `Debug, Clone, PartialEq`; errors implement `std::error::Error` by hand (no thiserror — zero deps so `cargo check` needs no network beyond the empty registry; use `[dependencies]` empty).

### Steps

1. Create `domainforge-core/src/projection/domain/rust.rs`: `emit` + `project_domain_rust_in_memory`.
2. Layout: `Cargo.toml` (name `<slug(ns)>_domain`, edition 2021, no deps), `README.md`, `src/lib.rs` (module tree + re-exports), `src/domain/mod.rs`, `value_objects.rs` (Quantity{amount: f64 is WRONG — use `amount: rust_decimal`-free plain `String`-carried decimal? No: emit `amount: f64` is lossy; emit `amount: i64` scaled is over-engineering — emit `amount: f64` with a doc-comment noting precision, OR carry the quantity as `String`. DECISION: `pub amount: f64` + doc-comment; quantities in the fixture are integers), `entities.rs`, `errors.rs` (`pub enum DomainError` with one variant per policy/pattern error + `Display` + `std::error::Error` impl — enum, not trait objects, is idiomatic here), `commands.rs`, `events.rs`, `aggregates.rs` (structs + `impl` blocks; methods return `Result<EventX, DomainError>`), `src/ports/mod.rs`, `repositories.rs` (one trait per aggregate: `fn get(&self, id: &str) -> Result<Option<X>, DomainError>; fn save(&self, a: &X) -> Result<(), DomainError>`), `bus.rs` (`trait EventBus { fn publish(&self, event: DomainEvent) -> Result<(), DomainError>; }` — requires `enum DomainEvent` wrapping all events; same for `CommandBus`/`enum Command`), `read_model.rs`, `src/container.rs` (`pub struct Container { pub <x>_repository: Box<dyn XRepository>, ... }`), and a smoke test as `tests/smoke.rs` (constructs commands/events/VOs, asserts pattern VO rejects bad input via the generated regex check — implement regex matching WITHOUT the regex crate: v1 pattern VO in Rust validates non-empty + documents the regex in a doc-comment, since zero-dep is the harder constraint; the Python/TS versions do full regex).
3. Wire CLI: `#[value(name = "domain-rust")] DomainRust`, match arm, `run_domain_rust`.
4. Rust unit tests in `rust.rs`: file set, key content (`pub trait PurchaseOrderRepository`, `RequireApprovalViolation` variant), determinism.
5. Gate script `scripts/verify/projection-targets/domain-rust.sh`: project fixture; structural asserts; then `cargo check --offline` falling back to `cargo check` in the temp dir (zero-dep crate, so offline works), then `cargo test` (runs the smoke test). Register in `all.sh`.

### Gate

```bash
cargo test --features cli --manifest-path domainforge-core/Cargo.toml projection::domain::rust
bash scripts/verify/projection-targets/domain-rust.sh
```

**Done when:** `cargo check` + `cargo test` pass on the generated crate; deliberately breaking an emitted trait signature makes the script fail (teeth-check).

**Redesign trigger:** If `cargo check` inside a `cargo test` environment collides with the workspace (nested-workspace error), emit `[workspace]` (empty table) in the generated Cargo.toml — the standard opt-out.

---

## Task 6 — CI jobs + docs (coherence · last)

**Goal:** CI proves all three gates on every push; docs describe the new family accurately — no claim ahead of a passing gate.

### Steps

1. Add three CI jobs to `.github/workflows/ci.yml` mirroring `verify-dspy` (:696): `verify-domain-python` (setup python + `pip install mypy`, run the script), `verify-domain-typescript` (setup node, run the script), `verify-domain-rust` (rust toolchain already present, run the script). Copy the existing jobs' checkout/build-CLI steps verbatim.
2. `docs/projection-families.md`: add a "Code targets" row group (three rows) to the second table, operator family **Code**, with the mapping-table summary and CLI formats.
3. `docs/projection-target-implementation-status.md`: add the three targets with toolchain notes (compileall+mypy / tsc / cargo check+test), note the Task 1 annotation plumbing, and that Roles/Policies/Metrics/Patterns/Instances ARE now consumed by these targets (updating the "not yet consumed" caveat).
4. New how-to `docs/how-tos/project-domain-code.md`: one page — command lines for all three formats, the normative mapping table (copy from this plan), the `@cqrs` annotation switch, and "what you still write by hand" (policy predicate bodies, infrastructure adapters).

### Gate

```bash
bash scripts/verify/projection-targets/domain-python.sh && \
bash scripts/verify/projection-targets/domain-typescript.sh && \
bash scripts/verify/projection-targets/domain-rust.sh
cargo test --features cli --manifest-path domainforge-core/Cargo.toml
```

**Done when:** All three scripts and the full suite exit 0 locally; CI YAML lints (`actionlint` if available, else visual diff against verify-dspy).

**Redesign trigger:** none plausible.

---

## Final acceptance checklist (whole plan)

- [ ] `@cqrs` flow annotations reach `ResolvedFlow`; reverting the copy fails the test *(Task 1)*
- [ ] `DomainIr::from_graph` maps the fixture per the normative table; deterministic across runs *(Task 2)*
- [ ] `--format domain-python` emits a package passing compileall + mypy + unittest smoke *(Task 3)*
- [ ] `--format domain-typescript` emits a package passing `tsc --noEmit` strict *(Task 4)*
- [ ] `--format domain-rust` emits a crate passing `cargo check` + `cargo test` *(Task 5)*
- [ ] Three CI verify jobs green; docs updated; how-to exists *(Task 6)*
- [ ] `cargo clippy -- -D warnings` and `cargo fmt --check` exit 0
- [ ] All three targets byte-identical run-to-run on the fixture (asserted in each renderer's determinism test)

## Guardrails (do not violate)

- **Do NOT modify `fixtures/projection_cell/basic/model.sea`** — other targets' gate scripts assert its exact flow set (e.g. tla.sh asserts `TransferPurchaseOrder_Buyer_Supplier`). Test `@cqrs` behavior with inline sources in unit tests only.
- **No new graph API** beyond the Task 1 annotation copy. If an IR mapping needs data the graph can't give, degrade that row to README prose and record it in the status doc.
- **No bindings work** — projection export is CLI-only by convention (status doc §Bindings).
- **Zero runtime dependencies in all three generated packages** — stdlib only; the generated code must build offline.
- **Renderers never touch `Graph`** — IR only (CORE PRINCIPLE). Reviewers should reject any `graph.` call in `python.rs` / `typescript.rs` / `rust.rs`.
- All identifiers via `projection/ids.rs`; all flow reads via `projection/flows.rs` (ADR-011 §2).
- Generated aggregates/ports stop at the **port / abstract base** boundary: no in-memory repository impls, no bus impls, no persistence — that is the consumer's job.
- Commit hygiene: Task 1 (parser plumbing) in its own commit; each language target in its own commit; docs/CI last.
