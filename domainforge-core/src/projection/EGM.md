# Projection Kernel & Family Index

## Overview
The projection subsystem: one shared kernel (artifact sink, identity minting, flow resolution, contract lookup, registry, YAML emission, Protobuf engine, Buf integration) plus 22 target-specific projection families. Every family emits through the same `ArtifactSink` so the CLI and the language bindings are byte-for-byte identical, and every family mints identity through the same `ids` helpers and resolves flows through the same `flows` helpers.

## Interface

Kernel modules:
- `[mod.rs]` — module declarations and the public re-exports: `find_mapping_rule`, `find_projection_override`, `ProjectionExporter`, `ProjectionRegistry`, and the Protobuf type surface (`ProtobufEngine`, `ProtoFile`, `CompatibilityChecker`, `SchemaHistory`, …).
- `[engine.rs:ProjectionExporter]` — the `export_entity` / `export_flow` trait every exporter implements, taking an optional `MappingContract` and `ProjectionContract`.
- `[sink.rs:ArtifactSink]` — the single artifact sink: `Dir(&Path)` for the CLI, `Memory { prefix, map }` for the bindings/WASM.
- `[sink.rs:ArtifactSink::write]` — writes one artifact under the output root by relative path.
- `[registry.rs:ProjectionRegistry]` — read-only index over a `Graph` for mapping/projection contracts.
- `[registry.rs:ProjectionRegistry::new]` — constructs the registry from a borrowed graph.
- `[registry.rs:ProjectionRegistry::find_mappings_for_target]` — all `MappingContract`s whose `target_format` matches.
- `[registry.rs:ProjectionRegistry::find_projections_for_target]` — all `ProjectionContract`s whose `target_format` matches.
- `[registry.rs:ProjectionRegistry::get_mapping_by_name]` — name lookup for a mapping contract.
- `[registry.rs:ProjectionRegistry::get_projection_by_name]` — name lookup for a projection contract.
- `[contracts.rs:find_mapping_rule]` — case-insensitive primitive-type plus exact-name lookup of a `MappingRule` inside a contract.
- `[contracts.rs:find_projection_override]` — the same lookup for a `ProjectionOverride`.
- `[ids.rs:content_hash]` — deterministic content hash of a U+0001-joined tuple (xxh64 seed 42, 16 hex chars).
- `[ids.rs:element_id]` — family-tagged deterministic element id, so the same part tuple yields distinct ids under different families.
- `[ids.rs:sanitize_qname]` — arbitrary string → safe XML QName / IRI fragment.
- `[ids.rs:slug]` — the single blessed lowercase ASCII slug function (ADR-011 §2).
- `[ids.rs:pascal]` — PascalCase identifier for type-name positions.
- `[ids.rs:ident]` — identifier sanitizer preserving case.
- `[ids.rs:sanitize_filename]` — filesystem-safe filename fragment.
- `[ids.rs:collision_suffix]` — hash suffix appended on sanitization collision.
- `[ids.rs:NameRegistrar]` — collision-tracking registrar (`new`, `register`) returning stable, unique, sanitized names.
- `[flows.rs:ResolvedFlow]` — a flow with resource/endpoints resolved to display names, plus `quantity`, `namespace`, and `annotations`.
- `[flows.rs:FlowError]` — structured failures: `DanglingReference { flow_id }` and `MultiNamespace { namespaces }`.
- `[flows.rs:collect_flows]` — resolves every graph flow to display names, sorted by `(resource, from, to)`.
- `[flows.rs:model_namespace]` — derives the single model namespace, erroring on multiple.
- `[yaml.rs:to_yaml]` — shared YAML serialization (block scalars disabled, no anchors, single-scalar strings).

