# SDS-005: Knowledge Graph Module

**System:** DomainForge  
**Component:** Knowledge Graph Integration  
**Version:** 1.0  
**Date:** 2025-12-14  
**Status:** Implemented

---

## 1. Overview

The Knowledge Graph module provides bidirectional conversion between SEA semantic graphs and RDF knowledge graphs. It supports:

- Export to Turtle and RDF-XML formats
- Import from Turtle and RDF-XML files
- SHACL shape generation and validation
- Round-trip fidelity for SEA concepts

---

## 2. Architecture

```
┌────────────────────────────────────────────────────┐
│                   kg.rs Module                      │
├────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐               │
│  │ KnowledgeGraph│  │   Triple    │               │
│  │   struct     │  │   struct    │               │
│  └──────────────┘  └──────────────┘               │
│  ┌──────────────┐  ┌──────────────┐               │
│  │ ShaclShape   │  │ShaclProperty │               │
│  └──────────────┘  └──────────────┘               │
├────────────────────────────────────────────────────┤
│  Export Methods     │  Import Methods              │
│  - to_turtle()      │  - from_turtle()             │
│  - to_rdfxml()      │  - from_graph()              │
│  - to_graph()       │                              │
└────────────────────────────────────────────────────┘
         ↓                       ↑
┌────────────────────────────────────────────────────┐
│                kg_import.rs Module                  │
│  - import_kg_turtle()                              │
│  - import_kg_rdfxml()                              │
└────────────────────────────────────────────────────┘
```

---

## 3. Core Types

### 3.1 Triple

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Triple {
    pub subject: String,    // e.g., "sea:Warehouse"
    pub predicate: String,  // e.g., "rdf:type"
    pub object: String,     // e.g., "sea:Entity"
}
```

### 3.2 KnowledgeGraph

```rust
pub struct KnowledgeGraph {
    pub triples: Vec<Triple>,
    pub shapes: Vec<ShaclShape>,
}

impl KnowledgeGraph {
    pub fn new() -> Self;
    pub fn from_graph(graph: &Graph) -> Result<Self, KgError>;
    pub fn to_turtle(&self) -> String;
    pub fn from_turtle(turtle: &str) -> Result<Self, KgError>;
    pub fn to_graph(&self) -> Result<Graph, KgError>;
    pub fn validate_shacl(&self) -> Result<Vec<Violation>, KgError>;
}
```

### 3.3 SHACL Shape

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaclShape {
    pub target_class: String,
    pub properties: Vec<ShaclProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaclProperty {
    pub path: String,
    pub datatype: Option<String>,
    pub min_count: Option<u32>,
    pub max_count: Option<u32>,
    pub min_exclusive: Option<String>,
}
```

---

## 4. RDF Vocabulary

### 4.1 SEA Namespace

| URI                          | Prefix | Description            |
| ---------------------------- | ------ | ---------------------- |
| `http://domainforge.ai/sea#` | `sea:` | SEA ontology namespace |

### 4.2 Class Mappings

| SEA Concept | RDF Class      |
| ----------- | -------------- |
| Entity      | `sea:Entity`   |
| Resource    | `sea:Resource` |
| Flow        | `sea:Flow`     |
| Role        | `sea:Role`     |
| Relation    | `sea:Relation` |
| Pattern     | `sea:Pattern`  |

### 4.3 Property Mappings

| SEA Attribute   | RDF Property      |
| --------------- | ----------------- |
| name            | `rdfs:label`      |
| namespace       | `sea:namespace`   |
| unit            | `sea:unit`        |
| quantity        | `sea:quantity`    |
| from (Flow)     | `sea:from`        |
| to (Flow)       | `sea:to`          |
| resource (Flow) | `sea:hasResource` |

---

## 5. Export Process

### 5.1 Graph to Turtle

```rust
impl KnowledgeGraph {
    pub fn from_graph(graph: &Graph) -> Result<Self, KgError> {
        let mut kg = Self::new();

        // Export entities
        for entity in graph.all_entities() {
            kg.triples.push(Triple {
                subject: format!("sea:{}", uri_encode(entity.name())),
                predicate: "rdf:type".to_string(),
                object: "sea:Entity".to_string(),
            });

            kg.triples.push(Triple {
                subject: format!("sea:{}", uri_encode(entity.name())),
                predicate: "rdfs:label".to_string(),
                object: format!("\"{}\"", escape_literal(entity.name())),
            });

            kg.triples.push(Triple {
                subject: format!("sea:{}", uri_encode(entity.name())),
                predicate: "sea:namespace".to_string(),
                object: format!("\"{}\"", escape_literal(entity.namespace())),
            });
        }

        // Export resources, flows, etc.
        // ...

        // Generate SHACL shapes
        kg.shapes.push(ShaclShape {
            target_class: "sea:Flow".to_string(),
            properties: vec![
                ShaclProperty {
                    path: "sea:quantity".to_string(),
                    datatype: Some("xsd:decimal".to_string()),
                    min_count: None,
                    max_count: None,
                    min_exclusive: Some("0".to_string()),
                },
                // ...
            ],
        });

        Ok(kg)
    }
}
```

