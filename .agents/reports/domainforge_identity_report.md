# DomainForge Identity Report

Date: 2026-06-10  
Method: code-first architectural investigation. Documentation, README language, comments, and repository names were treated as claims unless supported by implemented behavior.

## Executive Judgment

DomainForge is primarily a **semantic modeling language and graph-centered projection framework**.

Its implemented center is not a wire protocol, not a conventional compiler, and not an ontology engine. The code implements:

1. a textual `.sea` language,
2. a semantic Rust model built from named primitives,
3. a canonical in-memory `Graph`,
4. validation through policy expression evaluation,
5. projections/imports to interoperability formats,
6. language bindings over the same Rust core.

The best short answer is:

> DomainForge is a graph-backed semantic modeling DSL for enterprise architecture concepts, with policy evaluation and projection tooling for CALM, RDF/SBVR/KG, Protobuf, Python, TypeScript, and WASM consumers.

Primary classification: **semantic modeling language + projection engine**.  
Secondary classifications: DSL, knowledge representation framework, schema/projection system, policy-validation engine, interoperability substrate.  
Misleading classifications: protocol, protocol definition language, ontology engine, full compiler, general-purpose programming language.

## Phase 1: Repository Census

### Packages and Crates

- Root Cargo workspace: `Cargo.toml` contains one member, `sea-core`.
- Rust core crate: `sea-core/Cargo.toml`, package `sea-core`, version `0.11.0`.
- CLI binary: `sea-core/src/bin/sea.rs`, gated by the `cli` feature.
- Python package: `pyproject.toml`, package `sea-dsl`, built by maturin from `sea-core`.
- TypeScript package: `package.json`, package `@domainforge/sea`, built with napi-rs from `sea-core`.
- WASM surface: `sea-core/src/wasm/`.

### Major Modules

Core modules exposed by `sea-core/src/lib.rs`:

- `parser`: Pest grammar, AST, AST-to-graph conversion, linting, pretty printing, profiles.
- `primitives`: semantic primitives such as `Entity`, `Resource`, `Flow`, `Instance`, `Role`, `Relation`, `Metric`, `MappingContract`, `ProjectionContract`.
- `graph`: canonical graph store and validation surface.
- `policy`: expression model, normalization, quantifiers, type inference, three-valued evaluation.
- `projection`: projection contracts, registry, Protobuf engine, Buf integration.
- `calm`: CALM import/export.
- `kg` and `kg_import`: RDF/Turtle/RDF-XML knowledge graph export/import plus SHACL-related structures.
- `sbvr`: SBVR/XMI export/import model.
- `registry` and `module`: `.sea-registry.toml` namespace binding and import validation.
- `authority`: authority policy environment, fact requirements, resolver, trace, decision model.
- `semantic_pack`: semantic pack schema, validation, signing, diff, alias/concept resolution.
- `python`, `typescript`, `wasm`: bindings over the Rust core.

### Entry Points

- Public Rust API: `sea-core/src/lib.rs` re-exports `Graph`, `ConceptId`, `parse`, `parse_to_graph`, `KnowledgeGraph`, KG import functions, SBVR, registry, units, and validation types.
- CLI: `sea-core/src/bin/sea.rs` dispatches to `parse`, `validate`, `import`, `project`, `format`, `test`, `validate-kg`, `normalize`, `registry`, `authority`, and `pack`.
- Python module: `sea_dsl` exposes primitives, `Graph`, policy expression types, authority evaluation, units, formatter functions, and semantic pack functions.
- TypeScript module: `index.d.ts` exposes `Graph`, primitives, policy enums, authority evaluation, formatter functions, and semantic pack functions.
- WASM module: `sea-core/src/wasm/mod.rs` exports primitives, graph, policy, semantic pack, units, formatter, and authority bindings.

### Core Data Structures

The central structure is `Graph` in `sea-core/src/graph/mod.rs`.

It stores:

- entities
- roles
- resources
- flows
- relations
- resource instances
- entity instances
- policies
- patterns
- concept changes
- metrics
- mappings
- projections
- entity-role assignments
- graph evaluation config

`Graph` uses `IndexMap` for deterministic iteration. It validates references when adding flows, instances, roles, and relations. Named concepts use `ConceptId::from_concept(namespace, name)`; flows use UUID-backed IDs because the implementation treats flows as events rather than named concepts.

### Parsers, Interpreters, Generators, Runtimes

