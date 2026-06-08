use super::canonical_json;
use super::diagnostics::{
    DiagnosticSeverity, SemanticDiagnostic, SemanticDiagnosticCode, SemanticTruth,
    SemanticValidationResult, SemanticValidationStatus, ValidationMode, ValidationOptions,
};
use super::resolver::{self, ResolveRequest};
use super::schema::{ConceptStatus, PackRef, PackSetRef, SemanticPack, SignatureState, SourceRef};

/// Validate a semantic pack itself (internal consistency).
pub fn validate_semantic_pack(
    pack: &SemanticPack,
) -> Result<Vec<SemanticDiagnostic>, SemanticPackError> {
    let mut diagnostics = Vec::new();
    let pack_ref = make_pack_ref(pack);

    // Schema version check
    if pack.schema_version != "0.3" {
        diagnostics.push(SemanticDiagnostic {
            code: SemanticDiagnosticCode::PackSchemaMismatch,
            severity: DiagnosticSeverity::Error,
            semantic_truth: SemanticTruth::Invalid,
            message: format!(
                "Expected schema_version '0.3', got '{}'",
                pack.schema_version
            ),
            source_ref: SourceRef::pack_uri(&pack.pack_id),
            pack_ref: pack_ref.clone(),
            suggestions: vec![],
            recoverability_hint: "Update schema_version to 0.3".to_string(),
        });
    }

    // Duplicate concept IDs
    let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for c in &pack.concepts {
        if !seen_ids.insert(c.id.clone()) {
            diagnostics.push(SemanticDiagnostic {
                code: SemanticDiagnosticCode::DuplicateConceptId,
                severity: DiagnosticSeverity::Error,
                semantic_truth: SemanticTruth::Invalid,
                message: format!("Duplicate concept ID: '{}'", c.id),
                source_ref: SourceRef::pack_uri(&pack.pack_id),
                pack_ref: pack_ref.clone(),
                suggestions: vec![],
                recoverability_hint: "Rename one of the duplicate concepts".to_string(),
            });
        }
    }

    // Duplicate canonical names targeting different IDs
    let mut seen_names: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for c in &pack.concepts {
        let norm = resolver::normalize_lookup_key(&c.canonical_name);
        if let Some(existing_id) = seen_names.get(&norm) {
            if existing_id != &c.id {
                diagnostics.push(SemanticDiagnostic {
                    code: SemanticDiagnosticCode::DuplicateCanonicalName,
                    severity: DiagnosticSeverity::Error,
                    semantic_truth: SemanticTruth::Invalid,
                    message: format!(
                        "Duplicate canonical name '{}' targets '{}' and '{}'",
                        c.canonical_name, existing_id, c.id
                    ),
                    source_ref: SourceRef::pack_uri(&pack.pack_id),
                    pack_ref: pack_ref.clone(),
                    suggestions: vec![],
                    recoverability_hint: "Ensure canonical names are unique".to_string(),
                });
            }
        } else {
            seen_names.insert(norm, c.id.clone());
        }
    }

    // Missing definitions on active concepts
    for c in &pack.concepts {
        if matches!(c.status, ConceptStatus::Active | ConceptStatus::Proposed) {
            if c.definition.text.is_empty() {
                diagnostics.push(SemanticDiagnostic {
                    code: SemanticDiagnosticCode::MissingDefinition,
                    severity: DiagnosticSeverity::Warning,
                    semantic_truth: SemanticTruth::Unknown,
                    message: format!("Active concept '{}' has no definition", c.id),
                    source_ref: SourceRef::pack_uri(&pack.pack_id),
                    pack_ref: pack_ref.clone(),
                    suggestions: vec![],
                    recoverability_hint: "Add a definition text".to_string(),
                });
            }
            if c.owner.is_empty() {
                diagnostics.push(SemanticDiagnostic {
                    code: SemanticDiagnosticCode::MissingOwner,
                    severity: DiagnosticSeverity::Warning,
                    semantic_truth: SemanticTruth::Unknown,
                    message: format!("Active concept '{}' has no owner", c.id),
                    source_ref: SourceRef::pack_uri(&pack.pack_id),
                    pack_ref: pack_ref.clone(),
                    suggestions: vec![],
                    recoverability_hint: "Assign an owner".to_string(),
                });
            }
        }
    }

    // Alias target validation
    let concept_ids: std::collections::HashSet<&str> =
        pack.concepts.iter().map(|c| c.id.as_str()).collect();
    for a in &pack.aliases {
        if !concept_ids.contains(a.target_concept_id.as_str()) {
            diagnostics.push(SemanticDiagnostic {
                code: SemanticDiagnosticCode::UnknownConcept,
                severity: DiagnosticSeverity::Error,
                semantic_truth: SemanticTruth::Invalid,
                message: format!(
                    "Alias '{}' targets unknown concept '{}'",
                    a.alias, a.target_concept_id
                ),
                source_ref: SourceRef::pack_uri(&pack.pack_id),
                pack_ref: pack_ref.clone(),
                suggestions: vec![],
                recoverability_hint: "Add the target concept or fix the alias".to_string(),
            });
        }
    }

    // Relation endpoint validation
    for r in &pack.relations {
        if !concept_ids.contains(r.subject_id.as_str()) {
            diagnostics.push(SemanticDiagnostic {
                code: SemanticDiagnosticCode::InvalidRelation,
                severity: DiagnosticSeverity::Error,
                semantic_truth: SemanticTruth::Invalid,
                message: format!(
                    "Relation '{}' references unknown subject '{}'",
                    r.id, r.subject_id
                ),
                source_ref: SourceRef::pack_uri(&pack.pack_id),
                pack_ref: pack_ref.clone(),
                suggestions: vec![],
                recoverability_hint: "Add the missing concept or fix the relation".to_string(),
            });
        }
        if !concept_ids.contains(r.object_id.as_str()) {
            diagnostics.push(SemanticDiagnostic {
                code: SemanticDiagnosticCode::InvalidRelation,
                severity: DiagnosticSeverity::Error,
                semantic_truth: SemanticTruth::Invalid,
                message: format!(
                    "Relation '{}' references unknown object '{}'",
                    r.id, r.object_id
                ),
                source_ref: SourceRef::pack_uri(&pack.pack_id),
                pack_ref: pack_ref.clone(),
                suggestions: vec![],
                recoverability_hint: "Add the missing concept or fix the relation".to_string(),
            });
        }
    }

    // Metric dimension/unit validation
    let dimension_ids: std::collections::HashSet<&str> =
        pack.dimensions.iter().map(|d| d.id.as_str()).collect();
    let unit_ids: std::collections::HashSet<&str> =
        pack.units.iter().map(|u| u.id.as_str()).collect();
    for m in &pack.metrics {
        if let Some(ref dim_id) = m.dimension_id {
            if !dimension_ids.contains(dim_id.as_str()) {
                diagnostics.push(SemanticDiagnostic {
                    code: SemanticDiagnosticCode::UnknownDimension,
                    severity: DiagnosticSeverity::Warning,
                    semantic_truth: SemanticTruth::Unknown,
                    message: format!(
                        "Metric '{}' references unknown dimension '{}'",
                        m.id, dim_id
                    ),
                    source_ref: SourceRef::pack_uri(&pack.pack_id),
                    pack_ref: pack_ref.clone(),
                    suggestions: vec![],
                    recoverability_hint: "Add the missing dimension".to_string(),
                });
            }
        }
        if let Some(ref unit_id) = m.unit_id {
            if !unit_ids.contains(unit_id.as_str()) {
                diagnostics.push(SemanticDiagnostic {
                    code: SemanticDiagnosticCode::UnknownUnit,
                    severity: DiagnosticSeverity::Warning,
                    semantic_truth: SemanticTruth::Unknown,
                    message: format!("Metric '{}' references unknown unit '{}'", m.id, unit_id),
                    source_ref: SourceRef::pack_uri(&pack.pack_id),
                    pack_ref: pack_ref.clone(),
                    suggestions: vec![],
                    recoverability_hint: "Add the missing unit".to_string(),
                });
            }
        }
    }

    Ok(diagnostics)
}

