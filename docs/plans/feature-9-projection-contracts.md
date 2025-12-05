# Feature 9: Projection Contracts Implementation Plan

This plan details the implementation of **Projection Contracts**, a key component of Phase 3: Ergonomics. This feature allows users to define custom mappings and overrides for exporting SEA models to external formats (CALM, Knowledge Graph, SBVR) directly within the DSL.

## 1. Overview

Currently, projection logic is hardcoded in Rust (e.g., `sea-core/src/calm/export.rs`). Users cannot customize how an `Entity` maps to a CALM `Node` or how a `Flow` maps to an RDF triple without modifying the core codebase.

**Projection Contracts** introduce two new top-level declarations:

1. **`Mapping`**: Defines a reusable transformation contract for a specific target format.
2. **`Projection`**: Applies specific overrides to default behavior for a set of concepts.

## 2. Proposed Syntax

### Mapping Declaration

Defines a reusable contract for mapping SEA primitives to target structures.

```sea
Mapping "payment_to_calm" for calm {
  // Map Entity to CALM Node with specific type and metadata
  Entity "PaymentProcessor" -> Node {
    node_type: "service",
    metadata: {
      "team": "payments",
      "tier": "critical"
    }
  }

  // Map Flow to CALM Relationship
  Flow "Payment" -> Relationship {
    relationship_type: "dataflow",
    metadata: {
      "encrypted": true
    }
  }
}
```

### Projection Override

Overrides default projection behavior for specific instances.

```sea
Projection "custom_kg" for kg {
  // Override RDF class and property mappings
  Entity "Vendor" {
    rdf_class: "org:Organization"
    properties: {
      "name" -> "foaf:name",
      "credit_limit" -> "fin:creditLimit"
    }
  }
}
```

## 3. Implementation Steps

### Step 1: Grammar (`sea-core/grammar/sea.pest`)

Add new rules to the grammar to support `Mapping` and `Projection` declarations.

```pest
// Add to declaration rule
declaration = { ... | mapping_decl | projection_decl }

// Mapping declaration
mapping_decl = {
    ^"Mapping" ~ string_literal ~ ^"for" ~ target_format ~ "{" ~ mapping_rule* ~ "}"
}

target_format = { ^"calm" | ^"kg" | ^"sbvr" }

mapping_rule = {
    primitive_type ~ string_literal ~ "->" ~ target_structure
}

primitive_type = { ^"Entity" | ^"Resource" | ^"Flow" | ^"Policy" | ^"Instance" }

target_structure = {
    identifier ~ "{" ~ (mapping_field ~ ","?)* ~ "}"
}

mapping_field = {
    identifier ~ ":" ~ (string_literal | object_literal | boolean)
}

// Projection override
projection_decl = {
    ^"Projection" ~ string_literal ~ ^"for" ~ target_format ~ "{" ~ projection_rule* ~ "}"
}

projection_rule = {
    primitive_type ~ string_literal ~ "{" ~ (projection_field ~ ","?)* ~ "}"
}

projection_field = {
    identifier ~ ":" ~ (string_literal | property_mapping)
}

property_mapping = {
    "{" ~ (identifier ~ "->" ~ string_literal ~ ","?)* ~ "}"
}
```

### Step 2: AST (`sea-core/src/parser/ast.rs`)

