# Primitives API Reference

This reference documents the core data structures used throughout DomainForge across Rust, Python, and TypeScript bindings. It mirrors the definitions in `sea-core/src/primitives/` and the exposed binding classes in `sea-core/src/python/` and `sea-core/src/typescript/`.

## Types

- `AttributeValue`: Union used for attribute payloads. Acceptable values include:
  - `null`
  - `boolean`
  - `string`
  - `Decimal` (arbitrary-precision decimal represented as a `string` in JSON)
  - `bytes` (base64 encoded strings in JSON)
  - `array` of `AttributeValue`
  - `object` maps (`string` -> `AttributeValue`)

- `AttributeMap`: A map of `string` -> `AttributeValue`. Implementations use `IndexMap` for deterministic iteration.

- `Decimal`: Arbitrary-precision decimal used for quantities and numeric values. In JSON, `Decimal` values are serialized as strings to preserve precision (for example, `"1.07"`).

## Overview

The SEA DSL produces a graph of primitives:

- **Entity**: actors or systems participating in flows.
- **Resource**: what is being transferred or measured.
- **Flow**: movement of a resource between entities.
- **ResourceInstance**: concrete instance of a resource with optional quantity.
- **Role**: label describing a participant category.
- **Relation**: predicate connecting roles, optionally tied to a flow.
- **Policy**: expression evaluated against the graph.

Each section below lists fields, constructors, methods, validation rules, and serialization notes.

## Entity

### Fields

- `id: Uuid` — stable identifier used in references.
- `name: String` — user-supplied name from the DSL.
- `namespace: Option<String>` — defaults to `"default"` when omitted.
- `attributes: IndexMap<String, AttributeValue>` — arbitrary key/value metadata.

### Constructors

- Rust: `Entity::new(name, namespace, attributes)`
- Python: `Entity(name: str, namespace: Optional[str] = None, attributes: Optional[dict] = None)`
- TypeScript: `new Entity(name: string, namespace?: string, attributes?: Record<string, AttributeValue>)`

### Methods

- Accessors for `id`, `name`, `namespace`, `attributes` (read-only in bindings).
- Equality based on `id`.

### Validation

- Names must be unique per namespace.
- Attributes accept string, number, or boolean values; nested objects are not supported.

## Resource

### Fields

- `id: Uuid`
- `name: String`
- `namespace: Option<String>`
- `unit: Option<String>` — concrete unit if bound.
- `has_units: bool` — true when declared with `units` keyword.
- `attributes: AttributeMap`

### Constructors

- Rust: `Resource::new(name, namespace, unit, has_units, attributes)`
- Python: `Resource(name, namespace=None, unit=None, has_units=False, attributes=None)`
- TypeScript: `new Resource(name, namespace?, unit?, hasUnits?, attributes?)`

### Methods

- Getters for `id`, `name`, `namespace`, `unit`, `has_units`, `attributes`.
- `is_dimensional()` in Rust to check units.

### Validation

- If `unit` is set, it must resolve to a declared unit; otherwise validation fails.
- If `has_units` is false, providing quantities with units will error during validation.

## Flow

### Fields

- `id: Uuid`
- `resource_id: Uuid`
- `from: Uuid`
- `to: Uuid`
- `quantity: Option<Quantity>` — `Quantity { value: Decimal, unit: Option<String> }`
- `namespace: Option<String>`
- `attributes: AttributeMap`

### Constructors

- Rust: `Flow::new(resource_id, from, to, quantity, namespace, attributes)`
- Python: `Flow(resource_id: str, from_id: str, to_id: str, quantity: Optional[Quantity] = None, namespace: Optional[str] = None, attributes: Optional[dict] = None)`
- TypeScript: `new Flow(resourceId: string, fromId: string, toId: string, quantity?: Quantity, namespace?: string, attributes?: Record<string, AttributeValue>)`

### Methods

- Accessors for `id`, `resource_id`, `from_id`, `to_id`, `quantity`, `namespace`, `attributes`.
- `quantity.as_unit("<Unit>")` is available in Rust for conversions; bindings expose quantity fields directly.

### Validation

- `resource_id`, `from`, and `to` must refer to existing resources/entities in the same namespace.
- Quantity units must match the resource dimension or be convertible.

## ResourceInstance

### Fields

- `id: Uuid`
- `name: String` — instance name from DSL `Instance "name"`.
- `resource_id: Uuid`
- `quantity: Option<Quantity>`
- `namespace: Option<String>`
- `attributes: AttributeMap`

### Constructors

- Rust: `ResourceInstance::new(name, resource_id, quantity, namespace, attributes)`
- Python: `ResourceInstance(name: str, resource_id: str, quantity: Optional[Quantity] = None, namespace: Optional[str] = None, attributes: Optional[dict] = None)`
- TypeScript: `new ResourceInstance(name: string, resourceId: string, quantity?: Quantity, namespace?: string, attributes?: Record<string, AttributeValue>)`

### Methods

- Accessors for `id`, `name`, `resource_id`, `quantity`, `namespace`, `attributes`.
- Instances can be added to a graph via `Graph.add_instance`/`addInstance`.

### Validation

- Resource must exist.
- Quantity validation mirrors flows.

## Role

### Fields

- `id: Uuid`
- `name: String`
- `namespace: Option<String>`
- `attributes: AttributeMap`

### Constructors

- Rust: `Role::new(name, namespace, attributes)`
- Python: `Role(name: str, namespace: Optional[str] = None, attributes: Optional[dict] = None)`
- TypeScript: `new Role(name: string, namespace?: string, attributes?: Record<string, AttributeValue>)`

### Methods

- Accessors: `id`, `name`, `namespace`, `attributes`.
- Roles are used primarily as references from relations.

