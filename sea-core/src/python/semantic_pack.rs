use crate::semantic_pack::{
    build_semantic_pack as rust_build_semantic_pack, compute_pack_content_hash,
    diff_packs as rust_diff_packs, normalize_lookup_key as rust_normalize_lookup_key,
    resolve_concept as rust_resolve_concept,
    validate_graph_with_pack as rust_validate_graph_with_pack,
    validate_semantic_pack as rust_validate_semantic_pack, AliasStatus as RustAliasStatus,
    ApprovalState as RustApprovalState, ConceptKind as RustConceptKind,
    ConceptStatus as RustConceptStatus, DeprecatedPolicy as RustDeprecatedPolicy,
    DiagnosticSeverity as RustDiagnosticSeverity, PackBuildInput, SemanticPack as RustSemanticPack,
    SemanticTruth as RustSemanticTruth, SemanticValidationResult as RustSemanticValidationResult,
    SemanticValidationStatus as RustSemanticValidationStatus, SignatureState as RustSignatureState,
    UnknownConceptPolicy as RustUnknownConceptPolicy, ValidationMode as RustValidationMode,
    ValidationOptions,
};
#[cfg(feature = "signing")]
use crate::semantic_pack::{
    sign_pack as rust_sign_pack, verify_pack_signature as rust_verify_pack_signature,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

// =============================================================================
// Enums
// =============================================================================

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, PartialEq)]
pub enum SemanticTruth {
    Valid,
    Invalid,
    Unknown,
}

impl From<SemanticTruth> for RustSemanticTruth {
    fn from(v: SemanticTruth) -> Self {
        match v {
            SemanticTruth::Valid => RustSemanticTruth::Valid,
            SemanticTruth::Invalid => RustSemanticTruth::Invalid,
            SemanticTruth::Unknown => RustSemanticTruth::Unknown,
        }
    }
}

impl From<RustSemanticTruth> for SemanticTruth {
    fn from(v: RustSemanticTruth) -> Self {
        match v {
            RustSemanticTruth::Valid => SemanticTruth::Valid,
            RustSemanticTruth::Invalid => SemanticTruth::Invalid,
            RustSemanticTruth::Unknown => SemanticTruth::Unknown,
        }
    }
}

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

impl From<DiagnosticSeverity> for RustDiagnosticSeverity {
    fn from(v: DiagnosticSeverity) -> Self {
        match v {
            DiagnosticSeverity::Error => RustDiagnosticSeverity::Error,
            DiagnosticSeverity::Warning => RustDiagnosticSeverity::Warning,
            DiagnosticSeverity::Info => RustDiagnosticSeverity::Info,
            DiagnosticSeverity::Hint => RustDiagnosticSeverity::Hint,
        }
    }
}

impl From<RustDiagnosticSeverity> for DiagnosticSeverity {
    fn from(v: RustDiagnosticSeverity) -> Self {
        match v {
            RustDiagnosticSeverity::Error => DiagnosticSeverity::Error,
            RustDiagnosticSeverity::Warning => DiagnosticSeverity::Warning,
            RustDiagnosticSeverity::Info => DiagnosticSeverity::Info,
            RustDiagnosticSeverity::Hint => DiagnosticSeverity::Hint,
        }
    }
}

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, PartialEq)]
pub enum ValidationMode {
    Off,
    Warn,
    Strict,
}

impl From<ValidationMode> for RustValidationMode {
    fn from(v: ValidationMode) -> Self {
        match v {
            ValidationMode::Off => RustValidationMode::Off,
            ValidationMode::Warn => RustValidationMode::Warn,
            ValidationMode::Strict => RustValidationMode::Strict,
        }
    }
}

impl From<RustValidationMode> for ValidationMode {
    fn from(v: RustValidationMode) -> Self {
        match v {
            RustValidationMode::Off => ValidationMode::Off,
            RustValidationMode::Warn => ValidationMode::Warn,
            RustValidationMode::Strict => ValidationMode::Strict,
        }
    }
}

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, PartialEq)]
pub enum ApprovalState {
    Candidate,
    Approved,
    Rejected,
}

