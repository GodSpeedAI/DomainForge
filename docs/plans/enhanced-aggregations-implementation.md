# Implementation Plan - Enhanced Aggregations (Step 4)

This plan outlines the implementation of **Enhanced Aggregations** for the DomainForge DSL, specifically adding support for `group_by` clauses and window functions (`over` syntax). This corresponds to **Step 4** in the `dsl-completeness-roadmap.md`.

## Goal Description

The goal is to enable more sophisticated data analysis within policies by allowing:

1.  **Grouping**: Aggregating data based on specific keys (e.g., sum of payments _per vendor_).
2.  **Windowing**: Aggregating data over a sliding window (e.g., average throughput _over the last hour_).

## User Review Required

> [!IMPORTANT] > **Syntax Decision**:
> The `group_by` syntax introduces a block structure that is new to expressions.
>
> ```sea
> group_by(f in flows: f.to.name) {
>   sum(f.quantity) <= 1000
> }
> ```
>
> This implies that the expression inside the block is evaluated _for each group_. If _any_ group violates the condition, the policy is violated.

> [!NOTE] > **Window Semantics**:
> Window functions like `over last 1 "hour"` require access to temporal data. This assumes that the underlying data source provides timestamps. For the purpose of this implementation, we will assume `flows` have a `timestamp` or `created_at` field, or we will use a generic `timestamp` field.

## Proposed Changes

### Grammar (`sea-core/grammar/sea.pest`)

#### [MODIFY] `sea.pest`

- Add `group_by_expr` rule.
- Add `window_clause` rule to `aggregation_comprehension`.
- Update `primary_expr` to include `group_by_expr`.

```pest
// New Group By Expression
// Syntax: group_by(f in flows: f.to.name) { sum(f.quantity) > 10 }
group_by_expr = {
    ^"group_by" ~ "(" ~ identifier ~ "in" ~ collection ~ ("where" ~ expression)? ~ ":" ~ expression ~ ")" ~ "{" ~ expression ~ "}"
}

// Update Aggregation Comprehension to support Windowing
// Syntax: avg(f in flows over last 1 "hour": f.quantity)
aggregation_comprehension = {
    identifier ~ "in" ~ collection ~ window_clause? ~ "where" ~ expression ~ ":" ~ expression ~ ("as" ~ string_literal)?
}

window_clause = {
    ^"over" ~ ^"last" ~ number ~ string_literal
}

// Update primary_expr
primary_expr = {
    // ... existing ...
    group_by_expr |
    // ...
}
```

### AST (`sea-core/src/parser/ast.rs` & `sea-core/src/policy/expression.rs`)

#### [MODIFY] `sea-core/src/policy/expression.rs`

- Add `GroupBy` variant to `Expression` enum.
- Update `AggregationComprehension` variant to include `window`.

```rust
pub enum Expression {
    // ...
    GroupBy {
        variable: String,
        collection: Collection, // e.g., Flows
        filter: Option<Box<Expression>>,
        key: Box<Expression>,
        condition: Box<Expression>, // The expression to evaluate per group
    },
    AggregationComprehension {
        function: AggregateFunction,
        variable: String,
        collection: Collection,
        window: Option<WindowSpec>, // New field
        predicate: Box<Expression>,
        projection: Box<Expression>,
        target_unit: Option<String>,
    },
    // ...
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowSpec {
    pub duration: i64, // Value
    pub unit: String,  // Unit (e.g., "hour")
}
```

#### [MODIFY] `sea-core/src/parser/ast.rs`

- Update `parse_expression` and related functions to handle `group_by` and `window` syntax.
- Implement `parse_group_by_expr`.
- Update `parse_aggregation_expr` to parse `window_clause`.

### Semantics & Evaluation (`sea-core/src/policy/core.rs` & `sea-core/src/policy/expression.rs`)

#### [MODIFY] `sea-core/src/policy/expression.rs`

- Implement `evaluate_group_by` logic.

  1.  Filter collection based on `where` clause.
  2.  Group items by evaluating the `key` expression.
  3.  Iterate over groups.
  4.  For each group, create a context where aggregations are scoped to that group.
  5.  Evaluate `condition`.
  6.  If any group fails, return `False`.

- Update `evaluate_aggregation_comprehension` to handle `window`.
  1.  If `window` is present, filter items based on timestamp (assuming `timestamp` field exists on items).
  2.  Proceed with existing aggregation logic.

#### [MODIFY] `sea-core/src/policy/core.rs`

- Update `evaluate_expression_boolean` and `evaluate_expression_three_valued` to dispatch `Expression::GroupBy`.

### Validation (`sea-core/src/policy/validator.rs` or `core.rs`)

#### [MODIFY] `sea-core/src/policy/core.rs`

- Ensure that inside a `group_by` block, expressions either refer to the `key` or are `aggregations`. Accessing individual item properties directly (without aggregation) should be ambiguous/illegal unless it's the grouping key.

## Status: Completed (with Known Issue)

### Completed Features

1. **Grammar**: Added `group_by_expr` and `window_clause` rules. Updated `aggregation_comprehension` and `aggregation_simple`.
2. **AST**: Added `GroupBy` variant to `Expression` and `WindowSpec` struct.
3. **Parser**: Implemented parsing logic for new grammar rules.
4. **Semantics**: Implemented `expand` logic for `GroupBy` and window filtering in aggregations.
5. **Testing**: Added integration tests covering basic grouping and negative cases.

### Known Issue (Resolved)

- **WHERE Clause Parsing**: Filters that start with member access now parse and evaluate correctly in `group_by` expressions. Numeric comparisons now normalize JSON number equality (e.g., `1` vs `1.0`), which previously caused filtered groups to fail the condition check.

## Verification Plan

### Automated Tests

- [x] `test_group_by_entity`: Verifies basic grouping by entity.
- [x] `test_group_by_count`: Verifies grouping with `count` aggregation.
- [x] `test_group_by_entity_fail`: Verifies that unsatisfied conditions in groups correctly fail the policy.
- [x] `test_group_by_with_filter`: Verifies `where` clause filtering (Enabled after parser/evaluator fix).

### Manual Verification

Run the tests using:

```bash
cargo test --test aggregation_enhanced_tests
```

- Verify that error messages are clear when using invalid syntax or referencing non-grouped variables inside `group_by`.
