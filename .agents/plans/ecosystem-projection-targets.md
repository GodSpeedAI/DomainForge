# Implementation Plan — Ecosystem-Unlocking Projection Targets

**Created:** 2026-07-06

**Source of truth:** This plan executes the "ecosystem-unlocking projection targets" directive (user brief, 2026-07-06). Strategic goal, quoted: *"Unlock mature external ecosystems through DomainForge's canonical semantic model while preserving deterministic identity, semantic traceability, and projection consistency. An intuitive cli experience."*

**Originating context:** Planning request on `main` with the Lean and AI-learning projections freshly landed (untracked in git at plan time). Those two projections ARE the reference architecture for everything below.

**Status of the work today:**

- **Done (do not re-plan):** Lean 4 projection (`--format lean`), AI-learning projections (`--format ai-learning` / `ai-llm` / `graph-ml` / `cep-eval`), Protobuf (`--format proto`). The brief's "Lean" row is already shipped; the "Learning loop" operator is partially shipped (datasets exist; ZenML pipeline emission does not).
- **Partial:** RDF/OWL — `kg.rs` already has `to_turtle` (kg.rs:428), `to_rdf_xml` (kg.rs:950), `validate_shacl` (kg.rs:790), `export_rdf` (kg.rs:1345), and a `validate-kg` CLI command. What's missing is exposure as a first-class `project --format rdf` target with deterministic IRI policy, JSON-LD, and OWL axioms.
- **Missing:** CMMN, BPMN, ArchiMate, OTel SemConv, ZenML, BAML, DSPy — and a small shared kernel so these are a *family*, not nine one-off exporters.

---

## 0. How to use this plan (agent operating instructions)

- Execute tasks in the order given. Dependencies: Task 1 (kernel) blocks everything. Task 2 (RDF) is independent after 1. Tasks 3→4→5 (BPMN→CMMN→ArchiMate) are ordered because they share XML/XSD infrastructure built in Task 3. Tasks 6 (OTel) is independent. Tasks 7→8→9 (BAML→DSPy→ZenML) are ordered because they share the AI-recipe infrastructure. Task 10 (CLI UX + docs) is last and depends on whichever targets have landed.
- **Every task ends with a verification gate.** Do not mark a task done until its gate command exits 0.
- **CORE PRINCIPLE: One projection = one IR module + one renderer + one `emit(graph, …, &mut ArtifactSink)` + one `project_<x>_in_memory` binding surface + one proving fixture + one external-toolchain CI verify job. Never render directly from the AST inside the CLI; never duplicate output logic in bindings or tests.** This is the established pattern — copy `domainforge-core/src/projection/lean/mod.rs`, not `protobuf.rs` (protobuf predates the pattern).
- Match surrounding code style; the idioms to mirror live in `domainforge-core/src/projection/lean/` (structure, determinism, docs comments) and `domainforge-core/src/projection/ai_learning/` (recipe, provenance, sink).
- Fixtures are the spec: `fixtures/<family>/basic/domain/model.sea` plus expected-output assertions in `domainforge-core/tests/<family>_projection_tests.rs`. Change expected content deliberately, in the same commit as the code that justifies it.
- Byte determinism is non-negotiable: fixed `--created-at` in, identical bytes out. Every new projection copies the `output_is_byte_deterministic` test from `domainforge-core/tests/lean_projection_tests.rs`.

### Global verification gates (must stay green after EVERY task)

```bash
just ci-test-rust        # justfile:163 — cargo workspace tests (cli feature)
just ci-test-python      # justfile:168
just ci-test-ts          # justfile:173
cargo clippy --workspace --features cli -- -D warnings
```

Rebuild steps if you touched bindings (`domainforge-core/src/{python,typescript,wasm}/graph.rs`) before running the language gates:

```bash
just ci-test-python      # rebuilds the pyo3 module as part of the recipe
just ci-test-ts          # rebuilds the napi binding as part of the recipe
```

---

