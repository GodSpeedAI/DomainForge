---
sds_section: "8. Deployment & Operations"
diagram_type: "C4Deployment"
component_ids: ["CORE-API-RustCore", "BIND-API-PyO3", "BIND-API-TypeScript", "BIND-API-WebAssembly", "OPS-PIPE-ReleaseAutomation", "DOC-PIPE-DocGenerator"]
implements_adrs: ["ADR-002", "ADR-007", "ADR-008"]
satisfies_requirements: ["REQ-005", "REQ-006", "REQ-007", "REQ-008", "REQ-015", "REQ-016", "REQ-018", "REQ-019"]
related_diagrams:
  - sds-deployment-cicd-pipeline.md
  - sds-deployment-scaling-strategy.md
  - sds-deployment-observability.md
updated: "2025-11-01"
reviewed_by: "DevOps Team"
purpose: "Depicts build and runtime infrastructure for SEA DSL, including CI/CD nodes and artifact destinations."
---

## Deployment Topology

```mermaid
C4Deployment
    %% Source: docs/specs/sds.md SDS-008..SDS-015
    %% Implements: ADR-002, ADR-007, ADR-008
    %% Satisfies: REQ-005, REQ-006, REQ-007, REQ-008, REQ-015, REQ-016, REQ-018, REQ-019
    %% Components: CORE-API-RustCore, BIND-API-PyO3, BIND-API-TypeScript, BIND-API-WebAssembly, OPS-PIPE-ReleaseAutomation, DOC-PIPE-DocGenerator

    AddDeploymentNodeTag("ControlPlane", $bgColor="#ecfdf5", $borderColor="#047857", $fontColor="#065f46")
    AddDeploymentNodeTag("Runner", $bgColor="#dbeafe", $borderColor="#1d4ed8", $fontColor="#1e3a8a")
    AddDeploymentNodeTag("Registry", $bgColor="#fff7ed", $borderColor="#f97316", $fontColor="#7c2d12")

    Deployment_Node(control, "GitHub Actions Control Plane", "SaaS", $tags="ControlPlane") {
        Deployment_Node(runners, "Self-hosted Runners", "Ubuntu/macOS/Windows", $tags="Runner") {
            Container(ci_pipeline, "OPS-PIPE-ReleaseAutomation", "GitHub Workflow", "Build/test/publish orchestrator")
            Container(doc_pipeline, "DOC-PIPE-DocGenerator", "Docs workflow", "Generates docs")
        }
    }

    Deployment_Node(cargo, "crates.io", "SaaS Registry", $tags="Registry") {
        Container(rust_pkg, "sea-core crate", "Rust package", "CORE-API-RustCore distribution")
    }

    Deployment_Node(pypi, "PyPI", "SaaS Registry", $tags="Registry") {
        Container(py_pkg, "sea-python wheel", "Python package", "BIND-API-PyO3 distribution")
    }

    Deployment_Node(npm, "npm Registry", "SaaS Registry", $tags="Registry") {
        Container(ts_pkg, "sea-ts package", "npm package", "BIND-API-TypeScript distribution")
    }

    Deployment_Node(cdn, "Edge CDN", "Cloudflare/Workers", $tags="Runner") {
        Container(wasm_pkg, "sea-wasm module", "WASM package", "BIND-API-WebAssembly distribution")
    }

    Rel(ci_pipeline, rust_pkg, "cargo publish", "OIDC token")
    Rel(ci_pipeline, py_pkg, "maturin publish", "PyPI token")
    Rel(ci_pipeline, ts_pkg, "npm publish", "npm token")
    Rel(ci_pipeline, wasm_pkg, "upload artifacts", "API key")
    Rel(doc_pipeline, cdn, "Publish documentation", "Static site deploy")
```

### Design Rationale
- Self-hosted runners cover target architectures enumerated in SDS-014.
- Separate doc pipeline ensures REQ-019 documentation updates per release.

### Related Components
- Detailed pipeline steps are available in [sds-deployment-cicd-pipeline](sds-deployment-cicd-pipeline.md).
