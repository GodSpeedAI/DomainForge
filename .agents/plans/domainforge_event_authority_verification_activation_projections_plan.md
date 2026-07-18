# Implementation Plan — DomainForge Event, Authority, Verification, and Activation Projections

**Created:** 2026-07-08
**Revised:** 2026-07-08 (grounding correction — see note below)
**Source of truth:** Current repository code, `justfile`, `.github/workflows/ci.yml`, and `docs/`. Earlier external notes named `/mnt/data/semantic-projection.md`, `/mnt/data/write-in-sea.md`, and `/mnt/data/domainforge-strategic-archive.md`; those files are not present in this workspace and must not be treated as available implementation evidence.
**Originating context:** The projection strategy identified eight additional targets — AsyncAPI, CloudEvents, Gauge, Cedar, TLA+, Alloy, Dagger, and Devbox — as the missing event, authority, verification, and activation layer around existing semantic projections.
**Revision note:** The first version of this plan and its supporting audit cited a `domainforge-core/src/projection/otel/` module, a `domainforge-core/tests/common/projection_harness.rs` test harness, a `docs/projection-families.md` doc, and a `fixtures/<family>/basic/domain/model.sea` fixture convention as existing precedent. **None of these exist in this repository.** They were fabricated. This revision replaces every such claim with verified paths and states plainly, where nothing exists yet, that the task must establish the convention rather than "follow" one.
**Status of the work today:** DomainForge already has a `.sea` source language and a projection posture for `calm`, `kg` (RDF/Turtle), and `protobuf`/`proto`. Repo audit on 2026-07-08 found no implementation for the eight new targets, no per-family projection module pattern, no per-family fixture convention, and no existing projection bindings exposed through Python/TypeScript/WASM. The current implementation spine is a CLI `ProjectFormat` enum (`Calm`, `Kg`, `Protobuf`, `Proto`) plus flat modules under `domainforge-core/src/projection/`; `ProjectionRegistry` is only for SEA `Mapping`/`Projection` contracts, not the CLI target registry. These eight targets remain roadmap until their target gates pass.

---

## 0. How to use this plan (agent operating instructions)

- Execute tasks in the order given. Dependencies:
  - Task 1 is mandatory before all implementation.
  - Task 2 → Task 3 are dependent: AsyncAPI reuses CloudEvents event/message semantics.
  - Task 4 → Task 5 are dependent: Dagger should invoke or validate the Devbox environment.
  - Task 6 can start after Task 1, but its final integration should be called by Dagger in Task 5 or Task 10.
  - Tasks 7, 8, and 9 can start after Task 1; Task 10 integrates them into one projected organization fixture.
- **Every task ends with a verification gate.** Do not mark a task done until its gate command exits 0.
- **SEA remains the canonical semantic source; every target must be generated from a projection-specific IR, rendered deterministically, validated with a target-native or target-aware gate, and traced back to stable `.sea` element identity. Do not change the SEA grammar unless Task 1 proves an unavoidable missing primitive.**
- Match surrounding code style and the pinned locations below:
  - parser / AST: `domainforge-core/grammar/sea.pest`, `domainforge-core/src/parser/ast.rs`
  - CLI projection target enum / dispatcher: `domainforge-core/src/cli/project.rs`
  - projection modules and shared sink/identity helpers: `domainforge-core/src/projection/` (currently `buf.rs`, `contracts.rs`, `engine.rs`, `mod.rs`, `protobuf.rs`, `registry.rs` — flat, no per-family submodule pattern exists yet; Task 1 establishes the first one)
  - fixtures: new dedicated tree at `fixtures/projection_cell/<family>/model.sea` (see Task 1 — this is a **new** convention, not a mirror of an existing one)
  - integration tests: `domainforge-core/tests/<family>_projection_tests.rs` (matches the existing flat naming convention, e.g. `protobuf_projection_tests.rs`, `calm_round_trip_tests.rs`)
  - binding tests (only if a task adds bindings — see bindings guardrail below): `tests/test_<family>.py`, `typescript-tests/<family>.test.ts`
  - CLI docs: `docs/reference/cli-commands.md`
- **Bindings guardrail (corrected):** none of the existing projection families (`calm`, `kg`, `protobuf`) are exposed through `domainforge-core/src/python/`, `domainforge-core/src/typescript/`, or `domainforge-core/src/wasm/` today — those directories only bind `authority`, `error`, `formatter`, `graph`, `policy`, `primitives`, `registry`, `semantic_pack`, `units`. Projection export is CLI-only in the current codebase. Treat bindings for a new target as **opt-in per task**, added only when the task explicitly justifies the need (e.g. a downstream consumer requires in-memory access), not as a default with an exception clause.
- The projection outputs are generated artifacts. Do not hand-edit generated outputs to make gates pass. Follow `docs/reference/generated-artifacts-policy.md`: committed generated files are allowed only as explicit, documented `fixtures/` test fixtures; everything else must be regenerated from a clean checkout, and gates should prefer temp output plus byte-determinism/content assertions over committed generated-output trees.
- Runtime wiring belongs in Dagger/Devbox/CubeSandbox/manifest artifacts, not in `.sea`, unless it expresses durable semantic intent.

### Global verification gates (must stay green after EVERY task)

