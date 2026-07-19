//! SEA application contract model (ADR-013, Milestone 0).

pub mod canonical;
pub mod contract;
pub mod diagnostic;
pub mod envelope;
pub mod policy_context;
pub mod resolve;
pub(crate) mod validate;

pub use canonical::{
    canonical_decimal, canonical_typed_value, document_self_hash, input_fingerprint,
    semantic_pack_set_hash, source_set_hash,
};
pub use contract::*;
pub use diagnostic::{
    ApplicationDiagnostic, ApplicationDiagnosticCode, ApplicationDiagnosticContext,
};
pub use envelope::{
    resolve_semantic_envelope, resolve_semantic_envelope_with_packs, semantic_closure_hash,
    validate_application_contract_document_json, validate_semantic_envelope_document_json,
    CanonicalSemanticEnvelope, CanonicalSemanticEnvelopeDocument,
};
pub use policy_context::{evaluate_precondition, EvaluationResult};
pub use resolve::{
    resolve_application_contract, resolve_application_contract_json,
    resolve_application_contract_with_packs, resolve_application_graph,
};
