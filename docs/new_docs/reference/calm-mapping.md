
# SEA DSL ↔ CALM Mapping Specification

**Version:** 1.0
**Date:** 2025-11-07
**Status:** Implementation
**Aligned With:** ADR-006 (CALM), PRD-014 (CALM Interop), SDS-013 (CALM Serialization)

---

## Overview

FINOS CALM (Common Architecture Language Model) represents architecture as code using nodes and relationships. SEA DSL uses domain primitives with graph semantics. This document specifies the bidirectional mapping between these two representations.

## Mapping Strategy

### Core Principles

1. **Lossless Conversion**: All SEA DSL information must be preserved during export/import cycles
2. **CALM Compliance**: Exported JSON must validate against FINOS CALM v1.0 schema
3. **Metadata Preservation**: Domain-specific attributes stored in `metadata` field with `sea:` namespace prefix
4. **ID Mapping**: UUIDs converted to string format for CALM; remapping handled during import

### Mapping Table

| SEA Primitive | CALM Equivalent | Mapping Notes |
|---------------|-----------------|---------------|
| Entity | Node (type: actor/location) | Direct mapping; namespace preserved |
| Resource | Node (type: resource) | Unit stored in metadata |
| Flow | Relationship (type: flow) | Quantity attribute mapped to relationship metadata |
| Instance | Node (type: instance) + Relationship (ownership) | Composite mapping with parent entity link |
| Policy | Node (type: constraint) | SBVR expression stored in metadata |
| Entity Relationships | Relationship (type: association/aggregation) | Map association/aggregation links between entities |
| Instance-to-Entity Links | Relationship (type: ownership) | Map instance containment relationships |
| Inheritance | Node hierarchy + Relationship (type: inheritance) | Preserve type hierarchies as node relationships |
| Composition | Node composition + Relationship (type: composition) | Map composite object structures |
| Policy Propagation | Constraint propagation rules | Define how policies propagate across related nodes/flows |
| Domain Attributes | Node metadata | Preserve units, timestamps, provenance, ownership |
| Metadata Preservation | Node/relationship properties | Maintain domain-specific attributes during conversion |

---

## Detailed Mappings

### 1. Entity → CALM Node

**SEA Entity Structure:**
```rust
Entity {
    id: Uuid,
    name: String,
    namespace: String,
    attributes: HashMap<String, Value>
}
```

**CALM Node Structure:**
```json
{
  "unique-id": "uuid-here",
  "node-type": "actor",
  "name": "Warehouse A",
  "namespace": "logistics",
  "metadata": {
    "sea:primitive": "Entity",
    "sea:attributes": {
      "capacity": 10000,
      "location": "Building 5"
    }
  }
}
```

**Mapping Rules:**
- `Entity.id` → `unique-id` (converted to string)
- `Entity.name` → `name`
- `Entity.namespace` → `namespace`
- `Entity.attributes` → `metadata.sea:attributes`
- Add marker: `metadata.sea:primitive = "Entity"`

### 2. Resource → CALM Node

**SEA Resource Structure:**
```rust
Resource {
    id: Uuid,
    name: String,
    unit: String,
    attributes: HashMap<String, Value>
}
```

**CALM Node Structure:**
```json
{
  "unique-id": "resource-uuid",
  "node-type": "resource",
  "name": "Cameras",
  "metadata": {
    "sea:primitive": "Resource",
    "sea:unit": "units",
    "sea:attributes": {
      "model": "HD-2000",
      "cost_per_unit": 299.99
    }
  }
}
```

**Mapping Rules:**
- `Resource.id` → `unique-id`
- `Resource.name` → `name`
- `Resource.unit` → `metadata.sea:unit`
- `Resource.attributes` → `metadata.sea:attributes`
- `node-type` = `"resource"`

### 3. Flow → CALM Relationship

**SEA Flow Structure:**
```rust
Flow {
    id: Uuid,
    resource_id: Uuid,
    from_entity_id: Uuid,
    to_entity_id: Uuid,
    quantity: i64
}
```

**CALM Relationship Structure:**
```json
{
  "unique-id": "flow-uuid",
  "relationship-type": {
    "flow": {
      "resource": "resource-uuid",
      "quantity": 100
    }
  },
  "parties": {
    "source": "warehouse-uuid",
    "destination": "factory-uuid"
  }
}
```

