# domainforge-core `src` — Canonical Rust Core

## Overview
Crate root for the canonical DomainForge implementation: the SEA DSL parser,
graph model, policy evaluator, projections, knowledge-graph/SBVR interchange,
application-contract resolution, and the Python/TypeScript/WASM bindings. Every
other language surface is a projection of this crate.

## Interface
- `[lib.rs:VERSION]` — crate version from `env!("CARGO_PKG_VERSION")`.
- `[lib.rs]` — unconditional modules: `application`, `authority`, `calm`, `concept_id`, `error`, `formatter`, `graph`, `kg`, `kg_import`, `module`, `parser`, `patterns`, `policy`, `primitives`, `projection`, `registry`, `sbvr`, `semantic_pack`, `semantic_version`, `units`, `uuid_module`, `validation_error`, `validation_result`.
- `[lib.rs]` — feature-gated modules: `cli` (feature `cli`), `python` (feature `python`), `typescript` (feature `typescript`), `wasm` (feature `wasm`).
- `[lib.rs]` — re-exports: `ConceptId`, `Graph`, `KgError`, `KnowledgeGraph`, `import_kg_rdfxml`, `import_kg_turtle`, `ImportError`, `parse`, `parse_to_graph`, `parse_to_graph_with_options`, `ParseOptions`, `Pattern`, `NamespaceBinding`, `NamespaceRegistry`, `RegistryError`, `SbvrError`, `SbvrModel`, `SemanticVersion`, `try_unit_from_string`, `unit_from_string`, `Dimension`, `Unit`, `UnitError`, `UnitRegistry`, `format_uuid`, `generate_uuid_v7`, `parse_uuid`, `ErrorCode`, `Position`, `SourceRange`, `ValidationError`, `ValidationResult`.
- `[lib.rs]` — TypeScript-only aliased re-exports: `TsEntity`, `TsFlow`, `TsInstance`, `TsResource`, `TsResourceInstance`, `TsGraph`, `TsNamespaceBinding`, `TsNamespaceRegistry`.
- `[lib.rs:domainforge]` — `#[pymodule] fn domainforge(m)` under feature `python`; registers `__version__`, the `python::primitives` classes, `python::graph::Graph`, the `python::registry` classes, the `python::policy` enums/types, the `python::authority` classes plus `evaluate_authority`, the `python::units` classes, `python::formatter::{format_source, check_format}`, and the `python::semantic_pack` classes/functions.
- `[concept_id.rs:ConceptId]` (`+from_concept/from_uuid/as_uuid/from_legacy_uuid`, `Display`, `From<Uuid>`, `AsRef<Uuid>`) — deterministic concept identity.
- `[patterns.rs:Pattern]` (`+new/id/name/namespace/regex/is_match`) — regex-backed named patterns.
- `[semantic_version.rs:SemanticVersion]` (`+new/parse/bump_major/bump_minor/bump_patch`, `Display`, `Default`) — `major.minor.patch`.
- `[uuid_module.rs:generate_uuid_v7]`, `[uuid_module.rs:parse_uuid]`, `[uuid_module.rs:format_uuid]` — UUID v7 helpers.
- `[validation_error.rs:ErrorCode]` (`+as_str/description`), `[validation_error.rs:Position]` (`+new`), `[validation_error.rs:SourceRange]` (`+new/from_line_col/single_position`), `[validation_error.rs:ReferenceType]`, `[validation_error.rs:ValidationError]` (`+error_code/range/location_string/with_suggestion/with_types` plus the constructor family), `[validation_error.rs:FUZZY_MATCH_THRESHOLD]` (2) — the diagnostic vocabulary.
- `[validation_result.rs:ValidationResult]` (`+new`) — `total_policies`, `violations`, `error_count`.
- `[kg.rs:KnowledgeGraph]` (`+new/from_graph/to_turtle/from_turtle/to_graph/validate_shacl/to_rdf_xml/escape_turtle_literal/escape_xml/export_rdf`), `[kg.rs:Triple]`, `[kg.rs:ShaclShape]`, `[kg.rs:ShaclProperty]`, `[kg.rs:KgError]` — RDF/Turtle/SHACL interchange.
- `[kg_import.rs:import_kg_turtle]`, `[kg_import.rs:import_kg_rdfxml]`, `[kg_import.rs:ImportError]` — RDF to `Graph` import.
- `[sbvr.rs:SbvrModel]` (`+new/from_graph/to_xmi/from_xmi/to_graph/export_sbvr`), `[sbvr.rs:SbvrTerm]`, `[sbvr.rs:TermType]`, `[sbvr.rs:SbvrFactType]`, `[sbvr.rs:SbvrBusinessRule]`, `[sbvr.rs:RuleType]`, `[sbvr.rs:SbvrError]` — SBVR vocabulary to/from XMI and to/from `Graph`.
- `[test_utils.rs]` — `#[cfg(test)]` only; a proptest asserting `uuid_module::generate_uuid_v7()` never panics. Not part of the public surface.
- Child contracts: `./application/EGM.md`; `./authority/EGM.md`; `./bin/EGM.md`; `./calm/EGM.md`; `./cli/EGM.md`; `./error/EGM.md`; `./formatter/EGM.md`; `./graph/EGM.md`; `./module/EGM.md`; `./parser/EGM.md`; `./policy/EGM.md`; `./primitives/EGM.md`; `./projection/EGM.md`; `./python/EGM.md`; `./registry/EGM.md`; `./semantic_pack/EGM.md`; `./typescript/EGM.md`; `./units/EGM.md`; `./wasm/EGM.md`.

## Invariants
- `[concept_id.rs:ConceptId::from_concept]` — the ID is a UUIDv5 over `"{namespace}::{name}"` in a fixed namespace UUID, so the same namespace+name always yields the same `ConceptId` and different namespaces never collide.
- `[patterns.rs:Pattern::new]` — the regex is compiled eagerly at construction so an invalid pattern is rejected at parse time rather than at first match; `[patterns.rs:Pattern::is_match]` then reuses a lazily cached compiled `Regex` in a `OnceCell`.
- `[semantic_version.rs:SemanticVersion::parse]` — input must have exactly three dot-separated numeric components; `1.2` and `1.2.3.4` are errors.
- `[validation_result.rs:ValidationResult::new]` — `error_count` is derived from the passed violations (counting `Severity::Error`), never supplied independently.
- `[kg_import.rs:ImportError]` — RDF import failures are reported as `ShaclValidation` or `Other`, never swallowed.
- `[kg.rs:KnowledgeGraph::to_graph]` — round-tripping through `to_turtle`/`from_turtle`/`to_graph` reconstructs `Entity`/`Resource`/`Flow`/`Relation` only; `sea:EntityInstance` and `sea:Policy` triples are silently dropped rather than erroring, so this path must not be used for instance/policy round-tripping (the Canonical Semantic Envelope is the supported carrier).
- `[lib.rs]` — `cli`, `python`, `typescript`, and `wasm` are feature-gated; code outside them must not assume those modules exist.
- `[lib.rs]` — the `lol_alloc` global allocator is installed only under `all(feature = "lol_alloc", target_arch = "wasm32")`, so non-WASM targets never pull wasm-only allocator symbols.

## Side Effects
- `[lib.rs]` — installs a process-global `#[global_allocator]` when `all(feature = "lol_alloc", target_arch = "wasm32")`.
- `[lib.rs:domainforge]` — registers classes and functions into the CPython `domainforge` module at import time (feature `python`).
- `[kg.rs:KnowledgeGraph::from_graph]` — writes warnings to stderr via `eprintln!` when skipping an invalid RDF term for `rdf_class`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
