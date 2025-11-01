---
sds_section: "5. Interface & API Design"
diagram_type: "Documentation"
component_ids: ["CORE-API-RustCore", "BIND-API-PyO3", "BIND-API-TypeScript", "BIND-API-WebAssembly", "VAL-SVC-PolicyEvaluator"]
implements_adrs: ["ADR-002", "ADR-004", "ADR-007"]
satisfies_requirements: ["REQ-003", "REQ-005", "REQ-006", "REQ-007", "REQ-008", "REQ-012", "REQ-013", "REQ-015", "REQ-017"]
related_diagrams:
  - sds-api-model-definition.md
  - sds-api-validation-streaming.md
  - sds-api-ffi-error-handling.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Lists API-level sequence diagrams covering definition, validation, and error handling workflows."
---

## Interface Assets

- `sds-api-model-definition.md`: Sequence diagram for defining primitives via Rust core API.
- `sds-api-validation-streaming.md`: Sequence for synchronous + streaming validation flows.
- `sds-api-ffi-error-handling.md`: Sequence of cross-language error propagation and mitigation.