The repo already has a working CI/local verification stack in `justfile` and `.github/workflows/ci.yml` — Task 1 must extend that stack, not replace it with a parallel `scripts/verify/` tree invented from nothing. After Task 1, run:

```bash
just ci-test-rust        # cargo test --verbose --workspace --features cli
just rust-test           # cargo test -p domainforge-core --features cli
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
bash scripts/verify/projection-targets/all.sh   # created in Task 1; wraps the above plus per-target validators
```

Notes on the commands above (verified 2026-07-08, do not re-derive):

- CI does **not** run `cargo test --all-features` or `--workspace --all-features` for the test job; it runs `cargo test --verbose --workspace --features cli` (see `justfile:163-165`, `ci.yml` "Test Rust" job). The `python`, `typescript`, and `wasm` Cargo features are native-extension bindings (pyo3 / napi / wasm-bindgen) built and tested in **separate** CI jobs via `just ci-test-python`, `just ci-test-ts`, and a dedicated WASM job — they are not combined with `--all-features` in one `cargo test` invocation. Do not assume `--all-features` compiles for `cargo test`; only `cargo clippy --all-targets --all-features` is confirmed safe (`ci.yml:58`).
- `cargo run -p domainforge-core --features cli -- validate <path>` is valid; the bin target is named `domainforge` (`domainforge-core/Cargo.toml:96`, feature `cli` gates `clap`/`colored`/`three_valued_logic`/`signing`, see `domainforge-core/Cargo.toml:77-87`).
- The workspace (`/Cargo.toml`) has exactly one member: `domainforge-core`. `--workspace` flags are correct but only ever apply to this one crate today.

Rebuild steps if you touched generated bindings, CLI snapshots, or committed generated examples before running the gates:

```bash
cargo build -p domainforge-core --features cli
bash scripts/verify/projection-targets/regenerate-fixtures.sh
```

---

## Key facts already discovered (do not re-derive) — corrected 2026-07-08

Re-check line numbers before implementation if the branch has moved. Every row below was independently verified against the working tree; rows present in the original plan that pointed at nonexistent files have been removed or corrected.

| Thing | Location | Status |
|---|---|---|
| SEA grammar source; do not change unless unavoidable | `domainforge-core/grammar/sea.pest:373` `target_format = { ^"calm" \| ^"kg" \| ^"sbvr" \| ^"protobuf" \| ^"proto" }` | Verified |
| SEA parser / AST owner | `domainforge-core/src/parser/ast.rs:155` `AstNode`; `~1960` `parse_target_format`; `~2093` `parse_projection` | Verified (re-check exact lines; file is large and may have shifted) |
| CLI projection target enum / dispatcher | `domainforge-core/src/cli/project.rs:64-69` `ProjectFormat { Calm, Kg, Protobuf, Proto }`; dispatch `match` at `~111-141` | Verified — **only 4 variants, no `Sbvr`, no `Dsl`, despite grammar allowing `sbvr` and docs describing `rdf`/`sbvr`/`dsl`** |
| SEA mapping/projection contract registry, not a CLI target registry | `domainforge-core/src/projection/registry.rs:5,14` `ProjectionRegistry::find_mappings_for_target` | Verified |
| Actual projection module set (flat, no family submodule pattern yet) | `domainforge-core/src/projection/{buf,contracts,engine,mod,protobuf,registry}.rs` | Verified — **there is no `otel/`, `bpmn/`, or `baml/` submodule anywhere in the repo; any reference to one in an earlier draft was fabricated** |
| Shared projection module registration | `domainforge-core/src/projection/mod.rs` | Verified |
| Existing CLI projection command | `domainforge-core/src/cli/project.rs` args struct + `run()` | Verified |
| Existing fixture layout (the only one that exists) | `fixtures/semantic_packs/acme_procurement/{domain,tests,review}/*.sea` — flat `.sea` files per concern | Verified — **there is no `fixtures/<family>/basic/domain/model.sea` tree anywhere; that convention does not exist and must be established fresh (see Task 1)** |
| Test naming convention to follow | `domainforge-core/tests/{protobuf_projection_tests,calm_round_trip_tests,sbvr_parsing_tests,...}.rs` — flat files per family/concern, ~80 total | Verified |
| No shared test harness module exists | N/A | Verified — **`domainforge-core/tests/common/` does not exist; do not reference a shared harness that isn't there** |
| Python/TS bindings that exist today | `domainforge-core/src/python/{authority,error,formatter,graph,policy,primitives,registry,semantic_pack,units}.rs`; same set under `src/typescript/`; `src/wasm/` has the same minus `registry.rs` | Verified — **no `projection` binding exists in any of the three; projection export is CLI-only today** |
| CLI docs (stale relative to code — reconcile in Task 1) | `docs/reference/cli-commands.md:70-98` describes `--format <calm\|rdf\|sbvr\|dsl\|protobuf>`, but the enum has no `Sbvr`/`Dsl` variant; `rdf` maps to the `Kg` variant's extension-based branch, not a separate format name | Verified drift — must be reconciled, not cited as ground truth as-is |
| Docs page for projection targets | **No `docs/projection-families.md` exists.** Real docs entry points are `docs/reference/cli-commands.md` and `docs/reference/generated-artifacts-policy.md`. Task 1 creates a new status doc; do not invent a nonexistent index page. | Corrected |
| Generated-artifact policy governing fixtures/scripts | `docs/reference/generated-artifacts-policy.md` — scripts/tests/fixtures are source truth; generated output must not be hand-edited; committed generated files only allowed as documented `fixtures/` test fixtures | Verified — supports Task 1's fixture and script additions |
| Existing local/CI verification stack to extend, not replace | `justfile` (`ci-test-rust`, `ci-test-python`, `ci-test-ts`, `rust-test`, `python-test`, `ts-test`, `all-tests`, `ai-validate`); `.github/workflows/ci.yml` (separate Lint/Rust/Python/TypeScript/WASM/Integration jobs) | Verified — Task 1's new `scripts/verify/` tree must call into these, not duplicate them |
| Fixture created by this plan | `fixtures/projection_cell/basic/model.sea` (see Task 1 — deliberately new convention, flat file, not a `basic/domain/model.sea` subtree, matching the flat style seen in `fixtures/semantic_packs/acme_procurement/domain/*.sea`) | New |
| All-target verification entrypoint created by this plan | `scripts/verify/projection-targets/all.sh` | New |