Define AST nodes to capture the parsed structure.

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TargetFormat {
    Calm,
    Kg,
    Sbvr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MappingRule {
    pub primitive_type: String,
    pub primitive_name: String,
    pub target_type: String,
    pub fields: HashMap<String, JsonValue>,
}

// Add to AstNode enum
MappingDecl {
    name: String,
    target: TargetFormat,
    rules: Vec<MappingRule>,
},
ProjectionDecl {
    name: String,
    target: TargetFormat,
    overrides: Vec<ProjectionOverride>,
}
```

### Step 3: Primitives (`sea-core/src/primitives/`)

Create new primitive types to store the compiled contracts in the graph.

**`sea-core/src/primitives/mapping.rs`**:

```rust
pub struct MappingContract {
    pub id: ConceptId,
    pub name: String,
    pub namespace: String,
    pub target_format: TargetFormat,
    pub rules: Vec<MappingRule>,
}
```

**`sea-core/src/primitives/projection.rs`**:

```rust
pub struct ProjectionContract {
    pub id: ConceptId,
    pub name: String,
    pub namespace: String,
    pub target_format: TargetFormat,
    pub overrides: Vec<ProjectionOverride>,
}
```

### Step 4: Graph Integration (`sea-core/src/graph/mod.rs`)

Update the `Graph` struct to store mappings and projections.

```rust
pub struct Graph {
    // ... existing fields
    pub mappings: IndexMap<ConceptId, MappingContract>,
    pub projections: IndexMap<ConceptId, ProjectionContract>,
}

impl Graph {
    pub fn add_mapping(&mut self, mapping: MappingContract) { ... }
    pub fn add_projection(&mut self, projection: ProjectionContract) { ... }
}
```

### Step 5: Projection Engine (`sea-core/src/projection/`)

Create a new module to handle the application of contracts during export.

**Structure**:

```
sea-core/src/projection/
├── mod.rs           # Trait definitions
├── contracts.rs     # Contract logic
├── engine.rs        # Application logic
└── registry.rs      # Lookup helpers
```

**`ProjectionExporter` Trait**:

```rust
pub trait ProjectionExporter {
    type Output;
    fn export_entity(&self, entity: &Entity, contract: Option<&MappingContract>) -> Self::Output;
    // ... other primitives
}
```

### Step 6: Export Updates

Refactor existing exporters to use the Projection Engine.

**`sea-core/src/calm/export.rs`**:

- Update `export_entity`, `export_flow`, etc. to accept an optional `MappingContract`.
- If a contract exists and matches the primitive, use the contract's rules to generate the `CalmNode`.
- Fallback to default behavior if no rule matches.

**`sea-core/src/kg.rs`**:

- Update RDF generation to check for `ProjectionContract` overrides.
- Use overridden RDF classes and properties if specified.

### Step 7: Bindings

Expose the new primitives to Python and TypeScript.

- **Python**: `sea-core/src/python/primitives.rs`
- **TypeScript**: `sea-core/src/typescript/primitives.rs`

## 4. Testing Strategy

1. **Grammar Tests**: Verify that valid `Mapping` and `Projection` declarations parse correctly.
2. **AST Tests**: Ensure parsed AST nodes contain correct data.
3. **Graph Tests**: Verify that contracts are correctly stored and retrieved from the Graph.
4. **Export Tests**:
   - Create a graph with entities and a mapping contract.
   - Export to CALM and verify that the output matches the contract (e.g., custom `node_type`).
   - Export to KG and verify custom RDF predicates.
5. **Round-trip Tests**: Ensure that contracts can be serialized/deserialized (if applicable).

## 5. Work Breakdown

| Task                  | Description                               | Estimated Effort |
| --------------------- | ----------------------------------------- | ---------------- |
| **Grammar & AST**     | Update `sea.pest` and `ast.rs`            | 2 days           |
| **Primitives**        | Create `mapping.rs` and `projection.rs`   | 1 day            |
| **Graph**             | Update `Graph` struct and methods         | 1 day            |
| **Projection Engine** | Implement core logic in `src/projection/` | 3 days           |
| **Export Refactor**   | Update CALM and KG exporters              | 2 days           |
| **Bindings**          | Update Python and TS bindings             | 1 day            |
| **Testing**           | Unit and integration tests                | 2 days           |
| **Total**             |                                           | **~12 days**     |

## 6. Implementation Status

### ✅ Completed

1. **Grammar (`sea-core/grammar/sea.pest`)**: Added `mapping_decl` and `projection_decl` rules with full support for target formats, mapping rules, and projection overrides.

2. **AST (`sea-core/src/parser/ast.rs`)**:

   - Added `TargetFormat` enum (Calm, Kg, Sbvr)
   - Added `MappingRule` and `ProjectionOverride` structs
   - Added `MappingDecl` and `ProjectionDecl` to `AstNode` enum
   - Implemented parsing functions: `parse_mapping`, `parse_projection`, `parse_target_format`, `parse_mapping_rule`, `parse_projection_rule`, `parse_object_literal`, `parse_property_mapping`
   - Integrated into `ast_to_graph_with_options` to create contracts and add them to the graph

3. **Primitives**:

   - Created `sea-core/src/primitives/mapping_contract.rs` with `MappingContract` struct
   - Created `sea-core/src/primitives/projection_contract.rs` with `ProjectionContract` struct
   - Added Serde derives for serialization support
   - Exported from `sea-core/src/primitives/mod.rs`

4. **Graph Integration (`sea-core/src/graph/mod.rs`)**:

   - Added `mappings: IndexMap<ConceptId, MappingContract>` field
   - Added `projections: IndexMap<ConceptId, ProjectionContract>` field
   - Implemented `add_mapping`, `add_projection`, `get_mapping`, `get_projection`, `has_mapping`, `has_projection`, `all_mappings`, `all_projections`, `mapping_count`, `projection_count` methods
   - Updated `merge` and `is_empty` methods to handle new fields

5. **Projection Engine (`sea-core/src/projection/`)**:

   - Created `contracts.rs` with helper functions `find_mapping_rule` and `find_projection_override`
   - Created `engine.rs` with `ProjectionExporter` trait
   - Created `registry.rs` with `ProjectionRegistry` for finding contracts by target format
   - Exported module from `sea-core/src/lib.rs`

6. **Export Updates**:

   - **CALM** (`sea-core/src/calm/export.rs`): Updated `export_entity` and `export_flow` to accept and apply mapping/projection contracts
   - **KG** (`sea-core/src/kg.rs`): Updated `from_graph` to use projection overrides for RDF class and property mappings

7. **Bindings**:

   - **Python** (`sea-core/src/python/primitives.rs`): Added `Mapping` and `Projection` classes with getters for `name` and `target_format`
   - **TypeScript** (`sea-core/src/typescript/primitives.rs`): Added `Mapping` and `Projection` classes with getters
   - Registered classes in Python module (`sea-core/src/lib.rs`)

8. **Testing** (`sea-core/tests/projection_contracts_tests.rs`):
   - `test_parse_mapping_and_projection`: Verifies grammar and AST parsing
   - `test_graph_integration`: Verifies contracts are added to graph
   - `test_calm_export_with_mapping`: Verifies CALM export applies mapping rules
   - `test_kg_export_with_projection`: Verifies KG export applies projection overrides
   - All tests passing ✅

### 📝 Notes

- The implementation follows the plan closely with minor adjustments:
  - Renamed primitive files to `mapping_contract.rs` and `projection_contract.rs` to avoid module name conflicts
  - Used `ConceptId::from_concept()` instead of `ConceptId::new()` for ID generation
  - Simplified the projection engine to focus on core functionality
  - Export functions updated to use `ProjectionRegistry` for finding applicable contracts

### 🎯 Next Steps (Optional Enhancements)

1. Add more comprehensive export tests for SBVR format
2. Implement validation to ensure mapping/projection rules reference valid primitives
3. Add CLI commands to list and inspect projection contracts
4. Add support for conditional mappings based on attributes
5. Document projection contracts in user-facing documentation