- Parser: `sea-core/grammar/sea.pest` and `sea-core/src/parser/ast.rs`.
- AST-to-IR conversion: `ast_to_graph_with_options` in `sea-core/src/parser/ast.rs`.
- Interpreter/evaluator: policy evaluation in `sea-core/src/policy/core.rs`.
- Generators/projections:
  - CALM JSON: `sea-core/src/calm/export.rs`.
  - RDF Turtle/RDF-XML: `sea-core/src/kg.rs`.
  - SBVR XMI: `sea-core/src/sbvr.rs`.
  - Protobuf `.proto`: `sea-core/src/projection/protobuf.rs`.
  - AST JSON schema types: `sea-core/src/parser/ast_schema.rs`.
- Importers:
  - CALM JSON: `sea-core/src/calm/import.rs`.
  - KG Turtle/RDF-XML: `sea-core/src/kg_import.rs`.
  - SBVR XMI: `sea-core/src/sbvr.rs`.
- Runtime components:
  - Policy evaluation over a `Graph`.
  - Authority decision evaluation over fact sources.
  - Semantic pack validation and signing.

### Storage Systems

There is no durable database engine in the repository. Storage is file-based:

- `.sea` source files.
- `.sea-registry.toml` namespace registry.
- JSON/Turtle/RDF/XML/Proto projection outputs.
- Protobuf schema history files in `SchemaHistory`.
- Semantic pack JSON structures.

## Phase 2: Semantic Center of Gravity

### Canonical Representation

The canonical implementation representation is **`Graph`**, not `.sea`, not AST, and not RDF.

Evidence:

- `parse_to_graph` is exported from `sea-core/src/lib.rs`.
- CLI `parse` defaults to Graph output unless `--ast` is requested.
- CLI `validate` parses `.sea` into `Graph`, then calls `Graph::validate`.
- CLI `project` parses `.sea` into `Graph`, then projects the graph to CALM, KG, or Protobuf.
- CALM import returns `Graph`.
- KG import returns `Graph`.
- SBVR import converts to `Graph`.
- Protobuf projection takes `&Graph`.
- Policy evaluation takes `&Graph`.
- Python and TypeScript golden tests parse `.sea` directly into `Graph`.

The AST is important but transitional. It preserves source structure, spans, file metadata, and declaration syntax. The graph is the semantic IR used by validators, projections, imports, tests, and bindings.

### Major Workflows

#### `.sea` to Graph

Input: `.sea` source text.  
Internal path: Pest parser -> AST -> multi-pass `ast_to_graph_with_options`.  
Output: `Graph`.

The graph conversion:

- registers dimensions and units,
- validates and registers regex patterns,
- registers concept changes,
- adds roles, entities, resources,
- resolves flows by entity/resource name,
- resolves relations by role/resource name,
- adds entity instances,
- adds policies,
- adds metrics,
- adds mapping contracts,
- adds projection contracts.

#### `.sea` to Validation Result

Input: `.sea` file or directory.  
Internal path: namespace registry/module resolver -> AST -> `Graph` -> `Graph::validate`.  
Output: human, JSON, or LSP-like validation result.

`Graph::validate` evaluates all policies and collects violations. Policy evaluation expands expressions against the graph and supports boolean or three-valued logic.

#### `.sea` to Projection

Input: `.sea` file.  
Internal path: parse -> `Graph` -> projection-specific engine.  
Outputs:

- CALM JSON.
- RDF Turtle or RDF/XML.
- Protobuf `.proto`.

#### External Format to Graph

Input: CALM JSON, KG Turtle/RDF-XML, or SBVR XMI.  
Internal path: importer -> `Graph`.  
Output: graph statistics or downstream `Graph` use.

#### Graph to Language Bindings

Input: API calls from Python, TypeScript, or WASM.  
Internal path: binding wrappers -> Rust core primitives and graph.  
Output: language-native objects backed by core behavior.

### Source of Truth

There are two source-of-truth layers:

1. **For persisted authoring:** `.sea` files are the human-authored source format.
2. **For implemented semantics:** `Graph` is the canonical representation that validation and projection consume.

The repository gives `Graph` semantic authority. `.sea` is the dominant authoring syntax, but external formats can also import into `Graph`, and bindings can construct `Graph` directly.

## Phase 3: Language Analysis

`.sea` is a **domain-specific semantic modeling language**.

