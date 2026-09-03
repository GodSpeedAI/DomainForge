# DomainForge Source Map

This document connects high-level concepts, capabilities, and invariants to their concrete implementation locations in the codebase. Use this map to navigate directly from architectural intent to code, tests, and configuration.

---

## 1. Core Domain Concepts & Primitives

| Concept / Type | Source File | Key Symbols | Responsibility | Test Locations |
|---|---|---|---|---|
| **ConceptId** | `domainforge-core/src/concept_id.rs` | `ConceptId`, `ConceptId::from_concept()` | Deterministic UUID v5 identifier generated from namespace + concept name. | `src/concept_id.rs` (unit tests) |
| **Entity** | `domainforge-core/src/primitives/entity.rs` | `Entity`, `Entity::new()`, `Entity::concept_id()` | Domain actor, participant, or bounded context element. | `tests/entity_tests.rs` |
| **Resource** | `domainforge-core/src/primitives/resource.rs` | `Resource`, `Resource::new()`, `Resource::unit()` | Material, asset, message, or currency that moves between entities. | `tests/resource_tests.rs` |
| **Flow** | `domainforge-core/src/primitives/flow.rs` | `Flow`, `Flow::new()`, `Flow::quantity()` | Directed movement of a quantity of resource from one entity to another. | `tests/flow_tests.rs` |
| **Instance** | `domainforge-core/src/primitives/instance.rs` | `Instance`, `Instance::new()`, `Instance::fields()` | Concrete data record for an entity with assigned field values. | `tests/instance_tests.rs` |
| **ResourceInstance** | `domainforge-core/src/primitives/resource_instance.rs` | `ResourceInstance` | Concrete physical or digital instance of a resource. | `tests/instance_tests.rs` |
| **Role & Binding** | `domainforge-core/src/primitives/role.rs` | `Role`, `Role::new()` | Abstract organizational responsibility bound to entities. | `tests/role_binding_tests.rs` |
| **Relation** | `domainforge-core/src/primitives/relation.rs` | `RelationType`, `Relation` | Explicit semantic relationship between entities via flows. | `tests/roles_relations_tests.rs` |
| **Metric** | `domainforge-core/src/primitives/metric.rs` | `Metric`, `MetricAnnotation` | Observability rule tracking a quantified aggregation expression over time. | `tests/metric_tests.rs` |
| **Pattern** | `domainforge-core/src/patterns.rs` | `Pattern`, `Pattern::new()` | Regular expression constraint applied to string fields. | `tests/pattern_semantics_tests.rs` |

---

## 2. In-Memory Graph & Validation

| Subsystem / Capability | Source File | Key Symbols | Responsibility | Test Locations |
|---|---|---|---|---|
| **Graph Store** | `domainforge-core/src/graph/mod.rs` | `Graph`, `GraphConfig`, `Graph::new()`, `Graph::validate()` | Central in-memory store utilizing `IndexMap` for deterministic iteration. | `tests/graph_tests.rs`, `tests/phase_14_determinism_tests.rs` |
| **Entity Instance Validation** | `domainforge-core/src/graph/entity_validation.rs` | `Graph::validate_entity_instances()` | Validates entity instance field values against declared entity contracts. | `tests/entity_instance_validation_tests.rs` |
| **AST to Graph Lowering** | `domainforge-core/src/parser/ast_convert.rs` | `ast_to_graph()`, `ast_to_graph_with_options()` | Transforms parsed AST declarations into semantic primitives in the Graph. | `tests/parser_integration_tests.rs` |
| **Validation Error System** | `domainforge-core/src/validation_error.rs` | `ValidationError`, `ErrorCode`, `Position`, `SourceRange` | Structured diagnostics with error codes (`E001`–`E599`) and fuzzy suggestions. | `tests/phase_15_validation_error_tests.rs` |
| **Validation Result** | `domainforge-core/src/validation_result.rs` | `ValidationResult` | Aggregates policy violations and evaluation outcome. | `tests/validate_tests.rs` |

---

