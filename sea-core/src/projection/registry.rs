use crate::graph::Graph;
use crate::parser::ast::TargetFormat;
use crate::primitives::{MappingContract, ProjectionContract};

pub struct ProjectionRegistry<'a> {
    graph: &'a Graph,
}

impl<'a> ProjectionRegistry<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self { graph }
    }

    pub fn find_mappings_for_target(&self, target: &TargetFormat) -> Vec<&MappingContract> {
        self.graph
            .all_mappings()
            .into_iter()
            .filter(|m| m.target_format() == target)
            .collect()
    }

    pub fn find_projections_for_target(&self, target: &TargetFormat) -> Vec<&ProjectionContract> {
        self.graph
            .all_projections()
            .into_iter()
            .filter(|p| p.target_format() == target)
            .collect()
    }

    pub fn get_mapping_by_name(&self, name: &str) -> Option<&MappingContract> {
        self.graph
            .all_mappings()
            .into_iter()
            .find(|m| m.name() == name)
    }

    pub fn get_projection_by_name(&self, name: &str) -> Option<&ProjectionContract> {
        self.graph
            .all_projections()
            .into_iter()
            .find(|p| p.name() == name)
    }
}
