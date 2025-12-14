# SDS-002: SEA Core Architecture

**System:** DomainForge  
**Component:** sea-core  
**Version:** 0.2.1  
**Date:** 2025-12-14  
**Status:** Implemented

---

## 1. Overview

`sea-core` is the Rust implementation of the SEA Domain-Specific Language. It provides:

- **Parsing**: Convert `.sea` source text to AST and semantic graph
- **Validation**: Enforce semantic rules and constraints
- **Policy Evaluation**: Execute business logic policies
- **Projection**: Export to external formats (JSON, YAML, DDD, Protobuf)
- **Multi-language Bindings**: Python, TypeScript, WASM

---

## 2. High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         sea-core                                │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────────┐   │
│  │  Parser  │→ │   AST    │→ │  Graph   │→ │  Projection   │   │
│  │  (pest)  │  │          │  │          │  │    Engine     │   │
│  └──────────┘  └──────────┘  └──────────┘  └───────────────┘   │
│        ↓                          ↓                              │
│  ┌──────────┐              ┌──────────┐                          │
│  │  Linter  │              │  Policy  │                          │
│  │          │              │ Evaluator│                          │
│  └──────────┘              └──────────┘                          │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────────┐   │
│  │  Python  │  │   Node   │  │   WASM   │  │     CLI       │   │
│  │ Bindings │  │ Bindings │  │ Bindings │  │    (clap)     │   │
│  └──────────┘  └──────────┘  └──────────┘  └───────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Module Structure

```
sea-core/src/
├── lib.rs              # Public API and feature-gated re-exports
├── bin/sea.rs          # CLI entry point
│
├── parser/             # DSL parsing
│   ├── mod.rs          # Parser API
│   ├── ast.rs          # AST types and construction
│   ├── error.rs        # Parse errors
│   ├── lint.rs         # Linting rules
│   ├── printer.rs      # Pretty printer
│   └── profiles.rs     # Profile enforcement
│
├── grammar/            # Pest grammar
│   └── sea.pest        # SEA grammar definition
│
├── primitives/         # Core domain types
│   ├── entity.rs       # Entity primitive
│   ├── resource.rs     # Resource primitive
│   ├── flow.rs         # Flow primitive
│   ├── instance.rs     # Instance primitive
│   ├── role.rs         # Role primitive
│   ├── relation.rs     # Relation types
│   ├── policy.rs       # Policy primitive
│   ├── metric.rs       # Metric primitive
│   ├── concept_change.rs  # Temporal changes
│   ├── mapping_contract.rs
│   └── projection_contract.rs
│
├── graph/              # Semantic graph
│   ├── mod.rs          # Graph structure
│   └── to_ast.rs       # Graph → AST conversion
│
├── policy/             # Policy evaluation
│   ├── mod.rs          # Policy engine
│   ├── expression.rs   # Expression evaluation
│   └── three_valued.rs # Three-valued logic
│
├── projection/         # Output generation
│   ├── mod.rs          # Projection API
│   ├── contracts.rs    # Mapping/projection contracts
│   ├── engine.rs       # Projection executor
│   └── registry.rs     # Projection registry
│
├── registry/           # Namespace management
│   └── mod.rs          # NamespaceRegistry
│
├── module/             # Module resolution
│   └── resolver.rs     # Import handling
│
├── error/              # Error handling
│   ├── mod.rs          # Error types
│   ├── diagnostics.rs  # Formatters
│   └── fuzzy.rs        # Did-you-mean suggestions
│
├── cli/                # CLI commands
│   ├── mod.rs          # Command definitions
│   ├── validate.rs
│   ├── project.rs
│   ├── format.rs
│   ├── import.rs
│   └── test.rs
│
├── python/             # Python bindings (feature: python)
├── typescript/         # Node.js bindings (feature: typescript)
└── wasm/               # WASM bindings (feature: wasm)
```

---

## 4. Core Types

### 4.1 Semantic Primitives

