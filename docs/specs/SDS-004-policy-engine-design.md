# SDS-004: Policy Engine Design

**System:** DomainForge  
**Component:** Policy Evaluation Engine  
**Version:** 1.0  
**Date:** 2025-12-14  
**Status:** Implemented

---

## 1. Overview

The Policy Engine evaluates business rules expressed in SEA-DSL against the semantic graph. It supports:

- Three-valued logic (True, False, Unknown)
- Universal and existential quantifiers
- Aggregation functions
- Configurable severity levels

---

## 2. Architecture

```
Policy Expression (String)
         │
         ▼
┌─────────────────┐
│ Expression      │
│ Parser          │
└────────┬────────┘
         │ Expression AST
         ▼
┌─────────────────┐
│ Type Inference  │
└────────┬────────┘
         │ Typed Expression
         ▼
┌─────────────────┐     ┌─────────────┐
│ Evaluator       │ ←── │ Graph       │
└────────┬────────┘     └─────────────┘
         │ TruthValue + Violations
         ▼
    Evaluation Result
```

---

## 3. Core Types

### 3.1 Policy

```rust
pub struct Policy {
    pub id: ConceptId,
    pub name: String,
    pub expression: Expression,
    pub severity: Severity,
    pub description: Option<String>,
}

pub enum Severity {
    Error,   // Blocks validation
    Warning, // Reported but passes
    Info,    // Informational
}
```

### 3.2 Expression AST

```rust
pub enum Expression {
    // Literals
    Literal(Value),

    // Variable reference
    Variable(String),

    // Field access: entity.name
    FieldAccess {
        object: Box<Expression>,
        field: String,
    },

    // Binary operations: a + b, x > y
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },

    // Unary operations: not x
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
    },

    // Quantifiers: forall x in Domain: body
    Quantifier {
        kind: QuantifierKind,
        variable: String,
        domain: String,
        filter: Option<Box<Expression>>,
        body: Box<Expression>,
    },

    // Aggregations: sum(expr), count(expr)
    Aggregation {
        function: AggregateFunction,
        domain: String,
        filter: Option<Box<Expression>>,
        expression: Box<Expression>,
    },
}

pub enum QuantifierKind {
    ForAll,
    Exists,
}

pub enum AggregateFunction {
    Sum,
    Count,
    Min,
    Max,
    Avg,
}
```

### 3.3 Three-Valued Logic

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TruthValue {
    True,
    False,
    Unknown,
}

impl TruthValue {
    pub fn and(self, other: TruthValue) -> TruthValue {
        match (self, other) {
            (TruthValue::False, _) | (_, TruthValue::False) => TruthValue::False,
            (TruthValue::Unknown, _) | (_, TruthValue::Unknown) => TruthValue::Unknown,
            _ => TruthValue::True,
        }
    }

    pub fn or(self, other: TruthValue) -> TruthValue {
        match (self, other) {
            (TruthValue::True, _) | (_, TruthValue::True) => TruthValue::True,
            (TruthValue::Unknown, _) | (_, TruthValue::Unknown) => TruthValue::Unknown,
            _ => TruthValue::False,
        }
    }