---

## Task 1 — Pin current projection architecture and create the projection-target harness  (foundation · P0)

**Goal:** The repo contains one canonical projection-cell fixture and runnable verification scripts that prove the baseline still works, built on top of the repo's real `just`/CI conventions rather than a parallel invented stack.

**Why this shape:** The eight targets must not become eight ad-hoc exporters, and the harness that ties them together must not be built on fabricated precedent. The first settlement is locating the *actual* existing projection architecture (there is less of one than earlier drafts assumed) and creating one harness all later targets can plug into.

### Steps

1. Re-verify the "Key facts already discovered" table above against the current branch. If line numbers moved, update them. If a row's file no longer exists, treat that as a real regression to flag, not something to route around silently.
2. Create a minimal fixture at `fixtures/projection_cell/basic/model.sea` using valid existing SEA syntax only. There is no established `<family>/basic/domain/model.sea` subtree to match — follow the flatter style used in `fixtures/semantic_packs/acme_procurement/domain/*.sea` instead, and document in a header comment that this is a new fixture family introduced for these eight targets. Include:
   - one governed action or flow
   - one role/principal concept
   - one resource
   - one policy
   - one metric
   - one event-like flow or relation that CloudEvents/AsyncAPI can project
   - one scenario-like structure if existing SEA supports it; otherwise model it with existing `Pattern`, `Policy`, `Metric`, `Mapping`, or `Projection` primitives.
3. Add `scripts/verify/domainforge-baseline.sh` that shells out to the existing `just` recipes (`just ci-test-rust`, `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`) rather than re-implementing them. This keeps one source of truth for what "baseline green" means.
4. Add `scripts/verify/projection-targets/all.sh`. Initially it should call the baseline script plus `cargo run -p domainforge-core --features cli -- validate fixtures/projection_cell/basic/model.sea`. As each target lands, append that target's verification script.
5. Add `scripts/verify/projection-targets/regenerate-fixtures.sh` as a deliberate, reviewed fixture regeneration entrypoint. It may be a no-op until target renderers exist.
6. Create `docs/projection-target-implementation-status.md` stating these eight targets are planned/partial until their gates pass. Do not reference or create a `docs/projections/` or `docs/projection-families.md` hierarchy — no such index exists in this repo; the closest real entry points are `docs/reference/cli-commands.md` and `docs/reference/generated-artifacts-policy.md`.
7. Reconcile `docs/reference/cli-commands.md`: it currently documents `--format <calm|rdf|sbvr|dsl|protobuf>` against a `ProjectFormat` enum that only has `Calm`, `Kg`, `Protobuf`, `Proto`. Either fix the doc to match the real enum (rename `rdf`→`kg`, drop `sbvr`/`dsl` or note them as grammar-only/not CLI-exposed) or file this as a tracked pre-existing bug — do not let new target docs get added next to a doc section that already disagrees with the code.
8. Keep SEA `Mapping`/`Projection` contract grammar unchanged for these targets unless implementation explicitly needs user-authored overrides. The current grammar target list (`calm`, `kg`, `sbvr`, `protobuf`, `proto`) is for `Mapping`/`Projection` declarations, not the CLI `ProjectFormat` surface — do not conflate the two when deciding whether grammar changes are "unavoidable."

### Gate

```bash
bash scripts/verify/domainforge-baseline.sh
bash scripts/verify/projection-targets/all.sh
grep -R "PIN IN TASK 1" .agents/plans/domainforge_event_authority_verification_activation_projections_plan.md && exit 1 || exit 0
cargo run -p domainforge-core --features cli -- validate fixtures/projection_cell/basic/model.sea
```

**Done when:** Baseline gates exit 0, the fixture validates, this plan has no remaining `PIN IN TASK 1`, the `docs/reference/cli-commands.md` drift from step 7 is resolved or explicitly tracked, and deliberately corrupting `fixtures/projection_cell/basic/model.sea` makes validation fail.

**Redesign trigger:** If adding these targets makes `domainforge-core/src/cli/project.rs` materially harder to maintain, first extract a small internal CLI projection dispatch table. Do not replace `ProjectionRegistry`; it is a graph contract lookup for SEA mappings/projections, unrelated to the CLI target enum.

---