**Mapping Rules:**
- `Flow.id` → `unique-id`
- `Flow.resource_id` → `relationship-type.flow.resource`
- `Flow.quantity` → `relationship-type.flow.quantity`
- `Flow.from_entity_id` → `parties.source`
- `Flow.to_entity_id` → `parties.destination`

### 4. Instance → CALM Node + Relationship

**SEA Instance Structure:**
```rust
Instance {
    id: Uuid,
  resource_id: Uuid,
    entity_id: Uuid,
    attributes: HashMap<String, Value>
}
```

**CALM Node Structure:**
```json
{
  "unique-id": "instance-uuid",
  "node-type": "instance",
  "name": "Warehouse A Instance",
  "metadata": {
    "sea:primitive": "Instance",
    "sea:entity_id": "entity-uuid",
    "sea:attributes": {
      "timestamp": "2025-11-07T12:00:00Z",
      "status": "active"
    }
  }
}
```

**CALM Ownership Relationship:**
```json
{
  "unique-id": "ownership-uuid",
  "relationship-type": "ownership",
  "parties": {
    "owner": "entity-uuid",
    "owned": "instance-uuid"
  }
}
```

**Mapping Rules:**
- `Instance.id` → `unique-id`
- `Instance.entity_id` → `metadata.sea:entity_id` AND `ownership relationship`
- `Instance.attributes` → `metadata.sea:attributes`
- Generate ownership relationship linking instance to parent entity

### Entity Relationships - Association → CALM Relationship

**CALM Relationship Structure (Association):**
```json
{
  "unique-id": "assoc-1",
  "relationship-type": "association",
  "parties": {
    "source": "entity-1",
    "destination": "entity-2"
  }
}
```

**Mapping Rules:**
- Associations are exported as `relationship-type: "association"` with `source` and `destination` set to the `unique-id` of the participating nodes.
- During import, associations are stored on the source entity as a metadata attribute `sea:associations` which contains an array of objects `{ "type": "association", "target": "<unique-id>" }` to preserve a lightweight representation that avoids adding extra primitives.


### 5. Policy → CALM Node

**SEA Policy Structure:**
```rust
Policy {
    id: Uuid,
    name: String,
    expression: PolicyExpression
}
  // Optional policy metadata fields exposed in mappings
  modality: Option<String>,  // e.g., "Obligation", "Permission"
  kind: Option<String>,      // e.g., enforcement kind or modality subtype
  priority: Option<i32>,     // integer priority for conflict resolution
```

**CALM Node Structure:**
```json
{
  "unique-id": "policy-uuid",
  "node-type": "constraint",
  "name": "Capacity Limit",
  "metadata": {
    "sea:primitive": "Policy",
    "sea:expression": "ForAll entity: entity.capacity <= 10000",
    "sea:expression_type": "SBVR"
  }
}
```

**Mapping Rules:**
- `Policy.id` → `unique-id`
- `Policy.name` → `name`
- `Policy.expression` → `metadata.sea:expression` (serialized as string)
- `node-type` = `"constraint"`
 - `Policy.modality` → `metadata.sea:modality` (e.g., `Obligation`, `Prohibition`, `Permission`)
 - `Policy.kind` → `metadata.sea:kind` (e.g., `Constraint`, `Derivation`, `Obligation`)
 - `Policy.priority` → `metadata.sea:priority`
 - `metadata.sea:expression_type` → `"SBVR"` or `"SEA"` - indicates whether expression uses SBVR or SEA DSL format

### PolicyExpression Serialization

The `PolicyExpression` enum serializes to SBVR (Semantics of Business Vocabulary and Rules) textual syntax for storage in `metadata.sea:expression`. Supported variants and their canonical SBVR syntax:

**Supported Variants:**
- `AtomicComparison(left, op, right)` → `"left op right"` (e.g., `"entity.capacity <= 10000"`)
- `QuantifierForAll(var, collection, condition)` → `"ForAll var in collection: condition"`
- `QuantifierExists(var, collection, condition)` → `"Exists var in collection: condition"`
- `LogicalAnd(left, right)` → `"left AND right"`
- `LogicalOr(left, right)` → `"left OR right"`
- `LogicalNot(expr)` → `"NOT expr"`
- `FunctionCall(name, args)` → `"name(args)"` (e.g., `"count(flows)"`)