/// Validate a parsed graph against a semantic pack.
pub fn validate_graph_with_pack(
    pack: &SemanticPack,
    _source_uri: &str,
    options: &ValidationOptions,
) -> SemanticValidationResult {
    let pack_ref = make_pack_ref(pack);

    // Check unsigned requirement
    if options.require_signed_pack
        && pack.trust.signature_state != SignatureState::Signed
        && !options.allow_unsigned_test_fixtures
    {
        let mut result = empty_result(&pack_ref, options);
        result.status = SemanticValidationStatus::Blocked;
        result.diagnostics.push(SemanticDiagnostic {
            code: SemanticDiagnosticCode::PackUnsigned,
            severity: DiagnosticSeverity::Error,
            semantic_truth: SemanticTruth::Invalid,
            message: format!(
                "Pack '{}' is unsigned but signature is required",
                pack.pack_id
            ),
            source_ref: SourceRef::pack_uri(&pack.pack_id),
            pack_ref: pack_ref.clone(),
            suggestions: vec![],
            recoverability_hint: "Sign the pack or use --allow-unsigned-for-test-fixtures"
                .to_string(),
        });
        return result;
    }

    // Run pack self-validation first
    let mut diagnostics = match validate_semantic_pack(pack) {
        Ok(d) => d,
        Err(e) => {
            let mut result = empty_result(&pack_ref, options);
            result.status = SemanticValidationStatus::Failed;
            result.diagnostics.push(SemanticDiagnostic {
                code: SemanticDiagnosticCode::PackSchemaMismatch,
                severity: DiagnosticSeverity::Error,
                semantic_truth: SemanticTruth::Invalid,
                message: format!("Pack self-validation failed: {:?}", e),
                source_ref: SourceRef::synthetic("internal"),
                pack_ref: pack_ref.clone(),
                suggestions: vec![],
                recoverability_hint: "Check pack structure and retry".to_string(),
            });
            return result;
        }
    };

    // Sort diagnostics deterministically
    super::canonical_json::sort_diagnostics(&mut diagnostics);

    // Compute status per §4.7
    let status = compute_status(&diagnostics, options);

    SemanticValidationResult {
        status,
        diagnostics,
        pack_set_ref: PackSetRef {
            merged_pack_hash: pack_ref.pack_content_hash.clone(),
            pack_refs: vec![pack_ref.clone()],
        },
        input_hash: String::new(),
        validation_mode: options.mode,
        expected_hashes: vec![],
        unsigned_fixture_bypass_used: options.allow_unsigned_test_fixtures,
        first_approved_version_bypass_used: false,
        evidence_ref: None,
    }
}

