use serde::{Deserialize, Serialize};

use super::schema::{PackRef, PackSetRef, SourceRef, Suggestion};

// ---------------------------------------------------------------------------
// 4.5 SemanticTruth and DiagnosticSeverity
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SemanticTruth {
    Valid,
    Invalid,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

// ---------------------------------------------------------------------------
// 4.6 SemanticDiagnosticCode
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SemanticDiagnosticCode {
    UnknownConcept,
    DeprecatedConcept,
    AmbiguousConcept,
    AmbiguousAlias,
    AliasConflict,
    RejectedConcept,
    ProposedConcept,
    InvalidRelation,
    InvalidPredicate,
    UnknownMetric,
    UnknownDimension,
    UnknownUnit,
    UnitMismatch,
    DimensionMismatch,
    MissingDefinition,
    MissingOwner,
    DuplicateCanonicalName,
    DuplicateConceptId,
    UnreviewedConcept,
    PackUnavailable,
    PackSchemaMismatch,
    PackVersionMismatch,
    PackHashMismatch,
    PackUnsigned,
    PackSignatureInvalid,
    PackSetConflict,
    MappingRequired,
    MeaningVersionNotBumped,
    MeaningVersionBaselineMissing,
    AmbiguousAliasGroup,
}

impl SemanticDiagnosticCode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::UnknownConcept => "unknown_concept",
            Self::DeprecatedConcept => "deprecated_concept",
            Self::AmbiguousConcept => "ambiguous_concept",
            Self::AmbiguousAlias => "ambiguous_alias",
            Self::AliasConflict => "alias_conflict",
            Self::RejectedConcept => "rejected_concept",
            Self::ProposedConcept => "proposed_concept",
            Self::InvalidRelation => "invalid_relation",
            Self::InvalidPredicate => "invalid_predicate",
            Self::UnknownMetric => "unknown_metric",
            Self::UnknownDimension => "unknown_dimension",
            Self::UnknownUnit => "unknown_unit",
            Self::UnitMismatch => "unit_mismatch",
            Self::DimensionMismatch => "dimension_mismatch",
            Self::MissingDefinition => "missing_definition",
            Self::MissingOwner => "missing_owner",
            Self::DuplicateCanonicalName => "duplicate_canonical_name",
            Self::DuplicateConceptId => "duplicate_concept_id",
            Self::UnreviewedConcept => "unreviewed_concept",
            Self::PackUnavailable => "pack_unavailable",
            Self::PackSchemaMismatch => "pack_schema_mismatch",
            Self::PackVersionMismatch => "pack_version_mismatch",
            Self::PackHashMismatch => "pack_hash_mismatch",
            Self::PackUnsigned => "pack_unsigned",
            Self::PackSignatureInvalid => "pack_signature_invalid",
            Self::PackSetConflict => "pack_set_conflict",
            Self::MappingRequired => "mapping_required",
            Self::MeaningVersionNotBumped => "meaning_version_not_bumped",
            Self::MeaningVersionBaselineMissing => "meaning_version_baseline_missing",
            Self::AmbiguousAliasGroup => "ambiguous_alias_group",
        }
    }
}

// ---------------------------------------------------------------------------
// 4.6 SemanticDiagnostic
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticDiagnostic {
    pub code: SemanticDiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub semantic_truth: SemanticTruth,
    pub message: String,
    pub source_ref: SourceRef,
    pub pack_ref: PackRef,
    #[serde(default)]
    pub suggestions: Vec<Suggestion>,
    pub recoverability_hint: String,
}

// ---------------------------------------------------------------------------
// 4.7 SemanticValidationResult
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticValidationResult {
    pub status: SemanticValidationStatus,
    pub diagnostics: Vec<SemanticDiagnostic>,
    pub pack_set_ref: PackSetRef,
    pub input_hash: String,
    pub validation_mode: ValidationMode,
    pub expected_hashes: Vec<ExpectedHashCheck>,
    pub unsigned_fixture_bypass_used: bool,
    pub first_approved_version_bypass_used: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_ref: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SemanticValidationStatus {
    Passed,
    Failed,
    Unknown,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExpectedHashCheck {
    pub path: String,
    pub expected_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_hash: Option<String>,
    pub matched: bool,
}

// ---------------------------------------------------------------------------
// ValidationMode / policies (11.2)
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationMode {
    Off,
    Warn,
    Strict,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UnknownConceptPolicy {
    Ignore,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeprecatedPolicy {
    Allow,
    Warn,
    ErrorInStrict,
    ErrorAlways,
}

// ---------------------------------------------------------------------------
// ValidationOptions (11.2)
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOptions {
    pub mode: ValidationMode,
    pub unknown_concept_policy: UnknownConceptPolicy,
    pub deprecated_policy: DeprecatedPolicy,
    pub require_signed_pack: bool,
    pub allow_unsigned_test_fixtures: bool,
}

impl Default for ValidationOptions {
    fn default() -> Self {
        Self {
            mode: ValidationMode::Warn,
            unknown_concept_policy: UnknownConceptPolicy::Warning,
            deprecated_policy: DeprecatedPolicy::Warn,
            require_signed_pack: false,
            allow_unsigned_test_fixtures: false,
        }
    }
}

impl ValidationOptions {
    pub fn strict() -> Self {
        Self {
            mode: ValidationMode::Strict,
            unknown_concept_policy: UnknownConceptPolicy::Error,
            deprecated_policy: DeprecatedPolicy::ErrorInStrict,
            require_signed_pack: true,
            allow_unsigned_test_fixtures: false,
        }
    }
}