- `[protobuf.rs:ProtobufEngine]` — the `.proto` projection engine (`project`, `project_with_options`, `project_with_full_options`, `project_multi_file`, `flows_to_services`, `collect_response_messages`).
- `[protobuf.rs:ProtoFile]` / `[protobuf.rs:ProtoMessage]` / `[protobuf.rs:ProtoField]` / `[protobuf.rs:ProtoService]` / `[protobuf.rs:ProtoRpcMethod]` / `[protobuf.rs:ProtoEnum]` / `[protobuf.rs:ProtoType]` / `[protobuf.rs:ScalarType]` / `[protobuf.rs:WellKnownType]` / `[protobuf.rs:ProtoOptions]` / `[protobuf.rs:ProtoCustomOption]` / `[protobuf.rs:ProtoOptionValue]` / `[protobuf.rs:StreamingMode]` — the Protobuf IR.
- `[protobuf.rs:map_sea_type_to_proto]` / `[protobuf.rs:infer_proto_type_from_value]` — SEA type/value → proto type mapping.
- `[protobuf.rs:validate_proto_package_namespace]` — proto package/namespace validation.
- `[protobuf.rs:validate_output_path]` — path-traversal guard for any artifact write.
- `[protobuf.rs:CompatibilityChecker]` / `[protobuf.rs:CompatibilityMode]` / `[protobuf.rs:CompatibilityResult]` / `[protobuf.rs:CompatibilityViolation]` / `[protobuf.rs:ViolationType]` — schema compatibility checking (`check`, `apply_backward_compatibility`).
- `[protobuf.rs:SchemaHistory]` — on-disk proto schema history (`new`, `load`, `save`, `check_and_update`).
- `[buf.rs:BufIntegration]` — optional `buf` CLI integration (`new`, `with_path`, `is_available`, `get_version`, `lint`, `breaking_check`).
- `[buf.rs:BufLanguage]` — target language for plugin config (`to_plugin_config`, `parse`).
- `[buf.rs:BufError]` / `[buf.rs:BufLintResult]` / `[buf.rs:BufLintIssue]` / `[buf.rs:BufBreakingResult]` — Buf operation results and errors.
- `[buf.rs:BufIntegration::generate_buf_yaml]` / `[buf.rs:BufIntegration::generate_buf_gen_yaml]` — config file content generation.

Child projection families (each emits through `ArtifactSink` and exposes a `project_*_in_memory` binding surface):
- AI learning: `./ai_learning/EGM.md` — LLM/graph-ML/CEP datasets, recipe, provenance, splits, reports.
- Alloy: `./alloy/EGM.md` — Alloy `.als` signatures and flow facts.
- ArchiMate: `./archimate/EGM.md` — ArchiMate 3.0 Model Exchange XML.
- AsyncAPI: `./asyncapi/EGM.md` — AsyncAPI 3.0 channels/operations/messages.
- BAML: `./baml/EGM.md` — BAML capability project.
- BPMN: `./bpmn/EGM.md` — BPMN 2.0 process XML.
- Cedar: `./cedar/EGM.md` — Cedar schema and permit policies.
- Cell: `./cell/EGM.md` — cell environment IR, Devbox/Mise/deps/sandbox/authority/evidence, `cell.lock`.
- CloudEvents: `./cloudevents/EGM.md` — CloudEvents 1.0 JSONL envelopes.
- CMMN: `./cmmn/EGM.md` — CMMN 1.1 case XML.
- Dagger: `./dagger/EGM.md` — Dagger module manifest plus Python module.
- Devbox: `./devbox/EGM.md` — `devbox.json` domain-manifest environment.
- Domain: `./domain/EGM.md` — language-neutral DDD/CQRS IR with Python/TypeScript/Rust renderers.
- DSPy: `./dspy/EGM.md` — DSPy optimization program, metric, optimizer, config.
- Gauge: `./gauge/EGM.md` — Gauge `.spec` scenarios.
- Lean: `./lean/EGM.md` — Lean 4 Lake package with groundable/deferred proofs.
- OpenShell: `./openshell/EGM.md` — OpenShell sandbox policy, sandbox, mapping report, lock.
- OTel: `./otel/EGM.md` — OTel SemConv registry plus Rust/Python/TypeScript attribute constants.
- RDF: `./rdf/EGM.md` — Turtle, JSON-LD, and OWL ontology artifacts.
- Rego: `./rego/EGM.md` — OPA/Rego bundle, fidelity and provenance reports.
- TLA+: `./tla/EGM.md` — TLA+ spec and TLC config.
- ZenML: `./zenml/EGM.md` — ZenML learning-pipeline package.

