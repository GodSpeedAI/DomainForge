---
sds_section: "9. Testing & QA"
diagram_type: "Sequence"
component_ids: ["OPS-PIPE-ReleaseAutomation", "CORE-API-RustCore", "BIND-API-PyO3", "BIND-API-TypeScript", "MIG-ADPT-ERP5Adapter"]
implements_adrs: ["ADR-002", "ADR-005", "ADR-007", "ADR-008"]
satisfies_requirements: ["REQ-005", "REQ-006", "REQ-007", "REQ-009", "REQ-012", "REQ-013", "REQ-015", "REQ-017", "REQ-018", "REQ-019"]
related_diagrams:
  - sds-component-language-parity.md
  - ../06-data/sds-data-migration-strategy.md
  - ../08-deployment/sds-deployment-cicd-pipeline.md
updated: "2025-11-01"
reviewed_by: "QA Team"
purpose: "Shows integration test orchestration across bindings, migration adapters, and parity harness."
---

## Integration Test Orchestration

```mermaid
sequenceDiagram
    %% Source: docs/specs/sds.md SDS-009..SDS-014
    %% Implements: ADR-002, ADR-005, ADR-007, ADR-008
    %% Satisfies: REQ-005, REQ-006, REQ-007, REQ-009, REQ-012, REQ-013, REQ-015, REQ-017, REQ-018, REQ-019
    %% Components: OPS-PIPE-ReleaseAutomation, CORE-API-RustCore, BIND-API-PyO3, BIND-API-TypeScript, MIG-ADPT-ERP5Adapter

    participant CI as OPS-PIPE-ReleaseAutomation
    participant Rust as CORE-API-RustCore
    participant Py as BIND-API-PyO3
    participant Ts as BIND-API-TypeScript
    participant ERP5 as MIG-ADPT-ERP5Adapter

    CI->>Rust: run cargo test --all-features
    Rust-->>CI: success
    CI->>Py: pytest tests/ --baseline fixtures.json
    Py-->>CI: results.json
    CI->>Ts: npm test -- fixtures.json
    Ts-->>CI: results.json
    CI->>Rust: compare parity(python, typescript)
    Rust-->>CI: parity_ok?

    CI->>ERP5: migrate_fixture(sample.erp5)
    ERP5->>Rust: import_model()
    Rust-->>ERP5: model_handle
    ERP5->>Rust: validate()
    Rust-->>ERP5: ValidationResult
    ERP5-->>CI: migration_report

    alt parity mismatch or migration failure
        CI-->>CI: fail pipeline (block release)
    else success
        CI-->>CI: proceed to publish
    end
```

### Design Rationale
- Integration tests chain parity and migration to ensure data correctness before release.
- Fixtures reused to maintain deterministic coverage across languages.

### Related Components
- Migration steps described in [sds-data-migration-strategy](../06-data/sds-data-migration-strategy.md).