## Task 2 — Add CloudEvents projection  (event interoperability · P0)

**Goal:** DomainForge can project `.sea` event/action/flow semantics into deterministic CloudEvents-compatible event definitions, schemas, constants, and examples, with a validator proving required envelope fields and SEA traceability.

**Why this shape:** CloudEvents should be the standard event envelope, not the source of event meaning. Meaning stays in `.sea`; CloudEvents makes occurrences portable.

### Steps

1. Add a projection-specific `EventEnvelopeIR` as a new module under `domainforge-core/src/projection/cloudevents/` (this establishes the first per-family submodule — there is no prior one to mirror; keep it self-contained: IR types + renderer in the same module, following the general shape of `domainforge-core/src/projection/protobuf.rs`'s IR-to-output flow, adapted rather than copied since protobuf's file is a single large module, not a family-with-submodules pattern). Required fields:
   - `semantic_event_id`
   - `event_type`
   - `source`
   - `subject`
   - `data_schema_ref`
   - `data_content_type`
   - `correlation_id`
   - `causation_id`
   - `sea_model_hash`
   - `sea_element_id`
   - `policy_ref`
   - `evidence_ref`
2. Add a `cloudevents` target to `ProjectFormat`, the CLI dispatcher in `domainforge-core/src/cli/project.rs`, and `domainforge-core/src/projection/mod.rs`'s module registration. Add SEA `target_format` grammar support only if user-authored `Mapping`/`Projection` overrides are required (the grammar's `target_format` list governs `Mapping`/`Projection` declarations, not this CLI enum — adding a CLI variant does not require a grammar change).
3. Render deterministic outputs under:
   - `projections/events/cloudevents/event-types.yaml`
   - `projections/events/cloudevents/schemas/*.schema.json`
   - `projections/events/cloudevents/examples/*.json`
   - `projections/events/cloudevents/constants.rs`
4. Do not add Python/TypeScript/WASM binding surfaces for this target unless a concrete downstream consumer needs in-memory access — none of the existing families expose one today.
5. Add a target-aware validator script at `scripts/verify/projection-targets/cloudevents.sh` that:
   - regenerates CloudEvents output from `fixtures/projection_cell/basic/model.sea`
   - validates every example has `id`, `source`, `specversion`, `type`, and `data` or `dataref`
   - validates extensions `sea_model_hash`, `sea_element_id`, and `evidence_ref`
   - verifies deterministic regeneration by generating twice with a fixed `--created-at` and comparing output trees
6. Add a Rust integration test at `domainforge-core/tests/cloudevents_projection_tests.rs`, matching the flat naming convention of existing files like `protobuf_projection_tests.rs`.
7. Add `docs/cloudevents-projections.md` (new file — there is no `docs/projections/` directory to nest it under) and update `docs/reference/cli-commands.md` and `docs/projection-target-implementation-status.md` with status only after the gate passes.
8. Append `bash scripts/verify/projection-targets/cloudevents.sh` to `scripts/verify/projection-targets/all.sh`.

### Gate

```bash
bash scripts/verify/projection-targets/cloudevents.sh
bash scripts/verify/projection-targets/all.sh
```

**Done when:** CloudEvents projection regenerates byte-stable outputs, every event example validates required CloudEvents envelope fields plus SEA traceability extensions, and removing `sea_model_hash` from a generated example in the validator test fixture makes the CloudEvents gate fail.

**Redesign trigger:** If existing SEA cannot identify event-producing flows/actions without ambiguity, do not change grammar immediately. First use a projection-side convention over existing flow/relation annotations and document the narrower projection scope.

---

## Task 3 — Add AsyncAPI projection using CloudEvents messages  (event contracts · P0)

**Goal:** DomainForge can project producer/consumer/channel/message contracts into deterministic AsyncAPI documents that reference CloudEvents message envelopes and payload schemas.

**Why this shape:** CloudEvents answers "what is the portable event envelope?" AsyncAPI answers "who sends or receives which messages over which channels/protocols?"

### Steps

1. Add `AsyncApiIR` under `domainforge-core/src/projection/asyncapi/`:
   - `application`
   - `role: sender | receiver | both`
   - `server`
   - `protocol`
   - `channel`
   - `operation`
   - `action: send | receive`
   - `message`
   - `payload_schema_ref`
   - `cloud_event_type_ref`
   - `correlation_id`
   - `bindings`
2. Add an `asyncapi` target to `ProjectFormat`, the CLI dispatcher, and `projection/mod.rs`.
3. Render deterministic outputs under:
   - `projections/contracts/asyncapi/domain.asyncapi.yaml`
   - `projections/contracts/asyncapi/messages/*.schema.json`
   - `projections/contracts/asyncapi/examples/*.json`
4. Reuse the CloudEvents schemas/examples from Task 2 rather than duplicating event envelope rendering.
5. Add `scripts/verify/projection-targets/asyncapi.sh` that:
   - regenerates AsyncAPI output
   - validates the YAML structurally
   - verifies every message references either a generated CloudEvents schema or a CloudEvents type
   - verifies at least one sender and one receiver contract in the fixture
   - verifies deterministic output by generating twice with a fixed `--created-at` and comparing output trees
6. Add docs at `docs/asyncapi-projections.md` after the gate passes.
7. Append the AsyncAPI script to `scripts/verify/projection-targets/all.sh`.

