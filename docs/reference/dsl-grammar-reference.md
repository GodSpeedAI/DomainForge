# Reference: SEA DSL Grammar Specification

This document provides an exhaustive reference for the Semantic Enterprise Architecture (SEA) Domain-Specific Language syntax, as defined in `domainforge-core/grammar/sea.pest`.

---

## 1. File Structure & Header Annotations

A SEA file begins with optional file annotations, followed by import declarations, followed by domain declarations.

```sea
@namespace "string"
@version "semver"
@owner "string"
@profile "string"

import { Symbol1, Symbol2 as Alias } from "relative/path.sea"
import * as Alias from "relative/path.sea"

export declaration
```

---

## 2. Core Declarations

### Entity Declaration
```sea
Entity "Name"
Entity "Name" in domain_name
Entity "Name" v1.0.0 in domain_name
Entity "Name" {
  key id: uuid,
  name: string(min_length 1, max_length 100),
  status: enum_name optional default "PENDING"
}
```

### Resource Declaration
```sea
Resource "Name"
Resource "Name" in domain_name
Resource "Name" unit_name
Resource "Name" unit_name in domain_name
```

### Flow Declaration
```sea
Flow "ResourceName" from "SourceEntity" to "TargetEntity"
Flow "ResourceName" from "SourceEntity" to "TargetEntity" quantity 100
Flow "ResourceName" @cqrs { "kind": "command" } from "A" to "B" quantity 50
```

### Instance Declaration
```sea
Instance instance_name of "EntityName" {
  field_name: "string_value",
  amount: 100 "USD",
  is_active: true
}
```

### Role & Role Binding Declarations
```sea
Role "Approver"
Role "Approver" in governance

role_binding "Approver" for "CaseEntity"
```

### Relation Declaration
```sea
Relation "Payment"
  subject: "Payer"
  predicate: "pays"
  object: "Payee"
  via: flow "Money"
```

### Dimension & Unit Declarations
```sea
dimension "DimensionName"

unit "UnitName" of "DimensionName" factor 1 base "BaseUnit"
```

---

## 3. Application Contract Declarations (ADR-013)

### Record Declaration
```sea
record RecordName {
  field_name: field_type optional (constraints) default literal
}
```

#### Field Types:
- Scalar: `string`, `int`, `decimal`, `bool`, `timestamp`, `uuid`
- Quantity: `quantity<DimensionName.UnitName>`
- Reference: `ref<EntityName>`
- List: `list<scalar_type | ref_type | named_type>`

#### Field Constraints:
- String / List length: `min_length N`, `max_length N`, `min_items N`, `max_items N`
- Numeric bounds: `min N`, `max N`, `exclusive_min N`, `exclusive_max N`
- Regex pattern: `pattern PatternName`

### Enum Declaration
```sea
enum EnumName {
  VARIANT_A = "value_a",
  VARIANT_B = "value_b"
}
```

### Operation Declaration
```sea
operation OperationName {
  intent "Human readable description"
  direction inbound | outbound | internal
  actor "RoleOrEntity" | anonymous
  access public | policy_governed by PolicyName at precondition fails with FailureCode
  input RecordName
  output RecordName
  state "EntityName"
  effect creates | mutates | reads "EntityName"
  transaction single_aggregate | read_only
  idempotency keyed_by field_name | inherent | not_applicable(read_only, "reason")
  concurrency unique_key field_name | optimistic_version field_name | read_snapshot
  failure FailureName for input_validation | policy | missing_state "description"
  evidence operation_trace
  lifecycle synchronous_request_response
}
```

---

## 4. Policy Declarations

```sea
policy policy_name as: expression

policy policy_name per Constraint | Derivation | Obligation Obligation | Prohibition | Permission priority N as: expression
  @rationale "Why this policy exists"
  @tags ["tag1", "tag2"]
```

---

## 5. Expression Operators & Precedence

| Precedence | Category | Operators |
|---|---|---|
| 1 (Lowest) | Logical Disjunction | `or` |
| 2 | Logical Conjunction | `and` |
| 3 | Logical Negation | `not` |
| 4 | Comparison | `=`, `!=`, `<`, `<=`, `>`, `>=`, `matches`, `contains`, `startswith`, `endswith`, `before`, `after`, `during`, `has_role` |
| 5 | Addition / Subtraction | `+`, `-` |
| 6 | Multiplication / Division | `*`, `/` |
| 7 | Unary / Cast | `-`, `as "USD"` |
| 8 (Highest) | Primary Expressions | Literals, `forall`, `exists`, `exists_unique`, `count()`, `sum()`, `min()`, `max()`, `avg()`, member access (`f.quantity`), parentheses |

---

## 6. Literals Syntax

- **Strings**: `"simple string"` or `"""multi-line string"""`
- **Numbers**: `100`, `-42`, `3.14159`
- **Booleans**: `true`, `false`
- **Quantities**: `100 "USD"`, `50.5 "kg"`
- **Timestamps**: `"2026-09-02T20:00:00Z"` (ISO 8601 with required timezone)
- **Arrays**: `["item1", "item2"]`

---

## Source Trail
- `domainforge-core/grammar/sea.pest` — Formal PEG grammar definition
- `domainforge-core/src/parser/ast.rs` — Rust AST node structures
