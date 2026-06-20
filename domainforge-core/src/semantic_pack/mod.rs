pub mod builder;
pub mod canonical_json;
pub mod diagnostics;
pub mod diff;
pub mod pack_set;
pub mod resolver;
pub mod review;
pub mod schema;
#[cfg(feature = "signing")]
pub mod signing;
pub mod validator;

// Re-export main public types (§11.1)
pub use builder::{build_semantic_pack, PackBuildInput, PackBuildOutput};
pub use canonical_json::{canonical_json, compute_pack_content_hash, compute_sha256};
pub use diagnostics::{
    DeprecatedPolicy, DiagnosticSeverity, SemanticDiagnostic, SemanticDiagnosticCode,
    SemanticTruth, SemanticValidationResult, SemanticValidationStatus, UnknownConceptPolicy,
    ValidationMode, ValidationOptions,
};
pub use diff::{diff_packs, PackDiff};
pub use pack_set::{merge_packs, ConflictType, PackSet};
pub use resolver::{normalize_lookup_key, resolve_concept, ResolveRequest, ResolveResult};
pub use schema::{
    AliasDef, AliasStatus, ApprovalState, CompatibilityInfo, ConceptDef, ConceptDefinition,
    ConceptKind, ConceptStatus, GeneratorInfo, PackRef, PackSetRef, PackTrust, ReviewDecision,
    ReviewRecord, SemanticPack, SignatureState, SourceRef,
};
#[cfg(feature = "signing")]
pub use signing::{derive_signer_id, sign_pack, verify_pack_signature, SignOutput};
pub use validator::{validate_graph_with_pack, validate_semantic_pack, validate_term};
