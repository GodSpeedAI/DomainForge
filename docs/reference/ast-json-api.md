# AST & JSON Output Reference

This reference documents the Abstract Syntax Tree (AST) structure and JSON output capabilities in DomainForge. It covers the AST schema, how to generate JSON output from SEA DSL source code, and the relationship between the `Ast` and `Graph` structures.

## Overview

DomainForge provides multiple ways to work with parsed SEA DSL content as structured data:

1. **AST (Abstract Syntax Tree)**: The direct representation of parsed SEA source code, preserving declaration order and source location information.
2. **Graph**: The semantic domain model built from the AST, with entities, resources, flows, and policies organized for validation and querying.
3. **JSON Output**: Serialized representations of both structures for interoperability with other tools.

## AST Schema

The JSON schema for AST output is defined in `schemas/ast-v2.schema.json`. This schema is **programmatically generated** from the Rust type definitions to ensure accuracy.

> [!IMPORTANT]
> The schema is auto-generated from Rust types using [`schemars`](https://docs.rs/schemars). Do not edit `ast-v2.schema.json` manually—regenerate it instead.

### Schema Location

| File                         | Description                                             |
| ---------------------------- | ------------------------------------------------------- |
| `schemas/ast-v2.schema.json` | Current programmatically-generated schema (recommended) |
| `schemas/ast-v1.schema.json` | Legacy hand-written schema (deprecated)                 |

### Top-Level Structure

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Ast",
  "description": "Abstract Syntax Tree for SEA DSL",
  "type": "object",
  "required": ["declarations", "metadata"],
  "properties": {
    "declarations": {
      "type": "array",
      "items": { "$ref": "#/definitions/SpannedAstNode" }
    },
    "metadata": { "$ref": "#/definitions/FileMetadata" }
  }
}
```

### Regenerating the Schema

To regenerate the AST JSON schema from the Rust type definitions:

```bash
cd sea-core
cargo test --lib generate_ast_schema -- --ignored --nocapture
```

This runs the schema generation test which:

1. Uses `schemars` to derive a JSON Schema from the Rust `Ast` type
2. Writes the output to `schemas/ast-v2.schema.json`

> [!TIP]
> Run this command after modifying any AST-related types in `sea-core/src/parser/ast.rs` or `sea-core/src/policy/expression.rs`.

### Schema Generation Implementation

The schema is generated from types defined in `sea-core/src/parser/ast_schema.rs`. This module mirrors the AST types with `JsonSchema` derives:

```rust
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Ast {
    pub metadata: FileMetadata,
    pub declarations: Vec<SpannedAstNode>,
}

