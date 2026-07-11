# Write In SEA

Goal: Write valid `.sea` files for DomainForge models, policies, metrics, imports, and projection contracts.

SEA is the DomainForge source language. The canonical implementation is the Rust parser in `domainforge-core/grammar/sea.pest` and `domainforge-core/src/parser/ast.rs`. Use this guide when authoring `.sea` files by hand.

## Prerequisites

- DomainForge CLI installed or built from this repo.
- A text editor that saves files as UTF-8.
- Optional: `.sea-registry.toml` when the file imports modules by namespace.

## Start With A Minimal File

Create `model.sea`:

```sea
@namespace "logistics"
@version "1.0.0"
@owner "architecture-team"
@profile "default"

Entity "Warehouse"
Entity "Factory"

Resource "Camera" units

Flow "Camera" from "Warehouse" to "Factory" quantity 100

Policy positive_flows as:
    forall f in flows: (f.quantity > 0)
```

Validate it:

```bash
domainforge validate --format human model.sea
```

If you are working from the repository, build the CLI first:

```bash
cargo install --path domainforge-core --features cli
```

## File Shape

A `.sea` file has an optional header, then zero or more declarations.

```sea
@namespace "finance.payments"
@version "1.2.3"
@owner "payments-team"
@profile "cloud"

import { Money, Ledger as Book } from "finance.common"
import * as core from "std:core"

export Entity "PaymentService"
```

Rules:

- Header annotations use string values: `@namespace`, `@version`, `@owner`, and `@profile`.
- Imports must use `import { Name } from "module"` or `import * as alias from "module"`.
- Exports wrap a normal declaration: `export Entity "Customer"`.
- `//` starts a line comment. `#` is not a SEA comment.
- Keywords are case-insensitive. The formatter prints capitalized keywords.

## Names, Identifiers, And Literals

Use quoted names for domain concepts:

```sea
Entity "Accounts Payable"
Resource "Purchase Order" units
```

Use identifiers for policy names, instance names, aliases, and unquoted domains:

```sea
Policy credit_limit_check as: true
Instance vendor_123 of "Vendor"
Entity "Vendor" in procurement
```

Identifier rules:

- Start with a letter or `_`.
- Continue with letters, numbers, or `_`.
- Do not include dots, hyphens, spaces, or slashes.
- Use `@namespace "finance.payments"` for dotted namespace strings. Do not write `in finance.payments`; `in` expects an identifier.

Literal rules:

- Strings use double quotes and JSON-style escapes: `"line\nnext"`, `"quote: \""`.
- Multiline names and strings use triple quotes: `"""Long\nName"""`.
- Numbers are decimal: `42`, `19.99`, `-10`. Scientific notation is not supported.
- Booleans are `true` and `false`.
- Quantity literals combine a number and quoted unit: `100 "USD"`, `1000 "ms"`.
- Time literals require a timezone: `"2026-07-06T12:30:00Z"` or `"2026-07-06T12:30:00-04:00"`.
- Intervals use function syntax: `interval("09:00", "17:00")`.

## Declare Core Model Elements

### Entities

Entities are actors, systems, concepts, or domain objects.

```sea
Entity "Vendor" in procurement

Entity "VendorV2" v2.0.0
    @replaces "Vendor" v1.0.0
    @changes ["added credit_limit", "added payment_terms"]
    in procurement
```

Semantics:

- Entity IDs come from namespace plus name.
- If `in <domain>` is omitted, the file `@namespace`, parser default namespace, or `"default"` is used.
- Duplicate entity names in the same namespace fail validation.

### Resources

Resources are things that flow between entities.

```sea
Resource "Money" USD in finance
Resource "Camera" units
Resource "Invoice" in finance
```

Semantics:

- The optional unit is an identifier, not a quoted string.
- If no unit is provided, DomainForge uses `units`.
- `Resource "Invoice" in finance` sets only the domain. It does not set a unit named `in`.

### Flows

Flows connect declared entities through a declared resource.

```sea
Flow "Money" from "Customer" to "PaymentProcessor" quantity 1000

Flow "Command"
    @cqrs { "kind": "command" }
    @tx { "transactional": false }
    from "Client" to "Server"
```

Semantics:

- The resource, source entity, and target entity must already exist somewhere in the parsed graph.
- `quantity` is optional and defaults to zero in the graph.
- Flow references are unqualified names. If the same name exists in multiple namespaces and the default namespace does not disambiguate it, validation fails as ambiguous.
- Custom flow annotations accept strings, string arrays, booleans, numbers, and object literals.

### Roles And Relations

Roles describe participant positions. Relations connect roles and may reference a resource through `via: flow`.

