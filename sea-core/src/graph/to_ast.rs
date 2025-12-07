use crate::graph::Graph;
use crate::parser::ast::{Ast, AstNode, FileMetadata};
use crate::primitives::{Entity, Resource};

const DEFAULT_NAMESPACE: &str = "default";

fn map_domain(namespace: &str) -> Option<String> {
    if namespace != DEFAULT_NAMESPACE {
        Some(namespace.to_string())
    } else {
        None
    }
}

impl Graph {
    pub fn to_ast(&self) -> Ast {
        let mut declarations = Vec::new();

        // Entities
        let mut entities: Vec<&Entity> = self.entities.values().collect();
        entities.sort_by(|a, b| {
            a.name()
                .cmp(b.name())
                .then_with(|| a.namespace().cmp(b.namespace()))
        });
        for entity in entities {
            declarations.push(AstNode::Entity {
                name: entity.name().to_string(),
                version: entity.version().map(|v| v.to_string()),
                annotations: Default::default(),
                domain: map_domain(entity.namespace()),
            });
        }

        // Resources
        let mut resources: Vec<&Resource> = self.resources.values().collect();
        resources.sort_by(|a, b| {
            a.name()
                .cmp(b.name())
                .then_with(|| a.namespace().cmp(b.namespace()))
        });
        for resource in resources {
            declarations.push(AstNode::Resource {
                name: resource.name().to_string(),
                unit_name: None,
                domain: map_domain(resource.namespace()),
            });
        }

        Ast {
            metadata: FileMetadata::default(),
            declarations,
        }
    }
}