It is more than configuration because the grammar includes:

- declarations: entity, resource, flow, pattern, role, relation, instance, policy, concept change, metric, mapping, projection, dimension, unit,
- module syntax: imports and exports,
- file metadata annotations: namespace, version, owner, profile,
- policy expressions: boolean operators, comparison, arithmetic, string predicates, temporal predicates, role predicates,
- quantifiers: forall, exists, exists unique,
- aggregations and group-by expressions,
- unit-aware quantity literals and casts.

It is not a general programming language:

- no user-defined functions,
- no imperative control flow,
- no mutation statements,
- no loops beyond declarative quantifiers/aggregations,
- no runtime process model.

It is not only a schema language:

- it models flows between entities,
- it evaluates policies over graph facts,
- it emits violations,
- it projects to RDF/CALM/SBVR/Protobuf,
- it has governance concepts such as authority packs and semantic packs.

It is closest to a semantic modeling DSL with an embedded constraint/policy expression language.

## Phase 4: Protocol Analysis

DomainForge does **not** currently qualify as a protocol in the strict sense.

### Protocol Criteria

1. Shared vocabulary: **Yes.**  
   Implemented primitives and semantic packs define vocabulary: entities, resources, flows, roles, relations, policies, metrics, units, concepts, aliases.

2. Shared semantics: **Partial yes.**  
   The graph conversion, policy evaluator, unit system, namespace resolution, and projection rules assign semantics to `.sea` declarations.

3. State transitions: **Weak.**  
   There are concept changes, versions, policy evaluation states, semantic pack trust states, Protobuf compatibility modes, and authority decisions. But there is no implemented protocol state machine for independent participants exchanging messages.

4. Interoperability between independent systems: **Partial yes.**  
   DomainForge projects to CALM, RDF, SBVR, and Protobuf. Those outputs support interoperability. The repository itself does not define a message exchange protocol between independent systems.

5. Compliance requirements: **Partial yes.**  
   There are validation errors, semantic pack validation modes, SHACL shapes, CALM schema tests, Protobuf compatibility checks, and authority conformance tests. These are compliance-like checks, but not a formal protocol conformance suite for participants.

6. Canonical meaning preservation: **Partial yes.**  
   Round-trip tests exist for CALM, and deterministic tests exist for Protobuf and RDF escaping. But not all projections are complete or bidirectionally lossless.

### Protocol Artifacts, If Interpreted Broadly

If one stretches the word "protocol," the artifacts would be:

- vocabulary: primitives and semantic packs,
- boundary format: `.sea` files and projection outputs,
- participants: authors, validators, projectors, downstream CALM/RDF/Protobuf/SBVR consumers,
- messages: `.sea`, CALM JSON, Turtle/RDF-XML, SBVR XMI, `.proto`, semantic pack JSON,
- state transitions: validation pass/fail, authority allow/deny/escalate, pack approval/signature states, Protobuf compatibility changes.

But this is better described as an **interoperability substrate** than a protocol. A protocol normally specifies participant roles, message exchange, ordering, state transitions, and conformance at a boundary. DomainForge mainly specifies a semantic model and projections.

## Phase 5: Projection Analysis

### Rust

Input: direct Rust API or parsed `.sea`.  
Internal representation: `Graph` and primitive structs.  
Output: Rust objects and serialized `Graph`.

Rust is the canonical implementation, not a projection target in the usual sense.

### Python

Input: Python API calls or `Graph.parse` over `.sea`.  
Internal path: PyO3 wrappers -> Rust core.  
Output: Python-accessible graph, primitives, policy evaluation, formatter, units, authority, semantic pack functions.

The Python package wraps the Rust core. It does not duplicate business logic.

### TypeScript

Input: TypeScript API calls or `Graph.parse` over `.sea`.  
Internal path: napi-rs wrappers -> Rust core.  
Output: TypeScript-accessible graph, primitives, policy, authority, semantic pack, formatter, units.

TypeScript tests exercise parity against Rust behavior.

### WASM

Input: browser/JS/WASM calls.  
Internal path: wasm-bindgen wrappers -> Rust core.  
Output: WASM-accessible graph, primitives, policy, semantic pack, units, formatter, authority.

### JSON Schema / AST JSON

Input: `.sea`.  
Internal path: parse -> internal AST -> `ast_schema` mirror types.  
Output: stable AST JSON and optional JSON Schema generation through `schemars`.