### Gate

```bash
bash scripts/verify/projection-targets/cloudevents.sh
bash scripts/verify/projection-targets/asyncapi.sh
bash scripts/verify/projection-targets/all.sh
```

**Done when:** AsyncAPI output validates, references CloudEvents event semantics instead of duplicating them, and changing a generated message reference to a non-existent CloudEvents type makes the AsyncAPI gate fail.

**Redesign trigger:** If no reliable channel/protocol semantics are available from `.sea`, project a minimal channel-neutral AsyncAPI contract first and file a model gap for richer protocol/broker mappings.

---

## Task 4 — Add Devbox config projection  (reproducible environment · P0)

**Goal:** DomainForge can project the tool environment required to validate and activate a projected organization cell into a deterministic `devbox.json` plus scripts.

**Why this shape:** Devbox should provide tool availability. It should not orchestrate work. Dagger owns ordered execution.

### Steps

1. Add `DevEnvironmentIR` under `domainforge-core/src/projection/devbox/`:
   - `packages`
   - `package_versions`
   - `scripts`
   - `services`
   - `environment_variables`
   - `shell_hooks`
   - `platform_constraints`
   - `generated_tool_commands`
2. Add a `devbox` target to `ProjectFormat`, the CLI dispatcher, and `projection/mod.rs`.
3. Render:
   - `projections/runtime/devbox/devbox.json`
   - `projections/runtime/devbox/README.md`
4. Include the minimum tools needed by this plan, using exact package names available in the repo's chosen Devbox/Nixpkgs conventions:
   - Rust toolchain or repo-required build tools
   - DomainForge CLI build requirements
   - Dagger CLI if available
   - Node or Python only where required by validators
   - AsyncAPI validator/tooling if available
   - Gauge, Cedar, Alloy, and TLA+ tooling only if available in a reliable package source; otherwise include scripts that explain the missing optional validator and keep the target's validation deterministic.
5. Add `scripts/verify/projection-targets/devbox.sh` that:
   - regenerates the Devbox projection
   - checks `devbox.json` is valid JSON
   - verifies required script names exist
   - runs `devbox run validate` or the repo-equivalent validation command if Devbox is installed
   - gracefully skips the actual Devbox shell execution only when Devbox is absent, while still validating generated JSON and scripts.
6. Add docs at `docs/devbox-projections.md` after the gate passes.
7. Append the Devbox script to `scripts/verify/projection-targets/all.sh`.

### Gate

```bash
bash scripts/verify/projection-targets/devbox.sh
bash scripts/verify/projection-targets/all.sh
```

**Done when:** `devbox.json` is deterministic, valid JSON, declares scripts for `validate`, `project`, and `verify-settlement`, and deleting the `validate` script makes the Devbox gate fail.

**Redesign trigger:** If Devbox package availability is unstable for formal tools, keep Devbox focused on core DomainForge/Dagger tools and let Dagger containers provide specialized validators.

---

## Task 5 — Add Dagger projection for boot, validation, replay, and evidence collection  (activation · P0)

**Goal:** DomainForge can project a Dagger module/pipeline that validates projections, boots the projected cell where applicable, runs target checks, and writes evidence artifacts.

**Why this shape:** Dagger is the activation harness. It turns generated projections from files into a replayable machine. Runtime wiring should be captured here, not dumped into `.sea`.

### Steps

1. Add `DeliveryPipelineIR` under `domainforge-core/src/projection/dagger/`:
   - `inputs`
   - `generated_artifacts`
   - `validation_steps`
   - `boot_steps`
   - `services`
   - `secrets_needed`
   - `evidence_outputs`
   - `cache_keys`
   - `settlement_checks`
2. Add a `dagger` target to `ProjectFormat`, the CLI dispatcher, and `projection/mod.rs`.
3. Render under:
   - `projections/runtime/dagger/dagger.json`
   - `projections/runtime/dagger/src/main.py` or repo-preferred Dagger SDK language
   - `projections/runtime/dagger/README.md`
4. Implement Dagger functions with stable names:
   - `project`
   - `validate`
   - `validate_cloudevents`
   - `validate_asyncapi`
   - `validate_devbox`
   - `validate_cedar`
   - `validate_gauge`
   - `validate_alloy`
   - `validate_tla`
   - `collect_evidence`
   - `verify_settlement`
5. Before Cedar/Gauge/Alloy/TLA+ exist, those Dagger functions may return "target not implemented" evidence only if `docs/projection-target-implementation-status.md` still marks them as pending. After each target lands, replace the placeholder with a real validator call.
6. Add `scripts/verify/projection-targets/dagger.sh` that:
   - regenerates the Dagger projection
   - validates generated Dagger files are syntactically loadable
   - runs `dagger call validate` when Dagger is installed
   - writes evidence output under a temp directory and verifies files exist
   - verifies deterministic regeneration by generating twice with a fixed `--created-at` and comparing output trees
7. Add docs at `docs/dagger-projections.md` after the gate passes.
8. Append the Dagger script to `scripts/verify/projection-targets/all.sh`.

### Gate

```bash
bash scripts/verify/projection-targets/devbox.sh
bash scripts/verify/projection-targets/dagger.sh
bash scripts/verify/projection-targets/all.sh
```

