use serde::Deserialize;

use super::canonical_json;
use super::diagnostics::{
    DiagnosticSeverity, SemanticDiagnostic, SemanticDiagnosticCode, SemanticTruth,
};
use super::review::{self, DefinitionHashResult};
use super::schema::{
    AliasDef, ApprovalState, CompatibilityInfo, ConceptDef, ConceptStatus, GeneratorInfo, PackRef,
    PackTrust, ReviewRecord, SemanticPack, SignatureState, SourceRef,
};

/// Input for building a semantic pack.
#[derive(Debug, Clone, Deserialize)]
pub struct PackBuildInput {
    pub org_id: String,
    pub domain_id: String,
    pub pack_version: String,
    pub meaning_version: String,
    pub approval: ApprovalState,
    pub concepts: Vec<ConceptDef>,
    pub relations: Vec<super::schema::RelationDef>,
    pub metrics: Vec<super::schema::MetricDef>,
    pub dimensions: Vec<super::schema::DimensionDef>,
    pub units: Vec<super::schema::UnitDef>,
    pub aliases: Vec<AliasDef>,
    pub mapping_rules: Vec<super::schema::MappingRuleDef>,
    pub review_records: Vec<ReviewRecord>,
    pub previous_pack: Option<SemanticPack>,
    pub allow_first_approved_version: bool,
    pub source_graph_hash: String,
}

/// Output from building a semantic pack.
#[derive(Debug, Clone)]
pub struct PackBuildOutput {
    pub pack: SemanticPack,
    pub pack_content_hash: String,
    pub meaning_fingerprint: String,
    pub pre_pack_diagnostics: Vec<SemanticDiagnostic>,
    pub build_warnings: Vec<SemanticDiagnostic>,
}