| Type            | Description                                           |
| --------------- | ----------------------------------------------------- |
| `Entity`        | Domain concept with attributes and optional relations |
| `Resource`      | Quantifiable items that flow between entities         |
| `Flow`          | Movement of resources between entities                |
| `Instance`      | Concrete instantiation of an entity with values       |
| `Role`          | Named capability assigned to entities                 |
| `Relation`      | Typed relationship between entities                   |
| `Policy`        | Executable business rule with expression              |
| `Metric`        | Observable measurement with thresholds                |
| `ConceptChange` | Temporal evolution of a concept                       |

### 4.2 Graph

```rust
pub struct Graph {
    entities: IndexMap<ConceptId, Entity>,
    roles: IndexMap<ConceptId, Role>,
    resources: IndexMap<ConceptId, Resource>,
    flows: IndexMap<ConceptId, Flow>,
    relations: IndexMap<ConceptId, RelationType>,
    instances: IndexMap<ConceptId, ResourceInstance>,
    entity_instances: IndexMap<ConceptId, Instance>,
    policies: IndexMap<ConceptId, Policy>,
    patterns: IndexMap<ConceptId, Pattern>,
    concept_changes: IndexMap<ConceptId, ConceptChange>,
    metrics: IndexMap<ConceptId, Metric>,
    mappings: IndexMap<ConceptId, MappingContract>,
    projections: IndexMap<ConceptId, ProjectionContract>,
    config: GraphConfig,
}
```

### 4.3 ConceptId

Stable, content-addressable identifier for all graph nodes:

```rust
pub struct ConceptId {
    namespace: String,
    name: String,
    uuid: Uuid,  // v5 deterministic from namespace:name
}
```

---

## 5. Data Flow

### 5.1 Parse Pipeline

```
Source Text (.sea)
       │
       ▼
┌─────────────────┐
│   pest Parser   │  ← sea.pest grammar
└────────┬────────┘
         │ Pairs
         ▼
┌─────────────────┐
│   AST Builder   │  ← ast.rs
└────────┬────────┘
         │ Ast
         ▼
┌─────────────────┐
│ Graph Builder   │  ← ast_to_graph()
└────────┬────────┘
         │ Graph
         ▼
┌─────────────────┐
│   Validation    │  ← Reference checks, type checks
└────────┬────────┘
         │ Result<Graph>
         ▼
    Ready for use
```

### 5.2 Policy Evaluation

```rust
// Three-valued logic for null-safe evaluation
enum TruthValue {
    True,
    False,
    Unknown,  // NULL propagation
}

impl Graph {
    pub fn evaluate_policies(&self) -> Vec<Violation> {
        self.policies
            .values()
            .flat_map(|policy| policy.evaluate(self))
            .collect()
    }
}
```

---

## 6. Feature Flags

| Feature              | Dependencies      | Description                  |
| -------------------- | ----------------- | ---------------------------- |
| `python`             | pyo3, pythonize   | Python bindings              |
| `typescript`         | napi, napi-derive | Node.js bindings             |
| `wasm`               | wasm-bindgen      | WebAssembly bindings         |
| `cli`                | clap, colored     | CLI binary                   |
| `shacl`              | oxigraph          | SHACL validation             |
| `three_valued_logic` | (none)            | NULL-aware policy evaluation |

---

## 7. Serialization

Graph and primitives use `serde` for serialization:

```rust
#[derive(Serialize, Deserialize)]
pub struct Entity { /* ... */ }

// JSON round-trip
let json = serde_json::to_string(&graph)?;
let restored: Graph = serde_json::from_str(&json)?;
```

---

## 8. Extension Points

| Extension        | Mechanism                            |
| ---------------- | ------------------------------------ |
| New primitives   | Add to `primitives/`, update `Graph` |
| New projections  | Implement in `projection/`, register |
| New CLI commands | Add subcommand in `cli/`             |
| New bindings     | Feature-gate new binding module      |

---

## Related Documents

- [ADR-005: Multi-Language Support Strategy](./ADR-005-multi-language-support-strategy.md)
- [ADR-006: Error Handling Strategy](./ADR-006-error-handling-strategy.md)
- [SDS-003: Parser and Semantic Graph](./SDS-003-parser-semantic-graph.md)
- [PRD-002: SEA CLI Tooling](./PRD-002-sea-cli-tooling.md)
