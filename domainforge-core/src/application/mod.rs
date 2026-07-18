//! SEA application contract model (ADR-013, Milestone 0).

pub mod contract;
pub mod diagnostic;

pub use contract::*;
pub use diagnostic::{
    ApplicationDiagnostic, ApplicationDiagnosticCode, ApplicationDiagnosticContext,
};
