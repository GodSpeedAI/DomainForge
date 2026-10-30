# schemas — JSON Schema and Vendored Specification Assets

## Overview
A data-only directory: JSON Schema documents describing DomainForge's serialized
interchange formats, plus vendored third-party specification artifacts (XSD and
JSON Schema) used as validation references. The only directory here containing
executable source is `baml/`, which ships a checker script.

Direct `.json` schema data (no EGM contract — these are data, not source):
- `application-contract-v1.schema.json` — draft-07 schema, `$id`
  `https://domainforge.dev/schemas/application-contract-v1.schema.json`, title
  "DomainForge Application Contract Document"; loaded by path in
  `domainforge-core/tests/application_schema_tests.rs`.
- `canonical-semantic-envelope-v1.schema.json` — draft-07 schema, `$id`
  `https://domainforge.dev/schemas/canonical-semantic-envelope-v1.schema.json`,
  title "DomainForge Canonical Semantic Envelope Document"; loaded by path in
  `domainforge-core/tests/application_schema_tests.rs`.
- `ast-v1.schema.json` — draft-07, title "SEA DSL Abstract Syntax Tree".
- `ast-v2.schema.json` — draft-07, title "Ast".
- `ast-v3.schema.json` — draft-07, title "Ast"; the live AST contract, emitted by
  `[domainforge-core/src/parser/ast_schema.rs:generate_ast_schema]` via
  `schema_for!(Ast)` and referenced by
  `[domainforge-core/src/python/graph.rs]` and
  `[domainforge-core/src/wasm/graph.rs]` as the `parse --ast` output shape.
- `sea-registry.schema.json` — title "SEA Registry Schema"; validated against a
  sample by `[domainforge-core/tests/registry_schema_tests.rs:registry_schema_validates_sample]`.

## Subdirectories & Boundaries
- baml: `./baml/EGM.md` — Python structural well-formedness checker for generated
  `.baml` source, plus its rationale doc.
- archimate: no EGM contract (vendored XSDs only — `archimate3_Diagram.xsd`,
  `archimate3_Model.xsd`, `archimate3_View.xsd`, `xml.xsd`, `VENDORED.md`); no
  executable source.
- asyncapi: no EGM contract (vendored `3.0.0.json` AsyncAPI 3.0 schema plus
  `VENDORED.md`); consumed by `asyncapi_spec_validation_tests`, referenced from
  `scripts/verify/projection-targets/asyncapi.sh`.
- bpmn: no EGM contract (vendored `BPMN20.xsd`, `BPMNDI.xsd`, `DC.xsd`, `DI.xsd`,
  `Semantic.xsd`, `VENDORED.md`); no executable source.
- cep_eval: no EGM contract (`answer_key.schema.json`,
  `benchmark_case.schema.json`, `dataset.schema.json`); JSON data only.
- cmmn: no EGM contract (vendored `CMMN11.xsd`, `CMMN11CaseModel.xsd`,
  `CMMNDI11.xsd`, `DC.xsd`, `DI.xsd`, `VENDORED.md`); no executable source.
- otel: no EGM contract (`semconv_registry.schema.json`, `VENDORED.md`); JSON data
  only.

## Verification
- Run: `cargo test -p domainforge-core --features cli --test registry_schema_tests`
- Run: `cargo test -p domainforge-core --features cli --test application_schema_tests`
