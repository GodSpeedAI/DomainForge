use super::models::{CalmModel, NodeType, Parties, RelationshipType};
use crate::primitives::{Entity, Flow, Instance, Resource};
use crate::units::unit_from_string;
use crate::ConceptId;
use crate::Graph;
use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

pub fn import(calm_json: Value) -> Result<Graph, String> {
    let calm_model: CalmModel = serde_json::from_value(calm_json)
        .map_err(|e| format!("Failed to parse CALM model: {}", e))?;

    let mut graph = Graph::new();
    let mut id_map: HashMap<String, ConceptId> = HashMap::new();

    // First pass: import entities and resources to populate id_map
    for node in &calm_model.nodes {
        import_entity_or_resource_node(node, &mut graph, &mut id_map)?;
    }

    // Second pass: import instances now that referenced IDs are available
    for node in &calm_model.nodes {
        import_instance_node(node, &mut graph, &mut id_map)?;
    }

    for relationship in &calm_model.relationships {
        import_relationship(relationship, &mut graph, &id_map)?;
    }

    Ok(graph)
}

fn import_entity_or_resource_node(
    node: &super::models::CalmNode,
    graph: &mut Graph,
    id_map: &mut HashMap<String, ConceptId>,
) -> Result<(), String> {
    let calm_id = &node.unique_id;

    let sea_id = match &node.node_type {
        NodeType::Actor | NodeType::Location => {
            let entity = import_entity(node)?;
            let id = entity.id().clone();
            graph.add_entity(entity)?;
            id
        }
        NodeType::Resource => {
            let resource = import_resource(node)?;
            let id = resource.id().clone();
            graph.add_resource(resource)?;
            id
        }
        _ => return Ok(()), // Skip instances and constraints in first pass
    };

    id_map.insert(calm_id.clone(), sea_id);
    Ok(())
}

fn import_instance_node(
    node: &super::models::CalmNode,
    graph: &mut Graph,
    id_map: &mut HashMap<String, ConceptId>,
) -> Result<(), String> {
    let calm_id = &node.unique_id;

    let sea_id = match &node.node_type {
        NodeType::Instance => {
            let instance = import_instance(node, id_map)?;
            let id = instance.id().clone();
            graph.add_instance(instance)?;
            id
        }
        _ => return Ok(()), // Skip entities, resources, and constraints in second pass
    };

    id_map.insert(calm_id.clone(), sea_id);
    Ok(())
}

fn import_entity(node: &super::models::CalmNode) -> Result<Entity, String> {
    let entity = if let Some(ns) = &node.namespace {
        Entity::new_with_namespace(node.name.clone(), ns.clone())
    } else {
        Entity::new(node.name.clone())
    };

    Ok(entity)
}

fn import_resource(node: &super::models::CalmNode) -> Result<Resource, String> {
    let unit = node
        .metadata
        .get("sea:unit")
        .and_then(|v| v.as_str())
        .unwrap_or("units")
        .to_string();

    let unit_obj = unit_from_string(unit);

    let resource = if let Some(ns) = &node.namespace {
        Resource::new_with_namespace(node.name.clone(), unit_obj, ns.clone())
    } else {
        Resource::new(node.name.clone(), unit_obj)
    };

    Ok(resource)
}

fn import_instance(
    node: &super::models::CalmNode,
    id_map: &HashMap<String, ConceptId>,
) -> Result<Instance, String> {
    let entity_id_str = node
        .metadata
        .get("sea:entity_id")
        .and_then(|v| v.as_str())
        .ok_or("Missing sea:entity_id in instance metadata")?;

    let resource_id_str = node
        .metadata
        .get("sea:resource_id")
        .and_then(|v| v.as_str())
        .ok_or("Missing sea:resource_id in instance metadata")?;

    let entity_id = id_map
        .get(entity_id_str)
        .ok_or_else(|| format!("Unknown entity ID: {}", entity_id_str))?;

    let resource_id = id_map
        .get(resource_id_str)
        .ok_or_else(|| format!("Unknown resource ID: {}", resource_id_str))?;

    let instance = if let Some(ns) = &node.namespace {
        Instance::new_with_namespace(resource_id.clone(), entity_id.clone(), ns.clone())
    } else {
        Instance::new(resource_id.clone(), entity_id.clone())
    };

    Ok(instance)
}

