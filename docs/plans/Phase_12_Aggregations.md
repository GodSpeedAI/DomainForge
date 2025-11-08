# 🧭 Phase 12: Aggregation Operators

**Status:** Planned
**Priority:** P0 — Critical Gap (PRD Requirement)
**Created:** 2025-11-08
**Estimated Duration:** 10 days
**Complexity:** Medium

---

## 1. Objectives and Context

**Goal:** Implement aggregation operators (count, sum, min, max, avg) in the policy expression language as specified in PRD-003.

**Problem Statement:**
PRD-003 explicitly requires "aggregations (count, sum, min, max, avg)" but the current Expression AST has no Aggregation variant. Policies needing aggregate calculations require ad-hoc host-language code, breaking the declarative model.

**Scope:**

- ✅ Aggregation expression variant (count, sum, min, max, avg)
- ✅ Collection filtering with where clauses
- ✅ Field accessors for aggregation (e.g., sum(flows.quantity))
- ✅ Unit-aware aggregations (requires Phase 11)
- ✅ DSL syntax: `count(flows where resource = "Camera")`
- ❌ NO window functions or complex SQL-style aggregations in MVP

**Dependencies:**

- **Prerequisite:** Phase 11 (unit-aware sum/avg requires Dimension)
- **Blocks:** Phase 17 (SBVR export needs aggregate mapping)

**Key Deliverable:** `Policy::evaluate()` supports aggregate expressions

---

## 2. Architecture & Design

