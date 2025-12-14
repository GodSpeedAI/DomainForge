# ADR-007: Policy Evaluation Engine

**Status:** Accepted  
**Date:** 2025-12-14  
**Deciders:** DomainForge Architecture Team

## Context

SEA-DSL policies express business rules as logical expressions over the semantic graph. These policies must:

1. Evaluate efficiently over large graphs (10,000+ entities)
2. Support quantified expressions (`forall`, `exists`)
3. Handle missing/null data gracefully
4. Provide actionable violation reports

Traditional two-valued boolean logic (true/false) fails when data is incomplete, leading to false positives or missed violations.

## Decision

### Three-Valued Logic

Implement a **three-valued logic** system based on Kleene's strong logic:

| Expression        | Result  |
| ----------------- | ------- |
| true AND unknown  | unknown |
| false AND unknown | false   |
| true OR unknown   | true    |
| false OR unknown  | unknown |
| NOT unknown       | unknown |

```rust
pub enum TruthValue {
    True,
    False,
    Unknown,  // NULL propagation
}

impl TruthValue {
    pub fn and(self, other: TruthValue) -> TruthValue {
        match (self, other) {
            (TruthValue::False, _) | (_, TruthValue::False) => TruthValue::False,
            (TruthValue::Unknown, _) | (_, TruthValue::Unknown) => TruthValue::Unknown,
            _ => TruthValue::True,
        }
    }
}
```

### Quantifier Evaluation

Support universal and existential quantifiers over graph collections:

```sea
// Universal: must hold for ALL matching entities
Policy all_flows_positive as:
    forall f in Flow: f.quantity > 0

// Existential: must hold for AT LEAST ONE entity
Policy has_active_warehouse as:
    exists e in Entity where e.type = "Warehouse": e.active = true
```

### Expression Evaluation

Expressions are parsed into an AST and evaluated against the graph:

```rust
pub enum Expression {
    Literal(Value),
    FieldAccess { object: Box<Expression>, field: String },
    BinaryOp { left: Box<Expression>, op: BinaryOperator, right: Box<Expression> },
    UnaryOp { op: UnaryOperator, operand: Box<Expression> },
    Quantifier { kind: QuantifierKind, variable: String, domain: String, filter: Option<Box<Expression>>, body: Box<Expression> },
}
```

## Consequences

### Positive

- **Null-safe**: Missing data doesn't cause false violations
- **Expressive**: Quantifiers enable complex business rules
- **Efficient**: Lazy evaluation and short-circuit logic
- **Debuggable**: Clear violation messages with context

### Negative

- **Complexity**: Three-valued logic less intuitive than boolean
- **Performance**: Quantifier evaluation is O(n) per policy

## Related

- [SDS-004: Policy Engine Design](./SDS-004-policy-engine-design.md)
- [ADR-001: SEA-DSL as Semantic Source of Truth](./ADR-001-sea-dsl-semantic-source-of-truth.md)
