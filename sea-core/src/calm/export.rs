use crate::Graph;
use crate::primitives::{Entity, Resource, Flow, Instance};
use super::models::{
    CalmModel, CalmNode, CalmRelationship,
    NodeType, RelationshipType, FlowDetails, Parties
};
use serde_json::{json, Value};
use std::collections::HashMap;

pub fn export(graph: &Graph) -> Result<Value, String> {
    let mut calm_model = CalmModel::new();
    
    calm_model.metadata.sea_timestamp = Some(
        chrono::Utc::now().to_rfc3339()
    );
    
    for entity in graph.all_entities() {
        calm_model.nodes.push(export_entity(entity));
    }
    
    for resource in graph.all_resources() {
        calm_model.nodes.push(export_resource(resource));
    }
    
    for instance in graph.all_instances() {
        let (node, relationship) = export_instance(instance);
        calm_model.nodes.push(node);
        calm_model.relationships.push(relationship);
    }
    
    for flow in graph.all_flows() {
        calm_model.relationships.push(export_flow(flow));
    }
    
    serde_json::to_value(&calm_model)
        .map_err(|e| format!("Failed to serialize CALM model: {}", e))
}

fn export_entity(entity: &Entity) -> CalmNode {
    let mut metadata = HashMap::new();
    metadata.insert("sea:primitive".to_string(), json!("Entity"));
    
    CalmNode {
        unique_id: entity.id().to_string(),
        node_type: NodeType::Actor,
        name: entity.name().to_string(),
        namespace: entity.namespace().map(|s| s.to_string()),
        metadata,
    }
}

fn export_resource(resource: &Resource) -> CalmNode {
    let mut metadata = HashMap::new();
    metadata.insert("sea:primitive".to_string(), json!("Resource"));
    metadata.insert("sea:unit".to_string(), json!(resource.unit()));
    
    CalmNode {
        unique_id: resource.id().to_string(),
        node_type: NodeType::Resource,
        name: resource.name().to_string(),
        namespace: resource.namespace().map(|s| s.to_string()),
        metadata,
    }
}

fn export_instance(instance: &Instance) -> (CalmNode, CalmRelationship) {
    let mut metadata = HashMap::new();
    metadata.insert("sea:primitive".to_string(), json!("Instance"));
    metadata.insert("sea:entity_id".to_string(), json!(instance.entity_id().to_string()));
    metadata.insert("sea:resource_id".to_string(), json!(instance.resource_id().to_string()));
    
    let node = CalmNode {
        unique_id: instance.id().to_string(),
        node_type: NodeType::Instance,
        name: format!("Instance of {}", instance.id()),
        namespace: instance.namespace().map(|s| s.to_string()),
        metadata,
    };
    
    let relationship = CalmRelationship {
        unique_id: format!("{}-ownership", instance.id()),
        relationship_type: RelationshipType::Simple("ownership".to_string()),
        parties: Parties::OwnerOwned {
            owner: instance.entity_id().to_string(),
            owned: instance.id().to_string(),
        },
    };
    
    (node, relationship)
}

fn export_flow(flow: &Flow) -> CalmRelationship {
    CalmRelationship {
        unique_id: flow.id().to_string(),
        relationship_type: RelationshipType::Flow {
            flow: FlowDetails {
                resource: flow.resource_id().to_string(),
                quantity: flow.quantity().to_string(),
            },
        },
        parties: Parties::SourceDestination {
            source: flow.from_id().to_string(),
            destination: flow.to_id().to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::*;
    
    #[test]
    fn test_export_empty_graph() {
        let graph = Graph::new();
        let result = export(&graph);
        assert!(result.is_ok());
        
        let calm = result.unwrap();
        assert_eq!(calm["version"], "2.0");
        assert_eq!(calm["nodes"].as_array().unwrap().len(), 0);
        assert_eq!(calm["relationships"].as_array().unwrap().len(), 0);
    }
    
    #[test]
    fn test_export_entity() {
        let mut graph = Graph::new();
        let entity = Entity::new_with_namespace("Warehouse".to_string(), "logistics".to_string());
        graph.add_entity(entity.clone()).unwrap();
        
        let result = export(&graph).unwrap();
        let nodes = result["nodes"].as_array().unwrap();
        
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["name"], "Warehouse");
        assert_eq!(nodes[0]["namespace"], "logistics");
        assert_eq!(nodes[0]["metadata"]["sea:primitive"], "Entity");
    }
    
    #[test]
    fn test_export_resource() {
        let mut graph = Graph::new();
        let resource = Resource::new("Cameras".to_string(), "units".to_string());
        graph.add_resource(resource).unwrap();
        
        let result = export(&graph).unwrap();
        let nodes = result["nodes"].as_array().unwrap();
        
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["name"], "Cameras");
        assert_eq!(nodes[0]["node-type"], "resource");
        assert_eq!(nodes[0]["metadata"]["sea:unit"], "units");
    }
    
    #[test]
    fn test_export_flow() {
        let mut graph = Graph::new();
        
        let entity1 = Entity::new("Warehouse".to_string());
        let entity2 = Entity::new("Factory".to_string());
        let resource = Resource::new("Cameras".to_string(), "units".to_string());
        
        let e1_id = *entity1.id();
        let e2_id = *entity2.id();
        let r_id = *resource.id();
        
        graph.add_entity(entity1).unwrap();
        graph.add_entity(entity2).unwrap();
        graph.add_resource(resource).unwrap();
        
        let flow = Flow::new(r_id, e1_id, e2_id, rust_decimal::Decimal::from(100));
        graph.add_flow(flow).unwrap();
        
        let result = export(&graph).unwrap();
        let relationships = result["relationships"].as_array().unwrap();
        
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0]["relationship-type"]["flow"]["quantity"], "100");
    }
}
