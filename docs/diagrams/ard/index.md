---
generated: 2025-11-01
---

# ADR Diagram Master Index

| ADR | Decision Topic | Diagram Types Covered | Key Insights Visualized |
|-----|----------------|-----------------------|-------------------------|
| [ADR-001](ADR-001/README.md) | Layered architecture for vocabulary, facts, rules | C4Context, Flowchart, C4Container, C4Component, Sequence | Clarifies layer responsibilities, dependency enforcement, and ripple effects across tooling |
| [ADR-002](ADR-002/README.md) | Rust core with multi-language bindings | C4Context, Flowchart, C4Container, C4Component, Sequence | Demonstrates shared Rust runtime, build pipeline coverage, and governance for synchronized releases |
| [ADR-003](ADR-003/README.md) | Graph-based domain model representation | C4Context, Flowchart, C4Container, C4Component, Sequence | Highlights typed multigraph structure, traversal mechanics, and quality safeguards |
| [ADR-004](ADR-004/README.md) | SBVR-aligned rule expression language | C4Context, Flowchart, C4Container, C4Component, Sequence | Shows SBVR semantic layers, authoring pipeline, and compliance workflow integration |
| [ADR-005](ADR-005/README.md) | Five core primitives design pattern | C4Context, Flowchart, C4Container, C4Component, Sequence | Outlines primitive roles, extensible runtime components, and governance of the primitive library |
| [ADR-006](ADR-006/README.md) | CALM interoperability for architecture-as-code | C4Context, Flowchart, C4Container, C4Component, Sequence | Maps CALM mapping components, round-trip validation, and integration network dependencies |
| [ADR-007](ADR-007/README.md) | Idiomatic language-specific bindings | C4Context, Flowchart, C4Container, C4Component, Sequence | Depicts language-specific facades, DX enforcement tooling, and adoption impacts |
| [ADR-008](ADR-008/README.md) | Lockstep versioning strategy | C4Context, Flowchart, C4Container, C4Component, Sequence | Illustrates version manifest governance, synchronized pipelines, and distribution alignment |

All diagrams embed traceability comments (`%% Source`, `%% Section`, `%% Generated`) and adhere to Mermaid syntax limits (≤15 nodes). Each C4 diagram includes an `!include` hint for future PlantUML interoperability.