## Key facts already discovered (do not re-derive)

| Thing | Location |
| --- | --- |
| Reference projection: `emit(graph, model_ref, created_at, sink) -> Vec<String>` + `project_lean_in_memory` | `domainforge-core/src/projection/lean/mod.rs:26` and `:64` |
| `ArtifactSink` (Dir / Memory) — currently misplaced inside ai_learning | `domainforge-core/src/projection/ai_learning/mod.rs:371` |
| Recipe pattern (JSON recipe, `from_json`, `validate`, seed, split ratios) | `domainforge-core/src/projection/ai_learning/recipe.rs:22,163,170` |
| Provenance struct (model hash, recipe hash, generation metadata) | `domainforge-core/src/projection/ai_learning/provenance.rs:6` |
| Deterministic ID convention: `compute_deterministic_hash` (xxh64 seed 42, 16-hex), record ids joined with `\u{1}` | `domainforge-core/src/projection/ai_learning/split.rs` (`record_id`), `domainforge-core/src/authority/` |
| CLI dispatch: `ProjectFormat` enum + `match args.format` | `domainforge-core/src/cli/project.rs:88,141`; `run_lean` at `:335`, `run_ai_learning` at `:372` |
| DSL-level `Projection`/`Mapping` contracts (scope + target + governance, no format detail) | `domainforge-core/src/projection/registry.rs`, `contracts.rs`, ADR-002 |
| Lean deliberately did NOT use the contract surface ("target is CLI-format-driven") | `docs/lean-projections.md` Non-goals |
| Bindings pattern: `export_lean` / `export_ai_learning` returning JSON map of path→content | `python/graph.rs:364`, `typescript/graph.rs:365`, `wasm/graph.rs:428` |
| Test oracle pattern: assert_cmd + `read_tree` + byte-determinism + fixed `FIXED_TS` | `domainforge-core/tests/lean_projection_tests.rs` |
| Three-language test parity convention | `tests/test_lean.py`, `typescript-tests/lean.test.ts` |
| External-toolchain CI precedent: `verify-lean` job runs real `lake build` | `.github/workflows/ci.yml` |
| Existing RDF machinery (turtle, RDF/XML, SHACL) | `domainforge-core/src/kg.rs:428,790,950,1345` |
| Vendored-schema precedent (put XSDs here, like cep_eval) | `schemas/cep_eval/`, `docs/specs/cep/VENDORED.md` |
| Doc pattern: one `docs/<family>-projections.md` + a section in `docs/reference/cli-commands.md` | `docs/lean-projections.md`, `docs/reference/cli-commands.md` |
| Flow/pattern data (input for BPMN/CMMN process semantics) | `domainforge-core/src/graph/`, gRPC-from-Flow precedent in `protobuf.rs` |

---

## Task 1 — Shared projection kernel (foundation · P0)

**Goal:** The cross-cutting machinery every new family needs lives in one place, and an ADR names the operator-family architecture so reviewers can reject one-off exporters.

**Why this shape:** The brief's core demand is "not a bag of one-off exporters." The repo already converged on a pattern; this task *names and relocates* it — it does not invent a plugin framework. No trait-object registry, no dynamic loading: nine statically-known targets do not justify one (deliberate simplification; revisit only if third-party targets become a requirement).

### Steps

