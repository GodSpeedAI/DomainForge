# Grammar Specification

This document is the canonical reference for the SEA (Semantic Entity Architecture) grammar. It mirrors the rules defined in `sea-core/grammar/sea.pest` and complements them with examples, edge cases, and validation notes so authors can reason about how the parser interprets DSL files.

## Reading this document

- **Scope**: Every rule currently accepted by the parser. Deprecated constructs are called out explicitly.
- **Notation**: Rules are written in Pest-style syntax alongside EBNF-like prose. Literals appear in quotes, identifiers in *italics*, and optional parts in square brackets.
- **Examples**: Each rule includes at least one minimal and one realistic example. Copy/paste them into `sea parse` to verify behavior.
- **Validation**: Notes explain when the semantic validator rejects syntactically valid constructs (e.g., unknown units or duplicate IDs).

## Top-level structure

A SEA file is a sequence of declarations. Whitespace and comments may appear between declarations.

```
file = { SOI ~ decl* ~ EOI }
```

Supported declaration types:

- Namespace block
- Dimension and Unit declarations
- Entity, Resource, Flow declarations
- Instances of resources
- Roles and relations
- Policies

Example (minimal file):

```
Namespace "default"
Entity "User"
Resource "Money" units
Flow "Money" from "User" to "User" quantity 1
```

## Lexical elements

### Identifiers and strings

Identifiers are double-quoted strings. They may include spaces and symbols except for an unescaped `"`. Escapes follow Rust string rules (e.g., `\"`).

```
quoted = _{ '"' ~ (\\("\\"|"\\\\"|"\\n"|"\\r"|"\\t") | (!'"' ~ ANY))* ~ '"' }
```

### Numbers

Quantities and unit factors use decimal numbers. Scientific notation is not supported.

```
number = @{ "-"? ~ ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)? }
```

- Leading zeros are allowed (`0.5`).
- Trailing decimals without a fraction (`10.`) are rejected by the parser.

### Comments and whitespace

Comments start with `//` and run to the end of the line. Blank lines and spaces are ignored except inside strings.

```
comment = _{ "//" ~ (!NEWLINE ~ ANY)* }
```

## Namespaces

```
namespace_decl = { "Namespace" ~ quoted }
```

- Optional. If omitted, the default namespace is `"default"`.
- Declarations that follow inherit the last declared namespace until a new namespace appears.

Example:

```
Namespace "finance"
Entity "Account"
Namespace "ops"
Entity "User"
```

## Dimensions and units

Dimensions introduce a physical dimension; units attach to a dimension and optionally specify conversion factors.

```
dimension_decl = { "Dimension" ~ quoted }
unit_decl = {
  "Unit" ~ quoted ~ "of" ~ quoted ~
  ["factor" ~ number] ~ ["base" ~ quoted]
}
```

Rules:

- `factor` multiplies the base unit to reach this unit. Defaults to `1`.
- `base` identifies the canonical unit within the dimension. At least one unit per dimension must declare a base.
- Circular `base` chains fail validation even if they parse.

Example:

```
Dimension "Currency"
Unit "USD" of "Currency" factor 1 base "USD"
Unit "EUR" of "Currency" factor 1.07 base "USD"
```

## Entities

Entities represent actors or systems.

```
entity_decl = { "Entity" ~ quoted }
```

Notes:

- IDs are case-sensitive; duplicates in the same namespace are rejected.
- Use namespaces to disambiguate logically distinct entities with the same display name.

## Resources

Resources describe what flows between entities.

```
resource_decl = { "Resource" ~ quoted ~ [resource_mod*] }
resource_mod = { "units" | "unit" ~ quoted }
```

- `units` marks the resource as dimensional without fixing a unit (e.g., `Resource "Money" units`).
- `unit "<Unit>"` binds the resource to a specific unit and is validated against known dimensions.

## Flows

Flows connect a resource between two entities and may assign quantities.

```
flow_decl = {
  "Flow" ~ quoted ~ "from" ~ quoted ~ "to" ~ quoted ~
  ["quantity" ~ number] ~ [quantity_unit]
}
quantity_unit = { quoted }
```

Rules:

- The resource name must match a declared resource (same namespace).
- `quantity` is optional. If present with units, units must align with the resource’s dimension.

Example:

```
Flow "Money" from "Alice" to "Bob" quantity 100 "USD"
```

## Instances

Instances attach concrete IDs and quantities to resources for runtime evaluation.

```
instance_decl = {
  "Instance" ~ quoted ~ "of" ~ quoted ~
  ["quantity" ~ number ~ [quoted]]
}
```

Notes:

- `Instance "order-1" of "Order"` associates a logical instance identifier with a resource definition.
- If a quantity unit is provided, it is validated against the resource unit/dimension.

## Roles

Roles label participants in relations or flows.

```
role_decl = { "Role" ~ quoted ~ [role_mod*] }
role_mod = { "namespace" ~ quoted | "attributes" ~ attributes }
```

- `attributes` mirrors entity attributes syntax (key/value pairs) and is optional.
- Role IDs must be unique per namespace.

## Relations

Relations connect two roles with a predicate, optionally tied to a flow.

```
relation_decl = {
  "Relation" ~ quoted ~
  "subject:" ~ quoted ~
  "predicate:" ~ quoted ~
  "object:" ~ quoted ~
  ["via:" ~ flow_ref]
}
flow_ref = { "flow" ~ quoted }
```

Rules:

- Subject/object names refer to previously declared roles.
- The optional `via: flow` binds the relation to a flow identifier.

Example:

```
Relation "Payment"
  subject: "Payer"
  predicate: "pays"
  object: "Payee"
  via: flow "Money"
```

## Attributes

Entities, resources, roles, and instances may carry attributes.

```
attributes = { "{" ~ (attribute_pair ~ ("," ~ attribute_pair)*)? ~ "}" }
attribute_pair = { quoted ~ ":" ~ attribute_value }
attribute_value = { quoted | number | "true" | "false" }
```

- Attribute maps do not preserve order.
- Boolean values are lowercase.

## Policies

Policies define logical assertions over graph elements.

```
policy_decl = { "Policy" ~ quoted ~ "as:" ~ policy_expr }
```

### Expressions

Core expression forms include comparisons, logical connectives, quantifiers, and arithmetic. The Pest grammar encodes precedence; high level:

- Literals: numbers, strings, booleans
- Accessors: `f.quantity`, `e.name`
- Comparators: `=`, `!=`, `<`, `<=`, `>`, `>=`, `matches`
- Logical: `and`, `or`, `not`
- Quantifiers: `forall`, `exists`

Example policy:

```
Policy payment_threshold as:
  forall f in flows where f.resource = "Money":
    f.quantity <= 10000 "USD"
```

### Quantifiers

```
quantifier = { ("forall" | "exists") ~ ident ~ "in" ~ collection ~ ["where" ~ predicate] ~ ":" ~ policy_expr }
collection = { "entities" | "resources" | "flows" | "instances" }
ident = { (LETTER | "_") ~ (LETTER | NUMBER | "_")* }
```

- The `where` clause filters before evaluating the body.
- Bodies may nest quantifiers.

### Matches operator

`matches` performs regex matching using Rust’s `regex` crate semantics.

```
predicate = { value ~ "matches" ~ quoted }
```

- Patterns are anchored by default if you include `^`/`$`; otherwise substring matches are allowed.
- Invalid regex patterns surface as validation errors.

## Errors and validation

Syntactically valid files may still fail semantic validation. Common checks:

- **Undefined references**: flows referencing unknown resources or entities; relations referencing unknown roles.
- **Duplicate IDs**: entities/resources/roles with the same name in a namespace.
- **Unit mismatches**: quantities expressed with units incompatible with the resource dimension.
- **Cardinality**: policies referencing empty collections when `forall` expects at least one element may evaluate to `Unknown` in three-valued mode.

Refer to [`../reference/error-codes.md`](./error-codes.md) for the exhaustive error catalog.

## Projections

The parser output feeds multiple projections:

- **Graph store**: internal representation used by validators and evaluators.
- **CALM export/import**: ensures bidirectional mapping of roles, relations, and units (see `calm-mapping.md`).
- **RDF/Turtle**: mapping to triples in `sea-core/src/kg.rs`.
- **SBVR**: fact types emitted for roles/relations.

## Extending the grammar

To add new constructs:

1. Update `sea-core/grammar/sea.pest` with the new rule.
2. Extend the AST types and parser in `sea-core/src/parser/`.
3. Add semantic validation and projections.
4. Update this document with the new rule, examples, and edge cases.

See the how-to in `docs/new_docs/how-tos/extend-grammar.md` for a guided walkthrough.

## See also

- [`primitives-api.md`](./primitives-api.md) for data structures produced by parsing.
- [`../how-tos/parse-sea-files.md`](../how-tos/parse-sea-files.md) for CLI and language-specific parsing examples.
- [`error-codes.md`](./error-codes.md) for validation failures.
- [`../explanations/semantic-modeling-concepts.md`](../explanations/semantic-modeling-concepts.md) for modeling guidance.