Static Rego assets: `rego/rego/lib.rego` and `rego/rego/decision.rego` are embedded at compile time via `include_str!` (`[rego/mod.rs:LIB_REGO]`, `[rego/mod.rs:DECISION_REGO]`). They are build inputs, not a Rust source directory, so they carry no contract of their own.

## Invariants
- `[sink.rs]` (module doc) — every renderer writes through `ArtifactSink`, so the CLI directory output and the binding in-memory output stay byte-for-byte identical.
- `[sink.rs:write_artifact]` — every filesystem artifact write is preceded by `protobuf::validate_output_path`, so a relative path can never escape the output root.
- `[sink.rs:ArtifactSink::write]` — the in-memory variant keys the map by `prefix + rel`, so families can nest artifacts under a shared prefix without filesystem access (WASM-safe).
- `[ids.rs]` (module doc) — hashing reuses `authority::compute_deterministic_hash`; the U+0001 separator is illegal in human-authored names, so a part tuple cannot collide by concatenation.
- `[ids.rs:element_id]` — the family tag is the first hashed field, so one part tuple yields distinct ids under different families.
- `[ids.rs:slug]` — the single blessed slug implementation every projection family MUST use instead of a private reimplementation (ADR-011 §2).
- `[flows.rs]` (module doc) — no family re-implements flow resolution (ADR-011 §2); the dangling-reference policy is uniform (a hard error, not a silent drop) because a partially projected authority/verification artifact must not look complete.
- `[flows.rs:model_namespace]` — the projection targets require a single namespace; multi-namespace graphs are a hard error.
- `[flows.rs:FlowError]` — `From<FlowError> for String` is implemented so existing `Result<_, String>` call sites keep compiling unchanged.
- `[yaml.rs:to_yaml]` — block scalars are disabled so every string stays a single scalar with no folding; ambiguous scalars are quoted by YAML 1.1 rules; anchors are never generated.
- `[protobuf.rs:validate_output_path]` — lexically rejects `ParentDir`/`RootDir` traversal even when the target file does not exist yet.
- `[protobuf.rs]` (module doc) — field numbering is deterministic so emitted schemas are stable across runs.
- `[buf.rs]` (module doc) — all buf operations gracefully degrade when the buf CLI is not installed; they never make a projection fail.
- `[registry.rs:ProjectionRegistry]` — lookups are read-only over the borrowed graph; the registry never mutates it.
- Determinism across every family rests on `ids` (content hashing, family tags) and on caller-supplied `created_at`; wall-clock and random per-parse flow UUIDs must not reach artifact contents.
- `[ids.rs:NameRegistrar]` — registering the same original twice returns the same id (no double-suffix), while a sanitization collision yields a distinct `<base>_<hash>` id.

## Side Effects
- `[sink.rs:write_artifact]` — creates parent directories (`std::fs::create_dir_all`) and writes the file (`std::fs::write`) under the output root.
- `[protobuf.rs:SchemaHistory::save]` — creates the history directory and writes one JSON file per proto package.
- `[protobuf.rs:SchemaHistory::load]` — reads the schema history file for a package from disk.
- `[protobuf.rs:SchemaHistory::check_and_update]` — persists the new schema when compatible (or in `Breaking` mode).
- `[buf.rs:BufIntegration::lint]` — writes a temporary `buf.yaml` into the proto directory when absent and removes it afterwards.
- `[buf.rs:BufIntegration::get_version]` / `[buf.rs:BufIntegration::lint]` / `[buf.rs:BufIntegration::breaking_check]` — spawn external `buf` processes via `std::process::Command`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