1. Move `ArtifactSink` from `domainforge-core/src/projection/ai_learning/mod.rs:371` to a new `domainforge-core/src/projection/sink.rs`; re-export from `ai_learning` so existing callers (`cli/project.rs:356`, bindings) keep compiling, then migrate call sites.
2. Add `domainforge-core/src/projection/ids.rs`: thin wrappers over `authority::compute_deterministic_hash` for projection-element IDs (`element_id(family: &str, parts: &[&str]) -> String`) and a deterministic IRI/QName sanitizer. Every new family MUST route IDs through this module so identity is uniform.
3. Extract the test helpers (`read_tree`, `FIXED_TS`, byte-determinism assertion) from `domainforge-core/tests/lean_projection_tests.rs` into `domainforge-core/tests/common/projection_harness.rs` (Rust integration-test `mod common` pattern).
4. Write `docs/specs/ADR-011-operator-backed-projection-families.md`: the operator-family table from the brief (Adaptive work/case→CMMN, Ordered process→BPMN, Enterprise architecture→ArchiMate, Semantic graph→RDF/OWL, Runtime observation→OTel, Learning loop→ZenML, Formal verification→Lean, AI representation→BAML, AI optimization→DSPy), the CORE PRINCIPLE above, the ID convention, and the decision that targets stay CLI-format-driven (per the Lean precedent) with ADR-002 `Projection` contracts governing scope/compatibility only.
5. Docs/coherence: link ADR-011 from `docs/specs/README.md`.

### Gate

```bash
cargo test --features cli -p domainforge-core --test lean_projection_tests --test ai_learning_tests && just ci-test-rust
```

**Done when:** All existing projection tests pass with `ArtifactSink` in its new home; ADR-011 exists; deleting `projection/ids.rs` makes the build fail (proves at least the Lean or ai_learning path was migrated onto it — teeth-check).

**Redesign trigger:** If moving `ArtifactSink` ripples into bindings signatures, keep the re-export permanently instead of migrating call sites — the re-export IS the fix.

---

## Task 2 — RDF / OWL projection (`--format rdf`) (semantic graph · P0)

**Goal:** `domainforge project --format rdf model.sea out/` emits a deterministic RDF dataset (Turtle + JSON-LD) with an explicit IRI policy and optional OWL axioms, validated by the existing SHACL path.

**Why this shape:** ~80% exists in `kg.rs`. This task is a *promotion*, not a build: wrap `Kg::from_graph` behind the family pattern, add JSON-LD and an `OntologyIR` layer only where kg.rs lacks one. Do NOT rewrite kg.rs.

### Steps

1. Create `domainforge-core/src/projection/rdf/mod.rs` with `emit(graph, model_ref, created_at, opts, sink)` that calls `Kg::from_graph` (kg.rs:145) and writes `model.ttl` (via `to_turtle`, kg.rs:428), `model.jsonld` (new serializer over the same triple set), and `ontology.owl.ttl` (classes/properties derived from entity/role/resource domains — the OntologyIR is the existing Kg triple structs plus an axiom pass; add structs only for what kg.rs cannot express: class axioms, property domains/ranges).
2. Deterministic IRIs: route all minted IRIs through `projection::ids` (Task 1) with a documented base-IRI policy (`--base-iri` flag, default derived from model namespace). Named-graph + provenance triples reuse `ai_learning/provenance.rs` fields.
3. CLI: add `Rdf` to `ProjectFormat` (`cli/project.rs:88`), `run_rdf` mirroring `run_lean` (`cli/project.rs:335`), directory-output validation identical to Lean's.
4. Bindings: `export_rdf_projection` in `python/graph.rs`, `typescript/graph.rs`, `wasm/graph.rs`, each delegating to `project_rdf_in_memory` (mirror `export_lean` at `python/graph.rs:364`).
5. Fixture + tests: `fixtures/rdf/basic/domain/model.sea`; `domainforge-core/tests/rdf_projection_tests.rs` using the Task-1 harness (file manifest, byte determinism, IRI stability across two runs); `tests/test_rdf.py`; `typescript-tests/rdf.test.ts`.
6. Validation gate wiring: CI job `verify-rdf` — project the fixture, run `domainforge validate-kg out/model.ttl` (needs `--features cli,shacl`) and a JSON-LD parse check.
7. Docs: `docs/rdf-projections.md` + `docs/reference/cli-commands.md` section (mirror the Lean sections).

### Gate

```bash
cargo test --features cli,shacl -p domainforge-core --test rdf_projection_tests
just ci-test-python && just ci-test-ts
```