This is an AST API schema, not the primary semantic representation.

### Protobuf

Input: `Graph`.  
Internal path: `ProtobufEngine::project*`.  
Output: `ProtoFile` IR -> `.proto` text.

Implemented features include message generation from entities/resources, deterministic ordering/field numbering, governance messages, optional gRPC service generation, multi-file output, schema history, compatibility checking, and Buf hooks.

### RDF / Knowledge Graph

Input: `Graph`.  
Internal path: `KnowledgeGraph::from_graph`.  
Output: Turtle or RDF/XML with SEA terms, RDF/RDFS/OWL/XSD/SHACL prefixes, triples, and shapes.

KG import parses Turtle/RDF-XML back to `Graph` for supported structures.

### SBVR

Input: `Graph`.  
Internal path: `SbvrModel::from_graph`.  
Output: SBVR XMI/XML.

Import from SBVR XMI to `Graph` is also implemented.

### CALM

Input: `Graph`.  
Internal path: CALM exporter.  
Output: CALM JSON model with nodes, relationships, and metadata.

CALM import reconstructs a `Graph`. Round-trip tests cover entities, resources, flows, instances, policies, patterns, and associations.

### Context Envelopes

There is authority request/decision and semantic pack JSON support, but no general "context envelope" projection appears as a first-class projection target in the source tree inspected.

## Phase 6: Architectural Comparison

### Protocol Buffers

Similarities:

- Projects semantic entities/resources to schema artifacts.
- Implements deterministic output and schema compatibility checks.
- Can generate gRPC service definitions.

Differences:

- Protobuf is primarily an IDL and binary serialization ecosystem.
- DomainForge's source model is semantic graph primitives and policies; Protobuf is one projection target.

Analogy succeeds: DomainForge can behave like a higher-level schema source for `.proto`.  
Analogy fails: DomainForge is not primarily a serialization protocol or wire format.

### Smithy

Similarities:

- Model-first design.
- Can target generated protocol/schema artifacts.
- Has semantic model and validation concerns.

Differences:

- Smithy is centered on service APIs and protocol bindings.
- DomainForge is centered on enterprise semantic primitives, flows, policies, KG/CALM/SBVR projections.

Analogy succeeds: both are model-to-projection systems.  
Analogy fails: DomainForge is not primarily service/API modeling.

### OpenAPI

Similarities:

- Documents/contracts can be consumed by external tools.
- Schema-like projection and validation goals.

Differences:

- OpenAPI describes HTTP APIs.
- DomainForge describes semantic enterprise concepts and flows.

Analogy succeeds: both support interoperability through machine-readable models.  
Analogy fails: DomainForge does not define HTTP operations, request/response shapes, or API servers.

### GraphQL

Similarities:

- Strongly shaped graph-like conceptual model.
- Schema-driven tooling analogy.

Differences:

- GraphQL specifies query execution over a service.
- DomainForge has no implemented query language or resolver runtime.

Analogy succeeds: graph vocabulary and typed model mindset.  
Analogy fails: DomainForge is not a query protocol.

### Terraform

Similarities:

- Declarative language.
- Describes desired structures.
- Validation and formatting tooling.

Differences:

- Terraform manages infrastructure state and applies changes through providers.
- DomainForge does not reconcile external infrastructure or maintain provider state.

Analogy succeeds: declarative authoring plus tooling.  
Analogy fails: no apply engine or state reconciliation.

### Kubernetes CRDs

Similarities:

- Schema-like extensible domain objects.
- Validation and structured API concepts.

Differences:

- CRDs extend a Kubernetes API server with reconciliation controllers.
- DomainForge has no cluster API, resource lifecycle, or controller model.

Analogy succeeds: custom semantic object definitions.  
Analogy fails: no operational control plane.

### OWL

Similarities:

- Semantic concepts can be projected to RDF/OWL-flavored outputs.
- Vocabulary and ontology-adjacent structures exist.

Differences:

- DomainForge does not implement OWL reasoning, class axioms, property restrictions, or description logic inference.

Analogy succeeds: semantic vocabulary export.  
Analogy fails: no ontology reasoning engine.

### RDF

Similarities:

- RDF/Turtle/RDF-XML projection and import exist.
- Knowledge graph triples and SHACL shapes exist.

Differences:

- RDF is a graph data model and serialization family.
- DomainForge's canonical model is Rust `Graph`; RDF is a projection/import format.

