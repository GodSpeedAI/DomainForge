use crate::semantic_pack::{
    build_semantic_pack, canonical_json, diff_packs, normalize_lookup_key, resolve_concept,
    validate_graph_with_pack, validate_semantic_pack, AliasStatus as RustAliasStatus,
    ApprovalState as RustApprovalState, ConceptKind as RustConceptKind,
    ConceptStatus as RustConceptStatus, DiagnosticSeverity as RustDiagnosticSeverity,
    PackBuildInput, SemanticPack, SemanticTruth as RustSemanticTruth,
    SemanticValidationStatus as RustSemanticValidationStatus, SignatureState as RustSignatureState,
    ValidationMode as RustValidationMode, ValidationOptions,
};
#[cfg(feature = "signing")]
use crate::semantic_pack::{derive_signer_id, sign_pack, verify_pack_signature};
use wasm_bindgen::prelude::*;

// =============================================================================
// Enums
// =============================================================================

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum SemanticTruth {
    Valid = 0,
    Invalid = 1,
    Unknown = 2,
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

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum DiagnosticSeverity {
    Error = 0,
    Warning = 1,
    Info = 2,
    Hint = 3,
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

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum ValidationMode {
    Off = 0,
    Warn = 1,
    Strict = 2,
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

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum ApprovalState {
    Candidate = 0,
    Approved = 1,
    Rejected = 2,
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

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum SignatureState {
    Unsigned = 0,
    Signed = 1,
    InvalidSignature = 2,
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

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum ConceptStatus {
    Active = 0,
    Proposed = 1,
    Deprecated = 2,
    Rejected = 3,
    ExternalOnly = 4,
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

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum ConceptKind {
    Entity = 0,
    Resource = 1,
    Role = 2,
    Flow = 3,
    Policy = 4,
    Metric = 5,
    Dimension = 6,
    Unit = 7,
    External = 8,
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

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum AliasStatus {
    Approved = 0,
    Deprecated = 1,
    Ambiguous = 2,
    Blocked = 3,
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

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum SemanticValidationStatus {
    Passed = 0,
    Failed = 1,
    Unknown = 2,
    Blocked = 3,
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
// Functions
// =============================================================================

#[wasm_bindgen(js_name = "semanticPackBuild")]
pub fn semantic_pack_build(input_json: &str) -> Result<String, JsError> {
    let input: PackBuildInput = serde_json::from_str(input_json)
        .map_err(|e| JsError::new(&format!("Invalid build input: {}", e)))?;

    let output = build_semantic_pack(input).map_err(|diagnostics| {
        let msg = format!(
            "Build failed with {} diagnostic(s): {}",
            diagnostics.len(),
            diagnostics
                .iter()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join("; ")
        );
        JsError::new(&msg)
    })?;

    let result = serde_json::json!({
        "pack": output.pack,
        "pack_content_hash": output.pack_content_hash,
        "meaning_fingerprint": output.meaning_fingerprint,
        "pre_pack_diagnostics": output.pre_pack_diagnostics,
        "build_warnings": output.build_warnings,
    });

    serde_json::to_string(&result).map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

#[wasm_bindgen(js_name = "semanticPackValidate")]
pub fn semantic_pack_validate(pack_json: &str) -> Result<String, JsError> {
    let pack: SemanticPack = serde_json::from_str(pack_json)
        .map_err(|e| JsError::new(&format!("Invalid pack JSON: {}", e)))?;

    let diagnostics = validate_semantic_pack(&pack)
        .map_err(|e| JsError::new(&format!("Validation error: {:?}", e)))?;

    serde_json::to_string(&diagnostics)
        .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

#[wasm_bindgen(js_name = "semanticPackValidateGraph")]
pub fn semantic_pack_validate_graph(
    pack_json: &str,
    source: &str,
    options_json: &str,
) -> Result<String, JsError> {
    let pack: SemanticPack = serde_json::from_str(pack_json)
        .map_err(|e| JsError::new(&format!("Invalid pack JSON: {}", e)))?;

    let options: ValidationOptions = if options_json.is_empty() {
        ValidationOptions::default()
    } else {
        serde_json::from_str(options_json)
            .map_err(|e| JsError::new(&format!("Invalid options JSON: {}", e)))?
    };

    let result = validate_graph_with_pack(&pack, source, &options);

    serde_json::to_string(&result).map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

#[cfg(feature = "signing")]
#[wasm_bindgen(js_name = "semanticPackSign")]
pub fn semantic_pack_sign(pack_json: &str, private_key_pem: &str) -> Result<String, JsError> {
    let mut pack: SemanticPack = serde_json::from_str(pack_json)
        .map_err(|e| JsError::new(&format!("Invalid pack JSON: {}", e)))?;

    let sign_output = sign_pack(&pack, private_key_pem.as_bytes())
        .map_err(|e| JsError::new(&format!("Signing error: {:?}", e)))?;
    let signer_id = derive_signer_id(private_key_pem.as_bytes())
        .map_err(|e| JsError::new(&format!("Key derivation error: {:?}", e)))?;

    pack.trust.signature_state = RustSignatureState::Signed;
    pack.trust.signature = Some(sign_output.signature);
    pack.trust.signature_alg = Some(sign_output.signature_alg);
    pack.trust.signed_by = Some(signer_id);

    serde_json::to_string(&pack).map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

#[cfg(not(feature = "signing"))]
#[wasm_bindgen(js_name = "semanticPackSign")]
pub fn semantic_pack_sign(_pack_json: &str, _private_key_pem: &str) -> Result<String, JsError> {
    Err(JsError::new("signing requires the 'signing' feature"))
}

#[cfg(feature = "signing")]
#[wasm_bindgen(js_name = "semanticPackVerify")]
pub fn semantic_pack_verify(pack_json: &str, public_key_pem: &str) -> Result<bool, JsError> {
    let pack: SemanticPack = serde_json::from_str(pack_json)
        .map_err(|e| JsError::new(&format!("Invalid pack JSON: {}", e)))?;

    verify_pack_signature(&pack, public_key_pem.as_bytes())
        .map(|_| true)
        .map_err(|e| JsError::new(&format!("Verification failed: {:?}", e)))
}

#[cfg(not(feature = "signing"))]
#[wasm_bindgen(js_name = "semanticPackVerify")]
pub fn semantic_pack_verify(_pack_json: &str, _public_key_pem: &str) -> Result<bool, JsError> {
    Err(JsError::new("signing requires the 'signing' feature"))
}

#[wasm_bindgen(js_name = "semanticPackDiff")]
pub fn semantic_pack_diff(old_json: &str, new_json: &str) -> Result<String, JsError> {
    let old_pack: SemanticPack = serde_json::from_str(old_json)
        .map_err(|e| JsError::new(&format!("Invalid old pack JSON: {}", e)))?;
    let new_pack: SemanticPack = serde_json::from_str(new_json)
        .map_err(|e| JsError::new(&format!("Invalid new pack JSON: {}", e)))?;

    let diff = diff_packs(&old_pack, &new_pack);

    serde_json::to_string(&diff).map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

#[wasm_bindgen(js_name = "semanticPackHash")]
pub fn semantic_pack_hash(pack_json: &str) -> Result<String, JsError> {
    let pack: SemanticPack = serde_json::from_str(pack_json)
        .map_err(|e| JsError::new(&format!("Invalid pack JSON: {}", e)))?;

    Ok(canonical_json::compute_pack_content_hash(&pack))
}

#[wasm_bindgen(js_name = "semanticNormalizeKey")]
pub fn semantic_normalize_key(text: &str) -> String {
    normalize_lookup_key(text)
}

#[wasm_bindgen(js_name = "semanticResolveConcept")]
pub fn semantic_resolve_concept(
    raw_text: &str,
    pack_json: &str,
    options_json: &str,
) -> Result<String, JsError> {
    let pack: SemanticPack = serde_json::from_str(pack_json)
        .map_err(|e| JsError::new(&format!("Invalid pack JSON: {}", e)))?;

    let options: ValidationOptions = if options_json.is_empty() {
        ValidationOptions::default()
    } else {
        serde_json::from_str(options_json)
            .map_err(|e| JsError::new(&format!("Invalid options JSON: {}", e)))?
    };

    let source_ref = crate::semantic_pack::schema::SourceRef::synthetic("wasm://resolve");
    let request = crate::semantic_pack::resolver::ResolveRequest {
        raw_text,
        expected_kind: None,
        source_ref,
    };

    let result = resolve_concept(&request, &pack, &options);

    let output = serde_json::json!({
        "resolved_concept_id": result.resolved_concept_id,
        "semantic_truth": serde_json::to_value(result.semantic_truth).unwrap_or_default(),
        "diagnostic_code": result.diagnostic_code.as_ref().map(|c| c.as_str().to_string()),
        "diagnostic_severity": serde_json::to_value(result.diagnostic_severity).unwrap_or_default(),
        "message": result.message,
        "suggestions": result.suggestions,
    });

    serde_json::to_string(&output).map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}
