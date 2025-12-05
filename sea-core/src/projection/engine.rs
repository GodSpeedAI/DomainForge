use crate::primitives::{Entity, Flow, MappingContract, ProjectionContract};

pub trait ProjectionExporter {
    type Output;

    fn export_entity(
        &self,
        entity: &Entity,
        mapping: Option<&MappingContract>,
        projection: Option<&ProjectionContract>,
    ) -> Self::Output;

    fn export_flow(
        &self,
        flow: &Flow,
        mapping: Option<&MappingContract>,
        projection: Option<&ProjectionContract>,
    ) -> Self::Output;
}
