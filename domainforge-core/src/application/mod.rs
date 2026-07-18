//! SEA application contract model (ADR-013, Milestone 0).

pub mod contract;
pub mod diagnostic;
pub mod resolve;
pub(crate) mod validate;

pub use contract::*;
pub use diagnostic::{
    ApplicationDiagnostic, ApplicationDiagnosticCode, ApplicationDiagnosticContext,
};
pub use resolve::{resolve_application_contract, resolve_application_contract_json};
