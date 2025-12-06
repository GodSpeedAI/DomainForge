use super::models::{CalmModel, NodeType, Parties, RelationshipType};
use crate::policy::Policy;
use crate::primitives::{Entity, Flow, Resource, ResourceInstance};
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
        // Import policies (constraints) as part of the first pass as they do not
        // reference other nodes.
        import_constraint_node(node, &mut graph, &mut id_map)?;
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
        _ => return Ok(()), // Skip instances (handled in second pass) and other unsupported node types
    };

    id_map.insert(calm_id.clone(), sea_id);
    Ok(())
}

fn import_constraint_node(
    node: &super::models::CalmNode,
    graph: &mut Graph,
    id_map: &mut HashMap<String, ConceptId>,
) -> Result<(), String> {
    let calm_id = &node.unique_id;

    if let NodeType::Constraint = node.node_type {
        // Extract SBVR/SEA expression from metadata
        let expr_str = node
            .metadata
            .get("sea:expression")
            .and_then(|v| v.as_str())
            .ok_or("Missing sea:expression for constraint node")?;

        // Parse the expression using the SEA parser; SBVR XMI exports expressions
        // in a compatible textual form (forall/exist style) - use the same parser.
        let expr = crate::parser::parse_expression_from_str(expr_str)
            .map_err(|e| format!("Failed to parse policy expression: {}", e))?;

        let ns = node
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string());
        let mut policy = Policy::new_with_namespace(node.name.clone(), ns.clone(), expr);

        // Optional metadata mapping
        if let Some(priority_val) = node.metadata.get("sea:priority") {
            if let Some(priority) = priority_val.as_i64() {
                policy = policy.with_priority(priority as i32);
            }
        }

        // Modality and kind mapping if present
        if let Some(modality) = node.metadata.get("sea:modality").and_then(|v| v.as_str()) {
            match modality {
                "Obligation" => {
                    policy = policy.with_modality(crate::policy::PolicyModality::Obligation)
                }
                "Prohibition" => {
                    policy = policy.with_modality(crate::policy::PolicyModality::Prohibition)
                }
                "Permission" => {
                    policy = policy.with_modality(crate::policy::PolicyModality::Permission)
                }
                _ => {}
            }
        }

        if let Some(kind) = node.metadata.get("sea:kind").and_then(|v| v.as_str()) {
            match kind {
                "Constraint" => policy = policy.with_kind(crate::policy::PolicyKind::Constraint),
                "Derivation" => policy = policy.with_kind(crate::policy::PolicyKind::Derivation),
                "Obligation" => policy = policy.with_kind(crate::policy::PolicyKind::Obligation),
                _ => {}
            }
        }

        let policy_id = policy.id.clone();
        graph.add_policy(policy)?;
        id_map.insert(calm_id.clone(), policy_id);
    }

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
        _ => return Ok(()), // Skip entities and resources (handled in first pass) and other unsupported node types
    };

    id_map.insert(calm_id.clone(), sea_id);
    Ok(())
}

fn import_entity(node: &super::models::CalmNode) -> Result<Entity, String> {
    let entity = if let Some(ns) = &node.namespace {
        Entity::new_with_namespace(node.name.clone(), ns.clone())
    } else {
        Entity::new_with_namespace(node.name.clone(), "default".to_string())
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
        Resource::new_with_namespace(node.name.clone(), unit_obj, "default".to_string())
    };

    Ok(resource)
}