Analogy succeeds: DomainForge can emit semantic graph facts.  
Analogy fails: RDF is not the internal source of truth.

### UML

Similarities:

- Models entities, relationships, and architecture concepts.
- Useful for conceptual architecture descriptions.

Differences:

- DomainForge has executable policy validation and projection pipelines.
- UML has broader diagram semantics and tooling conventions not implemented here.

Analogy succeeds: semantic modeling of systems.  
Analogy fails: DomainForge is code-first and graph/projection-oriented, not diagram-first.

### XSD

Similarities:

- Schema validation analogy.
- SBVR/CALM/XML-related outputs can be schema-checked.

Differences:

- XSD defines XML document structure.
- DomainForge defines semantic graph primitives and policies.

Analogy succeeds: contract validation mindset.  
Analogy fails: DomainForge is not an XML schema language.

### CUE

Similarities:

- Declarative constraints.
- Validation orientation.
- Can act as a source for generated artifacts.

Differences:

- CUE is a general data constraint and configuration language.
- DomainForge is domain-specific, graph-backed, and enterprise-architecture oriented.

Analogy succeeds: constraints plus data/model generation.  
Analogy fails: DomainForge's semantics are narrower and graph-native.

### Dhall

Similarities:

- Typed declarative configuration language analogy.
- Aims for safe machine-readable authoring.

Differences:

- Dhall is a total functional configuration language.
- DomainForge lacks functional abstraction and is centered on semantic primitives, policies, and projections.

Analogy succeeds: declarative source language.  
Analogy fails: DomainForge is not a general configuration calculus.

## Phase 7: Adversarial Classification

### If DomainForge is a DSL, why might that be wrong?

It is not only a DSL. The core API lets consumers construct `Graph` directly without `.sea`. Importers can build graphs from CALM, KG, and SBVR. The Rust semantic model and projection engines matter as much as textual syntax.

Verdict: DSL is correct but incomplete.

### If DomainForge is a protocol, why might that be wrong?

The repository lacks a formal participant model, exchange sequence, wire message lifecycle, transport boundary, or protocol state machine. It has semantic artifacts and projections, but those are not enough to make it a protocol.

Verdict: protocol is misleading unless used loosely.

### If DomainForge is a protocol definition language, why might that be wrong?

Protobuf output and gRPC service generation exist, but `.sea` is not primarily for defining service protocols. Its grammar centers on entities, resources, flows, policies, roles, relations, metrics, units, mappings, and projections.

Verdict: not primary. It can generate protocol-adjacent artifacts.

### If DomainForge is an ontology engine, why might that be wrong?

It emits RDF-like knowledge graphs and has semantic packs, aliases, concepts, definitions, and SHACL shapes. But it does not implement OWL reasoning, ontology classification, rule inference over RDF, or a triplestore-backed query engine.

Verdict: ontology-adjacent, not an ontology engine.

### If DomainForge is a semantic modeling language, why might that be wrong?

Some implemented features are operational tooling: CLI commands, package bindings, Protobuf schema compatibility, authority decisions, pack signing. Those extend beyond pure modeling.

Verdict: still the best primary classification because the operational tooling serves the semantic model.

### If DomainForge is a compiler, why might that be wrong?

It parses and generates artifacts, but it does not lower to executable code as its primary output. There is no optimizer, backend codegen pipeline, or runtime execution target comparable to a compiler.

Verdict: projection compiler in a narrow sense, not a general compiler.

### If DomainForge is a projection engine, why might that be wrong?

Projection is substantial, but the project also defines the source language, graph model, policy evaluator, importers, namespace registry, semantic packs, and authority layer.

Verdict: projection engine is a major secondary identity, not the whole identity.

### If DomainForge is an interoperability substrate, why might that be wrong?

The projections support interoperability, but the repository does not yet show broad independent-system integrations, formal conformance profiles, or message choreography.

Verdict: credible secondary identity, aspirational if stated as primary.

### If DomainForge is a schema system, why might that be wrong?

It has AST schema, registry schema, CALM schema tests, Protobuf schema output, and compatibility checks. But its model includes graph relations, flows, policies, metrics, and semantic packs beyond structural schema.

Verdict: schema system is too narrow.

### If DomainForge is a knowledge representation framework, why might that be wrong?

