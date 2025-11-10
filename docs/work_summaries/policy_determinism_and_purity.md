# Policy Evaluation Determinism and Purity Guarantees

## Overview

The SEA policy evaluation system provides strong guarantees about deterministic behavior and functional purity to ensure reliable, predictable, and reproducible policy evaluation.

## Deterministic Iteration

### IndexMap-Based Storage

All graph collections (entities, resources, flows, instances) use `IndexMap` instead of `HashMap` to ensure:

1. **Insertion-order preservation**: Items are iterated in the order they were added
2. **Stable iteration**: Multiple calls to iteration methods return items in the same order
3. **Deterministic serialization**: Graph serialization produces consistent output

### Guaranteed Iteration Order

The following methods guarantee deterministic iteration:

- `Graph::all_entities()` - Returns entities in insertion order
- `Graph::all_resources()` - Returns resources in insertion order
- `Graph::all_flows()` - Returns flows in insertion order
- `Graph::all_instances()` - Returns instances in insertion order

## Policy Evaluation Order

### Priority-Based Ordering

Policies can be explicitly ordered using the `priority` field:

```rust
let policy = Policy::new("my_policy", expression)
    .with_priority(10);
```

Lower priority values are evaluated first. Default priority is 0.

### Policy Kinds

Policies are categorized into three kinds:

1. **Constraint**: Boolean validation policies that verify conditions
2. **Derivation**: Policies that compute derived fields or values
3. **Obligation**: Policies that trigger events or side effects

## Purity Guarantees

### No Side Effects

Policy evaluation is **pure** and **side-effect free**:

- No I/O operations (file, network, database)
- No mutation of the graph state
- No mutation of external state
- No non-deterministic operations (random, time-based)

### Referential Transparency

Given the same graph state, policy evaluation always returns:

- The same `is_satisfied` result
- The same set of violations
- The same violation messages

### Must-Use Semantics

Policy evaluation results are marked with `#[must_use]` (where applicable) to ensure:

- Results are not silently ignored
- Violations are explicitly handled
- Evaluation errors are propagated

## Implementation Details

### Expression Expansion

Before evaluation, quantified expressions are expanded into boolean expressions:

```rust
pub fn evaluate(&self, graph: &Graph) -> Result<EvaluationResult, String> {
    let expanded = self.expression.expand(graph)?;
    let is_satisfied = self.evaluate_expression(&expanded, graph)?;
    // ...
}
```

This ensures:
- No lazy evaluation that could introduce non-determinism
- All data dependencies are resolved upfront
- Evaluation is a pure function of the expanded expression

### Type Safety

The type system enforces purity:

- Evaluation takes `&Graph` (immutable borrow)
- No `&mut` references that could allow mutation
- All intermediate values are owned or immutably borrowed

## Testing

Determinism and purity are verified through:

1. **Stable iteration tests**: Verify same order across multiple iterations
2. **Pure evaluation tests**: Verify same results from repeated evaluations
3. **No side-effect audits**: Code review ensures no I/O in evaluation paths

## Migration Notes

### From HashMap to IndexMap

The migration from `HashMap` to `IndexMap` is backward compatible:

- API surface remains unchanged
- Serialization format unchanged (both use serde)
- Performance characteristics similar for typical graph sizes

### Policy Priority

The `priority` field is optional:

- Default value is 0
- Existing policies without priority will evaluate in insertion order among same-priority policies
- Explicit priority allows fine-grained control

## Best Practices

1. **Set explicit priorities** for policies with dependencies
2. **Use PolicyKind** to document intent (Constraint vs Derivation vs Obligation)
3. **Leverage purity** for parallel evaluation (future enhancement)
4. **Test with multiple evaluations** to verify determinism

## Future Enhancements

- Parallel policy evaluation (enabled by purity guarantee)
- Policy dependency graph analysis
- Automatic priority inference from dependencies
- Incremental evaluation for large graphs
