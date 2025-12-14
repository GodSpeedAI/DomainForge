# ADR-008: Knowledge Graph Integration

**Status:** Accepted  
**Date:** 2025-12-14  
**Deciders:** DomainForge Architecture Team

## Context

Enterprise systems often maintain knowledge in RDF/OWL knowledge graphs. SEA needs to:

1. Export semantic models to standard RDF formats (Turtle, RDF-XML)
2. Import existing knowledge graphs into SEA models
3. Validate graph constraints using SHACL shapes
4. Enable interoperability with semantic web tooling

## Decision

### RDF Export

SEA graphs export to RDF using a canonical vocabulary:

| SEA Concept | RDF Mapping                                             |
| ----------- | ------------------------------------------------------- |
| Entity      | `sea:Entity` instance with `rdfs:label`                 |
| Resource    | `sea:Resource` instance with `sea:unit`                 |
| Flow        | `sea:Flow` with `sea:from`, `sea:to`, `sea:hasResource` |
| Relation    | `sea:Relation` with subject/object roles                |

### Turtle Format

```turtle
@prefix sea: <http://domainforge.ai/sea#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

sea:Warehouse a sea:Entity ;
    rdfs:label "Warehouse" ;
    sea:namespace "logistics" .

sea:Camera a sea:Resource ;
    rdfs:label "Camera" ;
    sea:unit "units" .
```

### SHACL Validation

Generate SHACL shapes from SEA policies and validate imported graphs:

```rust
pub struct ShaclShape {
    pub target_class: String,
    pub properties: Vec<ShaclProperty>,
}

pub struct ShaclProperty {
    pub path: String,
    pub datatype: Option<String>,
    pub min_count: Option<u32>,
    pub max_count: Option<u32>,
    pub min_exclusive: Option<String>,
}
```

### Bidirectional Round-Trip

```
SEA Graph
    ↓ to_turtle()
Turtle/RDF
    ↓ from_turtle()
SEA Graph (restored)
```

## Consequences

### Positive

- **Interoperability**: Standard formats enable tool integration
- **Semantic web**: Leverage existing RDF infrastructure
- **Validation**: SHACL provides schema-level constraint checking
- **Portability**: Export models for external consumption

### Negative

- **Lossy conversion**: Some SEA semantics may not map perfectly to RDF
- **Complexity**: RDF parsing adds dependency weight

## Related

- [SDS-005: Knowledge Graph Module](./SDS-005-knowledge-graph-module.md)
- [ADR-001: SEA-DSL as Semantic Source of Truth](./ADR-001-sea-dsl-semantic-source-of-truth.md)