fn import_relationship(
    relationship: &super::models::CalmRelationship,
    graph: &mut Graph,
    id_map: &HashMap<String, ConceptId>,
) -> Result<(), String> {
    match &relationship.relationship_type {
        RelationshipType::Flow { flow } => {
            let (source_id, dest_id) = match &relationship.parties {
                Parties::SourceDestination {
                    source,
                    destination,
                } => (source, destination),
                _ => {
                    return Err("Flow relationship must have source/destination parties".to_string())
                }
            };

            let source_uuid = id_map
                .get(source_id)
                .ok_or_else(|| format!("Unknown source ID: {}", source_id))?;

            let dest_uuid = id_map
                .get(dest_id)
                .ok_or_else(|| format!("Unknown destination ID: {}", dest_id))?;

            let resource_uuid = id_map
                .get(&flow.resource)
                .ok_or_else(|| format!("Unknown resource ID: {}", flow.resource))?;

            let quantity = Decimal::from_str(&flow.quantity)
                .map_err(|e| format!("Invalid quantity '{}': {}", flow.quantity, e))?;

            let flow_obj = Flow::new(
                resource_uuid.clone(),
                source_uuid.clone(),
                dest_uuid.clone(),
                quantity,
            );
            graph.add_flow(flow_obj)?;
        }
        RelationshipType::Simple(rel_type) => {
            let rel_type_text = rel_type.as_str();
            log::warn!(
                "Skipping unsupported Simple relationship '{}' in CALM import",
                rel_type_text
            );
            return Ok(());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_import_empty_model() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {
                "sea:exported": true,
                "sea:version": "0.0.1"
            },
            "nodes": [],
            "relationships": []
        });

        let result = import(calm_json);
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert_eq!(graph.entity_count(), 0);
        assert_eq!(graph.resource_count(), 0);
    }

    #[test]
    fn test_import_entity() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {
                "sea:exported": true,
                "sea:version": "0.0.1"
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
                }
            ],
            "relationships": []
        });

        let result = import(calm_json);
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert_eq!(graph.entity_count(), 1);

        let entity = graph.all_entities()[0];
        assert_eq!(entity.name(), "Warehouse");
        assert_eq!(entity.namespace(), "logistics");
    }

    #[test]
    fn test_import_resource() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {
                "sea:exported": true,
                "sea:version": "0.0.1"
            },
            "nodes": [
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
            "relationships": []
        });

        let result = import(calm_json);
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert_eq!(graph.resource_count(), 1);

        let resource = graph.all_resources()[0];
        assert_eq!(resource.name(), "Cameras");
        assert_eq!(resource.unit().symbol(), "units");
    }

    #[test]
    fn test_import_flow() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {
                "sea:exported": true,
                "sea:version": "0.0.1"
            },
            "nodes": [
                {
                    "unique-id": "entity-1",
                    "node-type": "actor",
                    "name": "Warehouse",
                    "metadata": {
                        "sea:primitive": "Entity"
                    }
                },
                {
                    "unique-id": "entity-2",
                    "node-type": "actor",
                    "name": "Factory",
                    "metadata": {
                        "sea:primitive": "Entity"
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
                            "quantity": "100"
                        }
                    },
                    "parties": {
                        "source": "entity-1",
                        "destination": "entity-2"
                    }
                }
            ]
        });

        let result = import(calm_json);
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert_eq!(graph.flow_count(), 1);

        let flow = graph.all_flows()[0];
        assert_eq!(flow.quantity().to_string(), "100");
    }
}