impl From<ApprovalState> for RustApprovalState {
    fn from(v: ApprovalState) -> Self {
        match v {
            ApprovalState::Candidate => RustApprovalState::Candidate,
            ApprovalState::Approved => RustApprovalState::Approved,
            ApprovalState::Rejected => RustApprovalState::Rejected,
        }
    }
}

impl From<RustApprovalState> for ApprovalState {
    fn from(v: RustApprovalState) -> Self {
        match v {
            RustApprovalState::Candidate => ApprovalState::Candidate,
            RustApprovalState::Approved => ApprovalState::Approved,
            RustApprovalState::Rejected => ApprovalState::Rejected,
        }
    }
}

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, PartialEq)]
pub enum SignatureState {
    Unsigned,
    Signed,
    InvalidSignature,
}

impl From<SignatureState> for RustSignatureState {
    fn from(v: SignatureState) -> Self {
        match v {
            SignatureState::Unsigned => RustSignatureState::Unsigned,
            SignatureState::Signed => RustSignatureState::Signed,
            SignatureState::InvalidSignature => RustSignatureState::InvalidSignature,
        }
    }
}

impl From<RustSignatureState> for SignatureState {
    fn from(v: RustSignatureState) -> Self {
        match v {
            RustSignatureState::Unsigned => SignatureState::Unsigned,
            RustSignatureState::Signed => SignatureState::Signed,
            RustSignatureState::InvalidSignature => SignatureState::InvalidSignature,
        }
    }
}

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, PartialEq)]
pub enum ConceptStatus {
    Active,
    Proposed,
    Deprecated,
    Rejected,
    ExternalOnly,
}

impl From<ConceptStatus> for RustConceptStatus {
    fn from(v: ConceptStatus) -> Self {
        match v {
            ConceptStatus::Active => RustConceptStatus::Active,
            ConceptStatus::Proposed => RustConceptStatus::Proposed,
            ConceptStatus::Deprecated => RustConceptStatus::Deprecated,
            ConceptStatus::Rejected => RustConceptStatus::Rejected,
            ConceptStatus::ExternalOnly => RustConceptStatus::ExternalOnly,
        }
    }
}

impl From<RustConceptStatus> for ConceptStatus {
    fn from(v: RustConceptStatus) -> Self {
        match v {
            RustConceptStatus::Active => ConceptStatus::Active,
            RustConceptStatus::Proposed => ConceptStatus::Proposed,
            RustConceptStatus::Deprecated => ConceptStatus::Deprecated,
            RustConceptStatus::Rejected => ConceptStatus::Rejected,
            RustConceptStatus::ExternalOnly => ConceptStatus::ExternalOnly,
        }
    }
}

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, PartialEq)]
pub enum ConceptKind {
    Entity,
    Resource,
    Role,
    Flow,
    Policy,
    Metric,
    Dimension,
    Unit,
    External,
}

impl From<ConceptKind> for RustConceptKind {
    fn from(v: ConceptKind) -> Self {
        match v {
            ConceptKind::Entity => RustConceptKind::Entity,
            ConceptKind::Resource => RustConceptKind::Resource,
            ConceptKind::Role => RustConceptKind::Role,
            ConceptKind::Flow => RustConceptKind::Flow,
            ConceptKind::Policy => RustConceptKind::Policy,
            ConceptKind::Metric => RustConceptKind::Metric,
            ConceptKind::Dimension => RustConceptKind::Dimension,
            ConceptKind::Unit => RustConceptKind::Unit,
            ConceptKind::External => RustConceptKind::External,
        }
    }
}

impl From<RustConceptKind> for ConceptKind {
    fn from(v: RustConceptKind) -> Self {
        match v {
            RustConceptKind::Entity => ConceptKind::Entity,
            RustConceptKind::Resource => ConceptKind::Resource,
            RustConceptKind::Role => ConceptKind::Role,
            RustConceptKind::Flow => ConceptKind::Flow,
            RustConceptKind::Policy => ConceptKind::Policy,
            RustConceptKind::Metric => ConceptKind::Metric,
            RustConceptKind::Dimension => ConceptKind::Dimension,
            RustConceptKind::Unit => ConceptKind::Unit,
            RustConceptKind::External => ConceptKind::External,
        }
    }
}

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, PartialEq)]
pub enum AliasStatus {
    Approved,
    Deprecated,
    Ambiguous,
    Blocked,
}