**Done when:** A clean checkout can regenerate the Dagger projection and run the target validation entrypoint; evidence files are produced; removing the `verify_settlement` function makes the Dagger gate fail.

**Redesign trigger:** If Dagger SDK generation is too heavy for first implementation, render a minimal Dagger module that shells out to `scripts/verify/projection-targets/all.sh`, then promote typed Dagger functions in a later task.

---

## Task 6 — Add Cedar projection for authorization decisions  (authority · P1)

**Goal:** DomainForge can project role/resource/action/context semantics into Cedar schema, policies, entity examples, request examples, and expected authorization decisions.

**Why this shape:** Cedar should handle principal-action-resource-context authorization. SEA-Forge or the policy gateway still owns consequential action, evidence, and fail-closed execution.

### Steps

1. Add `AuthorizationIR` under `domainforge-core/src/projection/cedar/`:
   - `namespace`
   - `principal_types`
   - `resource_types`
   - `actions`
   - `action_groups`
   - `context_shape`
   - `role_membership`
   - `permission_rules`
   - `forbids`
   - `policy_templates`
   - `test_requests`
2. Add a `cedar` target to `ProjectFormat`, the CLI dispatcher, and `projection/mod.rs`.
3. Render:
   - `governance/cedar/schema.cedarschema`
   - `governance/cedar/policies.cedar`
   - `governance/cedar/entities.json`
   - `governance/cedar/requests/*.json`
   - `governance/cedar/expected_decisions/*.json`
4. Include allow, deny, missing-context, wrong-principal-type, wrong-resource-type, and forbid-override examples.
5. Add `scripts/verify/projection-targets/cedar.sh` that:
   - regenerates Cedar outputs
   - runs Cedar validation if the CLI/library is available
   - otherwise runs a deterministic structural validator that checks schema/policy/entity/request files exist and expected decision fixtures are complete
   - verifies byte-stable outputs by generating twice with a fixed `--created-at` and comparing output trees
6. Update Dagger's `validate_cedar` function to call the Cedar verification script.
7. Add docs at `docs/cedar-projections.md` after the gate passes.
8. Append the Cedar script to `scripts/verify/projection-targets/all.sh`.

### Gate

```bash
bash scripts/verify/projection-targets/cedar.sh
bash scripts/verify/projection-targets/dagger.sh
bash scripts/verify/projection-targets/all.sh
```

**Done when:** Cedar outputs are deterministic, include schema/policy/entities/requests/expected decisions, and flipping one expected deny to allow makes the Cedar gate fail.

**Redesign trigger:** If `.sea` policy semantics cannot be mapped safely to Cedar, implement a conservative projection that emits schema and test requests only, and file a model gap for policy expression mapping.

---

## Task 7 — Add Gauge projection for executable acceptance behavior  (behavioral settlement · P1)

**Goal:** DomainForge can project business-readable acceptance specifications and step stubs from domain flows, policies, metrics, and scenarios.

**Why this shape:** Gauge is not a new semantic operator. It is one backend for behavioral verification.

### Steps

1. Add `AcceptanceSpecIR` under `domainforge-core/src/projection/gauge/`:
   - `specification`
   - `scenario`
   - `context`
   - `preconditions`
   - `steps`
   - `parameters`
   - `expected_outcomes`
   - `tags`
   - `linked_policy`
   - `linked_flow`
   - `linked_metric`
2. Add a `gauge` target to `ProjectFormat`, the CLI dispatcher, and `projection/mod.rs`.
3. Render:
   - `projections/tests/gauge/specs/*.spec`
   - `projections/tests/gauge/concepts/*.cpt`
   - `projections/tests/gauge/step_stubs/`
   - `projections/tests/gauge/env/default/default.properties`