fn import_instance(
    node: &super::models::CalmNode,
    id_map: &HashMap<String, ConceptId>,
) -> Result<ResourceInstance, String> {
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
        ResourceInstance::new_with_namespace(resource_id.clone(), entity_id.clone(), ns.clone())
    } else {
        ResourceInstance::new_with_namespace(
            resource_id.clone(),
            entity_id.clone(),
            "default".to_string(),
        )
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
            if rel_type_text == "ownership" {
                return Err("Ownership relationships should be modeled as Instances, not Simple relationships".to_string());
            }
            if rel_type_text == "association" {
                // Map association relationships into the Graph. We use the
                // Graph::add_association helper which stores associations as an attribute
                // on the source entity for simplicity.
                let (source_id, dest_id) = match &relationship.parties {
                    Parties::SourceDestination {
                        source,
                        destination,
                    } => (source, destination),
                    _ => {
                        return Err(
                            "Association relationship must have source/destination parties"
                                .to_string(),
                        )
                    }
                };
                let source_uuid = id_map
                    .get(source_id)
                    .ok_or_else(|| format!("Unknown source ID: {}", source_id))?;

                let dest_uuid = id_map
                    .get(dest_id)
                    .ok_or_else(|| format!("Unknown destination ID: {}", dest_id))?;

                graph.add_association(source_uuid, dest_uuid, "association")?;
                return Ok(());
            }
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
                "sea:version": crate::VERSION
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
                "sea:version": crate::VERSION
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
                "sea:version": crate::VERSION
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
                "sea:version": crate::VERSION
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

    #[test]
    fn test_import_instance() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {
                "sea:exported": true,
                "sea:version": crate::VERSION
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
                    "unique-id": "resource-1",
                    "node-type": "resource",
                    "name": "Cameras",
                    "metadata": {
                        "sea:primitive": "Resource",
                        "sea:unit": "units"
                    }
                },
                {
                    "unique-id": "instance-1",
                    "node-type": "instance",
                    "name": "Camera-001",
                    "metadata": {
                        "sea:primitive": "Instance",
                        "sea:entity_id": "entity-1",
                        "sea:resource_id": "resource-1"
                    }
                }
            ],
            "relationships": []
        });

        let result = import(calm_json);
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert_eq!(graph.instance_count(), 1);

        let instance = graph.all_instances()[0];
        // Verify the instance references the correct entities and resources
        assert_eq!(
            graph.get_entity(instance.entity_id()).unwrap().name(),
            "Warehouse"
        );
        assert_eq!(
            graph.get_resource(instance.resource_id()).unwrap().name(),
            "Cameras"
        );
    }

    #[test]
    fn test_import_instance_with_namespace() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {
                "sea:exported": true,
                "sea:version": crate::VERSION
            },
            "nodes": [
                {
                    "unique-id": "entity-1",
                    "node-type": "actor",
                    "name": "Warehouse",
                    "namespace": "logistics",
                    "metadata": {
                        "sea:primitive": "Entity"
                    }
                },
                {
                    "unique-id": "resource-1",
                    "node-type": "resource",
                    "name": "Cameras",
                    "namespace": "security",
                    "metadata": {
                        "sea:primitive": "Resource",
                        "sea:unit": "units"
                    }
                },
                {
                    "unique-id": "instance-1",
                    "node-type": "instance",
                    "name": "Camera-001",
                    "namespace": "monitoring",
                    "metadata": {
                        "sea:primitive": "Instance",
                        "sea:entity_id": "entity-1",
                        "sea:resource_id": "resource-1"
                    }
                }
            ],
            "relationships": []
        });

        let result = import(calm_json);
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert_eq!(graph.instance_count(), 1);

        let instance = graph.all_instances()[0];
        // Verify the instance references the correct entities and resources
        assert_eq!(
            graph.get_entity(instance.entity_id()).unwrap().name(),
            "Warehouse"
        );
        assert_eq!(
            graph.get_resource(instance.resource_id()).unwrap().name(),
            "Cameras"
        );
        assert_eq!(instance.namespace(), "monitoring");
    }

    #[test]
    fn test_import_instance_missing_entity_id() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {
                "sea:exported": true,
                "sea:version": crate::VERSION
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
                    "unique-id": "resource-1",
                    "node-type": "resource",
                    "name": "Cameras",
                    "metadata": {
                        "sea:primitive": "Resource",
                        "sea:unit": "units"
                    }
                },
                {
                    "unique-id": "instance-1",
                    "node-type": "instance",
                    "name": "Camera-001",
                    "metadata": {
                        "sea:primitive": "Instance",
                        "sea:resource_id": "resource-1"
                        // Missing sea:entity_id
                    }
                }
            ],
            "relationships": []
        });

        let result = import(calm_json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing sea:entity_id"));
    }

    #[test]
    fn test_import_instance_missing_resource_id() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {
                "sea:exported": true,
                "sea:version": crate::VERSION
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
                    "unique-id": "resource-1",
                    "node-type": "resource",
                    "name": "Cameras",
                    "metadata": {
                        "sea:primitive": "Resource",
                        "sea:unit": "units"
                    }
                },
                {
                    "unique-id": "instance-1",
                    "node-type": "instance",
                    "name": "Camera-001",
                    "metadata": {
                        "sea:primitive": "Instance",
                        "sea:entity_id": "entity-1"
                        // Missing sea:resource_id
                    }
                }
            ],
            "relationships": []
        });

        let result = import(calm_json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing sea:resource_id"));
    }

    #[test]
    fn test_import_instance_unknown_entity_id() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {
                "sea:exported": true,
                "sea:version": crate::VERSION
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
                    "unique-id": "resource-1",
                    "node-type": "resource",
                    "name": "Cameras",
                    "metadata": {
                        "sea:primitive": "Resource",
                        "sea:unit": "units"
                    }
                },
                {
                    "unique-id": "instance-1",
                    "node-type": "instance",
                    "name": "Camera-001",
                    "metadata": {
                        "sea:primitive": "Instance",
                        "sea:entity_id": "nonexistent-entity",
                        "sea:resource_id": "resource-1"
                    }
                }
            ],
            "relationships": []
        });

        let result = import(calm_json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Unknown entity ID: nonexistent-entity"));
    }

    #[test]
    fn test_import_instance_unknown_resource_id() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {
                "sea:exported": true,
                "sea:version": crate::VERSION
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
                    "unique-id": "resource-1",
                    "node-type": "resource",
                    "name": "Cameras",
                    "metadata": {
                        "sea:primitive": "Resource",
                        "sea:unit": "units"
                    }
                },
                {
                    "unique-id": "instance-1",
                    "node-type": "instance",
                    "name": "Camera-001",
                    "metadata": {
                        "sea:primitive": "Instance",
                        "sea:entity_id": "entity-1",
                        "sea:resource_id": "nonexistent-resource"
                    }
                }
            ],
            "relationships": []
        });

        let result = import(calm_json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Unknown resource ID: nonexistent-resource"));
    }

    #[test]
    fn test_import_flow_invalid_quantity() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {
                "sea:exported": true,
                "sea:version": crate::VERSION
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
                            "quantity": "not_a_number"
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
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid quantity"));
    }

    #[test]
    fn test_import_flow_unknown_source() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {
                "sea:exported": true,
                "sea:version": crate::VERSION
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
                        "source": "nonexistent-entity",
                        "destination": "entity-1"
                    }
                }
            ]
        });

        let result = import(calm_json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Unknown source ID: nonexistent-entity"));
    }

    #[test]
    fn test_import_flow_unknown_destination() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {
                "sea:exported": true,
                "sea:version": crate::VERSION
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
                        "destination": "nonexistent-entity"
                    }
                }
            ]
        });

        let result = import(calm_json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Unknown destination ID: nonexistent-entity"));
    }

    #[test]
    fn test_import_entity_without_namespace() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {
                "sea:exported": true,
                "sea:version": crate::VERSION
            },
            "nodes": [
                {
                    "unique-id": "entity-1",
                    "node-type": "actor",
                    "name": "Warehouse",
                    // No namespace field
                    "metadata": {
                        "sea:primitive": "Entity"
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
        assert_eq!(entity.namespace(), "default");
    }

    #[test]
    fn test_import_resource_without_namespace() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {
                "sea:exported": true,
                "sea:version": crate::VERSION
            },
            "nodes": [
                {
                    "unique-id": "resource-1",
                    "node-type": "resource",
                    "name": "Cameras",
                    // No namespace field
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
        assert_eq!(resource.namespace(), "default");
    }

    #[test]
    fn test_import_flow_same_source_destination() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {
                "sea:exported": true,
                "sea:version": crate::VERSION
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
                        "destination": "entity-1"
                    }
                }
            ]
        });

        let result = import(calm_json);
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert_eq!(graph.flow_count(), 1);

        let flow = graph.all_flows()[0];
        assert_eq!(flow.from_id(), flow.to_id());
    }

    #[test]
    fn test_import_relationship_invalid_parties() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {
                "sea:exported": true,
                "sea:version": crate::VERSION
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
                        "owner": "entity-1",
                        "owned": "entity-1"
                    }
                }
            ]
        });

        let result = import(calm_json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Flow relationship must have source/destination parties"));
    }

    #[test]
    fn test_import_policy_node() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {"sea:exported": true, "sea:version": crate::VERSION},
            "nodes": [
                {
                    "unique-id": "entity-1",
                    "node-type": "actor",
                    "name": "Warehouse",
                    "metadata": {"sea:primitive": "Entity"}
                },
                {
                    "unique-id": "policy-1",
                    "node-type": "constraint",
                    "name": "MustHavePositiveQuantity",
                    "metadata": {
                        "sea:primitive": "Policy",
                        "sea:expression": "forall f in flows: (f.quantity > 0)",
                        "sea:expression_type": "SEA"
                    }
                }
            ],
            "relationships": []
        });

        let result = import(calm_json);
        assert!(result.is_ok());
        let graph = result.unwrap();
        assert_eq!(graph.policy_count(), 1);
        let policy = graph.all_policies()[0];
        assert_eq!(policy.name, "MustHavePositiveQuantity");
    }

    #[test]
    fn test_import_association_relationship() {
        let calm_json = json!({
            "version": "2.0",
            "metadata": {"sea:exported": true, "sea:version": crate::VERSION},
            "nodes": [
                {
                    "unique-id": "entity-1",
                    "node-type": "actor",
                    "name": "A",
                    "metadata": {"sea:primitive": "Entity"}
                },
                {
                    "unique-id": "entity-2",
                    "node-type": "actor",
                    "name": "B",
                    "metadata": {"sea:primitive": "Entity"}
                }
            ],
            "relationships": [
                {
                    "unique-id": "assoc-1",
                    "relationship-type": "association",
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
        assert_eq!(graph.entity_count(), 2);
        let _entities = graph.all_entities();
        // Get the first entity and check associations attribute
        let e1 = graph.find_entity_by_name("A").unwrap();
        let e1_ref = graph.get_entity(&e1).unwrap();
        let associations = e1_ref.get_attribute("associations").unwrap();
        assert!(associations.is_array());
        let arr = associations.as_array().unwrap();
        assert_eq!(
            arr[0]["target"],
            Value::String(graph.find_entity_by_name("B").unwrap().to_string())
        );
    }
}
