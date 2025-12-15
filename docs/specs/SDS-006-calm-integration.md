# SDS-006: CALM Integration

**System:** DomainForge  
**Component:** FINOS CALM Import/Export  
**Version:** 1.0  
**Date:** 2025-12-14  
**Status:** Implemented

---

## 1. Overview

The CALM (Common Architecture Language Model) integration enables DomainForge to:

- Export SEA models to FINOS CALM JSON format
- Import FINOS CALM architecture models into SEA
- Support enterprise architecture governance workflows
- Enable architecture-as-code practices

---

## 2. FINOS CALM Background

CALM is a standardized format for describing enterprise architectures:

```json
{
  "$schema": "https://calm.finos.org/draft/2024-03/meta/core.json",
  "nodes": [...],
  "relationships": [...]
}
```

Key CALM concepts:

- **Nodes**: Architectural components (services, systems, actors)
- **Relationships**: Connections between nodes
- **Interfaces**: API contracts and interactions

---

## 3. Architecture

```
┌────────────────────────────────────────────────────┐
│                 calm/ Module                        │
├────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐               │
│  │  CalmModel   │  │  CalmNode    │               │
│  └──────────────┘  └──────────────┘               │
│  ┌──────────────────────────────────┐             │
│  │       CalmRelationship           │             │
│  └──────────────────────────────────┘             │
├────────────────────────────────────────────────────┤
│  export.rs         │  import.rs                    │
│  - export()        │  - import()                   │
│  - graph_to_calm() │  - calm_to_graph()            │
├────────────────────────────────────────────────────┤
│  sbvr_import.rs                                    │
│  - import_sbvr_xmi()                               │
└────────────────────────────────────────────────────┘
```

---

## 4. Core Types

### 4.1 CalmModel

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalmModel {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub nodes: Vec<CalmNode>,
    pub relationships: Vec<CalmRelationship>,
}

impl CalmModel {
    pub fn new() -> Self {
        Self {
            schema: "https://calm.finos.org/draft/2024-03/meta/core.json".to_string(),
            nodes: Vec::new(),
            relationships: Vec::new(),
        }
    }
}
```

### 4.2 CalmNode

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalmNode {
    #[serde(rename = "unique-id")]
    pub unique_id: String,

    pub name: String,

    #[serde(rename = "node-type")]
    pub node_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub interfaces: Option<Vec<CalmInterface>>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
```

