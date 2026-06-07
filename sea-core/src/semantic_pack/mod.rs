pub mod builder;
pub mod canonical_json;
pub mod diagnostics;
pub mod diff;
pub mod pack_set;
pub mod review;
pub mod resolver;
pub mod schema;
pub mod signing;
pub mod validator;

// Re-export main public types (§11.1)
pub use builder::{PackBuildInput, PackBuildOutput, build_semantic_pack};
pub use canonical_json::{compute_pack_content_hash, compute_sha256, canonical_json};
pub use diagnostics::{
    DeprecatedPolicy, DiagnosticSeverity, SemanticDiagnostic, SemanticDiagnosticCode,
    SemanticTruth, SemanticValidationResult, SemanticValidationStatus, UnknownConceptPolicy,
    ValidationMode, ValidationOptions,
};
pub use diff::{PackDiff, diff_packs};
pub use pack_set::{ConflictType, PackSet, merge_packs};
pub use resolver::{resolve_concept, normalize_lookup_key, ResolveRequest, ResolveResult};
pub use schema::{
    AliasDef, AliasStatus, ApprovalState, CompatibilityInfo, ConceptDef, ConceptDefinition,
    ConceptKind, ConceptStatus, GeneratorInfo, PackRef, PackSetRef, PackTrust, ReviewDecision,
    ReviewRecord, SemanticPack, SignatureState, SourceRef,
};
pub use signing::{derive_signer_id, sign_pack, verify_pack_signature, SignOutput};
pub use validator::{validate_semantic_pack, validate_graph_with_pack, validate_term};
