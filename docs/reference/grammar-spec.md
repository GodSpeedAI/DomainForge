# Grammar Specification

Canonical reference for the SEA DSL grammar. This mirrors `sea-core/grammar/sea.pest` and demonstrates the supported syntax with copy/pasteable examples. Keywords are case-insensitive in the parser (`^"keyword"`), but documentation and pretty-printing use capitalized forms (e.g., `Entity`, `Resource`, `Flow`).

## File shape

```sea
program = { SOI ~ file_header? ~ declaration* ~ EOI }
file_header = { annotation* ~ import_decl* }
```

- **Annotations**: `@namespace`, `@version`, `@owner`, `@profile` with string values.
- **Imports**: `Import {Foo, Bar as Baz} from "path/to/file.sea"`, `Import * as alias from "module"`, or `Import { Service } from "std:core"`.

Example header:

```sea
@namespace "finance.payments"
@owner "platform-team"
@profile "cloud"
Import {Ledger as L} from "../common/ledger.sea"
Import { Service } from "std:core"
```

## Names, identifiers, literals

- **Names** (`name` rule): double-quoted string or `"""` multiline string.
- **Identifiers** (`identifier` rule): `[A-Za-z_][A-Za-z0-9_]*`.
- **String literal**: double-quoted with escapes; multiline uses triple quotes.
- **Number**: decimal with optional fraction and leading sign; no scientific notation.
- **Boolean**: `true` / `false`.
- **Comments/whitespace**: `// ...` to end of line; whitespace ignored outside strings.

## Declarations

Order is flexible; duplicates fail validation.

### Entity

```sea
Entity "Name" [v1.2.3] [@replaces "Old" v1.0.0] [@changes ["note1", "note2"]] [in domain]
```

Example:

```sea
Entity "VendorV2" v2.0.0
    @replaces "Vendor" v1.0.0
    @changes ["added credit_limit", "added payment_terms"]
    in procurement
```

### Resource

```sea
Resource "Name" [units|<unit>] [in domain]
```

Examples:

```sea
Resource "Money" USD in finance
Resource "Logs" units
```

### Flow

```sea
Flow "ResourceName" from "SourceEntity" to "TargetEntity" [quantity <number>]
```

Example:

```sea
Flow "Money" from "Customer" to "PaymentProcessor" quantity 1000
```

Custom flow annotations are supported using `@<identifier> <value>` with JSON-like values:

```sea
Flow "Money"
  @cqrs { "kind": "command" }
  @tx { "transactional": false }
from "Customer" to "PaymentProcessor" quantity 1000
```

### Pattern

```sea
Pattern "Email" matches "^[a-z0-9._%+-]+@[a-z0-9.-]+\\.[a-z]{2,}$"
```

### Role

```sea
Role "Approver" [in domain]
```

### Relation

```sea
Relation "Payment"
  subject: "Payer"
  predicate: "pays"
  object: "Payee"
  [via: flow "Money"]
```

### ConceptChange

```sea
ConceptChange "Vendor_v2_migration"
  @from_version v2.0.0
  @to_version v2.1.0
  @migration_policy mandatory
  @breaking_change true
```

### Instance

```sea
Instance instance_id of "EntityName" [{
    field: expression,
    other: expression
}]
```

Example:

```sea
Instance order_123 of "Order" {
    amount: 150,
    currency: "USD"
}
```

### Dimension and Unit

```sea
Dimension "Currency"
Unit "USD" of "Currency" factor 1 base "USD"
Unit "EUR" of "Currency" factor 1.07 base "USD"
```

### Policy

```sea
Policy policy_name [per Constraint|Derivation|Obligation] [Obligation|Prohibition|Permission] [priority <number>] [@rationale "..."] [@tags ["a","b"]] [v1.0.0] as:
    expression
```

Example:

```sea
Policy payment_threshold per Constraint Obligation priority 5 @tags ["finance"] as:
    forall f in flows: (f.resource = "Money" and f.quantity <= 10000)
```

### Metric