**Done when:** Fixture projects to identical bytes twice; `validate-kg` passes on the output; renaming one entity in the fixture changes exactly the IRIs derived from it (teeth-check for deterministic identity).

**Redesign trigger:** If JSON-LD needs a triple model kg.rs can't expose cleanly, emit Turtle-only in v1 and record JSON-LD as a follow-up in the doc's Non-goals — do not fork kg.rs.

---

## Task 3 — BPMN projection (`--format bpmn`) (ordered process · P0)

**Goal:** Flows and their patterns project to BPMN 2.0 XML that passes XSD validation and imports into bpmn.io, with a declared (initially non-executable) subset.

**Why this shape:** Flow patterns already drive gRPC service generation in `protobuf.rs`, proving the graph carries process semantics. BPMN is the highest-leverage structural target and builds the XML/XSD infrastructure CMMN and ArchiMate reuse. BPMNDI layout is **out of scope v1** (declare in Non-goals; import tools auto-layout).

### Steps

1. `domainforge-core/src/projection/bpmn/ir.rs`: `ProcessIR` — process, lane (from roles), start/end event, task (from flow steps), exclusive/parallel gateway (from flow branching), sequence flow, data object (from resources). Build ONLY what the graph can populate today; every IR element carries a `deterministic_id` from `projection::ids`.
2. `domainforge-core/src/projection/bpmn/xml.rs`: renderer to BPMN 2.0.2 XML. Reuse `kg.rs::escape_xml` (kg.rs:1223) — do not write a second XML escaper.
3. `domainforge-core/src/projection/bpmn/mod.rs`: `emit` + `project_bpmn_in_memory`, per the pattern.
4. Vendor the BPMN 2.0 XSDs into `schemas/bpmn/` with a `VENDORED.md` (mirror `docs/specs/cep/VENDORED.md`).
5. CLI `Bpmn` format + `run_bpmn`; bindings `export_bpmn`; fixture `fixtures/bpmn/basic/`; tests in all three languages per the pattern.
6. CI `verify-bpmn` job: project fixture, `xmllint --schema schemas/bpmn/BPMN20.xsd out/*.bpmn --noout`.
7. Docs: `docs/bpmn-projections.md` (must state the executable-subset declaration and the "semantics live in the model, layout is the tool's job" stance) + cli-commands.md section.

### Gate

```bash
cargo test --features cli -p domainforge-core --test bpmn_projection_tests
xmllint --schema schemas/bpmn/BPMN20.xsd /tmp/bpmn-out/*.bpmn --noout
```

**Done when:** Fixture output is byte-deterministic and XSD-valid; deleting a sequence-flow element from the renderer makes the XSD gate fail (teeth-check).

**Redesign trigger:** If the graph's flow patterns cannot express gateways at all, ship straight-line processes v1 and add a `redesign:` note in the doc — do not invent new DSL syntax inside this task (that is a separate spec-level decision).

---

## Task 4 — CMMN projection (`--format cmmn`) (adaptive work / case · P1)

**Goal:** Policies + roles + resources project to CMMN 1.1 XML (case, case-file items from resources, human tasks from role-gated actions, milestones, entry criteria from policy conditions) that passes the CMMN 1.1 XSD.

**Why this shape:** Reuses Task 3's XML renderer utilities and XSD-gate CI shape wholesale. The interesting mapping is policy→sentry/entry-criterion; the authority-policy structures (`AuthorityPolicy`, modality, priority — see `domainforge-core/tests/gen_ai_learning_fixture.rs`) are the input, mirroring how ai_learning consumes them.

### Steps