    pub fn not(self) -> TruthValue {
        match self {
            TruthValue::True => TruthValue::False,
            TruthValue::False => TruthValue::True,
            TruthValue::Unknown => TruthValue::Unknown,
        }
    }
}
```

---

## 4. Evaluation Algorithm

### 4.1 Quantifier Evaluation

```rust
fn evaluate_quantifier(
    &self,
    kind: QuantifierKind,
    variable: &str,
    domain: &str,
    filter: Option<&Expression>,
    body: &Expression,
    graph: &Graph,
    bindings: &mut HashMap<String, Value>,
) -> (TruthValue, Vec<Violation>) {
    let items = self.resolve_domain(domain, graph);
    let filtered = self.apply_filter(items, filter, bindings);

    match kind {
        QuantifierKind::ForAll => {
            // True if body holds for ALL items
            // False if body is False for ANY item
            // Unknown if body is Unknown for some and True for rest
            let mut result = TruthValue::True;
            let mut violations = Vec::new();

            for item in filtered {
                bindings.insert(variable.to_string(), item.clone());
                let (value, mut item_violations) = self.evaluate(body, graph, bindings);

                match value {
                    TruthValue::False => {
                        violations.append(&mut item_violations);
                        result = TruthValue::False;
                    }
                    TruthValue::Unknown if result == TruthValue::True => {
                        result = TruthValue::Unknown;
                    }
                    _ => {}
                }
            }

            (result, violations)
        }

        QuantifierKind::Exists => {
            // True if body holds for ANY item
            // False if body is False for ALL items
            let mut result = TruthValue::False;

            for item in filtered {
                bindings.insert(variable.to_string(), item.clone());
                let (value, _) = self.evaluate(body, graph, bindings);

                match value {
                    TruthValue::True => return (TruthValue::True, Vec::new()),
                    TruthValue::Unknown if result == TruthValue::False => {
                        result = TruthValue::Unknown;
                    }
                    _ => {}
                }
            }

            (result, Vec::new())
        }
    }
}
```

### 4.2 Aggregation Evaluation

```rust
fn evaluate_aggregation(
    &self,
    function: AggregateFunction,
    domain: &str,
    filter: Option<&Expression>,
    expression: &Expression,
    graph: &Graph,
    bindings: &mut HashMap<String, Value>,
) -> Value {
    let items = self.resolve_domain(domain, graph);
    let filtered = self.apply_filter(items, filter, bindings);

    let values: Vec<f64> = filtered
        .iter()
        .filter_map(|item| {
            bindings.insert("_".to_string(), item.clone());
            match self.evaluate_value(expression, graph, bindings) {
                Value::Number(n) => Some(n),
                _ => None,
            }
        })
        .collect();

    match function {
        AggregateFunction::Sum => Value::Number(values.iter().sum()),
        AggregateFunction::Count => Value::Number(values.len() as f64),
        AggregateFunction::Min => values.iter().cloned().min_by(|a, b| a.partial_cmp(b).unwrap())
            .map(Value::Number).unwrap_or(Value::Null),
        AggregateFunction::Max => values.iter().cloned().max_by(|a, b| a.partial_cmp(b).unwrap())
            .map(Value::Number).unwrap_or(Value::Null),
        AggregateFunction::Avg => {
            if values.is_empty() {
                Value::Null
            } else {
                Value::Number(values.iter().sum::<f64>() / values.len() as f64)
            }
        }
    }
}
```

---

## 5. Violation Reporting

```rust
pub struct Violation {
    pub policy_id: ConceptId,
    pub policy_name: String,
    pub severity: Severity,
    pub message: String,
    pub context: ViolationContext,
}

pub struct ViolationContext {
    pub entity_id: Option<ConceptId>,
    pub entity_name: Option<String>,
    pub field: Option<String>,
    pub expected: Option<String>,
    pub actual: Option<String>,
}
```

### Example Violation Output

```json
{
  "policy_name": "production_minimum",
  "severity": "error",
  "message": "Flow quantity 200 is less than minimum 500",
  "context": {
    "entity_name": "Assembly Line A",
    "field": "quantity",
    "expected": ">= 500",
    "actual": "200"
  }
}
```

---

## 6. Configuration

```rust
pub struct GraphConfig {
    /// Enable three-valued logic (True, False, NULL) for policy evaluation.
    pub use_three_valued_logic: bool,
}

impl Graph {
    pub fn set_evaluation_mode(&mut self, use_three_valued_logic: bool) {
        self.config.use_three_valued_logic = use_three_valued_logic;
    }
}
```

---

## 7. Performance Considerations

| Operation             | Complexity           | Mitigation                     |
| --------------------- | -------------------- | ------------------------------ |
| Quantifier evaluation | O(n) per quantifier  | Lazy evaluation, short-circuit |
| Aggregation           | O(n) per aggregation | Memoization where possible     |
| Field access          | O(1)                 | IndexMap for attribute lookup  |
| Policy batch          | O(p × n)             | Parallel evaluation option     |

---

## 8. Extension Points

- **Custom functions**: Register domain-specific aggregations
- **External data sources**: Fetch data during evaluation
- **Caching**: Memoize expensive sub-expressions

---

## Related Documents

- [ADR-007: Policy Evaluation Engine](./ADR-007-policy-evaluation-engine.md)
- [PRD-003: DSL Core Capabilities](./PRD-003-dsl-core-capabilities.md)
- [SDS-002: SEA Core Architecture](./SDS-002-sea-core-architecture.md)
