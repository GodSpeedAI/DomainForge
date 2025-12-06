# Graph Store Design

The Graph Store (`sea-core/src/graph/mod.rs`) is the in-memory database that holds the semantic model. It is designed for performance, correctness, and deterministic behavior.

## Data Structure: IndexMap

We use `indexmap::IndexMap` instead of the standard library's `HashMap` for all primary collections (Entities, Flows, Resources).

### Why?

1. **Determinism**: `HashMap` iteration order is randomized (SipHash). This means running the same policy on the same model could produce errors in a different order, or result in different generated code output (e.g., Terraform files changing order every run).
2. **Stability**: `IndexMap` preserves insertion order. This ensures that `forall` loops in policies always visit nodes in a predictable sequence.

## Node and Edge Model

The graph is not a generic adjacency matrix. It is a structured collection of typed stores:

```rust
pub struct Graph {
    pub entities: IndexMap<ConceptId, Entity>,
    pub resources: IndexMap<ConceptId, Resource>,
    pub flows: Vec<Flow>, // Flows are edges
    // ...
}
```

- **Nodes**: Entities and Resources are nodes. They are indexed by `ConceptId` (a hash of namespace + name).
- **Edges**: Flows are directed edges connecting nodes. They are stored separately to allow fast iteration over all interactions.

## Query Patterns

The graph supports specific access patterns optimized for policy evaluation:

- **Lookup by ID**: O(1) access to any Entity or Resource.
- **Flow Traversal**: "Get all flows originating from Entity X".
- **Type Filtering**: "Get all Resources of type 'database'".

## Performance Characteristics

- **Parsing**: Linear time relative to file size.
- **Graph Construction**: Near-linear. Resolution of references happens in a second pass.
- **Policy Evaluation**: Depends on complexity. Simple `forall` is O(N). Nested quantifiers can be O(N^2).

## See Also

- [Architecture Overview](architecture-overview.md)
- [Policy Evaluation Logic](policy-evaluation-logic.md)