1. `domainforge-core/src/projection/cmmn/ir.rs`: `CaseIR` (case, case file item, stage, human/discretionary task, milestone, sentry with entry/exit criterion, role). Populate from graph roles/resources and authority policies; skip event listeners v1 (no runtime-event source in the model — declared Non-goal).
2. `cmmn/xml.rs` renderer + `cmmn/mod.rs` `emit`/`project_cmmn_in_memory`.
3. Vendor CMMN 1.1 XSDs into `schemas/cmmn/` + VENDORED.md.
4. CLI `Cmmn` + `run_cmmn`; bindings `export_cmmn`; `fixtures/cmmn/basic/`; three-language tests; CI `verify-cmmn` (xmllint).
5. Docs: `docs/cmmn-projections.md` + cli-commands.md.

### Gate

```bash
cargo test --features cli -p domainforge-core --test cmmn_projection_tests
xmllint --schema schemas/cmmn/CMMN11.xsd /tmp/cmmn-out/*.cmmn --noout
```

**Done when:** XSD-valid, byte-deterministic output; a policy with a condition in the fixture demonstrably produces a sentry (asserted by test), and removing that policy removes the sentry (teeth-check).

**Redesign trigger:** If policy conditions can't lower to sentries meaningfully, emit milestone-only cases v1; the operator family still exists, thinner.

---

## Task 5 — ArchiMate projection (`--format archimate`) (enterprise architecture · P1)

**Goal:** The model projects to an ArchiMate Model Exchange File (XML) with business-layer elements (roles→business roles, entities/resources→business objects, flows→business processes/services, policies→requirements/drivers) and only spec-legal relations.

**Why this shape:** Third consumer of the Task-3 XML infra. The validation gate is the relation matrix: the renderer must consult a static allowed-relations table so it *cannot* emit an illegal pair — validation by construction beats post-hoc checking here.

### Steps

1. `domainforge-core/src/projection/archimate/ir.rs`: `ArchitectureIR` (element with layer+type, relation with type, view listing element refs). Include a `const ALLOWED_RELATIONS: &[(ElemKind, RelKind, ElemKind)]` table; the IR builder returns `Err` on any tuple not in it.
2. `archimate/xml.rs`: Model Exchange File renderer; one auto-generated view per stakeholder-ish grouping (start with: one business-layer view; per-stakeholder views are v2 — Non-goal note).
3. Vendor the Model Exchange File XSD into `schemas/archimate/`.
4. CLI `Archimate` + `run_archimate`; bindings `export_archimate`; `fixtures/archimate/basic/`; three-language tests; CI `verify-archimate` (xmllint).
5. Docs: `docs/archimate-projections.md` + cli-commands.md.

### Gate

```bash
cargo test --features cli -p domainforge-core --test archimate_projection_tests
xmllint --schema schemas/archimate/archimate3_Model.xsd /tmp/archimate-out/*.xml --noout
```

**Done when:** XSD-valid, deterministic output; a unit test feeding a forbidden relation tuple into the IR builder gets `Err` (teeth-check on the matrix).

**Redesign trigger:** None plausible — the exchange format is stable and the mapping is additive.

---

## Task 6 — OpenTelemetry SemConv projection (`--format otel-semconv`) (runtime observation · P0, independent)

**Goal:** Domain events/flows project to an OTel semantic-convention registry (YAML) plus generated attribute-constant files (Rust/Python/TS), with every attribute namespaced under the model's namespace and correlated to deterministic domain IDs.

**Why this shape:** This is the traceability keystone — it is what lets *runtime* telemetry point back at *model* identity, serving the strategic goal directly. Output is YAML+constants, so it needs none of the XML infra; it can proceed in parallel with Tasks 3–5.

### Steps

