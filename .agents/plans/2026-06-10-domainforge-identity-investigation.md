# DomainForge Identity Investigation Plan

**Goal:** Determine what DomainForge is from repository evidence, then write `.agent/reports/domainforge_identity_report.md`.

**Architecture:** Treat documentation and names as hypotheses. Use manifests, source modules, public APIs, parser/projection code, bindings, and tests as primary evidence. Keep findings adversarial by testing each possible classification against implemented behavior.

**Tech Stack:** Rust workspace, Pest grammar, PyO3 Python bindings, napi-rs TypeScript bindings, wasm-bindgen, Python/TypeScript integration tests.

## Tasks

1. Repository census
   - Identify crates/packages, entry points, public APIs, core data structures, storage, parsers, interpreters, generators, and projection systems.
   - Evidence: file paths, module declarations, manifests, CLI definitions, tests.

2. Semantic center of gravity
   - Trace major workflows from inputs to internal representations to outputs.
   - Determine which representation behaves as source of truth in code.

3. Language/protocol/projection analysis
   - Classify `.sea` by implemented capabilities.
   - Evaluate protocol criteria explicitly.
   - Inventory implemented projections and bindings.

4. Architectural comparison and falsification
   - Compare against Protobuf, Smithy, OpenAPI, GraphQL, Terraform, Kubernetes CRDs, OWL, RDF, UML, XSD, CUE, and Dhall.
   - Argue against DSL, protocol, ontology engine, compiler, projection engine, schema system, and knowledge representation classifications.

5. Report and handoff
   - Write `.agent/reports/domainforge_identity_report.md`.
   - Update `.agents/current_state.md` and `.agents/next_steps.md` with investigation evidence.