## 3. Language Ingestion, Grammar & Parser

| Component | Source File | Key Symbols | Responsibility | Test Locations |
|---|---|---|---|---|
| **Pest PEG Grammar** | `domainforge-core/grammar/sea.pest` | Rules: `program`, `entity_decl`, `flow_decl`, `policy_decl`, `operation_decl` | Authoritative syntax definition of the SEA DSL. | `tests/parser_tests.rs` |
| **AST Structures** | `domainforge-core/src/parser/ast.rs` | `Ast`, `AstNode`, `Declaration`, `Expression` | Typed in-memory syntax representation preserving source positions. | `tests/parser_ast_v3.rs` |
| **Top-Level Parser API** | `domainforge-core/src/parser/mod.rs` | `parse()`, `parse_to_graph()`, `ParseOptions` | Primary entry points for parsing SEA source code. | `tests/parser_tests.rs` |
| **Module Resolver** | `domainforge-core/src/module/resolver.rs` | `ModuleResolver`, `ModuleInfo`, `ModuleResolver::validate_entry()` | Transitive import closure, cycle detection, and namespace resolution. | `tests/module_resolution_tests.rs`, `tests/cli_module_closure_tests.rs` |
| **Namespace Registry** | `domainforge-core/src/registry/mod.rs` | `NamespaceRegistry`, `NamespaceBinding` | Reads `.sea-registry.toml` mappings for cross-file multi-module projects. | `tests/namespace_registry_tests.rs` |
| **Pretty Printer & Formatter** | `domainforge-core/src/parser/printer.rs` | `PrettyPrinter`, `format_source()`, `check_format()` | Formats SEA code canonically and checks for idempotency. | `tests/printer_tests.rs` |

---

## 4. Policy Engine & Three-Valued Logic

| Component | Source File | Key Symbols | Responsibility | Test Locations |
|---|---|---|---|---|
| **Three-Valued Logic** | `domainforge-core/src/policy/three_valued.rs` | `ThreeValuedBool`, `ThreeValuedBool::{True, False, Null}` | SQL-like Kleene three-valued truth logic and boolean algebraic operators. | `tests/three_valued_quantifiers_tests.rs` |
| **Policy Evaluation** | `domainforge-core/src/policy/core.rs` | `Policy`, `Policy::evaluate()`, `Policy::evaluate_with_mode()` | Evaluates policy expression AST against graph collections. | `tests/policy_tests.rs` |
| **Quantifier Engine** | `domainforge-core/src/policy/quantifier.rs` | `evaluate_quantifier()`, `forall`, `exists`, `exists_unique` | Evaluates collection quantifiers with null-safe handling. | `tests/three_valued_quantifiers_tests.rs` |
| **Expression Normalization** | `domainforge-core/src/policy/normalize.rs` | `normalize_expression()`, `NormalizedExpression` | Simplifies expressions into canonical conjunctive/disjunctive forms. | `tests/policy_arithmetic_where_tests.rs` |
| **Type Inference** | `domainforge-core/src/policy/type_inference.rs` | `infer_expression_type()` | Type checks expression operands and infer return types. | `tests/type_inference_tests.rs` |

---

## 5. Units & Dimensional Analysis

| Component | Source File | Key Symbols | Responsibility | Test Locations |
|---|---|---|---|---|
| **Unit Registry** | `domainforge-core/src/units/mod.rs` | `UnitRegistry`, `Dimension`, `Unit`, `unit_from_string()` | Manages base dimensions and derived units with scaling factors. | `tests/unit_tests.rs`, `tests/dimension_unit_tests.rs` |
| **Quantity Math** | `domainforge-core/src/primitives/quantity.rs` | `Quantity`, `Decimal` operations | Exact decimal arithmetic for physical and financial quantities. | `tests/quantity_tests.rs` |
| **Unit Mismatch Validation** | `domainforge-core/src/validation_error.rs` | `ErrorCode::E003_UnitMismatch`, `E200_DimensionMismatch` | Flags incompatible unit operations during flow and policy validation. | `tests/validation_unit_mismatch_tests.rs` |

