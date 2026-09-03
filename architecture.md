# DomainForge Architecture

The canonical architecture documentation has been unified and published in the documentation knowledge base:

👉 **[Read the Canonical Architecture Documentation (docs/architecture.md)](docs/architecture.md)**

### Quick Summary
- **Canonical Rust Core**: Pure, deterministic Rust implementation in `domainforge-core`.
- **Zero-Business-Logic Bindings**: Python (PyO3), TypeScript (napi-rs), and Browser (wasm-bindgen) wrap the Rust core directly.
- **Deterministic Storage**: `IndexMap` guarantees byte-identical projections across isolated runs.
- **Three-Valued Policy Engine**: SQL-like `True` / `False` / `Null` logic for enterprise rules and obligations.
- **Operator Families**: Projections into 17+ ecosystems (RDF/OWL, FINOS CALM, BPMN, CMMN, ArchiMate, OTel, BAML, DSPy, ZenML, Lean 4, TLA+, AsyncAPI, CloudEvents, Cedar, and DDD code).
- **Cryptographic Vocabulary Governance**: Ed25519-signed Semantic Packs for organization-wide terminology contracts.

See also:
- [Documentation Map](docs/documentation-map.md)
- [Source Map](docs/source-map.md)
- [Subsystem Guides](docs/subsystems/graph-store.md)
