# DomainForge Interactive Explainers

This directory contains standalone, zero-dependency interactive HTML explainers for DomainForge. Each explainer is a single, self-contained file with embedded styles, scripts, and domain models. They require no build step, no Node.js runtime, no package installation, and no internet access. You can open them directly via `file://` in any modern web browser.

Use these interactive representations when static diagrams or prose require too much mental simulation. They allow you to manipulate parameters, inspect intermediate compiler representations, step through three-valued policy evaluations, and explore all 29 target projection formats.

---

## Explainer Catalog

| Explainer | Reader Question Addressed | Form & Mechanism | Key Cognitive Affordance | Source Anchors |
|---|---|---|---|---|
| **[Projection Compiler Explorer](projection-compiler-explorer.html)** | *How does DomainForge compile `.sea` source into an in-memory semantic graph, and project it into 29 distinct target ecosystems while preserving byte-identical determinism?* | Interactive pipeline stepper & 29-target projection matrix comparator | Directly inspect intermediate representations (Pest CST, `IndexMap` Graph, Domain IR, `ArtifactSink`), switch between 3 real-world scenarios, toggle `--created-at` fixed timestamps to observe `xxh64` hash fixpoints, and preview generated output artifacts across all 9 operator families. | [`src/graph/mod.rs`](../../domainforge-core/src/graph/mod.rs), [`src/projection/ids.rs`](../../domainforge-core/src/projection/ids.rs), [`src/projection/sink.rs`](../../domainforge-core/src/projection/sink.rs), [`docs/projection-families.md`](../projection-families.md) |
| **[Three-Valued Policy Simulator](three-valued-policy-simulator.html)** | *Why does DomainForge use Kleene three-valued logic (`True`, `False`, `Null`), and how do quantifiers (`forall`, `exists`) and expression normalization prevent false-positive authorization grants on incomplete data?* | Interactive 3VL logic laboratory, collection quantifier stepper & AST normalizer | Test Kleene truth algebra tables with live operands, toggle field presence on collection instances to watch `fold_and` and `fold_or` accumulate `Null` states, step through De Morgan/annihilation AST rewrites, and observe the `--allow-unknown` CLI exit code switch. | [`src/policy/three_valued.rs`](../../domainforge-core/src/policy/three_valued.rs), [`src/policy/quantifier.rs`](../../domainforge-core/src/policy/quantifier.rs), [`src/policy/normalize.rs`](../../domainforge-core/src/policy/normalize.rs), [`docs/subsystems/policy-engine.md`](../subsystems/policy-engine.md) |

---

## Technical Architecture & Boundary Principles

These explainers adhere strictly to the **Repository Cognitive Representation Architecture** principles:

1. **Zero Runtime Dependencies**: No CDNs (`unpkg`, `cdnjs`), external web fonts, or remote assets. All CSS and JavaScript are strictly local and inline.
2. **Deterministic & Offline**: Directly openable from local disk via `file:///path/to/docs/explainers/<name>.html`.
3. **High-Fidelity Models**: The underlying data structures, token names, Kleene truth tables, quantifier short-circuiting rules, and projection IDs accurately match `domainforge-core` Rust implementations.
4. **Non-Invasive Documentation**: These tools serve comprehension and operational inspection; they never mutate repository state, alter build configurations, or execute side-effecting code.