/// Build a semantic pack from input.
pub fn build_semantic_pack(
    input: PackBuildInput,
) -> Result<PackBuildOutput, Vec<SemanticDiagnostic>> {
    let mut pre_pack_diagnostics = Vec::new();

    // Run pre-pack checks (§5.1)
    let mut build_warnings = Vec::new();
    run_pre_pack_checks(&input, &mut pre_pack_diagnostics, &mut build_warnings);

    // Check review coverage for approved builds
    if matches!(input.approval, ApprovalState::Approved) {
        check_review_coverage(&input, &mut pre_pack_diagnostics, &mut build_warnings);
    }

    // Build the pack
    let pack_id = format!(
        "{}/{}/{}",
        input.org_id, input.domain_id, input.pack_version
    );
    let generator = GeneratorInfo {
        name: "sea-core".to_string(),
        version: "0.3".to_string(),
    };

    // Compute review manifest hash
    let review_manifest_hash = review::compute_review_manifest_hash(&input.review_records);

    // Compute definition hashes for concepts
    let mut concepts = input.concepts;
    for c in &mut concepts {
        if c.definition.definition_hash.is_empty() {
            c.definition.definition_hash = canonical_json::compute_definition_hash(
                &c.definition.text,
                &c.examples,
                &c.counterexamples,
                &serde_json::to_value(&c.status)
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_default(),
            );
        }
    }

    // Normalize aliases
    let mut aliases = input.aliases;
    for a in &mut aliases {
        a.normalized_alias = super::resolver::normalize_lookup_key(&a.alias);
    }

    let mut pack = SemanticPack {
        schema_version: "0.3".to_string(),
        pack_id: pack_id.clone(),
        org_id: input.org_id,
        domain_id: input.domain_id,
        pack_version: input.pack_version,
        meaning_version: input.meaning_version,
        meaning_fingerprint: String::new(), // computed below
        source_graph_hash: input.source_graph_hash,
        build_config_hash: canonical_json::compute_sha256(b"{}"),
        review_manifest_hash,
        created_at: chrono::Utc::now().to_rfc3339(),
        generator,
        trust: PackTrust {
            approval_state: input.approval,
            signature_state: SignatureState::Unsigned,
            signed_by: None,
            signature_alg: None,
            signature: None,
        },
        concepts,
        relations: input.relations,
        metrics: input.metrics,
        dimensions: input.dimensions,
        units: input.units,
        aliases,
        mapping_rules: input.mapping_rules,
        compatibility: CompatibilityInfo::default(),
    };

    // Compute meaning fingerprint
    pack.meaning_fingerprint = canonical_json::compute_meaning_fingerprint(&pack);

    // Sort for canonicalization
    canonical_json::sort_pack_for_canonicalization(&mut pack);

    // Compute content hash
    let pack_content_hash = canonical_json::compute_pack_content_hash(&pack);

    // Check meaning version (§4.1)
    if let Some(ref prev) = input.previous_pack {
        if prev.meaning_fingerprint != pack.meaning_fingerprint {
            // Meaning changed — version must increase
            if !version_increased(&prev.meaning_version, &pack.meaning_version) {
                pre_pack_diagnostics.push(SemanticDiagnostic {
                    code: SemanticDiagnosticCode::MeaningVersionNotBumped,
                    severity: DiagnosticSeverity::Error,
                    semantic_truth: SemanticTruth::Invalid,
                    message: format!(
                        "meaning_fingerprint changed but meaning_version '{}' did not increase from '{}'",
                        pack.meaning_version, prev.meaning_version
                    ),
                    source_ref: SourceRef::pack_uri(&pack_id),
                    pack_ref: PackRef {
                        pack_id: pack_id.clone(),
                        pack_content_hash: pack_content_hash.clone(),
                        path_or_uri: format!("pack://{}", pack_id),
                        priority: 0,
                    },
                    suggestions: vec![],
                    recoverability_hint: "Bump meaning_version (MAJOR.MINOR.PATCH)".to_string(),
                });
            }
        }
    } else if matches!(input.approval, ApprovalState::Approved)
        && !input.allow_first_approved_version
    {
        pre_pack_diagnostics.push(SemanticDiagnostic {
            code: SemanticDiagnosticCode::MeaningVersionBaselineMissing,
            severity: DiagnosticSeverity::Error,
            semantic_truth: SemanticTruth::Unknown,
            message: "No previous pack provided for approved build; use --allow-first-approved-version for initial pack".to_string(),
            source_ref: SourceRef::pack_uri(&pack_id),
            pack_ref: PackRef {
                pack_id: pack_id.clone(),
                pack_content_hash: pack_content_hash.clone(),
                path_or_uri: format!("pack://{}", pack_id),
                priority: 0,
            },
            suggestions: vec![],
            recoverability_hint: "Provide --previous-pack or --allow-first-approved-version".to_string(),
        });
    }

    // Block approved builds on pre-pack errors
    if matches!(input.approval, ApprovalState::Approved) {
        let has_errors = pre_pack_diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error);
        if has_errors {
            return Err(pre_pack_diagnostics);
        }
    }

    let meaning_fingerprint = pack.meaning_fingerprint.clone();

    Ok(PackBuildOutput {
        pack,
        pack_content_hash,
        meaning_fingerprint,
        pre_pack_diagnostics,
        build_warnings,
    })
}

