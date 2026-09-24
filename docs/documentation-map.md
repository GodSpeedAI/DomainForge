# DomainForge Documentation Map

This document catalogs every page in the DomainForge documentation system. It provides a navigational index categorized by reader intent, Diátaxis quadrant, prerequisite knowledge, and related pages.

---

## 1. Documentation Index by Layer and Diátaxis

| Page | Diátaxis / Purpose | Primary Audience Need | Prerequisites | Related Pages |
|---|---|---|---|---|
| **[Documentation Portal](index.md)** | Orientation / Hub | "Where do I start?" Overview of the knowledge system. | None | [Orientation](orientation.md), [Mental Model](mental-model.md) |
| **[Orientation](orientation.md)** | Orientation | Fast 5-minute mental orientation, first journey, and reading tracks. | None | [Mental Model](mental-model.md), [First SEA Model](tutorials/01-first-sea-model.md) |
| **[Mental Model](mental-model.md)** | Mental Model | Understand core abstractions (Entity, Resource, Flow, Policy) without code clutter. | [Orientation](orientation.md) | [Architecture](architecture.md), [Graph Store](subsystems/graph-store.md) |
| **[System Architecture](architecture.md)** | Architecture Overview | Understand logical, runtime, dependency, and data architecture across components. | [Mental Model](mental-model.md) | [Subsystems](subsystems/parser-grammar.md), [Source Map](source-map.md) |
| **[Source Map](source-map.md)** | Source Map / Traceability | Find where concepts, types, and capabilities live in the repository codebase. | None | [Architecture](architecture.md), [Subsystems](subsystems/parser-grammar.md) |
| **[Troubleshooting Guide](troubleshooting.md)** | Operational / Runbook | Diagnose and recover from parse failures, policy violations, and pack drift. | None | [CLI Reference](reference/cli-reference.md), [Error Code Reference](reference/error-code-reference.md) |

---

## 2. Subsystem Guides (Layer 3 — Architecture & Implementation)

Subsystem guides explain why each component exists, what it owns, its internal mechanism, state invariants, failure modes, and implementation source trails.

| Subsystem Page | Primary Audience Need | Implementation Focus | Prerequisites |
|---|---|---|---|
| **[Parser & Grammar](subsystems/parser-grammar.md)** | "How does SEA text become an AST?" | Pest PEG parser, AST nodes, span tracking, and linter. | [Mental Model](mental-model.md) |
| **[Graph Store](subsystems/graph-store.md)** | "How is domain state held and queried?" | In-memory relational store, `IndexMap` determinism, `ConceptId` UUID v5. | [Mental Model](mental-model.md) |
| **[Policy Engine](subsystems/policy-engine.md)** | "How are business rules evaluated?" | Three-valued logic (`True`/`False`/`Null`), quantifiers, expression trees. | [Graph Store](subsystems/graph-store.md) |
| **[Units & Dimensions](subsystems/units-dimensions.md)** | "How does dimensional checking work?" | Unit registry, conversion factors, dimensional equivalence checks. | [Graph Store](subsystems/graph-store.md) |
| **[Semantic Packs](subsystems/semantic-packs.md)** | "How is organizational vocabulary governed?" | Pack schemas, canonical JSON, content hashing, Ed25519 detached signing. | [Architecture](architecture.md) |
| **[Authority Engine](subsystems/authority-engine.md)** | "How are access and compliance policies audited?" | Fact resolution, policy lowering, evidence traces, and execution environments. | [Policy Engine](subsystems/policy-engine.md) |
| **[Application Contracts](subsystems/application-contracts.md)** | "How are executable operations and state modeled?" | ADR-013 operations, records, enums, idempotency, and semantic envelopes. | [Graph Store](subsystems/graph-store.md) |
| **[Projections Engine](subsystems/projections-engine.md)** | "How does DomainForge export to 17+ targets?" | Operator family pattern (ADR-011), `ArtifactSink`, deterministic xxh64 hashing. | [Graph Store](subsystems/graph-store.md) |
| **[Language Bindings](subsystems/language-bindings.md)** | "How do Python, TypeScript, and WASM use the core?" | PyO3, napi-rs, wasm-bindgen wrapping layers, memory ownership, zero-copy rules. | [Architecture](architecture.md) |

---

## 3. Workflows & Execution Traces (Layer 4 — Dynamic Execution)

Workflows trace representative operations end to end through the system, detailing function call sequences, state transitions, and error branches.

| Workflow Page | Workflow Traced | Entry Point | Key Result |
|---|---|---|---|
| **[Parse and Validate](workflows/parse-and-validate.md)** | Parsing, transitive module resolution, and policy validation. | `domainforge validate` or `parse_to_graph()` | Validated `Graph` or structured diagnostics |
| **[Projection Generation](workflows/projection-generation.md)** | Model lowering into target-specific ASTs and file output. | `domainforge project --format <target>` | Generated artifacts via `ArtifactSink` |
| **[Pack Lifecycle](workflows/pack-lifecycle.md)** | Pack building, review verification, signing, and diffing. | `domainforge pack build / sign / diff` | Cryptographically signed `SemanticPack` JSON |
| **[Application Resolution](workflows/application-resolution.md)** | Resolving modular sources into canonical application envelopes. | `domainforge contract` / `envelope` | Canonical contract and envelope documents |
| **[Authority Evaluation](workflows/authority-evaluation.md)** | Evaluating runtime facts against authority policies. | `domainforge authority` / `AuthorityResolver` | `FinalDecision` and structured `AuthorityTrace` |

---

## 4. Explanations (Layer 5 — Diátaxis: Understanding)

