//! SEA application contract model (ADR-013, Milestone 0).

pub mod contract;
pub mod diagnostic;
pub mod policy_context;
pub mod resolve;
pub(crate) mod validate;

pub use contract::*;
pub use diagnostic::{
    ApplicationDiagnostic, ApplicationDiagnosticCode, ApplicationDiagnosticContext,
};
pub use policy_context::{evaluate_precondition, EvaluationResult};
pub use resolve::{resolve_application_contract, resolve_application_contract_json};
