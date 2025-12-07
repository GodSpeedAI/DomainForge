use crate::graph::Graph;
use crate::parser::ast::{Ast, AstNode, FileMetadata};

impl Graph {
    pub fn to_ast(&self) -> Ast {
        let mut declarations = Vec::new();

        // Entities
        for entity in self.entities.values() {
            declarations.push(AstNode::Entity {
                name: entity.name().to_string(),
                version: entity.version().map(|v| v.to_string()),
                annotations: Default::default(),
                domain: if entity.namespace() != "default" {
                    Some(entity.namespace().to_string())
                } else {
                    None
                },
            });
        }

        // Resources
        for resource in self.resources.values() {
            declarations.push(AstNode::Resource {
                name: resource.name().to_string(),
                unit_name: None,
                domain: if resource.namespace() != "default" {
                    Some(resource.namespace().to_string())
                } else {
                    None
                },
            });
        }

        Ast {
            metadata: FileMetadata::default(),
            declarations,
        }
    }
}
