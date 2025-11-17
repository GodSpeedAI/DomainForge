//! SEA Core — Rust implementation of DomainForge Domain Specific Language
//!
//! This library provides the five core primitives:
//! 1. Entity
//! 2. Resource
//! 3. Flow
//! 4. Instance
//! 5. Policy
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
//! assert_eq!(VERSION, "0.0.1");
//! ```

pub const VERSION: &str = "0.0.1";

pub mod calm;
pub mod kg_import;
pub mod concept_id;
pub mod graph;
pub mod kg;
pub mod parser;
pub mod policy;
pub mod primitives;
pub mod sbvr;
pub mod semantic_version;
pub mod units;
pub mod uuid_module;
pub mod validation_error;
pub mod validation_result;

#[cfg(feature = "python")]
pub mod python;

#[cfg(feature = "typescript")]
pub mod typescript;

#[cfg(feature = "typescript")]
pub use typescript::primitives::{
    Entity as TsEntity, Flow as TsFlow, Instance as TsInstance, Resource as TsResource,
};

#[cfg(feature = "typescript")]
pub use typescript::graph::Graph as TsGraph;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use concept_id::ConceptId;
pub use graph::Graph;
pub use kg::{KgError, KnowledgeGraph};
pub use parser::{parse, parse_to_graph};
pub use sbvr::{SbvrError, SbvrModel};
pub use kg_import::{import_kg_turtle, import_kg_rdfxml};
pub use semantic_version::SemanticVersion;
pub use units::{unit_from_string, Dimension, Unit, UnitError, UnitRegistry};
pub use uuid_module::{format_uuid, generate_uuid_v7, parse_uuid};
pub use validation_error::ValidationError;
pub use validation_result::ValidationResult;

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "0.0.1");
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
    m.add_class::<python::primitives::Instance>()?;
    m.add_class::<python::graph::Graph>()?;

    Ok(())
}