### Expression AST Extension

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    // ... existing variants

    Aggregation {
        function: AggregateFunction,
        collection: Box<Expression>,
        field: Option<String>,          // e.g., "quantity" in sum(flows.quantity)
        filter: Option<Box<Expression>>, // WHERE clause
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AggregateFunction {
    Count,     // count(flows)
    Sum,       // sum(flows.quantity)
    Min,       // min(flows.quantity)
    Max,       // max(flows.quantity)
    Avg,       // avg(flows.quantity)
}
```

### Grammar Extension

```pest
primary_expr = {
    "(" ~ expression ~ ")" |
    aggregation_expr |        // NEW
    quantified_expr |
    member_access |
    literal |
    identifier
}

aggregation_expr = {
    aggregate_fn ~ "(" ~
        collection ~
        ("." ~ identifier)? ~           // field selector
        (^"where" ~ expression)? ~      // filter
    ")"
}

aggregate_fn = {
    ^"count" | ^"sum" | ^"min" | ^"max" | ^"avg"
}
```

### Example Policies

```sea
// Count all flows
Policy flow_count as: count(flows) > 10

// Sum quantities of Camera flows
Policy camera_total as: sum(flows where resource = "Camera").quantity > 1000

// Average flow quantity must be at least 500
Policy min_average as: avg(flows.quantity) >= 500

// Ensure at least one large flow exists
Policy has_large_flow as: max(flows.quantity) > 10000
```

---

## 3. TDD Cycles

### Cycle 12A — Aggregation AST & Grammar

**Branch:** `feat/phase12-aggregation-ast`
**Owner:** Core Development

#### 12A — RED Phase

**Test File:** `sea-core/tests/aggregation_tests.rs`

```rust
use sea_core::policy::{Expression, AggregateFunction};

#[test]
fn test_count_aggregation() {
    let expr = Expression::aggregation(
        AggregateFunction::Count,
        Expression::variable("flows"),
        None,
        None,
    );

    assert!(matches!(expr, Expression::Aggregation { .. }));
}

#[test]
fn test_sum_with_field() {
    let expr = Expression::aggregation(
        AggregateFunction::Sum,
        Expression::variable("flows"),
        Some("quantity"),
        None,
    );

    match expr {
        Expression::Aggregation { function, field, .. } => {
            assert_eq!(function, AggregateFunction::Sum);
            assert_eq!(field, Some("quantity".to_string()));
        }
        _ => panic!("Expected Aggregation"),
    }
}

#[test]
fn test_aggregation_with_filter() {
    let filter = Expression::comparison("resource", "==", "Camera");
    let expr = Expression::aggregation(
        AggregateFunction::Count,
        Expression::variable("flows"),
        None,
        Some(filter),
    );

    match expr {
        Expression::Aggregation { filter: Some(_), .. } => {}
        _ => panic!("Expected filter"),
    }
}

#[test]
fn test_parse_count_syntax() {
    let source = r#"
        Policy flow_count as: count(flows) > 10
    "#;

    let ast = parse(source).unwrap();
    // Verify AST contains count aggregation
}

#[test]
fn test_parse_sum_with_where() {
    let source = r#"
        Policy camera_sum as: sum(flows where resource = "Camera").quantity > 1000
    "#;

    let ast = parse(source).unwrap();
    // Verify AST structure
}
```

**Expected Outcome:** Compilation errors - Expression::Aggregation doesn't exist.

#### 12A — GREEN Phase

**Update `sea-core/src/policy/expression.rs`:**

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(serde_json::Value),
    Variable(String),
    Binary { op: BinaryOp, left: Box<Expression>, right: Box<Expression> },
    Unary { op: UnaryOp, operand: Box<Expression> },
    Quantifier {
        quantifier: Quantifier,
        variable: String,
        collection: Box<Expression>,
        condition: Box<Expression>,
    },
    MemberAccess { object: String, member: String },

    // NEW: Aggregation
    Aggregation {
        function: AggregateFunction,
        collection: Box<Expression>,
        field: Option<String>,
        filter: Option<Box<Expression>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregateFunction {
    Count,
    Sum,
    Min,
    Max,
    Avg,
}

impl Expression {
    pub fn aggregation(
        function: AggregateFunction,
        collection: Expression,
        field: Option<impl Into<String>>,
        filter: Option<Expression>,
    ) -> Self {
        Expression::Aggregation {
            function,
            collection: Box::new(collection),
            field: field.map(|f| f.into()),
            filter: filter.map(Box::new),
        }
    }
}

impl std::fmt::Display for AggregateFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AggregateFunction::Count => write!(f, "COUNT"),
            AggregateFunction::Sum => write!(f, "SUM"),
            AggregateFunction::Min => write!(f, "MIN"),
            AggregateFunction::Max => write!(f, "MAX"),
            AggregateFunction::Avg => write!(f, "AVG"),
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // ... existing cases
            Expression::Aggregation { function, collection, field, filter } => {
                write!(f, "{}({}", function, collection)?;
                if let Some(fld) = field {
                    write!(f, ".{}", fld)?;
                }
                if let Some(flt) = filter {
                    write!(f, " WHERE {}", flt)?;
                }
                write!(f, ")")
            }
        }
    }
}
```

**Update grammar `sea-core/grammar/sea.pest`:**

```pest
primary_expr = {
    "(" ~ expression ~ ")" |
    aggregation_expr |
    quantified_expr |
    member_access |
    literal |
    identifier
}

aggregation_expr = {
    aggregate_fn ~ "(" ~
        collection ~
        ("." ~ identifier)? ~
        (^"where" ~ expression)? ~
    ")"
}

aggregate_fn = {
    ^"count" | ^"sum" | ^"min" | ^"max" | ^"avg"
}
```

**Label:** → **12A-GREEN**

#### 12A — REFACTOR Phase

Extract aggregation parsing into separate module `sea-core/src/parser/aggregation.rs`.

**Label:** → **12A-REFACTOR**

#### 12A — REGRESSION Phase

```bash
cargo test --test expression_tests
cargo test --test parser_tests
```

**Label:** → **12A-COMPLETE**

---

### Cycle 12B — Aggregation Evaluation

**Branch:** `feat/phase12-aggregation-eval`
**Depends On:** Cycle 12A

#### 12B — RED Phase

**Test File:** `sea-core/tests/aggregation_eval_tests.rs`

```rust
use sea_core::{Graph, Policy, Expression, AggregateFunction};
use sea_core::primitives::{Entity, Resource, Flow};
use sea_core::units::{Unit, Dimension};
use rust_decimal::Decimal;

#[test]
fn test_count_flows() {
    let mut graph = Graph::new();

    // Create 3 flows
    let warehouse = Entity::new("Warehouse");
    let factory = Entity::new("Factory");
    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1));
    let gold = Resource::new("Gold", kg);

    graph.add_entity(warehouse.clone()).unwrap();
    graph.add_entity(factory.clone()).unwrap();
    graph.add_resource(gold.clone()).unwrap();

    for i in 1..=3 {
        let flow = Flow::new(
            gold.id().clone(),
            warehouse.id().clone(),
            factory.id().clone(),
            Decimal::from(i * 100),
        );
        graph.add_flow(flow).unwrap();
    }

    // Policy: count(flows) == 3
    let policy = Policy::new(
        "Count Flows",
        Expression::binary(
            BinaryOp::Equal,
            Expression::aggregation(
                AggregateFunction::Count,
                Expression::variable("flows"),
                None,
                None,
            ),
            Expression::literal(3),
        ),
    );

    let result = policy.evaluate(&graph).unwrap();
    assert!(result.is_satisfied);
}

#[test]
fn test_sum_quantities() {
    let mut graph = build_graph_with_flows();

    // Flows: 100, 200, 300 kg
    // sum(flows.quantity) = 600

    let policy = Policy::new(
        "Sum Quantities",
        Expression::binary(
            BinaryOp::Equal,
            Expression::aggregation(
                AggregateFunction::Sum,
                Expression::variable("flows"),
                Some("quantity"),
                None,
            ),
            Expression::literal(600),
        ),
    );

    let result = policy.evaluate(&graph).unwrap();
    assert!(result.is_satisfied);
}

#[test]
fn test_avg_quantity() {
    let mut graph = build_graph_with_flows(); // 100, 200, 300

    // avg(flows.quantity) = 200
    let policy = Policy::new(
        "Average Quantity",
        Expression::binary(
            BinaryOp::Equal,
            Expression::aggregation(
                AggregateFunction::Avg,
                Expression::variable("flows"),
                Some("quantity"),
                None,
            ),
            Expression::literal(200),
        ),
    );

    let result = policy.evaluate(&graph).unwrap();
    assert!(result.is_satisfied);
}

#[test]
fn test_aggregation_with_filter() {
    let mut graph = Graph::new();

    // Create flows for different resources
    let warehouse = Entity::new("Warehouse");
    let factory = Entity::new("Factory");

    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1));
    let camera = Resource::new("Camera", Unit::new("units", "units", Dimension::Count, Decimal::from(1)));
    let gold = Resource::new("Gold", kg);

    graph.add_entity(warehouse.clone()).unwrap();
    graph.add_entity(factory.clone()).unwrap();
    graph.add_resource(camera.clone()).unwrap();
    graph.add_resource(gold.clone()).unwrap();

    // 2 camera flows
    for _ in 0..2 {
        graph.add_flow(Flow::new(
            camera.id().clone(),
            warehouse.id().clone(),
            factory.id().clone(),
            Decimal::from(100),
        )).unwrap();
    }

    // 1 gold flow
    graph.add_flow(Flow::new(
        gold.id().clone(),
        warehouse.id().clone(),
        factory.id().clone(),
        Decimal::from(50),
    )).unwrap();

    // count(flows where resource = "Camera") == 2
    let filter = Expression::binary(
        BinaryOp::Equal,
        Expression::member_access("flow", "resource"),
        Expression::literal("Camera"),
    );

    let policy = Policy::new(
        "Camera Flow Count",
        Expression::binary(
            BinaryOp::Equal,
            Expression::aggregation(
                AggregateFunction::Count,
                Expression::variable("flows"),
                None,
                Some(filter),
            ),
            Expression::literal(2),
        ),
    );

    let result = policy.evaluate(&graph).unwrap();
    assert!(result.is_satisfied);
}
```

**Expected Outcome:** Tests compile but fail - evaluation not implemented.

#### 12B — GREEN Phase

**Update `sea-core/src/policy/policy.rs`:**

```rust
impl Policy {
    fn evaluate_expression(&self, expr: &Expression, graph: &Graph) -> Result<bool, String> {
        match expr {
            // ... existing cases

            Expression::Aggregation { function, collection, field, filter } => {
                let value = self.evaluate_aggregation(function, collection, field, filter, graph)?;
                Ok(value.as_bool().unwrap_or(false))
            }
        }
    }

    fn evaluate_aggregation(
        &self,
        function: &AggregateFunction,
        collection: &Expression,
        field: &Option<String>,
        filter: &Option<Box<Expression>>,
        graph: &Graph,
    ) -> Result<serde_json::Value, String> {
        // Get the collection items
        let items = match collection {
            Expression::Variable(name) if name == "flows" => {
                graph.all_flows()
                    .map(|f| serde_json::to_value(f).unwrap())
                    .collect::<Vec<_>>()
            }
            Expression::Variable(name) if name == "entities" => {
                graph.all_entities()
                    .map(|e| serde_json::to_value(e).unwrap())
                    .collect::<Vec<_>>()
            }
            Expression::Variable(name) if name == "resources" => {
                graph.all_resources()
                    .map(|r| serde_json::to_value(r).unwrap())
                    .collect::<Vec<_>>()
            }
            Expression::Variable(name) if name == "instances" => {
                graph.all_instances()
                    .map(|i| serde_json::to_value(i).unwrap())
                    .collect::<Vec<_>>()
            }
            _ => return Err("Unsupported collection type".to_string()),
        };

        // Apply filter if present
        let filtered_items = if let Some(filter_expr) = filter {
            items.into_iter()
                .filter(|item| {
                    // Evaluate filter in context of this item
                    // This requires extending evaluation context
                    self.evaluate_filter(filter_expr, item, graph).unwrap_or(false)
                })
                .collect::<Vec<_>>()
        } else {
            items
        };

        // Apply aggregation function
        match function {
            AggregateFunction::Count => {
                Ok(serde_json::json!(filtered_items.len()))
            }

            AggregateFunction::Sum => {
                let field_name = field.as_ref()
                    .ok_or("Sum requires a field specification")?;

                let sum: Decimal = filtered_items.iter()
                    .filter_map(|item| {
                        item.get(field_name)
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<Decimal>().ok())
                    })
                    .sum();

                Ok(serde_json::json!(sum.to_string()))
            }

            AggregateFunction::Avg => {
                let field_name = field.as_ref()
                    .ok_or("Avg requires a field specification")?;

                let values: Vec<Decimal> = filtered_items.iter()
                    .filter_map(|item| {
                        item.get(field_name)
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<Decimal>().ok())
                    })
                    .collect();

                if values.is_empty() {
                    return Ok(serde_json::json!(null));
                }

                let sum: Decimal = values.iter().sum();
                let avg = sum / Decimal::from(values.len());

                Ok(serde_json::json!(avg.to_string()))
            }

            AggregateFunction::Min => {
                let field_name = field.as_ref()
                    .ok_or("Min requires a field specification")?;

                let min = filtered_items.iter()
                    .filter_map(|item| {
                        item.get(field_name)
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<Decimal>().ok())
                    })
                    .min();

                Ok(serde_json::json!(min.map(|d| d.to_string())))
            }

            AggregateFunction::Max => {
                let field_name = field.as_ref()
                    .ok_or("Max requires a field specification")?;

                let max = filtered_items.iter()
                    .filter_map(|item| {
                        item.get(field_name)
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<Decimal>().ok())
                    })
                    .max();

                Ok(serde_json::json!(max.map(|d| d.to_string())))
            }
        }
    }

    fn evaluate_filter(
        &self,
        filter: &Expression,
        item: &serde_json::Value,
        graph: &Graph,
    ) -> Result<bool, String> {
        // Create temporary context with item bindings
        // Evaluate filter expression in that context
        // This is similar to quantifier evaluation
        todo!("Implement filter evaluation")
    }
}
```

**Label:** → **12B-GREEN**

#### 12B — REFACTOR Phase

1. Extract aggregation evaluation into `sea-core/src/policy/aggregation.rs`
2. Add unit-aware aggregation (sum preserves dimension)
3. Optimize collection iteration

**Label:** → **12B-REFACTOR**

#### 12B — REGRESSION Phase

```bash
cargo test --test aggregation_tests
cargo test --test policy_tests
```

**Label:** → **12B-COMPLETE**

---

### Cycle 12C — Unit-Aware Aggregations

**Branch:** `feat/phase12-unit-aware-aggregations`
**Depends On:** Cycle 12B, Phase 11

#### 12C — RED Phase

```rust
#[test]
fn test_sum_preserves_units() {
    let mut graph = Graph::new();

    // Create flows in kg
    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1));
    let gold = Resource::new("Gold", kg.clone());

    // ... add flows

    // sum(flows.quantity) should return value with kg unit
    let policy = Policy::new(
        "Sum With Units",
        Expression::aggregation(
            AggregateFunction::Sum,
            Expression::variable("flows"),
            Some("quantity"),
            None,
        ),
    );

    let result = policy.evaluate_with_units(&graph).unwrap();
    assert_eq!(result.unit, Some(kg));
}

#[test]
fn test_sum_rejects_mixed_units() {
    let mut graph = Graph::new();

    // Flows with different units should error on sum
    // Flow 1: 100 kg
    // Flow 2: 50 L

    let policy = Policy::new(
        "Mixed Units Sum",
        Expression::aggregation(
            AggregateFunction::Sum,
            Expression::variable("flows"),
            Some("quantity"),
            None,
        ),
    );

    let result = policy.evaluate(&graph);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("incompatible units"));
}
```

#### 12C — GREEN Phase

Enhance aggregation to track and validate units.

**Label:** → **12C-GREEN**

#### 12C — REFACTOR Phase

Add unit inference for aggregation results.

**Label:** → **12C-REFACTOR**

#### 12C — REGRESSION Phase

Full unit + aggregation test suite.

**Label:** → **12C-COMPLETE**

---

### Cycle 12D — FFI Bindings

**Branch:** `feat/phase12-ffi-aggregations`
**Depends On:** Cycle 12C

#### 12D — RED Phase

**Python:**

```python
def test_count_aggregation():
    policy = Policy(
        "Count Flows",
        Expression.binary(
            BinaryOp.EQUAL,
            Expression.aggregation(
                AggregateFunction.COUNT,
                Expression.variable("flows"),
                None,
                None,
            ),
            Expression.literal(3),
        ),
    )
    assert policy.name == "Count Flows"
```

**TypeScript:**

```typescript
test('count aggregation', () => {
  const policy = new Policy(
    'Count Flows',
    Expression.binary(
      BinaryOp.Equal,
      Expression.aggregation(
        AggregateFunction.Count,
        Expression.variable('flows'),
        null,
        null,
      ),
      Expression.literal(3),
    ),
  );
  expect(policy.name).toBe('Count Flows');
});
```

#### 12D — GREEN Phase

Expose AggregateFunction enum and Expression.aggregation() in PyO3 and napi-rs bindings.

**Label:** → **12D-GREEN**

#### 12D — REFACTOR Phase

Add builder-style API for aggregations.

**Label:** → **12D-REFACTOR**

#### 12D — REGRESSION Phase

```bash
pytest tests/test_aggregations.py
npm test -- aggregation
```

**Label:** → **12D-COMPLETE**

---

## 4. Success Criteria

- [ ] All 5 aggregate functions (count, sum, min, max, avg) implemented
- [ ] WHERE clause filtering works
- [ ] Field accessors work (e.g., flows.quantity)
- [ ] Unit-aware aggregations validate dimension compatibility
- [ ] Parser supports DSL syntax: `count(flows where resource = "X")`
- [ ] FFI bindings expose aggregations in Python, TypeScript, WASM
- [ ] Performance: Aggregations over 10K items complete in <100ms
- [ ] PRD-003 acceptance criteria met

---

## 5. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Performance on large collections | High | Add lazy evaluation; consider iterators |
| Complex WHERE clause evaluation | Medium | Reuse quantifier evaluation infrastructure |
| Unit mixing in aggregations | Medium | Validate all items have compatible units |

---

## 6. Traceability

**Addresses:**

- Critique 2 (Aggregations)
- PRD-003 (Expression Language) acceptance criteria
- SDS-006 (Policy Evaluator)

**Updates Required:**

- `docs/specs/prd.md` § PRD-003 (mark aggregation criteria as ✅)
- `docs/specs/sds.md` § SDS-006 (add aggregation evaluation spec)
- `docs/specs/api_specification.md` § Policy API

---

## 7. Sign-Off Checklist

- [ ] All TDD cycles complete (12A-D)
- [ ] PRD-003 acceptance criteria verified
- [ ] Documentation updated
- [ ] Performance benchmarks pass
- [ ] Cross-language tests pass
- [ ] Code review approved
- [ ] Merged to main

**Completion Date:** _____________________