---

## 6. Semantic Packs & Governance

| Component | Source File | Key Symbols | Responsibility | Test Locations |
|---|---|---|---|---|
| **Pack Schema** | `domainforge-core/src/semantic_pack/schema.rs` | `SemanticPack`, `ConceptDefinition`, `ApprovalState`, `SignatureState` | Data structure representing the complete frozen vocabulary contract. | `tests/semantic_pack_build.rs` |
| **Pack Builder** | `domainforge-core/src/semantic_pack/builder.rs` | `build_semantic_pack()`, `compute_meaning_fingerprint()` | Extracts concepts from Graph and computes canonical content hashes. | `tests/semantic_pack_build.rs` |
| **Canonical JSON** | `domainforge-core/src/semantic_pack/canonical_json.rs` | `to_canonical_json()` | Deterministic UTF-8 serialization with sorted keys. | `tests/semantic_pack_build.rs` |
| **Cryptographic Signer** | `domainforge-core/src/semantic_pack/signing.rs` | `sign_pack()`, `verify_pack_signature()` | Ed25519 signing and verification over the pack content hash. | `tests/semantic_pack_signing.rs` |
| **Drift & Diff Engine** | `domainforge-core/src/semantic_pack/diff.rs` | `diff_packs()`, `DiffResult`, `SemanticDiffKind` | Detects breaking, additive, and metadata changes between two pack versions. | `tests/semantic_pack_compat_tests.rs` |
| **Pack Validator** | `domainforge-core/src/semantic_pack/validator.rs` | `validate_semantic_pack()`, `validate_graph_with_pack()` | Enforces pack authority rules against working models in CI and LSP. | `tests/semantic_pack_validate.rs` |

---

## 7. Authority Engine & Decision Auditing

| Component | Source File | Key Symbols | Responsibility | Test Locations |
|---|---|---|---|---|
| **Authority Types** | `domainforge-core/src/authority/types.rs` | `FinalDecision`, `PolicyModality`, `ClaimLevel`, `EvidenceItem` | Data models for authoritative policy decisions and claims. | `tests/authority_conformance_tests.rs` |
| **Compiler & Lowering** | `domainforge-core/src/authority/compiler.rs` | `PolicyCompiler`, `LoweredPolicy`, `CompatibilityLoweringAuditor` | Compiles high-level SEA policies into lowered evaluation rules. | `tests/authority_conformance_tests.rs` |
| **Fact Resolver** | `domainforge-core/src/authority/fact_resolver.rs` | `FactResolver`, `FactSourceRegistry` | Resolves runtime context facts required to evaluate authority claims. | `tests/authority_conformance_tests.rs` |
| **Evidence Trace Emitter** | `domainforge-core/src/authority/trace.rs` | `AuthorityTrace`, `AuthorityTraceEmitter`, `EvidenceSink` | Emits structured audit traces detailing every step of decision resolution. | `tests/authority_conformance_tests.rs` |

---

## 8. Application Contracts & Operations (ADR-013)

| Component | Source File | Key Symbols | Responsibility | Test Locations |
|---|---|---|---|---|
| **Application Contract** | `domainforge-core/src/application/contract.rs` | `ApplicationContract`, `OperationContract`, `RecordContract`, `EnumContract` | Formal specification of operations, input/output records, state, and errors. | `tests/application_contract_tests.rs` |
| **Contract Resolver** | `domainforge-core/src/application/resolve.rs` | `resolve_application_contract()`, `resolve_application_graph()` | Resolves modular AST sources into unified application contracts. | `tests/application_contract_tests.rs` |
| **Semantic Envelope** | `domainforge-core/src/application/envelope.rs` | `CanonicalSemanticEnvelope`, `resolve_semantic_envelope()` | Binds application contract, pack closure, and source hash into one artifact. | `tests/application_canonical_tests.rs` |
| **Diagnostics** | `domainforge-core/src/application/diagnostic.rs` | `ApplicationDiagnostic`, codes `APP001`–`APP014` | Precise diagnostics for operation and contract validation failures. | `tests/application_diagnostic_tests.rs` |