It represents concepts and exports knowledge graphs, but the canonical store is a typed Rust graph, not RDF. Reasoning is policy evaluation over graph facts, not general knowledge inference.

Verdict: partially true, secondary.

## Phase 8: Final Judgment

## What DomainForge Is

DomainForge is a **graph-centered semantic modeling language and projection framework**.

Its primary artifact is a semantic `Graph` built from `.sea` declarations or imported external models. Its primary job is to preserve and validate meaning across enterprise concepts: actors/entities, resources, flows, roles, relations, policies, metrics, units, mappings, projections, and semantic packs.

It is also a DSL because `.sea` is a real implemented language with grammar, parser, formatter, AST, module imports, declarations, and expressions.

It is also a projection engine because the graph projects to CALM, RDF/KG, SBVR, and Protobuf, with partial round-trip/import support.

It is also an interoperability substrate because it uses a single semantic graph to bridge multiple external formats and language bindings.

## What DomainForge Is Not

DomainForge is not primarily a protocol. It lacks a formal exchange protocol with participants, transport, messages, sequence, and state transitions.

It is not primarily a protocol definition language. Protobuf is a projection target, not the center.

It is not an ontology engine. It emits ontology-adjacent RDF/SHACL structures but does not implement OWL or RDF reasoning.

It is not a conventional compiler. It translates semantic models to artifacts but does not compile programs to an executable target.

It is not only a schema system. Structural schemas are one output; semantic graph meaning and policy evaluation are central.

It is not merely configuration. `.sea` carries semantics, constraints, relations, units, projection contracts, and graph-transformable meaning.

## What DomainForge Is Becoming

The architecture points toward a **semantic interoperability platform**:

- `.sea` as authoring syntax,
- `Graph` as semantic IR,
- policy engine as validation semantics,
- semantic packs as governed meaning/trust artifacts,
- authority layer as decision governance,
- projections as boundary adapters to external ecosystems,
- language bindings as distribution surfaces.

The trajectory is broader than "DSL" and narrower than "protocol." It is becoming a governed semantic model hub that can project into API schemas, architecture-as-code, knowledge graphs, and policy/authority workflows.

## Evidence Summary

High-value evidence:

- `sea-core/src/lib.rs`: public API exports `Graph`, parser, KG, SBVR, registry, units, validation, and bindings.
- `sea-core/grammar/sea.pest`: implemented `.sea` language grammar.
- `sea-core/src/parser/ast.rs`: AST and AST-to-Graph conversion; mappings/projections are graph nodes.
- `sea-core/src/graph/mod.rs`: canonical semantic graph and validation entry point.
- `sea-core/src/policy/core.rs`: policy evaluation over `Graph`.
- `sea-core/src/projection/protobuf.rs`: Graph-to-Protobuf engine and compatibility checking.
- `sea-core/src/calm/export.rs` and `sea-core/src/calm/import.rs`: Graph/CALM bidirectional path.
- `sea-core/src/kg.rs` and `sea-core/src/kg_import.rs`: Graph/RDF/KG projection and import path.
- `sea-core/src/sbvr.rs`: Graph/SBVR path.
- `sea-core/src/cli/project.rs`: CLI projection path always parses input to `Graph` before output.
- `sea-core/src/cli/validate.rs`: CLI validation path parses input to `Graph` before policy validation.
- `sea-core/tests/protobuf_projection_tests.rs`: parse `.sea` -> Graph -> ProtoFile -> `.proto`.
- `sea-core/tests/calm_round_trip_tests.rs`: Graph -> CALM -> Graph round-trip.
- `sea-core/tests/phase_17_export_tests.rs`: Graph -> SBVR/RDF/KG exports.
- `tests/test_golden_payment_flow.py` and `typescript-tests/golden-payment-flow.test.ts`: cross-language `.sea` -> Graph parity.
- `sea-core/tests/semantic_pack_consumer_smoke.rs`: semantic pack validation and trust-state behavior.

## Confidence

Confidence: **High**.

Reason: the conclusion is supported by multiple independent code paths. Parser, CLI, projections, imports, tests, and language bindings all converge on `Graph` as semantic IR. The protocol-negative conclusion is also high confidence because the repository contains no implemented participant protocol, transport, exchange sequence, or protocol conformance boundary. The only lower-confidence area is future trajectory: semantic packs and authority modules suggest a governed interoperability platform, but that is an architectural direction rather than a fully mature runtime protocol.