### Validation

- Names unique per namespace.
- Attributes follow the same rules as entities.

## Relation

### Fields

- `id: Uuid`
- `name: String`
- `subject_role_id: Uuid`
- `predicate: String`
- `object_role_id: Uuid`
- `via_flow_id: Option<Uuid>`
- `namespace: Option<String>`
- `attributes: AttributeMap`

### Constructors

- Rust: `Relation::new(name, namespace, subject_role, predicate, object_role, via_flow)`
- Python: `Relation(name: str, subject_role_id: str, predicate: str, object_role_id: str, via_flow_id: Optional[str] = None, namespace: Optional[str] = None, attributes: Optional[dict] = None)`
- TypeScript: `new Relation(name: string, subjectRoleId: string, predicate: string, objectRoleId: string, viaFlowId?: string, namespace?: string, attributes?: Record<string, AttributeValue>)`

### Methods

- Accessors for all fields.
- Relations are added to the graph via `Graph.add_relation`/`addRelation`.

### Validation

- Subject/object role IDs must exist.
- `via_flow_id`, if present, must point to a declared flow.
- Namespace alignment is enforced with the referenced roles/flows.

## Policy

Policies are represented as serialized JSON across bindings and evaluated via graph methods.

- Rust: `Graph::evaluate_policy(policy: Policy) -> EvaluationResult`
- Python: `Graph.evaluate_policy(policy_json: str) -> PolicyResult`
- TypeScript: `graph.evaluatePolicy(policyJson: string): PolicyResult`

Validation: policies must reference existing entities/resources/flows. Unknown references produce `Unknown` outcomes in three-valued logic.

## Quantity type

Quantities are reused by flows and instances.

- Fields: `value: Decimal`, `unit: Option<String>`
- Python: `Quantity(value: Decimal, unit: Optional[str])`
- TypeScript: `{ value: number | string; unit?: string }` — The TypeScript bindings return a `number` for convenience; however, when serializing to JSON the Decimal is represented as a `string` to preserve precision. Consumers should prefer string representation when round-tripping or when precise decimal fidelity is required.

Conversions follow the dimension/unit registry; mismatches are surfaced as validation errors.

## Graph methods (shared subset)

While not primitives themselves, the graph API is the primary way to create and retrieve primitives.

Common methods across bindings:

- Add: `add_entity`, `add_resource`, `add_flow`, `add_instance`, `add_role`, `add_relation`
- Counts: `entity_count`, `resource_count`, `flow_count`, `instance_count`, `role_count`, `relation_count`, `pattern_count`
- Lookup by ID: `get_entity`, `get_resource`, `get_flow`, `get_instance`
- Lookup by name: `find_entity_by_name`, `find_resource_by_name`, `find_role_by_name`
- Collection accessors: `all_entities`, `all_resources`, `all_flows`, `all_instances`, `all_roles`, `all_relations`

## Serialization

Primitives serialize to JSON for interop. The canonical Rust structures derive `Serialize`/`Deserialize`; bindings reuse those shapes.

Example `Resource` JSON:

```json
{
  "id": "3a3e5d08-9c4d-4f59-a8af-5f52a1d69c20",
  "name": "Money",
  "namespace": "finance",
  "unit": "USD",
  "has_units": true,
  "attributes": {"type": "currency"}
}
```

### CALM mapping

- Entities map to `models[n].entities`.
- Resources map to `models[n].resources` with units preserved.
- Roles/relations emit as typed facts (`subjectRole`, `predicate`, `objectRole`).
- Flows and instances become facts with `sea:id` linking back to DSL IDs.

See `calm-mapping.md` for full mapping tables.

## Equality and hashing

- Equality for primitives is defined on `id` only; names/attributes do not affect equality.
- Hashing uses UUID bytes to ensure stable map keys across projections.

## Thread safety

- Rust structs are `Send + Sync` where possible; binding wrappers ensure ownership rules are upheld.
- Python uses PyO3 `PyClass` wrappers; TypeScript uses `napi` classes backed by Arc-managed Rust data.

## Error handling

- Constructor errors surface as `PyValueError` in Python and `Error` in TypeScript when references are invalid.
- Semantic validation returns `ValidationError` (Rust) with codes listed in `error-codes.md`.

## Examples

### Creating primitives in Python

```python
from sea_dsl import Graph, Entity, Resource, Flow, Role, Relation

graph = Graph()
user = Entity("User")
money = Resource("Money", has_units=True)
payer = Role("Payer")
payee = Role("Payee")
flow = Flow(money.id, user.id, user.id)
relation = Relation("Payment", payer.id, "pays", payee.id, flow.id)

graph.add_entity(user)
graph.add_resource(money)
graph.add_flow(flow)
graph.add_role(payer)
graph.add_role(payee)
graph.add_relation(relation)
```

### Creating primitives in TypeScript

```ts
import { Graph, Entity, Resource, Flow, Role, Relation } from "@domainforge/sea";

const graph = new Graph();
const user = new Entity("User");
const money = new Resource("Money", undefined, undefined, true);
const payer = new Role("Payer");
const payee = new Role("Payee");
const flow = new Flow(money.id, user.id, user.id);
const relation = new Relation("Payment", payer.id, "pays", payee.id, flow.id);

graph.addEntity(user);
graph.addResource(money);
graph.addFlow(flow);
graph.addRole(payer);
graph.addRole(payee);
graph.addRelation(relation);
```

## See also

- `docs/new_docs/reference/python-api.md` and `typescript-api.md` for method-level binding docs.
- `docs/new_docs/reference/cli-commands.md` for CLI operations over these primitives.
- `docs/new_docs/how-tos/parse-sea-files.md` for creating graphs from DSL input.