---

## 9. Projections Engine & Operator Families

| Target Family | Source Directory / File | CLI Flag | Emitted Format | Test Locations |
|---|---|---|---|---|
| **Shared ID Minting** | `domainforge-core/src/projection/ids.rs` | N/A | Deterministic xxh64 (seed 42) hashing with U+0001 separator. | `src/projection/ids.rs` |
| **Artifact Sink** | `domainforge-core/src/projection/sink.rs` | N/A | Writes to filesystem (`Dir`) or in-memory map (`Memory`). | Unit tests in `sink.rs` |
| **RDF / OWL / KG** | `domainforge-core/src/projection/rdf/`, `src/kg.rs` | `--format rdf`, `--format kg` | Turtle, JSON-LD, OWL Ontology XML, RDF/XML. | `tests/rdf_projection_tests.rs`, `tests/turtle_instance_export_tests.rs` |
| **FINOS CALM** | `domainforge-core/src/calm/` | `--format calm` | FINOS Common Architecture Language Model JSON. | `tests/calm_round_trip_tests.rs` |
| **BPMN 2.0** | `domainforge-core/src/projection/bpmn/` | `--format bpmn` | BPMN 2.0 Process XML (definitions, tasks, flows). | `tests/bpmn_projection_tests.rs` |
| **CMMN 1.1** | `domainforge-core/src/projection/cmmn/` | `--format cmmn` | CMMN 1.1 Case XML (cases, tasks, sentries). | `tests/cmmn_projection_tests.rs` |
| **ArchiMate 3.0** | `domainforge-core/src/projection/archimate/` | `--format archimate` | ArchiMate 3.0 Model Exchange XML. | `tests/archimate_projection_tests.rs` |
| **OpenTelemetry** | `domainforge-core/src/projection/otel/` | `--format otel-semconv` | OpenTelemetry Semantic Convention registries & constants. | `tests/otel_projection_tests.rs` |
| **BAML** | `domainforge-core/src/projection/baml/` | `--format baml` | BAML `.baml` AI prompt templates and schema definitions. | `tests/baml_projection_tests.rs` |
| **DSPy** | `domainforge-core/src/projection/dspy/` | `--format dspy` | DSPy Python programs and signature definitions. | `tests/dspy_projection_tests.rs` |
| **ZenML** | `domainforge-core/src/projection/zenml/` | `--format zenml` | ZenML pipeline definitions and steps in Python. | `tests/zenml_projection_tests.rs` |
| **Lean 4** | `domainforge-core/src/projection/lean/` | `--format lean` | Lean 4 formal Lake packages and inductive definitions. | `tests/lean_projection_tests.rs` |
| **TLA+** | `domainforge-core/src/projection/tla/` | `--format tla` | Formal TLA+ specifications verified with SANY and TLC. | `fixtures/projection_cell/basic/model.sea` |
| **AsyncAPI** | `domainforge-core/src/projection/asyncapi/` | `--format asyncapi` | AsyncAPI 3.0 YAML specification validated against schema. | `tests/asyncapi_spec_validation_tests.rs` |
| **CloudEvents** | `domainforge-core/src/projection/cloudevents/` | `--format cloudevents` | CloudEvents 1.0 JSONL message streams. | `fixtures/projection_cell/basic/model.sea` |
| **Cell Environment** | `domainforge-core/src/projection/cell/` | `--format cell` | Hermetic dev shell with Devbox, Mise, and `cell.lock`. | `tests/cell_projection_tests.rs` |
| **Cedar** | `domainforge-core/src/projection/cedar/` | `--format cedar` | Cedar schema and permissive authorization policies. | `fixtures/projection_cell/basic/model.sea` |
| **Gauge** | `domainforge-core/src/projection/gauge/` | `--format gauge` | Gauge test specifications (one scenario per flow). | `fixtures/projection_cell/basic/model.sea` |
| **Alloy** | `domainforge-core/src/projection/alloy/` | `--format alloy` | Alloy relational models with flow facts. | `fixtures/projection_cell/basic/model.sea` |
| **Domain Code** | `domainforge-core/src/projection/domain/` | `--format domain-python / -typescript / -rust` | Complete DDD packages (aggregates, commands, events, ports). | `scripts/verify/projection-targets/domain-*.sh` |
| **Protocol Buffers** | `domainforge-core/src/projection/protobuf.rs` | `--format protobuf` | Protobuf `.proto` messages and gRPC service definitions. | `tests/protobuf_projection_tests.rs` |