4. Generate step stubs in Python (`tests/` in this repo is the existing Python test convention; `python3 -m pytest -q` is the local run path per `justfile`'s `python-test` recipe), since that gives the best fit with existing tooling.
5. Add `scripts/verify/projection-targets/gauge.sh` that:
   - regenerates Gauge outputs
   - validates required spec/scenario/step-stub files exist
   - runs `gauge validate` and `gauge run specs` if Gauge is installed
   - otherwise runs a deterministic structural validator
   - verifies byte-stable outputs by generating twice with a fixed `--created-at` and comparing output trees
6. Update Dagger's `validate_gauge` function.
7. Add docs at `docs/gauge-projections.md` after the gate passes.
8. Append the Gauge script to `scripts/verify/projection-targets/all.sh`.

### Gate

```bash
bash scripts/verify/projection-targets/gauge.sh
bash scripts/verify/projection-targets/dagger.sh
bash scripts/verify/projection-targets/all.sh
```

**Done when:** Gauge outputs are deterministic, every scenario has at least one executable or stubbed step, and deleting a generated step stub makes the Gauge gate fail.

**Redesign trigger:** If existing `.sea` cannot express scenario order, generate acceptance specs from policies/metrics only and mark scenario projection as pending richer scenario semantics.

---

## Task 8 — Add Alloy projection for relational counterexample checks  (structural verification · P1)

**Goal:** DomainForge can project entity/resource/role/policy/evidence relationships into Alloy signatures, facts, predicates, assertions, and bounded checks.

**Why this shape:** Alloy is the cheap formal method target for relational contradictions. Use it before TLA+ when the question is structure, not temporal execution.

### Steps

1. Add `RelationalModelIR` under `domainforge-core/src/projection/alloy/`:
   - `signatures`
   - `relations`
   - `multiplicities`
   - `facts`
   - `predicates`
   - `assertions`
   - `scopes`
   - `example_instances`
2. Add an `alloy` target to `ProjectFormat`, the CLI dispatcher, and `projection/mod.rs`.
3. Render:
   - `projections/verification/alloy/domain.als`
   - `projections/verification/alloy/checks.als`
   - `projections/verification/alloy/examples/`
4. Include checks for:
   - every resource has an owner if the model declares ownership
   - an actor cannot be both requester and approver where separation of duty is declared
   - evidence cannot exist without a case/action reference
   - a case cannot close while required evidence is missing
5. Add `scripts/verify/projection-targets/alloy.sh` that:
   - regenerates Alloy outputs
   - validates expected `.als` files exist
   - runs an Alloy headless check if available
   - otherwise runs structural checks for required signatures/assertions/scopes
   - verifies byte-stable outputs by generating twice with a fixed `--created-at` and comparing output trees
6. Update Dagger's `validate_alloy` function.
7. Add docs at `docs/alloy-projections.md` after the gate passes.
8. Append the Alloy script to `scripts/verify/projection-targets/all.sh`.

### Gate

```bash
bash scripts/verify/projection-targets/alloy.sh
bash scripts/verify/projection-targets/dagger.sh
bash scripts/verify/projection-targets/all.sh
```

**Done when:** Alloy outputs are deterministic, include at least one "find instance" predicate and one "check assertion," and deleting an assertion makes the Alloy gate fail.

**Redesign trigger:** If the generated Alloy model becomes too broad or slow, split into small projection modules: `ownership.als`, `authority.als`, `evidence.als`, and `case_lifecycle.als`.

---

## Task 9 — Add TLA+ projection for temporal safety/liveness checks  (temporal verification · P2)

**Goal:** DomainForge can project bounded temporal specifications for critical governed behavior, especially action authorization, side effects, evidence writes, and settlement ordering.

**Why this shape:** TLA+ is valuable only where the risk is temporal/concurrent behavior. Do not use it for ordinary static constraints that Alloy, SHACL, Lean, or policy tests cover more cheaply.

### Steps

1. Add `TemporalSpecIR` under `domainforge-core/src/projection/tla/`:
   - `variables`
   - `initial_state`
   - `actions`
   - `next_relation`
   - `invariants`
   - `temporal_properties`
   - `fairness_assumptions`
   - `allowed_stuttering`
   - `model_parameters`
2. Add a `tla` target to `ProjectFormat`, the CLI dispatcher, and `projection/mod.rs`. Use `tla` internally and document the display name as `TLA+`.
3. Render:
   - `projections/verification/tla/Domain.tla`
   - `projections/verification/tla/Domain.cfg`
   - `projections/verification/tla/README.md`
4. Start with one bounded property:
   - an approved action eventually writes evidence
   - settlement cannot occur before validation/evidence exists
   - an unapproved action never performs the side effect
5. Add `scripts/verify/projection-targets/tla.sh` that:
   - regenerates TLA+ outputs
   - validates expected `.tla` and `.cfg` files exist
   - runs TLC if available
   - otherwise checks required modules, variables, `Init`, `Next`, and invariants exist
   - verifies byte-stable outputs by generating twice with a fixed `--created-at` and comparing output trees
6. Update Dagger's `validate_tla` function.
7. Add docs at `docs/tla-projections.md` after the gate passes.
8. Append the TLA script to `scripts/verify/projection-targets/all.sh`.

### Gate

```bash
bash scripts/verify/projection-targets/tla.sh
bash scripts/verify/projection-targets/dagger.sh
bash scripts/verify/projection-targets/all.sh
```

**Done when:** TLA+ outputs are deterministic, include `Init`, `Next`, at least one invariant, and at least one eventuality/safety property; deleting the evidence-write property makes the TLA gate fail.

**Redesign trigger:** If generated TLA+ is too speculative from current semantics, reduce scope to one handoff protocol: `AuthorizedAction -> SideEffect -> EvidenceWritten -> Settlement`.

---

## Task 10 — Integrate all eight targets into one projected organization cell  (settlement · P0)

**Goal:** A single `.sea` fixture can regenerate all eight targets, validate them through Dagger/Devbox-aware scripts, collect evidence, and prove the projected cell can be replayed from committed representations.

**Why this shape:** The strategic win is not eight files. The win is a round-trip projection settlement: canonical meaning generates operational surfaces, validators prove them, Dagger replays them, evidence records the result.

### Steps

1. Update `fixtures/projection_cell/basic/model.sea` only if Tasks 2–9 prove missing semantic intent. Use existing SEA primitives first; do not change grammar.
2. Ensure every target output includes:
   - source `.sea` path
   - SEA model hash
   - target name
   - projection version
   - generated timestamp only if normalized or excluded from deterministic comparisons
   - semantic element IDs where applicable
3. Add `scripts/verify/projection-targets/roundtrip-cell.sh` that:
   - deletes a temp projection output directory
   - regenerates all eight targets from the fixture
   - runs every target-specific validation script
   - runs Dagger `verify_settlement` when available
   - writes evidence to a temp ledger path
   - reruns generation and proves deterministic equivalence
4. Update docs:
   - `docs/projection-target-implementation-status.md`
   - `docs/roundtrip-projection-settlement.md`
5. Add a final fixture/evidence example only if it is a small, reviewed example; otherwise rely on generated temp outputs plus deterministic tree comparisons.
6. Append `bash scripts/verify/projection-targets/roundtrip-cell.sh` to `scripts/verify/projection-targets/all.sh`.

### Gate

```bash
bash scripts/verify/projection-targets/roundtrip-cell.sh
bash scripts/verify/projection-targets/all.sh
```

**Done when:** A clean run can regenerate all eight projections from `fixtures/projection_cell/basic/model.sea`, validate them, collect evidence, rerun generation, prove deterministic equivalence, and deliberately breaking one target output makes the roundtrip gate fail.

**Redesign trigger:** If all eight targets make the first fixture too large, split into two fixtures:

- `fixtures/projection_cell/event_authority/model.sea` for CloudEvents, AsyncAPI, Cedar, Gauge
- `fixtures/projection_cell/verification_activation/model.sea` for Alloy, TLA+, Devbox, Dagger

---

## Final acceptance checklist (whole plan)

- [ ] Task 1 pins exact file:line architecture facts (all independently re-verified, none fabricated), creates the projection-cell fixture, reconciles `docs/reference/cli-commands.md` drift, and baseline gates fail when the fixture is corrupted. *(Task 1)*
- [ ] CloudEvents projection emits deterministic envelope definitions, schemas, constants, and examples with SEA traceability; removing `sea_model_hash` fails the gate. *(Task 2)*
- [ ] AsyncAPI projection emits deterministic event contracts that reference CloudEvents messages; breaking a message reference fails the gate. *(Task 3)*
- [ ] Devbox projection emits valid deterministic `devbox.json` with required validation scripts; deleting `validate` fails the gate. *(Task 4)*
- [ ] Dagger projection emits a runnable validation/evidence harness; removing `verify_settlement` fails the gate. *(Task 5)*
- [ ] Cedar projection emits schema, policies, entities, requests, and expected decisions; flipping an expected deny to allow fails the gate. *(Task 6)*
- [ ] Gauge projection emits specs, concepts, and step stubs; deleting a step stub fails the gate. *(Task 7)*
- [ ] Alloy projection emits signatures/facts/predicates/assertions; deleting an assertion fails the gate. *(Task 8)*
- [ ] TLA+ projection emits `Init`, `Next`, invariants, and temporal properties; deleting the evidence-write property fails the gate. *(Task 9)*
- [ ] Roundtrip cell regenerates all eight targets, validates them, collects evidence, reruns generation, and proves deterministic equivalence. *(Task 10)*
- [ ] `bash scripts/verify/domainforge-baseline.sh` exits 0.
- [ ] `bash scripts/verify/projection-targets/all.sh` exits 0.
- [ ] `cargo fmt --all --check` exits 0.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` exits 0.
- [ ] `just ci-test-rust` (`cargo test --verbose --workspace --features cli`) exits 0.
- [ ] `docs/projection-target-implementation-status.md` reflects reality — no status claim ahead of a passing gate.
- [ ] Any committed generated examples are regenerated deliberately and reviewed in the same commit as the code that justifies them, per `docs/reference/generated-artifacts-policy.md`.

## Guardrails (do not violate)

- Do **not** add new SEA grammar for these targets unless Task 1 proves that existing primitives cannot express the required semantic distinction. Remember: the grammar's `target_format` list governs `Mapping`/`Projection` declarations only, not the CLI `ProjectFormat` enum — a new CLI target does not by itself require a grammar change.
- Do **not** generate target files directly from raw parser nodes when the repo has or can support a semantic model layer. Use `semantic model -> projection-specific IR -> renderer -> validator -> deterministic test`.
- Do **not** use AsyncAPI as the event envelope. Use CloudEvents for envelope and AsyncAPI for event-driven API contracts.
- Do **not** use Cedar as a replacement for SEA-Forge authority/evidence. Cedar is an authorization decision projection only.
- Do **not** use TLA+ for static relationship constraints. Use Alloy for relational structure and TLA+ for temporal/concurrent behavior.
- Do **not** let Devbox orchestrate workflow. Devbox makes tools available; Dagger runs the workflow.
- Do **not** hand-edit generated artifacts to make tests pass. Fix the source, projection IR, renderer, validator, or deterministic fixture/test intentionally.
- Do **not** mark a projection "implemented" in docs until its target-specific gate and `all.sh` both pass.
- Do **not** cite a file, module, or convention as "existing precedent" without having verified it exists in the current working tree. Earlier drafts of this plan cited a nonexistent `otel` projection module, a nonexistent `tests/common/` harness, a nonexistent `docs/projection-families.md`, and a nonexistent `fixtures/<family>/basic/domain/model.sea` convention — treat any similarly unverified claim in this document as a bug to fix, not a pattern to follow.
- Do **not** add Python/TypeScript/WASM bindings for a new target by default. None of the existing projection families are bound today; add bindings only when a task explicitly justifies the need.
- Keep unrelated cleanup in separate commits. A projection target commit should include its IR, renderer, validator, fixture, docs, and gate together.