// Test that generates the schema file
#[test]
#[ignore]
fn generate_ast_schema() {
    let schema = schema_for!(Ast);
    let json = serde_json::to_string_pretty(&schema).unwrap();
    std::fs::write("../schemas/ast-v2.schema.json", &json).unwrap();
}
```

## AST Node Types

The schema includes all 15 AST node types:

| Node Type         | Description                            | Required Fields                                                             |
| ----------------- | -------------------------------------- | --------------------------------------------------------------------------- |
| `Export`          | Export wrapper for public declarations | `declaration`                                                               |
| `Entity`          | Entity declaration                     | `name`                                                                      |
| `Resource`        | Resource declaration                   | `name`                                                                      |
| `Flow`            | Resource transfer between entities     | `resource_name`, `from_entity`, `to_entity`                                 |
| `Pattern`         | Named regex for string validation      | `name`, `regex`                                                             |
| `Role`            | Participant category                   | `name`                                                                      |
| `Relation`        | Predicate connecting roles             | `name`, `subject_role`, `predicate`, `object_role`                          |
| `Dimension`       | Dimension declaration for units        | `name`                                                                      |
| `UnitDeclaration` | Custom unit definition                 | `symbol`, `dimension`, `factor`, `base_unit`                                |
| `Policy`          | Validation rule or constraint          | `name`, `metadata`, `expression`                                            |
| `Instance`        | Entity instance with field values      | `name`, `entity_type`                                                       |
| `ConceptChange`   | Version migration                      | `name`, `from_version`, `to_version`, `migration_policy`, `breaking_change` |
| `Metric`          | Observable metric                      | `name`, `expression`, `metadata`                                            |
| `MappingDecl`     | Format mapping rules                   | `name`, `target`, `rules`                                                   |
| `ProjectionDecl`  | Projection configuration               | `name`, `target`, `overrides`                                               |

### Expression Types

Policy and metric expressions use a rich expression language:

| Expression Type            | Description                                         |
| -------------------------- | --------------------------------------------------- |
| `Literal`                  | JSON literal value                                  |
| `QuantityLiteral`          | Decimal value with unit                             |
| `TimeLiteral`              | ISO 8601 timestamp                                  |
| `IntervalLiteral`          | Time interval (start, end)                          |
| `Variable`                 | Named variable reference                            |
| `Binary`                   | Binary operation (AND, OR, comparisons, arithmetic) |
| `Unary`                    | Unary operation (NOT, negate)                       |
| `Cast`                     | Type cast                                           |
| `Quantifier`               | ForAll, Exists, ExistsUnique                        |
| `MemberAccess`             | Object member access                                |
| `Aggregation`              | COUNT, SUM, MIN, MAX, AVG                           |
| `AggregationComprehension` | Aggregation with comprehension syntax               |
| `GroupBy`                  | Grouping operation                                  |

## Generating JSON Output

### Via Rust Library

The `Graph` struct implements `Serialize` from serde, enabling direct JSON serialization:

```rust
use sea_core::parser::parse_to_graph;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        Entity "Customer"
        Entity "Vendor"
        Resource "Payment" USD
        Flow "Payment" from "Customer" to "Vendor"
    "#;

    let graph = parse_to_graph(source)?;

    // Serialize the full graph to JSON
    let json = serde_json::to_string_pretty(&graph)?;
    println!("{}", json);

    Ok(())
}
```

To convert a `Graph` back to an `Ast` structure:

```rust
use sea_core::Graph;

let ast = graph.to_ast();
// The Ast can then be used for code generation or formatting
```

### Via WASM (JavaScript/TypeScript)

The WASM bindings expose a `toJSON()` method on the `Graph` class:

```typescript
import init, { Graph } from "sea_core";

async function parseAndSerialize() {
  await init();

  const source = `
        Entity "Customer"
        Entity "Vendor"
        Resource "Payment" USD
        Flow "Payment" from "Customer" to "Vendor"
    `;

  const graph = Graph.parse(source);
  const json = graph.toJSON();

  console.log(JSON.stringify(json, null, 2));
}
```

### Via CLI

The SEA CLI provides multiple commands for JSON output:

#### Validate with JSON Output

```bash
# Validate and output results as JSON
sea validate --format json input.sea
```

Output structure:

```json
{
  "error_count": 0,
  "violations": []
}
```

#### Project to CALM Format

```bash
# Export graph to CALM JSON format
sea project --format calm input.sea output.json
```

#### Project to Knowledge Graph

```bash
# Export to RDF/Turtle or XML
sea project --format kg input.sea output.ttl
sea project --format kg input.sea output.xml
```

## Graph JSON Structure

When serializing a `Graph` to JSON, the structure includes all domain primitives:

```json
{
  "entities": {
    "<uuid>": {
      "id": "<uuid>",
      "name": "Customer",
      "namespace": "default",
      "version": null,
      "attributes": {}
    }
  },
  "resources": {
    "<uuid>": {
      "id": "<uuid>",
      "name": "Payment",
      "namespace": "default",
      "unit": "USD",
      "has_units": true,
      "attributes": {}
    }
  },
  "flows": {
    "<uuid>": {
      "id": "<uuid>",
      "resource_id": "<uuid>",
      "from": "<uuid>",
      "to": "<uuid>",
      "quantity": null,
      "namespace": "default",
      "attributes": {}
    }
  },
  "roles": {},
  "relations": {},
  "instances": {},
  "entity_instances": {},
  "policies": {},
  "patterns": {},
  "concept_changes": {},
  "metrics": {},
  "mappings": {},
  "projections": {},
  "entity_roles": {},
  "config": {
    "use_three_valued_logic": true
  }
}
```

### Field Descriptions

| Field              | Type                            | Description                          |
| ------------------ | ------------------------------- | ------------------------------------ |
| `entities`         | `Map<UUID, Entity>`             | All declared entities in the graph   |
| `resources`        | `Map<UUID, Resource>`           | All declared resources               |
| `flows`            | `Map<UUID, Flow>`               | Resource transfers between entities  |
| `roles`            | `Map<UUID, Role>`               | Participant categories               |
| `relations`        | `Map<UUID, RelationType>`       | Predicates connecting roles          |
| `instances`        | `Map<UUID, ResourceInstance>`   | Concrete resource instances          |
| `entity_instances` | `Map<UUID, Instance>`           | Entity instances with field values   |
| `policies`         | `Map<UUID, Policy>`             | Validation rules and constraints     |
| `patterns`         | `Map<UUID, Pattern>`            | Regex patterns for string validation |
| `concept_changes`  | `Map<UUID, ConceptChange>`      | Version migration declarations       |
| `metrics`          | `Map<UUID, Metric>`             | Observable metrics                   |
| `mappings`         | `Map<UUID, MappingContract>`    | Format mapping declarations          |
| `projections`      | `Map<UUID, ProjectionContract>` | Projection declarations              |
| `entity_roles`     | `Map<UUID, Vec<UUID>>`          | Role assignments to entities         |
| `config`           | `GraphConfig`                   | Evaluation configuration             |

## Spanned AST Structure

The internal `Ast` structure contains `SpannedAstNode` elements, where each node includes source location:

```json
{
  "node": { "type": "Entity", "name": "Customer", ... },
  "line": 1,
  "column": 1
}
```

This enables precise error messages and IDE integration with accurate source positions.

## Converting Between AST and Graph

### AST → Graph (Parsing)

The standard parsing pipeline converts source code to a `Graph`:

```rust
use sea_core::parser::{parse_to_graph, parse_to_graph_with_options, ParseOptions};