Explanations address "Why?" questions, exploring design trade-offs, historical context, rejected alternatives, and invariants.

| Explanation Page | Topic & Core Question Addressed | Key Takeaway |
|---|---|---|
| **[Canonical Semantic Core](explanations/canonical-semantic-core.md)** | Why implement business logic exclusively in Rust? | Single source of truth across Python, TypeScript, and WASM without logic divergence. |
| **[IndexMap Determinism](explanations/indexmap-determinism.md)** | Why is `std::collections::HashMap` strictly banned? | Deterministic iteration order guarantees byte-identical projection outputs. |
| **[Three-Valued Logic Rationale](explanations/three-valued-logic-rationale.md)** | Why SQL-like `UNKNOWN` instead of strict boolean logic? | Distinguishes missing information from explicit rule failure in distributed enterprise domains. |
| **[ADR-013 Application Contract](explanations/adr-013-application-contract.md)** | Why add operations, typed records, and state contracts? | Eliminates informal prose and guessing; provides machine-checkable code generation boundaries. |
| **[Operator Family Design](explanations/operator-family-design.md)** | Why decouple projections as operator families (ADR-011)? | Allows expanding to dozens of external toolchains while reusing a common ID and sink kernel. |

---

## 5. Tutorials (Layer 6 — Diátaxis: Learning)

Tutorials guide newcomers through structured, successful hands-on experiences with real project tools.

| Tutorial | Learning Goal | Prerequisites | Time to Complete |
|---|---|---|---|
| **[First SEA Model](tutorials/01-first-sea-model.md)** | Write, parse, and validate a basic procurement domain in SEA DSL. | CLI installed | 10 minutes |
| **[Multi-Target Projections](tutorials/02-multi-target-projection.md)** | Project a SEA model into Python DDD code and BPMN 2.0 XML diagrams. | [First SEA Model](tutorials/01-first-sea-model.md) | 15 minutes |
| **[Building & Signing Packs](tutorials/03-building-signing-packs.md)** | Extract domain vocabulary, inspect definitions, and sign a semantic pack. | CLI installed | 15 minutes |

---

## 6. How-To Guides (Layer 7 — Diátaxis: Problem Solving)

How-to guides provide recipes for specific, real-world tasks maintainers and power users need to accomplish.

| How-To Guide | Goal | Primary Files Involved |
|---|---|---|
| **[Add a Projection Target](how-tos/add-projection-target.md)** | Add a new target format following the ADR-011 operator family pattern. | `src/projection/<family>/`, `src/cli/project.rs` |
| **[Add a Grammar Construct](how-tos/add-grammar-construct.md)** | Extend SEA syntax following the strict grammar-first workflow. | `grammar/sea.pest`, `src/parser/ast.rs` |
| **[Configure Module Resolution](how-tos/configure-module-resolution.md)** | Organize multi-file enterprise projects with `.sea-registry.toml`. | `.sea-registry.toml`, `src/module/resolver.rs` |
| **[Debug Policy Evaluations](how-tos/debug-policy-evaluations.md)** | Diagnose why a policy evaluated to `False` or `Null` (Unknown). | `src/policy/core.rs`, `domainforge normalize` |

---

## 7. Technical Reference (Layer 8 — Diátaxis: Information)

Reference documentation prioritizes precision, completeness, and exact contracts over narrative.

| Reference Page | Content | Format |
|---|---|---|
| **[DSL Grammar Reference](reference/dsl-grammar-reference.md)** | Exhaustive rule catalog for every keyword, declaration, and expression. | Grammar rules & syntax tables |
| **[CLI Reference](reference/cli-reference.md)** | All subcommands, options, exit codes, and environment variables. | Command flags & usage tables |
| **[Error Code Catalog](reference/error-code-reference.md)** | Complete listing of `E001`–`E599` and `APP001`–`APP014` error codes. | Code, description, cause, fix |
| **[Primitives API Reference](reference/primitives-api-reference.md)** | Rust core data types (`Entity`, `Resource`, `Flow`, `Instance`, `Policy`). | Struct fields, methods, types |
| **[Configuration Reference](reference/configuration-reference.md)** | Manifest specifications (`Cargo.toml`, `.sea-registry.toml`, Devbox). | Schema definitions & keys |

---

## 8. Reader Navigation Tracks

Depending on your immediate objective, follow these curated reading paths:

- **Track A: "I am new and want to understand what this does"**
  1. [Documentation Portal](index.md)
  2. [Orientation](orientation.md)
  3. [Mental Model](mental-model.md)
  4. [First SEA Model Tutorial](tutorials/01-first-sea-model.md)

- **Track B: "I am an engineer preparing to modify or extend the system"**
  1. [System Architecture](architecture.md)
  2. [Source Map](source-map.md)
  3. [Graph Store Subsystem](subsystems/graph-store.md)
  4. [Subsystem Guides](subsystems/parser-grammar.md)
  5. [How-To: Add a Projection Target](how-tos/add-projection-target.md)

- **Track C: "I need to understand why decisions were made"**
  1. [Canonical Semantic Core Explanation](explanations/canonical-semantic-core.md)
  2. [IndexMap Determinism Explanation](explanations/indexmap-determinism.md)
  3. [Three-Valued Logic Explanation](explanations/three-valued-logic-rationale.md)
  4. [ADR-013 Application Contract Explanation](explanations/adr-013-application-contract.md)

- **Track D: "I am debugging an issue or validating output"**
  1. [Troubleshooting Guide](troubleshooting.md)
  2. [Error Code Catalog](reference/error-code-reference.md)
  3. [CLI Reference](reference/cli-reference.md)
  4. [Proof Ledger](../PROOFS.md)
