# ADR: Canonical Expression Normalizer

**Status:** Proposed  
**Date:** 2025-12-16  
**Authors:** DomainForge Architecture Team

## Context

SEA-DSL policy expressions can be written in many syntactically equivalent forms:

- `a AND b` vs `b AND a`
- `true AND x` vs `x`
- Nested parentheses: `((a))` vs `a`

This creates challenges for:

1. **Hover signatures** - displaying a consistent, readable form
2. **Change detection** - determining if two policies are semantically equivalent
3. **Caching** - using normalized expressions as cache keys
4. **Testing** - golden snapshot tests need deterministic output

## Decision

Implement a canonical normalizer in `sea_core::policy` that transforms any `Expression` into a deterministic, minimal, canonical form.

## Design

### 1. Normalization Rules

| Category                   | Rule                                          | Example                             |
| -------------------------- | --------------------------------------------- | ----------------------------------- |
| **Commutativity**          | Sort operands of `AND`/`OR` lexicographically | `b AND a` → `a AND b`               |
| **Identity**               | Remove identity elements                      | `true AND x` → `x`                  |
| **Absorption**             | Remove absorbed terms                         | `a OR (a AND b)` → `a`              |
| **Parentheses**            | Flatten redundant nesting                     | `((a))` → `a`                       |
| **Variable naming**        | Alpha-rename quantifier vars                  | `forall x: ...` → `forall _v0: ...` |
| **Operator normalization** | Use canonical operators                       | `NOT (a = b)` → `a != b`            |

### 2. API

```rust
// In sea_core::policy::expression
impl Expression {
    /// Returns a canonical form of this expression
    pub fn normalize(&self) -> NormalizedExpression;

    /// Returns true if two expressions are semantically equivalent
    pub fn is_equivalent(&self, other: &Expression) -> bool {
        self.normalize() == other.normalize()
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct NormalizedExpression {
    inner: Expression,
    /// Stable hash for caching
    hash: u64,
}

impl Display for NormalizedExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Pretty-print in canonical form
    }
}
```

### 3. Implementation Strategy

**Phase 1: Core normalization**

- [ ] Implement `normalize()` for leaf nodes (Literal, Variable)
- [ ] Implement `normalize()` for binary operators with sorting
- [ ] Implement `normalize()` for unary operators
- [ ] Add stable hashing using `rustc_hash::FxHasher`

**Phase 2: Advanced normalization**

- [ ] Implement constant folding (`true AND x` → `x`)
- [ ] Implement absorption rules
- [ ] Implement alpha-renaming for quantifier variables

**Phase 3: Integration**

- [ ] Use in LSP hover signature generation
- [ ] Add `Policy::normalized_expression()` helper
- [ ] Add CLI option: `sea normalize <expr>`

### 4. Files to Modify

| File                                           | Changes                          |
| ---------------------------------------------- | -------------------------------- |
| `sea-core/src/policy/mod.rs`                   | Add `mod normalize;`             |
| `sea-core/src/policy/normalize.rs`             | **NEW** - Normalization logic    |
| `sea-core/src/policy/expression.rs`            | Add `normalize()` method         |
| `domainforge-lsp/src/hover/symbol_resolver.rs` | Use normalized form in signature |

## Consequences

### Positive

- Deterministic hover output
- Semantic equivalence checking
- Better caching with normalized keys
- Cleaner test assertions

### Negative

- Computational overhead for normalization
- Increased complexity in expression module
- Alpha-renaming may reduce readability

## Alternatives Considered

1. **No normalization** - Accept non-deterministic output (rejected: breaks testing)
2. **Simple sorting only** - Only sort commutative operators (insufficient for identity elimination)
3. **External SMT solver** - Use Z3 for semantic equivalence (rejected: too heavy)

## References

- [SDS-004: Policy Engine Design](./SDS-004-policy-engine-design.md)
- [hover_plan.yml](../archive/hover_plan.yml) - Mentions normalized form display