impl From<AliasStatus> for RustAliasStatus {
    fn from(v: AliasStatus) -> Self {
        match v {
            AliasStatus::Approved => RustAliasStatus::Approved,
            AliasStatus::Deprecated => RustAliasStatus::Deprecated,
            AliasStatus::Ambiguous => RustAliasStatus::Ambiguous,
            AliasStatus::Blocked => RustAliasStatus::Blocked,
        }
    }
}

impl From<RustAliasStatus> for AliasStatus {
    fn from(v: RustAliasStatus) -> Self {
        match v {
            RustAliasStatus::Approved => AliasStatus::Approved,
            RustAliasStatus::Deprecated => AliasStatus::Deprecated,
            RustAliasStatus::Ambiguous => AliasStatus::Ambiguous,
            RustAliasStatus::Blocked => AliasStatus::Blocked,
        }
    }
}

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, PartialEq)]
pub enum UnknownConceptPolicy {
    Ignore,
    Warning,
    Error,
}

impl From<UnknownConceptPolicy> for RustUnknownConceptPolicy {
    fn from(v: UnknownConceptPolicy) -> Self {
        match v {
            UnknownConceptPolicy::Ignore => RustUnknownConceptPolicy::Ignore,
            UnknownConceptPolicy::Warning => RustUnknownConceptPolicy::Warning,
            UnknownConceptPolicy::Error => RustUnknownConceptPolicy::Error,
        }
    }
}

impl From<RustUnknownConceptPolicy> for UnknownConceptPolicy {
    fn from(v: RustUnknownConceptPolicy) -> Self {
        match v {
            RustUnknownConceptPolicy::Ignore => UnknownConceptPolicy::Ignore,
            RustUnknownConceptPolicy::Warning => UnknownConceptPolicy::Warning,
            RustUnknownConceptPolicy::Error => UnknownConceptPolicy::Error,
        }
    }
}

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, PartialEq)]
pub enum DeprecatedPolicy {
    Allow,
    Warn,
    ErrorInStrict,
    ErrorAlways,
}

impl From<DeprecatedPolicy> for RustDeprecatedPolicy {
    fn from(v: DeprecatedPolicy) -> Self {
        match v {
            DeprecatedPolicy::Allow => RustDeprecatedPolicy::Allow,
            DeprecatedPolicy::Warn => RustDeprecatedPolicy::Warn,
            DeprecatedPolicy::ErrorInStrict => RustDeprecatedPolicy::ErrorInStrict,
            DeprecatedPolicy::ErrorAlways => RustDeprecatedPolicy::ErrorAlways,
        }
    }
}

impl From<RustDeprecatedPolicy> for DeprecatedPolicy {
    fn from(v: RustDeprecatedPolicy) -> Self {
        match v {
            RustDeprecatedPolicy::Allow => DeprecatedPolicy::Allow,
            RustDeprecatedPolicy::Warn => DeprecatedPolicy::Warn,
            RustDeprecatedPolicy::ErrorInStrict => DeprecatedPolicy::ErrorInStrict,
            RustDeprecatedPolicy::ErrorAlways => DeprecatedPolicy::ErrorAlways,
        }
    }
}

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, PartialEq)]
pub enum SemanticValidationStatus {
    Passed,
    Failed,
    Unknown,
    Blocked,
}

impl From<SemanticValidationStatus> for RustSemanticValidationStatus {
    fn from(v: SemanticValidationStatus) -> Self {
        match v {
            SemanticValidationStatus::Passed => RustSemanticValidationStatus::Passed,
            SemanticValidationStatus::Failed => RustSemanticValidationStatus::Failed,
            SemanticValidationStatus::Unknown => RustSemanticValidationStatus::Unknown,
            SemanticValidationStatus::Blocked => RustSemanticValidationStatus::Blocked,
        }
    }
}

impl From<RustSemanticValidationStatus> for SemanticValidationStatus {
    fn from(v: RustSemanticValidationStatus) -> Self {
        match v {
            RustSemanticValidationStatus::Passed => SemanticValidationStatus::Passed,
            RustSemanticValidationStatus::Failed => SemanticValidationStatus::Failed,
            RustSemanticValidationStatus::Unknown => SemanticValidationStatus::Unknown,
            RustSemanticValidationStatus::Blocked => SemanticValidationStatus::Blocked,
        }
    }
}