### 4.3 CalmRelationship

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalmRelationship {
    #[serde(rename = "unique-id")]
    pub unique_id: String,

    #[serde(rename = "relationship-type")]
    pub relationship_type: RelationshipType,

    pub parties: Parties,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parties {
    pub source: String,
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelationshipType {
    Connects,
    Interacts,
    Deployed,
    Composed,
}
```

---

## 5. Mapping Rules

### 5.1 SEA → CALM Mappings

| SEA Concept | CALM Mapping                                            |
| ----------- | ------------------------------------------------------- |
| Entity      | `CalmNode` with `node-type: "system"` or `"actor"`      |
| Resource    | `CalmNode` with `node-type: "data-asset"`               |
| Flow        | `CalmRelationship` with `relationship-type: "connects"` |
| Role        | `CalmNode` with `node-type: "actor"`                    |
| Relation    | `CalmRelationship` with appropriate type                |

### 5.2 Node Type Inference

```rust
fn infer_node_type(entity: &Entity) -> String {
    // Use entity attributes or naming conventions
    if entity.name().ends_with("Service") {
        "service".to_string()
    } else if entity.name().ends_with("Database") || entity.name().ends_with("Store") {
        "database".to_string()
    } else if entity.has_role("Actor") {
        "actor".to_string()
    } else {
        "system".to_string()
    }
}
```

---

## 6. Export Process

### 6.1 Graph to CALM

```rust
pub fn export(graph: &Graph) -> Result<CalmModel, CalmError> {
    let mut model = CalmModel::new();

    // Export entities as nodes
    for entity in graph.all_entities() {
        model.nodes.push(CalmNode {
            unique_id: entity.id().to_string(),
            name: entity.name().to_string(),
            node_type: infer_node_type(entity),
            description: entity.get_attribute("description")
                .and_then(|v| v.as_str().map(String::from)),
            interfaces: None,
            extra: HashMap::new(),
        });
    }

    // Export resources as data assets
    for resource in graph.all_resources() {
        model.nodes.push(CalmNode {
            unique_id: resource.id().to_string(),
            name: resource.name().to_string(),
            node_type: "data-asset".to_string(),
            description: None,
            interfaces: None,
            extra: HashMap::new(),
        });
    }

    // Export flows as relationships
    for flow in graph.all_flows() {
        model.relationships.push(CalmRelationship {
            unique_id: flow.id().to_string(),
            relationship_type: RelationshipType::Connects,
            parties: Parties {
                source: flow.from_id().to_string(),
                destination: flow.to_id().to_string(),
            },
            description: Some(format!(
                "{} (quantity: {})",
                graph.get_resource(flow.resource_id())
                    .map(|r| r.name())
                    .unwrap_or("unknown"),
                flow.quantity()
            )),
        });
    }

    Ok(model)
}
```

### 6.2 JSON Serialization

```rust
impl CalmModel {
    pub fn to_json(&self) -> Result<String, CalmError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| CalmError::Serialization(e.to_string()))
    }

    pub fn to_json_compact(&self) -> Result<String, CalmError> {
        serde_json::to_string(self)
            .map_err(|e| CalmError::Serialization(e.to_string()))
    }
}
```

---

## 7. Import Process

### 7.1 CALM to Graph

```rust
pub fn import(json: &str) -> Result<Graph, CalmError> {
    let model: CalmModel = serde_json::from_str(json)
        .map_err(|e| CalmError::Parse(e.to_string()))?;

    let mut graph = Graph::new();

    // Import nodes
    for node in model.nodes {
        match node.node_type.as_str() {
            "actor" | "system" | "service" => {
                let entity = Entity::new_with_namespace(
                    node.name.clone(),
                    "calm".to_string(),
                );
                graph.add_entity(entity)?;
            }
            "data-asset" => {
                let resource = Resource::new_with_namespace(
                    node.name.clone(),
                    Unit::count(),
                    "calm".to_string(),
                );
                graph.add_resource(resource)?;
            }
            _ => {
                // Unknown node types become entities
                let entity = Entity::new_with_namespace(
                    node.name.clone(),
                    "calm".to_string(),
                );
                graph.add_entity(entity)?;
            }
        }
    }

    // Import relationships as flows (where applicable)
    for rel in model.relationships {
        if rel.relationship_type == RelationshipType::Connects {
            // Create a synthetic resource for the flow
            let resource_name = format!("{}→{}", rel.parties.source, rel.parties.destination);
            // ...
        }
    }

    Ok(graph)
}
```

---

## 8. CLI Integration

```bash
# Export to CALM
sea project --format calm model.sea > architecture.json

# Import from CALM
sea import --from calm architecture.json > model.sea
```

---

## 9. Validation

### 9.1 CALM Schema Validation

```rust
pub fn validate_calm_schema(json: &str) -> Result<(), CalmError> {
    let model: CalmModel = serde_json::from_str(json)?;

    // Check required fields
    for node in &model.nodes {
        if node.unique_id.is_empty() {
            return Err(CalmError::Validation("Node missing unique-id".to_string()));
        }
        if node.name.is_empty() {
            return Err(CalmError::Validation("Node missing name".to_string()));
        }
    }

    // Check relationship references
    let node_ids: HashSet<_> = model.nodes.iter()
        .map(|n| n.unique_id.as_str())
        .collect();

    for rel in &model.relationships {
        if !node_ids.contains(rel.parties.source.as_str()) {
            return Err(CalmError::Validation(
                format!("Relationship references unknown source: {}", rel.parties.source)
            ));
        }
        if !node_ids.contains(rel.parties.destination.as_str()) {
            return Err(CalmError::Validation(
                format!("Relationship references unknown destination: {}", rel.parties.destination)
            ));
        }
    }

    Ok(())
}
```

---

## Related Documents

- [ADR-001: SEA-DSL as Semantic Source of Truth](./ADR-001-sea-dsl-semantic-source-of-truth.md)
- [SDS-002: SEA Core Architecture](./SDS-002-sea-core-architecture.md)
- [PRD-002: SEA CLI Tooling](./PRD-002-sea-cli-tooling.md)