fn run_pre_pack_checks(
    input: &PackBuildInput,
    diagnostics: &mut Vec<SemanticDiagnostic>,
    build_warnings: &mut Vec<SemanticDiagnostic>,
) {
    let dummy_ref = PackRef {
        pack_id: format!(
            "{}/{}/{}",
            input.org_id, input.domain_id, input.pack_version
        ),
        pack_content_hash: String::new(),
        path_or_uri: String::new(),
        priority: 0,
    };

    // Duplicate concept IDs
    let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for c in &input.concepts {
        if !seen_ids.insert(c.id.clone()) {
            diagnostics.push(SemanticDiagnostic {
                code: SemanticDiagnosticCode::DuplicateConceptId,
                severity: DiagnosticSeverity::Error,
                semantic_truth: SemanticTruth::Invalid,
                message: format!("Duplicate concept ID: '{}'", c.id),
                source_ref: SourceRef::pack_uri(&dummy_ref.pack_id),
                pack_ref: dummy_ref.clone(),
                suggestions: vec![],
                recoverability_hint: "Rename duplicate concepts".to_string(),
            });
        }
    }

    // Duplicate canonical names
    let mut seen_names: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for c in &input.concepts {
        let norm = super::resolver::normalize_lookup_key(&c.canonical_name);
        if let Some(existing) = seen_names.get(&norm) {
            if existing != &c.id {
                diagnostics.push(SemanticDiagnostic {
                    code: SemanticDiagnosticCode::DuplicateCanonicalName,
                    severity: DiagnosticSeverity::Error,
                    semantic_truth: SemanticTruth::Invalid,
                    message: format!(
                        "Duplicate canonical name '{}' for concepts '{}' and '{}'",
                        c.canonical_name, existing, c.id
                    ),
                    source_ref: SourceRef::pack_uri(&dummy_ref.pack_id),
                    pack_ref: dummy_ref.clone(),
                    suggestions: vec![],
                    recoverability_hint: "Ensure canonical names are unique".to_string(),
                });
            }
        } else {
            seen_names.insert(norm, c.id.clone());
        }
    }

    // Active concept without definition
    for c in &input.concepts {
        if matches!(c.status, ConceptStatus::Active) && c.definition.text.is_empty() {
            diagnostics.push(SemanticDiagnostic {
                code: SemanticDiagnosticCode::MissingDefinition,
                severity: DiagnosticSeverity::Warning,
                semantic_truth: SemanticTruth::Unknown,
                message: format!("Active concept '{}' missing definition", c.id),
                source_ref: SourceRef::pack_uri(&dummy_ref.pack_id),
                pack_ref: dummy_ref.clone(),
                suggestions: vec![],
                recoverability_hint: "Add definition text".to_string(),
            });
        }
    }

    // Active concept without owner
    for c in &input.concepts {
        if matches!(c.status, ConceptStatus::Active) && c.owner.is_empty() {
            diagnostics.push(SemanticDiagnostic {
                code: SemanticDiagnosticCode::MissingOwner,
                severity: DiagnosticSeverity::Warning,
                semantic_truth: SemanticTruth::Unknown,
                message: format!("Active concept '{}' missing owner", c.id),
                source_ref: SourceRef::pack_uri(&dummy_ref.pack_id),
                pack_ref: dummy_ref.clone(),
                suggestions: vec![],
                recoverability_hint: "Assign an owner".to_string(),
            });
        }
    }

    // Alias target missing
    let concept_ids: std::collections::HashSet<&str> =
        input.concepts.iter().map(|c| c.id.as_str()).collect();
    for a in &input.aliases {
        if !concept_ids.contains(a.target_concept_id.as_str()) {
            diagnostics.push(SemanticDiagnostic {
                code: SemanticDiagnosticCode::UnknownConcept,
                severity: DiagnosticSeverity::Error,
                semantic_truth: SemanticTruth::Invalid,
                message: format!(
                    "Alias '{}' targets missing concept '{}'",
                    a.alias, a.target_concept_id
                ),
                source_ref: SourceRef::pack_uri(&dummy_ref.pack_id),
                pack_ref: dummy_ref.clone(),
                suggestions: vec![],
                recoverability_hint: "Add the target concept".to_string(),
            });
        }
    }

    // Alias conflict detection (§5.1)
    let conflict_report = super::resolver::detect_alias_conflicts(&input.aliases);
    for key in &conflict_report.conflicting_keys {
        diagnostics.push(SemanticDiagnostic {
            code: SemanticDiagnosticCode::AliasConflict,
            severity: DiagnosticSeverity::Error,
            semantic_truth: SemanticTruth::Invalid,
            message: format!(
                "Alias conflict for key '{}': multiple targets through approved/deprecated entries",
                key
            ),
            source_ref: SourceRef::pack_uri(&dummy_ref.pack_id),
            pack_ref: dummy_ref.clone(),
            suggestions: vec![],
            recoverability_hint: "Resolve conflicting aliases".to_string(),
        });
    }
    for key in &conflict_report.ambiguous_only_keys {
        build_warnings_push(build_warnings, key, &dummy_ref);
    }
}