**Serialization Rules:**
- **Variables**: Direct string representation (e.g., `"entity"`, `"flow"`)
- **Literals**: String representation of values (e.g., `"10000"`, `"Camera"`)
- **Operators**: Symbolic representation (`<=`, `>=`, `=`, `!=`, `<`, `>`)
- **Collections**: Lowercase plural names (`entities`, `resources`, `flows`, `instances`)
- **Escaping**: Double quotes around string literals containing spaces or special characters
- **Nesting**: Parentheses for complex expressions: `"(condition1) AND (condition2)"`
- **Enums/Identifiers**: Direct name mapping without quotes

**Example Mappings:**
- `QuantifierForAll("entity", "entities", AtomicComparison("entity.capacity", "<=", "10000"))` → `"ForAll entity in entities: entity.capacity <= 10000"`
- `LogicalAnd(AtomicComparison("flow.quantity", ">", "0"), AtomicComparison("flow.resource", "=", "Camera"))` → `"(flow.quantity > 0) AND (flow.resource = Camera)"`

**Deserialization (Parsing):**
- Parse SBVR syntax back to `PolicyExpression` enum variants
- Validate collection names against known types
- Handle operator precedence (AND/OR before comparisons)
- Error on unsupported constructs with descriptive messages

**Metadata:**
- `metadata.sea:expression_type`: `"SBVR"`
- `metadata.sea:expression_version`: `"1.0"` (for future compatibility)

---

## Import Strategy

### Node Import Algorithm

```
For each CALM node:
  1. Read metadata.sea:primitive field
  2. Switch on primitive type:
     - "Entity" → Create Entity
     - "Resource" → Create Resource
     - "Instance" → Create Instance
     - "Policy" → Create Policy
  3. Map unique-id to internal UUID
  4. Store in id_map for relationship resolution
  5. Add to Graph
```

### Relationship Import Algorithm

```
For each CALM relationship:
  1. Resolve source/destination IDs using id_map
  2. Switch on relationship-type:
     - "flow" → Create Flow
     - "ownership" → Link Instance to Entity
     - "association" → Create Entity association
     - "composition" → Create composition link
  3. Add to Graph
```

### ID Remapping

During import, CALM `unique-id` values are mapped to new UUIDs:

```rust
let mut id_map: HashMap<String, Uuid> = HashMap::new();

for node in calm_nodes {
    let calm_id = node["unique-id"].as_str();
    let sea_id = Uuid::new_v4();  // Generate new UUID
    id_map.insert(calm_id.to_string(), sea_id);
}
```

---

## Round-Trip Validation

### Semantic Equivalence

Since UUIDs are regenerated during import, equivalence is tested semantically:

1. **Entity Matching**: Match by `(name, namespace)` tuple
2. **Flow Matching**: Match by `(source_name, destination_name, resource_name, quantity)`
3. **Attribute Preservation**: All custom attributes must be preserved exactly

### Validation Tests

```rust
#[test]
fn test_lossless_round_trip() {
    let graph1 = build_test_graph();

    // Export to CALM
    let calm_json = calm::export(&graph1)?;

    // Import back
    let graph2 = calm::import(calm_json)?;

    // Verify semantic equivalence
    assert_semantically_equivalent(&graph1, &graph2);
}
```

### Semantic Equivalence Definition

The `assert_semantically_equivalent` function compares two graphs for semantic equality, accounting for UUID regeneration during import:

- **Entities**: Match by `(name, namespace)` with identical attribute sets (keys equal, ordering ignored). Attribute values compared by deep equality with specific numeric rules: primitive ints/strings exact match, floats compared with epsilon (1e-9), NaN/inf handled explicitly, collections compared element-wise (preserve ordering unless domain says otherwise), maps compared by key/value deep-equality. Custom complex types must be canonicalized (sorted keys) before comparison.
- **Resources**: Match by `(name, unit)` with identical attribute sets using same comparison rules.
- **Flows**: Match by `(from_id, to_id, resource_id, quantity)` with quantity compared using numeric rules above. When name-based matching is used, the implementation may resolve names to IDs before asserting equivalence.
- **Instances**: Match by `(resource_id, entity_id, id)`; if the instance `id` is regenerated during import, matching may fall back to comparing `(resource_id, entity_id, attributes)`.
- **Policies**: Match by expression text and severity level.