// =============================================================================
// Structs
// =============================================================================

#[pyclass(skip_from_py_object)]
pub struct SemanticPack {
    inner: RustSemanticPack,
}

#[pymethods]
impl SemanticPack {
    #[staticmethod]
    fn from_json(json: &str) -> PyResult<Self> {
        let pack: RustSemanticPack = serde_json::from_str(json)
            .map_err(|e| PyValueError::new_err(format!("Invalid pack JSON: {}", e)))?;
        Ok(Self { inner: pack })
    }

    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner)
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
    }

    fn pack_id(&self) -> &str {
        &self.inner.pack_id
    }

    fn schema_version(&self) -> &str {
        &self.inner.schema_version
    }

    fn approval_state(&self) -> ApprovalState {
        self.inner.trust.approval_state.into()
    }

    fn signature_state(&self) -> SignatureState {
        self.inner.trust.signature_state.into()
    }

    fn concept_count(&self) -> usize {
        self.inner.concepts.len()
    }

    fn alias_count(&self) -> usize {
        self.inner.aliases.len()
    }

    fn meaning_version(&self) -> &str {
        &self.inner.meaning_version
    }

    fn meaning_fingerprint(&self) -> &str {
        &self.inner.meaning_fingerprint
    }

    fn pack_content_hash(&self) -> String {
        compute_pack_content_hash(&self.inner)
    }

    fn __repr__(&self) -> String {
        format!("<SemanticPack('{}')>", self.inner.pack_id)
    }
}

#[pyclass(skip_from_py_object)]
pub struct SemanticValidationResult {
    inner: RustSemanticValidationResult,
}

#[pymethods]
impl SemanticValidationResult {
    #[staticmethod]
    fn from_json(json: &str) -> PyResult<Self> {
        let result: RustSemanticValidationResult = serde_json::from_str(json)
            .map_err(|e| PyValueError::new_err(format!("Invalid result JSON: {}", e)))?;
        Ok(Self { inner: result })
    }

    fn status(&self) -> SemanticValidationStatus {
        self.inner.status.into()
    }

    fn diagnostics_json(&self) -> String {
        serde_json::to_string(&self.inner.diagnostics).unwrap_or_else(|_| "[]".to_string())
    }

    fn unsigned_fixture_bypass_used(&self) -> bool {
        self.inner.unsigned_fixture_bypass_used
    }

    fn first_approved_version_bypass_used(&self) -> bool {
        self.inner.first_approved_version_bypass_used
    }

    fn __repr__(&self) -> String {
        format!("<SemanticValidationResult(status={:?})>", self.inner.status)
    }
}

// =============================================================================
// Functions
// =============================================================================