### 5.2 Turtle Serialization

```rust
pub fn to_turtle(&self) -> String {
    let mut turtle = String::new();

    // Prefixes
    turtle.push_str("@prefix sea: <http://domainforge.ai/sea#> .\n");
    turtle.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
    turtle.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n");
    turtle.push_str("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n");
    turtle.push_str("@prefix sh: <http://www.w3.org/ns/shacl#> .\n\n");

    // Ontology classes
    turtle.push_str("# Ontology\n");
    turtle.push_str("sea:Entity a owl:Class .\n");
    // ...

    // Instance triples
    turtle.push_str("# Instances\n");
    for triple in &self.triples {
        turtle.push_str(&format!("{} {} {} .\n",
            triple.subject, triple.predicate, triple.object));
    }

    // SHACL shapes
    turtle.push_str("# SHACL Shapes\n");
    for shape in &self.shapes {
        // ...
    }

    turtle
}
```

---

## 6. Import Process

### 6.1 Turtle Parsing

```rust
pub fn from_turtle(turtle: &str) -> Result<Self, KgError> {
    let mut kg = Self::new();

    for line in turtle.lines() {
        let trimmed = line.trim();

        // Skip prefixes and comments
        if trimmed.is_empty()
            || trimmed.starts_with('@')
            || trimmed.starts_with('#') {
            continue;
        }

        // Parse triple
        let tokens = tokenize_triple_line(trimmed.strip_suffix('.').unwrap_or(trimmed));
        if tokens.len() == 3 {
            kg.triples.push(Triple {
                subject: normalize_token(&tokens[0]),
                predicate: normalize_token(&tokens[1]),
                object: normalize_token(&tokens[2]),
            });
        }
    }

    // Parse SHACL shapes
    // ...

    Ok(kg)
}
```

### 6.2 Conversion to SEA Graph

```rust
pub fn to_graph(&self) -> Result<Graph, KgError> {
    let mut graph = Graph::new();

    // First pass: collect entities and resources
    for triple in &self.triples {
        if triple.predicate == "rdf:type" && triple.object == "sea:Entity" {
            let name = extract_local_name(&triple.subject);
            let entity = Entity::new_with_namespace(name, "default".to_string());
            graph.add_entity(entity)?;
        }

        if triple.predicate == "rdf:type" && triple.object == "sea:Resource" {
            let name = extract_local_name(&triple.subject);
            let resource = Resource::new_with_namespace(name, Unit::count(), "default".to_string());
            graph.add_resource(resource)?;
        }
    }

    // Second pass: add flows with resolved references
    for triple in &self.triples {
        if triple.predicate == "rdf:type" && triple.object == "sea:Flow" {
            // Collect flow properties and add to graph
            // ...
        }
    }

    Ok(graph)
}
```

---

## 7. SHACL Validation

```rust
pub fn validate_shacl(&self) -> Result<Vec<Violation>, KgError> {
    let mut violations = Vec::new();

    for shape in &self.shapes {
        // Find all instances of target class
        let instances: Vec<_> = self.triples.iter()
            .filter(|t| t.predicate == "rdf:type" && t.object == shape.target_class)
            .collect();

        for instance in instances {
            for property in &shape.properties {
                // Check property constraints
                let values: Vec<_> = self.triples.iter()
                    .filter(|t| t.subject == instance.subject && t.predicate == property.path)
                    .collect();

                // min_count check
                if let Some(min) = property.min_count {
                    if values.len() < min as usize {
                        violations.push(Violation { /* ... */ });
                    }
                }

                // max_count check
                if let Some(max) = property.max_count {
                    if values.len() > max as usize {
                        violations.push(Violation { /* ... */ });
                    }
                }

                // datatype and min_exclusive checks
                // ...
            }
        }
    }

    Ok(violations)
}
```

---

## 8. Projection Override Support

The KG export respects projection contracts for custom RDF mappings:

```sea
Projection "Logistics KG" target kg {
    override Entity "Warehouse" {
        rdf_class: "logistics:Facility"
        properties: {
            name: "skos:prefLabel"
            namespace: "dct:subject"
        }
    }
}
```

---

## Related Documents

- [ADR-008: Knowledge Graph Integration](./ADR-008-knowledge-graph-integration.md)
- [SDS-002: SEA Core Architecture](./SDS-002-sea-core-architecture.md)
- [SDS-001: Protobuf Projection Engine](./SDS-001-protobuf-projection-engine.md)
