# BAML Capability Projection

## Overview
Compiles a `.sea` model plus its authority environment into a BAML project — typed enums/classes for the entity/resource/role domains, one typed policy-decision function, and a `test` block per resolver-grounded authority case.

## Interface
- `[mod.rs:emit]` — builds `AICapabilityIR` and writes `baml_src/domain.baml`, `baml_src/functions.baml`, `baml_src/tests.baml`, `baml_src/clients.baml`, `README.md` into an `ArtifactSink`.
- `[mod.rs:project_baml_in_memory]` — binding surface: graph + recipe JSON + authority-config JSON in, `BTreeMap<path, content>` out, no filesystem.
- `[ir.rs:BAML_FAMILY]` — the `element_id` family tag, `baml`.
- `[ir.rs:BAML_TARGET_VERSION]` — the pinned BAML language target, `0.68`.
- `[ir.rs:AICapabilityIR::build]` — builds the capability IR from an `AiLearningContext`.
- `[ir.rs:EnumDef]` / `[ir.rs:Variant]` / `[ir.rs:ClassDef]` / `[ir.rs:FieldDef]` / `[ir.rs:TestCase]` — the capability IR element types.
- `[render.rs:domain]` / `[render.rs:functions]` / `[render.rs:tests]` / `[render.rs:clients]` / `[render.rs:readme]` — the five renderers.

## Invariants
- `[ir.rs:AICapabilityIR::build]` — reuses the same resolver-grounded authority cases and policy information the ai-learning datasets are built from; the tests are the resolver's own decisions, never re-derived.
- `[mod.rs]` (module doc) — no artifact invents domain facts; the LLM client is a documented placeholder so generated code stays vendor-neutral.
- `[ir.rs]` (module doc) — every id is minted through `projection::ids` so identity is uniform across projection families, and all collections are sorted so rendered output is byte-deterministic for a fixed `created_at`.
- `[ir.rs:BAML_TARGET_VERSION]` — bumping requires changing this constant, regenerating the fixture, and confirming the `verify-baml` CI job stays green.
- Exactly the five listed artifacts are emitted.

## Side Effects
- `[mod.rs:emit]` — writes through `ArtifactSink::write` (filesystem write under the output root, or in-memory map insertion).

## Verification
- Run: `cargo test -p domainforge-core --features cli`
