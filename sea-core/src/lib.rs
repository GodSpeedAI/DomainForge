//! SEA Core — Rust implementation of DomainForge Domain Specific Language
//!
//! This library provides the core primitives:
//! 1. Entity - Domain entities and concepts
//! 2. Resource - Resources that flow between entities
//! 3. Flow - Movement of resources between entities
//! 4. Instance - Instances of entities with field values
//! 5. ResourceInstance - Instances of resources
//! 6. Mapping - Data transformation and mapping contracts
//! 7. Projection - Output format projections
//! 8. Policy - Validation and constraint rules
//!
//! ## Building
//!
//! ```bash
//! cargo build
//! cargo test
//! cargo doc --no-deps --open
//! ```
//!
//! # Example
//!
//! ```
//! use sea_core::VERSION;
//! assert_eq!(VERSION, "0.1.0");
//! ```

pub const VERSION: &str = "0.1.0";

// Use a compact allocator only when compiling to wasm to avoid pulling wasm-only symbols on other targets.
#[cfg(all(feature = "lol_alloc", target_arch = "wasm32"))]
#[global_allocator]
static ALLOC: lol_alloc::LockedAllocator<lol_alloc::FreeListAllocator> =
    lol_alloc::LockedAllocator::new(lol_alloc::FreeListAllocator::new());

pub mod calm;
pub mod concept_id;
pub mod error;
pub mod formatter;
pub mod graph;
pub mod kg;
pub mod kg_import;
pub mod module;
pub mod parser;
pub mod patterns;
pub mod policy;
pub mod primitives;
pub mod projection;
pub mod registry;
pub mod sbvr;
pub mod semantic_version;
pub mod units;
pub mod uuid_module;
pub mod validation_error;
pub mod validation_result;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "python")]
pub mod python;

#[cfg(feature = "typescript")]
pub mod typescript;

#[cfg(feature = "typescript")]
pub use typescript::primitives::{
    Entity as TsEntity, Flow as TsFlow, Instance as TsInstance, Resource as TsResource,
    ResourceInstance as TsResourceInstance,
};

#[cfg(feature = "typescript")]
pub use typescript::graph::Graph as TsGraph;
#[cfg(feature = "typescript")]
pub use typescript::registry::{
    NamespaceBinding as TsNamespaceBinding, NamespaceRegistry as TsNamespaceRegistry,
};

#[cfg(feature = "wasm")]
pub mod wasm;

pub use concept_id::ConceptId;
pub use graph::Graph;
pub use kg::{KgError, KnowledgeGraph};
pub use kg_import::{import_kg_rdfxml, import_kg_turtle, ImportError};
pub use parser::{parse, parse_to_graph, parse_to_graph_with_options, ParseOptions};
pub use patterns::Pattern;
pub use registry::{NamespaceBinding, NamespaceRegistry, RegistryError};
pub use sbvr::{SbvrError, SbvrModel};
pub use semantic_version::SemanticVersion;
pub use units::{unit_from_string, Dimension, Unit, UnitError, UnitRegistry};
pub use uuid_module::{format_uuid, generate_uuid_v7, parse_uuid};
pub use validation_error::{ErrorCode, Position, SourceRange, ValidationError};
pub use validation_result::ValidationResult;

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "0.1.0");
    }
}

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymodule]
fn sea_dsl(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", VERSION)?;

    m.add_class::<python::primitives::Entity>()?;
    m.add_class::<python::primitives::Resource>()?;
    m.add_class::<python::primitives::Flow>()?;
    m.add_class::<python::primitives::ResourceInstance>()?;
    m.add_class::<python::primitives::Instance>()?;
    m.add_class::<python::primitives::Mapping>()?;
    m.add_class::<python::primitives::Projection>()?;
    m.add_class::<python::graph::Graph>()?;
    m.add_class::<python::primitives::Role>()?;
    m.add_class::<python::primitives::Relation>()?;
    m.add_class::<python::registry::NamespaceRegistry>()?;
    m.add_class::<python::registry::NamespaceBinding>()?;
    m.add_class::<python::policy::Severity>()?;
    m.add_class::<python::policy::Violation>()?;
    m.add_class::<python::policy::EvaluationResult>()?;
    m.add_class::<python::policy::UnaryOp>()?;
    m.add_class::<python::policy::Quantifier>()?;
    m.add_class::<python::policy::BinaryOp>()?;
    m.add_class::<python::policy::AggregateFunction>()?;
    m.add_class::<python::policy::WindowSpec>()?;
    m.add_class::<python::policy::Expression>()?;
    m.add_class::<python::policy::NormalizedExpression>()?;
    m.add_class::<python::units::Dimension>()?;
    m.add_class::<python::units::Unit>()?;

    // Formatter functions
    m.add_function(pyo3::wrap_pyfunction!(python::formatter::format_source, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(python::formatter::check_format, m)?)?;

    Ok(())
}