#[pyfunction]
pub fn build_semantic_pack(input_json: &str) -> PyResult<(String, Vec<String>)> {
    let input: PackBuildInput = serde_json::from_str(input_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid build input JSON: {}", e)))?;
    match rust_build_semantic_pack(input) {
        Ok(output) => {
            let pack_json = serde_json::to_string(&output.pack)
                .map_err(|e| PyValueError::new_err(format!("Pack serialization error: {}", e)))?;
            let errors: Vec<String> = output
                .pre_pack_diagnostics
                .iter()
                .chain(output.build_warnings.iter())
                .map(|d| {
                    serde_json::to_string(d)
                        .unwrap_or_else(|_| serde_json::json!({"message": &d.message}).to_string())
                })
                .collect();
            Ok((pack_json, errors))
        }
        Err(diagnostics) => {
            let errors: Vec<String> = diagnostics
                .iter()
                .map(|d| {
                    serde_json::to_string(d)
                        .unwrap_or_else(|_| serde_json::json!({"message": &d.message}).to_string())
                })
                .collect();
            Ok((String::new(), errors))
        }
    }
}

#[pyfunction]
pub fn validate_semantic_pack(pack_json: &str) -> PyResult<Vec<String>> {
    let pack: RustSemanticPack = serde_json::from_str(pack_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid pack JSON: {}", e)))?;
    let diagnostics = rust_validate_semantic_pack(&pack)
        .map_err(|e| PyValueError::new_err(format!("Validation error: {:?}", e)))?;
    Ok(diagnostics
        .iter()
        .map(|d| {
            serde_json::to_string(d)
                .unwrap_or_else(|_| format!("{{\"message\": \"{}\"}}", d.message))
        })
        .collect())
}

#[pyfunction]
pub fn validate_graph_with_pack(
    pack_json: &str,
    source: &str,
    options_json: &str,
) -> PyResult<String> {
    let pack: RustSemanticPack = serde_json::from_str(pack_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid pack JSON: {}", e)))?;
    let options: ValidationOptions = serde_json::from_str(options_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid options JSON: {}", e)))?;
    let result = rust_validate_graph_with_pack(&pack, source, &options);
    serde_json::to_string(&result)
        .map_err(|e| PyValueError::new_err(format!("Result serialization error: {}", e)))
}

#[cfg(feature = "signing")]
#[pyfunction]
pub fn sign_pack(pack_json: &str, private_key_pem: &str) -> PyResult<String> {
    let mut pack: RustSemanticPack = serde_json::from_str(pack_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid pack JSON: {}", e)))?;
    let output = rust_sign_pack(&pack, private_key_pem.as_bytes())
        .map_err(|e| PyValueError::new_err(format!("Signing error: {:?}", e)))?;
    pack.trust.signature = Some(output.signature);
    pack.trust.signature_alg = Some(output.signature_alg);
    pack.trust.signature_state = RustSignatureState::Signed;
    serde_json::to_string(&pack)
        .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
}

#[cfg(not(feature = "signing"))]
#[pyfunction]
pub fn sign_pack(_pack_json: &str, _private_key_pem: &str) -> PyResult<String> {
    Err(PyValueError::new_err(
        "signing requires the 'signing' feature",
    ))
}

#[cfg(feature = "signing")]
#[pyfunction]
pub fn verify_pack_signature(pack_json: &str, public_key_pem: &str) -> PyResult<bool> {
    let pack: RustSemanticPack = serde_json::from_str(pack_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid pack JSON: {}", e)))?;
    match rust_verify_pack_signature(&pack, public_key_pem.as_bytes()) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(not(feature = "signing"))]
#[pyfunction]
pub fn verify_pack_signature(_pack_json: &str, _public_key_pem: &str) -> PyResult<bool> {
    Err(PyValueError::new_err(
        "signing requires the 'signing' feature",
    ))
}

#[pyfunction]
pub fn diff_packs(old_json: &str, new_json: &str) -> PyResult<String> {
    let old_pack: RustSemanticPack = serde_json::from_str(old_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid old pack JSON: {}", e)))?;
    let new_pack: RustSemanticPack = serde_json::from_str(new_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid new pack JSON: {}", e)))?;
    let diff = rust_diff_packs(&old_pack, &new_pack);
    serde_json::to_string(&diff)
        .map_err(|e| PyValueError::new_err(format!("Diff serialization error: {}", e)))
}

#[pyfunction]
pub fn compute_pack_hash(pack_json: &str) -> PyResult<String> {
    let pack: RustSemanticPack = serde_json::from_str(pack_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid pack JSON: {}", e)))?;
    Ok(compute_pack_content_hash(&pack))
}

#[pyfunction]
pub fn normalize_lookup_key(text: &str) -> String {
    rust_normalize_lookup_key(text)
}

#[pyfunction]
pub fn resolve_concept(raw_text: &str, pack_json: &str, options_json: &str) -> PyResult<String> {
    let pack: RustSemanticPack = serde_json::from_str(pack_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid pack JSON: {}", e)))?;
    let options: ValidationOptions = serde_json::from_str(options_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid options JSON: {}", e)))?;
    let request = crate::semantic_pack::resolver::ResolveRequest {
        raw_text,
        expected_kind: None,
        source_ref: crate::semantic_pack::schema::SourceRef::synthetic("python://resolve_concept"),
    };
    let result = rust_resolve_concept(&request, &pack, &options);
    serde_json::to_string(&serde_json::json!({
        "resolved_concept_id": result.resolved_concept_id,
        "semantic_truth": format!("{:?}", result.semantic_truth),
        "diagnostic_code": result.diagnostic_code.map(|c| c.as_str().to_string()),
        "message": result.message,
        "suggestions": result.suggestions,
    }))
    .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
}