```sea
Role "Payer"
Role "Payee"

Relation "Payment"
    subject: "Payer"
    predicate: "pays"
    object: "Payee"
    via: flow "Money"
```

Semantics:

- `subject` and `object` must reference declared roles.
- `via: flow "Money"` resolves against resources, not individual flow IDs.
- Relation names must be unique.

### Patterns

Patterns define named regular expressions for policy matching.

```sea
Pattern "EmailAddress" matches "^[^@\\s]+@[^@\\s]+\\.[^@\\s]+$"

Policy valid_email as:
    "user@example.com" matches "EmailAddress"
```

Semantics:

- Regexes are compiled during graph construction.
- Invalid regex syntax fails validation.

### Instances

Instances attach data to an entity type.

```sea
Entity "Vendor"

Instance vendor_123 of "Vendor" {
    name: "Acme Corp",
    credit_limit: 50000,
    active: true,
    balance: 1000 "USD"
}
```

Semantics:

- The entity type must exist in the same namespace.
- Instance names are identifiers.
- Field names must be unique within the instance.
- Instance fields can store strings, numbers, booleans, variables as strings, quantity literals, and time literals. Complex expressions are rejected when the AST converts the instance to graph data.

## Declare Units

Define dimensions before custom units:

```sea
Dimension "Currency"
Unit "USD" of "Currency" factor 1 base "USD"
Unit "EUR" of "Currency" factor 1.07 base "USD"

Resource "Money" USD
```

Semantics:

- Unit symbols are quoted in `Unit` declarations but unquoted in `Resource` declarations.
- A unit belongs to one dimension and has a factor relative to its base unit.
- Conflicting redefinitions of the same unit fail validation.
- Built-in time units include `s`, `ms`, `us`, and `ns`.
- Use `as "Unit"` in expressions to convert compatible units.

```sea
Policy latency_budget as:
    (1000 "ms" as "s") <= 1 "s"
```

## Write Policies

Policies use an identifier name, optional metadata, optional version, and an expression after `as:`.

```sea
Policy payment_limit
    per Constraint Obligation priority 5
    @rationale "Keep individual payments below the review threshold"
    @tags ["finance", "risk"]
    v1.0.0
    as:
        forall f in flows: (f.resource = "Money" and f.quantity <= 10000)
```

Defaults:

- Kind: `Constraint`.
- Modality: `Obligation`.
- Priority: `0`.
- Version: policy version if present, otherwise file `@version`, otherwise default semantic version.

Policy kinds:

- `Constraint`
- `Derivation`
- `Obligation`

Policy modalities:

- `Obligation`
- `Prohibition`
- `Permission`

## Expression Syntax

Operator precedence, from tightest to loosest:

1. Parentheses, literals, identifiers, member access, quantifiers, aggregations, and `group_by`.
2. Cast: `expr as "Unit"`.
3. Unary minus: `-value`.
4. Multiplication and division: `*`, `/`.
5. Addition and subtraction: `+`, `-`.
6. Comparisons.
7. `not`.
8. `and`.
9. `or`.

Comparison operators:

```sea
=  !=  >  <  >=  <=
contains  startswith  endswith  matches
before  after  during  has_role
```

Collections:

```sea
flows
entities
resources
instances
relations
```

Quantifiers:

```sea
Policy every_flow_positive as:
    forall f in flows: (f.quantity > 0)

Policy has_customer as:
    exists e in entities: (e.name = "Customer")

Policy one_money_resource as:
    exists_unique r in resources: (r.name = "Money")
```

Aggregations:

```sea
Metric "payment_count" as:
    count(f in flows where f.resource = "Money": f.quantity)

Metric "payment_total" as:
    sum(f in flows where f.resource = "Money": f.quantity as "USD")

Policy enough_volume as:
    sum(f in flows where f.resource = "Money": f.quantity as "USD") > 1000 "USD"
```

Rules:

- Supported aggregate functions are `count`, `sum`, `min`, `max`, and `avg`.
- Simple aggregations use `count(flows)` or `sum(flows.quantity)`.
- Comprehensions use `<var> in <collection> [over last N "unit"] [where predicate]: projection`.
- In a policy boolean context, compare an aggregation explicitly: `count(flows) > 0`, not just `count(flows)`.

Group by:

```sea
Policy destination_minimum as:
    group_by(f in flows where f.quantity > 0: f.to_entity) {
        sum(f.quantity) >= 100
    }
```

## Write Metrics

Metrics are named expressions with optional metadata annotations.

```sea
Metric "payment_success_rate" as:
    count(flows)
    @refresh_interval 60 "seconds"
    @window 1 "hour"
    @target 99.9
    @threshold 95
    @severity "warning"
    @unit "percent"
```