```rust
fn assert_semantically_equivalent(graph1: &Graph, graph2: &Graph) {
    // Implementation compares entities, resources, flows, instances, and policies
    // using the rules above, failing the test if any mismatch is found
}
```

---

## CALM Schema Compliance

All exports must validate against the FINOS CALM v1.0 JSON Schema:

- **Schema Source**: <https://github.com/finos/architecture-as-code>
- **Validation Tool**: `jsonschema` crate
- **Version**: CALM v1.0 (pinned)

### Required CALM Fields

- `nodes`: Array of node objects
- `relationships`: Array of relationship objects
- `version`: String (e.g., "1.0")
- `metadata`: Object with export metadata

### CALM Node Required Fields

- `unique-id`: String
- `node-type`: String (actor/location/resource/instance/constraint)
- `name`: String
- `metadata`: Object (optional but used for SEA attributes)

### CALM Relationship Required Fields

- `unique-id`: String
- `relationship-type`: String or Object
- `parties`: Object with source/destination or owner/owned

---

## Export Format Example

```json
{
  "version": "1.0",
  "metadata": {
    "sea:exported": true,
    "sea:version": "0.1.0",
    "sea:timestamp": "2025-11-07T12:00:00Z"
  },
  "nodes": [
    {
      "unique-id": "entity-1",
      "node-type": "actor",
      "name": "Warehouse",
      "namespace": "logistics",
      "metadata": {
        "sea:primitive": "Entity",
        "sea:attributes": {
          "capacity": 10000
        }
      }
    },
    {
      "unique-id": "resource-1",
      "node-type": "resource",
      "name": "Cameras",
      "metadata": {
        "sea:primitive": "Resource",
        "sea:unit": "units"
      }
    }
  ],
  "relationships": [
    {
      "unique-id": "flow-1",
      "relationship-type": {
        "flow": {
          "resource": "resource-1",
          "quantity": 100
        }
      },
      "parties": {
        "source": "entity-1",
        "destination": "entity-2"
      }
    }
  ]
}
```

---

## Error Handling

### Export Errors

- **Missing Required Field**: If Entity missing name/namespace → Return error
- **Invalid UUID**: Should never occur (internal consistency)
- **Serialization Failure**: If attributes contain non-JSON types → Return error

### Import Errors

- **Invalid JSON**: Parse error → Return detailed error message
- **Missing Required Field**: Missing `unique-id` → Skip node and log warning
- **Unresolved ID Reference**: Relationship references unknown node → Return error
- **Unknown Node Type**: Unrecognized `node-type` → Skip and log warning
- **Schema Validation Failure**: Does not match CALM schema → Return validation errors

---

## Future Extensions

1. **Provenance Tracking**: Add import/export history to metadata
2. **Versioning**: Support CALM schema version detection and migration
3. **Custom Node Types**: Allow user-defined node types beyond CALM spec
4. **Diff/Merge**: Support for comparing CALM models and merging changes
5. **Streaming**: Large model export/import with streaming JSON parser

---

## References

- [FINOS CALM Specification](https://github.com/finos/architecture-as-code)
- ADR-006: CALM Integration Architecture Decision
- PRD-014: CALM Interoperability Requirements
- SDS-013: CALM Serialization System Design

## Implementation pointers

- Export logic: `sea-core/src/calm/export.rs`
- Import logic: `sea-core/src/calm/import.rs`
- Round-trip tests: `sea-core/tests/calm_round_trip_tests.rs`

Bindings expose these projections via `Graph.export_calm()` (Python) and `graph.exportCalm()` (TypeScript). Validation codes mirror those in `error-codes.md` when CALM payloads lack required fields.

## See also

- `docs/new_docs/how-tos/export-to-calm.md` for end-to-end export steps.
- `docs/new_docs/how-tos/import-from-calm.md` for import and round-trip validation.