/// Validate a specific term against a pack (resolver-backed).
pub fn validate_term(
    raw_text: &str,
    pack: &SemanticPack,
    options: &ValidationOptions,
    source_ref: SourceRef,
) -> Option<SemanticDiagnostic> {
    let request = ResolveRequest {
        raw_text,
        expected_kind: None,
        source_ref: source_ref.clone(),
    };
    let result = resolver::resolve_concept(&request, pack, options);

    if let Some(code) = result.diagnostic_code {
        let pack_ref = make_pack_ref(pack);
        Some(SemanticDiagnostic {
            code,
            severity: result.diagnostic_severity,
            semantic_truth: result.semantic_truth,
            message: result.message,
            source_ref,
            pack_ref,
            suggestions: result.suggestions,
            recoverability_hint: String::new(),
        })
    } else {
        None
    }
}

fn compute_status(
    diagnostics: &[SemanticDiagnostic],
    options: &ValidationOptions,
) -> SemanticValidationStatus {
    if matches!(options.mode, ValidationMode::Off) {
        return SemanticValidationStatus::Passed;
    }

    let has_blocking_error = diagnostics
        .iter()
        .any(|d| d.severity == DiagnosticSeverity::Error);
    let has_unknown = diagnostics
        .iter()
        .any(|d| d.semantic_truth == SemanticTruth::Unknown);

    if has_blocking_error || (matches!(options.mode, ValidationMode::Strict) && has_unknown) {
        SemanticValidationStatus::Failed
    } else if has_unknown {
        SemanticValidationStatus::Unknown
    } else {
        SemanticValidationStatus::Passed
    }
}

fn make_pack_ref(pack: &SemanticPack) -> PackRef {
    PackRef {
        pack_id: pack.pack_id.clone(),
        pack_content_hash: canonical_json::compute_pack_content_hash(pack),
        path_or_uri: format!("pack://{}", pack.pack_id),
        priority: 0,
    }
}

fn empty_result(pack_ref: &PackRef, options: &ValidationOptions) -> SemanticValidationResult {
    SemanticValidationResult {
        status: SemanticValidationStatus::Passed,
        diagnostics: vec![],
        pack_set_ref: PackSetRef {
            merged_pack_hash: pack_ref.pack_content_hash.clone(),
            pack_refs: vec![pack_ref.clone()],
        },
        input_hash: String::new(),
        validation_mode: options.mode,
        expected_hashes: vec![],
        unsigned_fixture_bypass_used: false,
        first_approved_version_bypass_used: false,
        evidence_ref: None,
    }
}

#[derive(Debug, Clone)]
pub enum SemanticPackError {
    IoError(String),
    JsonError(String),
    ValidationError(String),
}
