use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalmModel {
    #[serde(default = "default_version")]
    pub version: String,

    pub metadata: CalmMetadata,
    pub nodes: Vec<CalmNode>,
    pub relationships: Vec<CalmRelationship>,
}

fn default_version() -> String {
    "2.0".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalmMetadata {
    #[serde(rename = "sea:exported")]
    pub sea_exported: bool,

    #[serde(rename = "sea:version")]
    pub sea_version: String,

    #[serde(rename = "sea:timestamp", skip_serializing_if = "Option::is_none")]
    pub sea_timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalmNode {
    #[serde(rename = "unique-id")]
    pub unique_id: String,

    #[serde(rename = "node-type")]
    pub node_type: NodeType,

    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,

    #[serde(default)]
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Actor,
    Location,
    Resource,
    Instance,
    Constraint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalmRelationship {
    #[serde(rename = "unique-id")]
    pub unique_id: String,

    #[serde(rename = "relationship-type")]
    pub relationship_type: RelationshipType,

    pub parties: Parties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RelationshipType {
    Flow { flow: FlowDetails },
    Simple(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowDetails {
    pub resource: String,
    pub quantity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Parties {
    SourceDestination { source: String, destination: String },
    OwnerOwned { owner: String, owned: String },
}

impl CalmModel {
    pub fn new() -> Self {
        CalmModel {
            version: "2.0".to_string(),
            metadata: CalmMetadata {
                sea_exported: true,
                sea_version: crate::VERSION.to_string(),
                sea_timestamp: None,
            },
            nodes: Vec::new(),
            relationships: Vec::new(),
        }
    }
}

impl Default for CalmModel {
    fn default() -> Self {
        Self::new()
    }
}
