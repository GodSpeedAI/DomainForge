use crate::parser::ast::{MappingRule, ProjectionOverride};
use crate::primitives::{MappingContract, ProjectionContract};

pub fn find_mapping_rule<'a>(
    contract: &'a MappingContract,
    primitive_type: &str,
    primitive_name: &str,
) -> Option<&'a MappingRule> {
    contract.rules().iter().find(|r| {
        r.primitive_type.eq_ignore_ascii_case(primitive_type) && r.primitive_name == primitive_name
    })
}

pub fn find_projection_override<'a>(
    contract: &'a ProjectionContract,
    primitive_type: &str,
    primitive_name: &str,
) -> Option<&'a ProjectionOverride> {
    contract.overrides().iter().find(|r| {
        r.primitive_type.eq_ignore_ascii_case(primitive_type) && r.primitive_name == primitive_name
    })
}
