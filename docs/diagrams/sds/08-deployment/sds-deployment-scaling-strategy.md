---
sds_section: "8. Deployment & Operations"
diagram_type: "Flowchart"
component_ids: ["CORE-API-RustCore", "BIND-API-WebAssembly", "OPS-PIPE-ReleaseAutomation"]
implements_adrs: ["ADR-002", "ADR-007", "ADR-008"]
satisfies_requirements: ["REQ-005", "REQ-008", "REQ-015", "REQ-016", "REQ-018"]
related_diagrams:
  - sds-deployment-topology.md
  - sds-deployment-observability.md
  - ../03-architecture/sds-architecture-e2e-flow.md
updated: "2025-11-01"
reviewed_by: "DevOps Team"
purpose: "Summarises scaling strategy and failover decisions for core validation workloads."
---

## Scaling & Failover Strategy

```mermaid
graph TD
    %% Source: docs/specs/sds.md SDS-006, SDS-011, SDS-014
    %% Implements: ADR-002, ADR-007, ADR-008
    %% Satisfies: REQ-005, REQ-008, REQ-015, REQ-016, REQ-018
    %% Components: CORE-API-RustCore, BIND-API-WebAssembly, OPS-PIPE-ReleaseAutomation

    Demand[Incoming Validation Demand]
    Branch{Runtime?}
    RustCore["CORE-API-RustCore (Server)"]
    WASMEdge["BIND-API-WebAssembly (Edge)"]
    ScaleRust{CPU > 70%?}
    AddRunner["Provision new runner"]
    ScaleWASM{Latency > 50ms?}
    DeployEdge["Deploy additional edge workers"]
    Benchmarks["Performance Benchmarks"]
    Alert["Raise Alert"]
    AutoTune["Adjust validation depth / chunk"]

    Demand --> Branch
    Branch -- Server --> RustCore
    Branch -- Edge --> WASMEdge

    RustCore --> ScaleRust
    ScaleRust -- Yes --> AddRunner
    ScaleRust -- No --> Benchmarks
    AddRunner --> Benchmarks

    WASMEdge --> ScaleWASM
    ScaleWASM -- Yes --> DeployEdge
    ScaleWASM -- No --> Benchmarks
    DeployEdge --> Benchmarks

    Benchmarks --> AutoTune
    AutoTune --> RustCore
    AutoTune --> WASMEdge
    Benchmarks --> Alert
```

### Design Rationale
- Dual-mode scaling: server runners and edge deployments.
- Benchmarks feed auto-tuning to maintain REQ-018 performance thresholds.

### Related Components
- Observability metrics feeding auto-tuning described in [sds-deployment-observability](sds-deployment-observability.md).