// Simple parsing
let graph = parse_to_graph(source)?;

// With options (namespace, registry, etc.)
let options = ParseOptions {
    default_namespace: Some("my_namespace".to_string()),
    ..Default::default()
};
let graph = parse_to_graph_with_options(source, &options)?;
```

### Graph → AST (Reconstruction)

Convert a `Graph` back to an `Ast` for code generation or formatting:

```rust
let ast = graph.to_ast();
// Note: Currently reconstructs Entity and Resource nodes only
```

> [!NOTE]
> The `to_ast()` method is a partial implementation that currently reconstructs only Entity and Resource nodes. Full round-trip support is planned for future releases.

## Projection Formats

SEA supports projecting graphs to various external formats:

| Format          | Command             | Output                  |
| --------------- | ------------------- | ----------------------- |
| CALM            | `--format calm`     | FINOS CALM JSON         |
| Knowledge Graph | `--format kg`       | RDF Turtle or XML       |
| Protobuf        | `--format protobuf` | Protocol Buffers schema |

### CALM Export Example

```bash
sea project --format calm domains/finance.sea output/finance.calm.json
```

Produces FINOS CALM-compatible JSON following the schema in `schemas/calm-v1.schema.json`.

## Validation and Error Handling

JSON output from validation includes structured error information:

```json
{
  "error_count": 2,
  "violations": [
    {
      "severity": "error",
      "policy_name": "reference_integrity",
      "message": "Entity 'Unknown' not found",
      "context": { "line": 5, "column": 12 }
    },
    {
      "severity": "warning",
      "policy_name": "naming_convention",
      "message": "Entity name should be PascalCase",
      "context": { "line": 3, "column": 8 }
    }
  ]
}
```

### Severity Levels

| Severity  | Description                                   |
| --------- | --------------------------------------------- |
| `error`   | Validation failure that blocks processing     |
| `warning` | Potential issue that doesn't block processing |
| `info`    | Informational message                         |

## Version Compatibility

| Schema               | Version | Status                       |
| -------------------- | ------- | ---------------------------- |
| `ast-v2.schema.json` | 2.0.0   | **Current** (auto-generated) |
| `ast-v1.schema.json` | 1.0.0   | Deprecated (hand-written)    |

- Minimum sea-core version: `0.6.0`
- JSON output is stable across minor versions

## See Also

- [primitives-api.md](primitives-api.md) — Core data structure reference
- [cli-commands.md](cli-commands.md) — CLI operations
- [calm-mapping.md](calm-mapping.md) — CALM format mapping details
- [wasm-api.md](wasm-api.md) — WASM bindings reference
- [typescript-api.md](typescript-api.md) — TypeScript API reference
- [python-api.md](python-api.md) — Python API reference