Rules:

- Duration annotations accept `second(s)`, `s`, `minute(s)`, `m`, `hour(s)`, `h`, `day(s)`, and `d`.
- Severity accepts `info`, `warning`, `error`, and `critical`.
- Unknown duration units or severities fail validation.

## Track Evolution

Use entity annotations and `ConceptChange` declarations to describe versioned changes.

```sea
Entity "VendorV2" v2.0.0
    @replaces "Vendor" v1.0.0
    @changes ["added payment_terms"]
    in procurement

ConceptChange "Vendor_v2_migration"
    @from_version v1.0.0
    @to_version v2.0.0
    @migration_policy mandatory
    @breaking_change true
```

Rules:

- Versions use semantic version syntax: `1.2.3`.
- `ConceptChange` requires `@from_version`, `@to_version`, and `@migration_policy`.
- `@breaking_change` is optional and defaults to `false`.

## Write Mapping And Projection Contracts

Mappings describe how primitives map to external target structures.

```sea
Mapping "calm_entities" for calm {
    Entity "PaymentService" -> Node { id: "payment_service", critical: true }
    Flow "Money" -> Edge { kind: "payment" }
}
```

Projections override exported fields for a target.

```sea
Projection "calm_names" for calm {
    Entity "PaymentService" {
        name: "payment-service",
        properties: { "owner" -> "team" }
    }
}
```

Targets:

- `calm`
- `kg`
- `sbvr`
- `protobuf` or `proto`

Supported primitive types in these contracts:

- `Entity`
- `Resource`
- `Flow`
- `Policy`
- `Instance`

## Use Imports Correctly

Use the current import grammar:

```sea
@namespace "acme.order"
import { Customer } from "acme.common"
import { Money as Currency } from "finance.common"
import * as common from "acme.common"

export Entity "Order"
```

Rules:

- Named imports support optional aliases with `as`.
- Wildcard imports require an alias.
- Module resolution requires parser options with both a namespace registry and entry path.
- Standard modules use strings such as `"std:core"`.
- The module resolver validates exports and detects dependency cycles.

## Semantic Validation Rules

A file can parse but still fail graph construction or validation. Common semantic rules:

- Entity, role, resource, pattern, metric, mapping, projection, policy, and instance IDs must be unique where their graph collection requires uniqueness.
- Flows require known source entities, target entities, and resources.
- Relations require known subject and object roles.
- Instances require a known entity type in the instance namespace.
- Patterns must contain valid regexes.
- Unknown profiles fail unless parser options allow profile warnings.
- Unit conversions must use compatible dimensions.
- Aggregations in policy boolean context require explicit comparison.
- Ambiguous unqualified references fail when multiple namespaces contain the same name.
- Policy evaluation uses three-valued logic by default. Missing data may evaluate to unknown and produce a violation according to policy modality.

## Authoring Checklist

1. Put file metadata at the top.
2. Declare dimensions and units before resources that use them.
3. Declare roles, entities, and resources before flows, relations, instances, policies, and metrics that reference them.
4. Use quoted names for concepts and identifiers for policy names, domains, aliases, and instance names.
5. Keep domains identifier-shaped after `in`; put dotted names in `@namespace`.
6. Compare aggregation results explicitly in policies.
7. Validate with `domainforge validate --format human <file>.sea`.
8. Use `domainforge parse --ast --format json <file>.sea` when you need to inspect parsed structure.

## Troubleshooting

- **Syntax error near a concept name**: Quote the concept name, for example `Entity "Customer"`.
- **Unexpected token near `as`**: Use `as:` only for `Policy` and `Metric` bodies. Use `as "Unit"` only as an expression cast.
- **Unknown entity or resource**: Add the missing declaration or fix the namespace/reference name.
- **Ambiguous reference**: Move the referenced declaration into the default namespace for the file or rename one declaration.
- **Unit conversion failed**: Define both units in the same dimension and verify their base units.
- **Metric duration failed**: Use seconds, minutes, hours, or days, or their short forms.
- **Import failed**: Confirm `.sea-registry.toml` maps the module string to a file and that imported symbols are exported.

## Links

- Tutorial: [First SEA Model](../tutorials/first-sea-model.md)
- How-tos: [Parse SEA Files](parse-sea-files.md), [Define Policies](define-policies.md), [Create Custom Units](create-custom-units.md), [Use Modules and Imports](use-modules-imports.md)
- Reference: [Grammar Spec](../reference/grammar-spec.md), [CLI Commands](../reference/cli-commands.md), [Primitives API](../reference/primitives-api.md), [Policy Evaluation Logic](../explanations/policy-evaluation-logic.md)