1. `domainforge-core/src/projection/otel/ir.rs`: `TelemetryIR` — attribute groups per entity/flow/policy-decision, span-kind suggestions per flow step, resource attributes carrying `domainforge.model.hash` and `domainforge.element.id` (values from `projection::ids`). Reserved-namespace guard: refuse to emit any attribute not prefixed with the model namespace (never `otel.*`, `service.*`, etc. — hard error).
2. `otel/yaml.rs`: SemConv registry YAML renderer (serde_yaml is likely already in-tree via recipes; verify before adding a dep).
3. `otel/constants.rs`: render `attributes.rs` / `attributes.py` / `attributes.ts` constant files from the same IR (one IR, three trivial renderers — no per-language logic beyond casing).
4. CLI `OtelSemconv` + runner; bindings `export_otel_semconv`; `fixtures/otel/basic/`; three-language tests asserting the constants files match the YAML (single-producer check).
5. CI `verify-otel`: project fixture, run `weaver registry check -r out/registry/` (OTel's official checker) — mirror the `verify-lean` job shape; if weaver install is too heavy for CI, gate on a JSON-schema validation of the YAML instead and note it.
6. Docs: `docs/otel-projections.md` (include the correlation story: span attr → deterministic element ID → model hash) + cli-commands.md.

### Gate

```bash
cargo test --features cli -p domainforge-core --test otel_projection_tests
```

**Done when:** Registry validates; a test proves an attribute name collision with a reserved OTel namespace is rejected (teeth-check); constants files and YAML are asserted to agree.

**Redesign trigger:** If SemConv registry schema churn (it has been unstable) breaks `weaver`, pin the weaver version in CI and record the pinned schema rev in the doc.

---

## Task 7 — BAML projection (`--format baml`) (AI representation · P1)

**Goal:** Entities/policies project to `.baml` files: classes/enums from entity domains, typed functions for policy-decision capabilities, prompts embedding the policy semantics, and BAML tests seeded from authority cases.

**Why this shape:** The AI family (Tasks 7–9) consumes `AiLearningContext` (`ai_learning/mod.rs:58`) — the authority cases and policy info already computed for datasets are exactly the examples/validators BAML needs. Build on it; do not re-derive cases.

### Steps

1. `domainforge-core/src/projection/baml/ir.rs`: `AICapabilityIR` — type defs (from entity/resource domains), function (input schema = authority request shape, output schema = decision + rationale), examples (from `AuthorityCase`), client block left as a documented placeholder the user fills (no vendor lock in generated code).
2. `baml/render.rs` + `baml/mod.rs` per the pattern; recipe support via a `baml` section added to the ai-recipe (`ai_learning/recipe.rs:38` `Projections` struct) rather than a new recipe format — one recipe system, not two.
3. CLI `Baml` + runner (accepts `--recipe` like other ai-* formats, `cli/project.rs:59`); bindings `export_baml`; `fixtures/baml/basic/` (reuse the manufacturing_quality fixture's authority env); three-language tests.
4. CI `verify-baml`: `baml-cli generate` + `baml-cli test --dry-run` over the projected fixture (pin baml-cli version).
5. Docs: `docs/baml-projections.md` + cli-commands.md.

### Gate

```bash
cargo test --features cli -p domainforge-core --test baml_projection_tests
```

**Done when:** Generated `.baml` passes `baml-cli generate` in CI; a fixture authority case appears verbatim as a BAML test (asserted); byte-deterministic.

**Redesign trigger:** If BAML's syntax churns faster than we can pin, emit against a pinned baml-cli version and state the supported version in the doc header — same policy as `LEAN_TOOLCHAIN` (`lean/mod.rs:23`).

---

## Task 8 — DSPy projection (`--format dspy`) (AI optimization · P2)

**Goal:** A runnable Python DSPy program: signatures from policy-decision capabilities, train/dev examples from the ai-learning splits, a metric derived from decision-label agreement, optimizer config with a declared regression threshold.

**Why this shape:** DSPy's train/dev sets ARE the ai-learning dataset with its deterministic `assign_split` (`ai_learning/split.rs`) — reuse the split verbatim so DSPy examples and LLM datasets never diverge. The projection emits *program + config*, it does not run optimization (that's the user's compute).

### Steps

1. `domainforge-core/src/projection/dspy/ir.rs`: `AIOptimizationIR` — signature per capability, module strategy (Predict v1; CoT as recipe option), example refs (pointers into the emitted dataset files, not copies), metric spec, optimizer block, artifact-version stamp from provenance.
2. `dspy/render.rs`: emits `program.py`, `optimize.py`, `metric.py`, `dspy.config.json`. Generated Python must be import-clean with only `dspy` installed.
3. Recipe: `dspy` section in the shared ai-recipe. CLI `Dspy` + runner; bindings `export_dspy`; fixture reusing `fixtures/ai_learning/manufacturing_quality/`; three-language tests (TS/wasm tests assert the file map only).
4. CI `verify-dspy`: `python -c "import ast; ast.parse(open(f).read())"` over emitted .py files + `pip install dspy && python -m py_compile` if CI budget allows (start with ast.parse; upgrade later).
5. Docs: `docs/dspy-projections.md` (must state: baseline-vs-optimized comparison and threshold enforcement happen in the *emitted* `optimize.py`, not in DomainForge) + cli-commands.md.

### Gate

```bash
cargo test --features cli -p domainforge-core --test dspy_projection_tests && just ci-test-python
```

**Done when:** Emitted Python parses; example refs resolve to files the ai-learning projection actually emits for the same fixture (cross-projection consistency test — the teeth-check).

**Redesign trigger:** If capability→signature mapping is too thin without richer prompt contracts, narrow v1 to authority-decision signatures only (the fixture proves that path end-to-end).

---

## Task 9 — ZenML projection (`--format zenml`) (learning loop · P2)

**Goal:** A ZenML pipeline package: `@step` functions (load dataset → train/eval → register) wired into a `@pipeline` DAG, with artifacts/metrics named by deterministic IDs and model-version grouping keyed to the model hash.

**Why this shape:** Closes the learning loop the ai-learning datasets opened: dataset emission (exists) → pipeline that consumes it (this task) → metrics back into provenance. Same reuse rule as DSPy: dataset paths are references to ai-learning output, single producer.

### Steps

1. `domainforge-core/src/projection/zenml/ir.rs`: `LearningPipelineIR` — dataset ref, step (name, inputs, outputs), DAG edges, metric names, model/capability version (= `source_model_hash`), promotion-gate threshold from recipe.
2. `zenml/render.rs`: emits `pipeline.py`, `steps.py`, `run.py`, `README.md`. Import-clean with only `zenml` installed; `run.py --dry-run` just constructs the pipeline object.
3. Recipe `zenml` section; CLI `Zenml` + runner; bindings `export_zenml`; fixture reuse (manufacturing_quality); three-language tests.
4. CI `verify-zenml`: ast.parse gate first; a pinned `pip install zenml` + pipeline-construction dry-run as a separate non-blocking job initially (zenml's dep tree is heavy — promote to blocking once stable).
5. Docs: `docs/zenml-projections.md` + cli-commands.md.

### Gate

```bash
cargo test --features cli -p domainforge-core --test zenml_projection_tests && just ci-test-python
```

**Done when:** Emitted pipeline parses and its dataset refs match ai-learning output paths for the same fixture (teeth-check); byte-deterministic.

**Redesign trigger:** If ZenML's decorator API shifts (it has), pin the version in the emitted `requirements.txt` and doc header — the projection targets a pinned version, period.

---

## Task 10 — CLI experience + documentation coherence (UX · P1, last)

**Goal:** A user can discover every projection from the CLI alone: `domainforge project --help` groups formats by operator family, errors name the family's doc page, and `docs/index.md` has one projection-families overview linking all family docs.

**Why this shape:** "An intuitive cli experience" is in the strategic goal verbatim. The mechanism already exists (clap derive on `ProjectFormat`); this is doc-comment and help-text work plus one overview doc — not a new CLI subsystem. No `--list-formats` subcommand: `--help` already lists formats; improving its grouping is the whole job (deliberate simplification).

### Steps

1. `domainforge-core/src/cli/project.rs`: give every `ProjectFormat` variant a doc comment of the form `"<family> operator: <one line>"` so `--help` reads as the operator table; ensure每 format-specific flag says which formats it applies to (protobuf flags already do — `cli/project.rs:19-35`; extend to new flags).
2. Error copy: each `run_<x>` failure message ends with `see docs/<family>-projections.md`.
3. `docs/projection-families.md`: the operator-family table from ADR-011, one row per family, linking each family doc + CLI section. Link from `docs/index.md` and README's projection mention.
4. Sweep `docs/reference/cli-commands.md` for consistency across all new sections (same heading shape as the existing `project > Lean-specific behavior` pattern).

### Gate

```bash
cargo run --features cli -- project --help | grep -ci "operator"   # ≥ number of shipped families
just ci-test-rust
```

**Done when:** `project --help` names every shipped family; every family doc is reachable from `docs/projection-families.md`; no doc claims a target that has no passing gate (audit against the acceptance checklist below).

**Redesign trigger:** None plausible.

---

## Final acceptance checklist (whole plan)

- [ ] `ArtifactSink`/ids/test-harness extracted; ADR-011 merged; existing lean + ai_learning tests green on the shared kernel *(Task 1)*
- [ ] `--format rdf` emits deterministic Turtle(+JSON-LD) passing `validate-kg`; IRI-stability teeth-check passes *(Task 2)*
- [ ] `--format bpmn` output XSD-valid, byte-deterministic, imports into bpmn.io (manual once, then XSD gate carries it) *(Task 3)*
- [ ] `--format cmmn` output XSD-valid; policy→sentry mapping proven by test *(Task 4)*
- [ ] `--format archimate` output XSD-valid; illegal-relation tuples rejected at IR build *(Task 5)*
- [ ] `--format otel-semconv` registry passes checker; reserved-namespace guard proven; constants/YAML agreement test *(Task 6)*
- [ ] `--format baml` passes `baml-cli generate`; authority cases appear as BAML tests *(Task 7)*
- [ ] `--format dspy` Python parses; dataset refs cross-check against ai-learning output *(Task 8)*
- [ ] `--format zenml` pipeline parses; dataset refs cross-check *(Task 9)*
- [ ] `project --help` reads as the operator-family table; `docs/projection-families.md` links every shipped family *(Task 10)*
- [ ] `cargo clippy --workspace --features cli -- -D warnings` exits 0.
- [ ] Every shipped family has: fixture dir, Rust/Python/TS tests, byte-determinism test, `verify-<family>` CI job, family doc, cli-commands.md section — audited against the CORE PRINCIPLE.
- [ ] All global gates green; Python/TS bindings rebuilt.

## Guardrails (do not violate)

- **No new DSL syntax in any task.** Every projection consumes the graph as-is. If a target needs semantics the model can't express, ship the thinner projection and file the DSL gap as a separate spec (ADR/PRD) — quoted constraint: projections "help DomainForge create complex unified systems from canonical semantics," they do not mutate the semantics.
- **No plugin/registry framework.** Nine static formats = one enum + one match arm each. Revisit only on a concrete third-party-target requirement.
- **Single producer per artifact.** Bindings and tests call `project_<x>_in_memory`; nothing re-implements rendering. Cross-family references (DSPy/ZenML → ai-learning datasets) are path references, never copies.
- **Determinism is a gate, not a goal.** Fixed `created_at` ⇒ identical bytes, always tested. All IDs through `projection::ids`; no wall-clock, no randomness, no HashMap iteration order in any renderer (BTreeMap, per `lean/mod.rs` doc comment).
- **Pinned external toolchains.** Every `verify-<family>` CI job pins its tool version (the `LEAN_TOOLCHAIN` precedent, `lean/mod.rs:23`); bump = constant change + green verify job, nothing else.
- **Ship order is negotiable between independent tasks; the kernel (Task 1) is not.** Each task lands as its own PR with its family fully green — no half-families on `main`.
- Commit hygiene: kernel extraction (Task 1) is its own PR titled `refactor(projection): extract shared sink/id/test kernel` before any new family lands.