---

## 10. Cross-Language Native Bindings

| Language Target | Core Rust Binding Source | Host Module Location | Key Bridge Functions | Verification Suite |
|---|---|---|---|---|
| **Python (PyO3)** | `domainforge-core/src/python/` | `domainforge-python/` | `Entity`, `Resource`, `Flow`, `Graph`, `evaluate_authority()` | `tests/test_*.py` |
| **TypeScript (napi-rs)** | `domainforge-core/src/typescript/` | `domainforge-typescript/` | `Entity`, `Resource`, `Flow`, `Graph`, `NamespaceRegistry` | `typescript-tests/*.test.ts` |
| **WebAssembly (wasm)** | `domainforge-core/src/wasm/` | In-memory / npm bundle | `WasmGraph`, `parse_to_graph_wasm()`, `evaluate_policy_wasm()` | `domainforge-core/tests/wasm_tests.rs` |

---

## 11. Command-Line Interface (CLI)

| Command | Implementation Source | Arguments & Handlers | Responsibility |
|---|---|---|---|
| `domainforge parse` | `domainforge-core/src/cli/parse.rs` | `ParseArgs`, `run()` | Parses SEA source and prints AST or Graph in human or JSON format. |
| `domainforge validate` | `domainforge-core/src/cli/validate.rs` | `ValidateArgs`, `run()` | Validates models against syntax, schema, units, and policies. |
| `domainforge project` | `domainforge-core/src/cli/project.rs` | `ProjectArgs`, `run()` | Lowers models into any of the 17+ projection targets. |
| `domainforge pack` | `domainforge-core/src/cli/pack.rs` | `PackArgs`, `run()` | Subcommands: `build`, `validate`, `sign`, `diff`. |
| `domainforge authority` | `domainforge-core/src/cli/authority.rs` | `AuthorityArgs`, `run()` | Evaluates authority policies against fact files. |
| `domainforge contract` | `domainforge-core/src/cli/contract.rs` | `ContractArgs`, `run()` | Resolves and prints ADR-013 Application Contract JSON. |
| `domainforge envelope` | `domainforge-core/src/cli/envelope.rs` | `EnvelopeArgs`, `run()` | Resolves and prints Canonical Semantic Envelope JSON. |
| `domainforge format` | `domainforge-core/src/cli/format.rs` | `FormatArgs`, `run()` | Canonical source formatting and idempotent formatting checks. |
| `domainforge normalize` | `domainforge-core/src/cli/normalize.rs` | `NormalizeArgs`, `run()` | Normalizes policy expressions into simplified forms. |

---

## 12. Self-Proving Harness & Proof Artifacts

| Proof Command | Script | Target Claim | Evidence Artifact |
|---|---|---|---|
| `just prove` | `justfile:170` | Full proof ledger execution | `evidence/latest/proof.json`, `proof.md` |
| `prove-language` | `scripts/prove/language.sh` | Parser validity & negative fixture rejection | `evidence/latest/fragments/language.json` |
| `prove-canonical` | `scripts/prove/canonical.sh` | Byte-level projection determinism | `evidence/latest/fragments/canonical.json` |
| `prove-projections` | `scripts/prove/projections.sh` | All projection targets pass native validators | `evidence/latest/fragments/projections.json` |
| `prove-drift` | `scripts/prove/drift.sh` | `pack diff` detects breaking drift accurately | `evidence/latest/fragments/drift.json` |