fn build_warnings_push(diagnostics: &mut Vec<SemanticDiagnostic>, key: &str, pack_ref: &PackRef) {
    diagnostics.push(SemanticDiagnostic {
        code: SemanticDiagnosticCode::AmbiguousAliasGroup,
        severity: DiagnosticSeverity::Warning,
        semantic_truth: SemanticTruth::Unknown,
        message: format!("Ambiguous-only alias group for key '{}'", key),
        source_ref: SourceRef::pack_uri(&pack_ref.pack_id),
        pack_ref: pack_ref.clone(),
        suggestions: vec![],
        recoverability_hint: String::new(),
    });
}

fn check_review_coverage(
    input: &PackBuildInput,
    diagnostics: &mut Vec<SemanticDiagnostic>,
    _build_warnings: &mut Vec<SemanticDiagnostic>,
) {
    let active_ids: Vec<String> = input
        .concepts
        .iter()
        .filter(|c| matches!(c.status, ConceptStatus::Active))
        .map(|c| c.id.clone())
        .collect();

    let gaps = review::validate_review_coverage(&active_ids, &input.review_records);
    for gap in gaps {
        diagnostics.push(SemanticDiagnostic {
            code: SemanticDiagnosticCode::UnreviewedConcept,
            severity: DiagnosticSeverity::Error,
            semantic_truth: SemanticTruth::Unknown,
            message: format!("Concept '{}' lacks review approval", gap.subject_id),
            source_ref: SourceRef::pack_uri(&format!(
                "{}/{}/{}",
                input.org_id, input.domain_id, input.pack_version
            )),
            pack_ref: PackRef {
                pack_id: format!(
                    "{}/{}/{}",
                    input.org_id, input.domain_id, input.pack_version
                ),
                pack_content_hash: String::new(),
                path_or_uri: String::new(),
                priority: 0,
            },
            suggestions: vec![],
            recoverability_hint: "Add review decision for this concept".to_string(),
        });
    }

    // Validate definition hashes match
    for c in &input.concepts {
        if c.definition.definition_hash.is_empty() {
            continue;
        }
        let hash_result = review::validate_definition_hashes(
            &c.id,
            &c.definition.definition_hash,
            &input.review_records,
        );
        match hash_result {
            DefinitionHashResult::Mismatch {
                reviewed_hash,
                current_hash,
            } => {
                diagnostics.push(SemanticDiagnostic {
                    code: SemanticDiagnosticCode::UnreviewedConcept,
                    severity: DiagnosticSeverity::Error,
                    semantic_truth: SemanticTruth::Unknown,
                    message: format!(
                        "Definition hash mismatch for '{}': reviewed='{}', current='{}'",
                        c.id, reviewed_hash, current_hash
                    ),
                    source_ref: SourceRef::pack_uri(&format!(
                        "{}/{}/{}",
                        input.org_id, input.domain_id, input.pack_version
                    )),
                    pack_ref: PackRef {
                        pack_id: format!(
                            "{}/{}/{}",
                            input.org_id, input.domain_id, input.pack_version
                        ),
                        pack_content_hash: String::new(),
                        path_or_uri: String::new(),
                        priority: 0,
                    },
                    suggestions: vec![],
                    recoverability_hint: "Re-review or record minor_amendment_no_semantic_change"
                        .to_string(),
                });
            }
            _ => {}
        }
    }
}

fn version_increased(old: &str, new: &str) -> bool {
    let parse = |s: &str| -> Vec<u32> { s.split('.').filter_map(|p| p.parse().ok()).collect() };
    let old_parts = parse(old);
    let new_parts = parse(new);
    for i in 0..std::cmp::max(old_parts.len(), new_parts.len()) {
        let o = old_parts.get(i).copied().unwrap_or(0);
        let n = new_parts.get(i).copied().unwrap_or(0);
        if n > o {
            return true;
        }
        if n < o {
            return false;
        }
    }
    false
}