```sea
Metric "name" as: expression
  [@refresh_interval <number> "seconds"]
  [@unit "USD"]
  [@threshold <number>]
  [@severity "critical"]
  [@target <number>]
  [@window <number> "seconds"]
```

### Mapping

```sea
Mapping "name" for calm|kg|sbvr {
    Entity "Customer" -> Target { "id": "customer_id" }
    Flow "Money" -> Target { "from": true }
}
```

### Projection

```sea
Projection "name" for calm|kg|sbvr {
    Entity "Customer" { "name": "customer_name" }
}
```

## Expressions (summary)

```sea
expression      = or_expr
or_expr         = and_expr ("or" and_expr)*
and_expr        = not_expr ("and" not_expr)*
not_expr        = "not" not_expr | comparison_expr
comparison_expr = additive_expr (comparison_op additive_expr)?
comparison_op   = >= | <= | != | = | > | < | contains | startswith | endswith | matches | before | after | during | has_role
additive_expr   = multiplicative_expr ((+|-) multiplicative_expr)*
multiplicative_expr = unary_expr ((*|/) unary_expr)*
unary_expr      = ("-" | "!")* cast_expr
cast_expr       = primary_expr ("as" string_literal)*
primary_expr    = (expression) | group_by_expr | member_access | aggregation_expr | quantified_expr | instance_reference | literal | identifier
```

Additional forms:

- **Quantifier**: `forall x in flows: (x.quantity > 0)` (the `in` keyword is case-insensitive).
- **Aggregation**: `sum(f in flows where f.resource = "Money": f.quantity as "USD")`.
- **Group by**: `group_by(f in flows: f.to) { sum(f.quantity) > 10 }`.
- **Member access**: `flow.quantity`, `entity.name`.
- **Casting**: `100 "ms" as "s"`, `f.quantity as "USD"`.
- **Literals**: strings, multiline strings, numbers, booleans, quantities (`100 "USD"`), time literals (`"2025-12-31T23:59:59Z"`), interval literals (`interval("09:00","17:00")`).

## Validation highlights

- Duplicate declarations in the same namespace are rejected.
- Unknown references (entities, resources, roles, units) fail validation.
- Unit conversions must align with declared dimensions or documented built-in units (see Time units below).
- ConceptChange annotations must include valid semantic versions.
- Policies and metrics use three-valued logic; undefined data may yield `Unknown`.

### Unit conversions & "as" usage

The SEA DSL supports unit literals and a general-purpose `as` coercion operator.

- **Quantity literal**: Write a number followed by a quoted unit (e.g., `100 "USD"`, `1_500 "USD"`, `100.5 "kg"`).
- **Explicit Casting**: Use the `as` operator to convert between compatible units in any expression.
    ```sea
    1000 "ms" as "s"          // Evaluates to 1 "s"
    f.quantity as "USD"       // Converts flow quantity to USD
    ```
- **Aggregation**: The `as` operator is commonly used in aggregations to normalize units before summing.
    ```sea
    sum(f in flows where f.resource = "Money": f.quantity as "USD")
    ```

- **Declaration `as:` introducer**: The `as` keyword is also used in `Policy` and `Metric` declarations to introduce expression bodies (e.g. `Policy name as:`). This is a distinct syntactic usage.

### Built-in Time units

Common time units are available without explicit declarations. The compiler preloads the `Time` dimension with the following units:

- `s` (base, seconds)
- `ms` (milliseconds, factor `0.001` to `s`)
- `us` (microseconds, factor `0.000001` to `s`)
- `ns` (nanoseconds, factor `0.000000001` to `s`)

Examples such as `1000 "ms" as "s"` rely on these built-ins. Projects can still declare additional time units, and all conversions must remain dimensionally compatible.

## Workflow for grammar changes

1. Update `sea-core/grammar/sea.pest`.
2. Update AST and parser in `sea-core/src/parser/`.
3. Add parser tests in `sea-core/tests/parser_*.rs`.
4. Update projections (CALM/KG) if affected.
5. Refresh this document and relevant examples.
