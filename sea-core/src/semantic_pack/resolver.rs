use super::diagnostics::{
    DiagnosticSeverity, SemanticDiagnosticCode, SemanticTruth, ValidationMode, ValidationOptions,
};
use super::schema::{AliasDef, AliasStatus, ConceptKind, ConceptStatus, SemanticPack, SourceRef};

use unicode_normalization::UnicodeNormalization;

// ---------------------------------------------------------------------------
// 8.1 Normalization
// ---------------------------------------------------------------------------
pub fn normalize_lookup_key(s: &str) -> String {
    let nfc: String = s.nfc().collect();
    let trimmed = nfc.trim();
    let collapsed: String = trimmed
        .split(|c: char| c.is_whitespace())
        .filter(|part| !part.is_empty())
        .collect::<Vec<&str>>()
        .join(" ");
    collapsed.to_lowercase()
}

/// Validate canonical ID format: [A-Za-z0-9._:/-]+
pub fn is_valid_canonical_id(id: &str) -> bool {
    !id.is_empty()
        && id.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == ':' || c == '/' || c == '-'
        })
}

// ---------------------------------------------------------------------------
// 8.2 ResolveRequest / ResolveResult
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct ResolveRequest<'a> {
    pub raw_text: &'a str,
    pub expected_kind: Option<ConceptKind>,
    pub source_ref: SourceRef,
}

#[derive(Debug, Clone)]
pub struct ResolveResult {
    pub resolved_concept_id: Option<String>,
    pub semantic_truth: SemanticTruth,
    pub diagnostic_code: Option<SemanticDiagnosticCode>,
    pub diagnostic_severity: DiagnosticSeverity,
    pub message: String,
    pub suggestions: Vec<super::schema::Suggestion>,
}

impl ResolveResult {
    pub fn valid(concept_id: String) -> Self {
        Self {
            resolved_concept_id: Some(concept_id),
            semantic_truth: SemanticTruth::Valid,
            diagnostic_code: None,
            diagnostic_severity: DiagnosticSeverity::Info,
            message: String::new(),
            suggestions: vec![],
        }
    }
}

// ---------------------------------------------------------------------------
// 8.3 Resolver algorithm — full candidate set
// ---------------------------------------------------------------------------
pub fn resolve_concept<'a>(
    request: &ResolveRequest<'a>,
    pack: &SemanticPack,
    options: &ValidationOptions,
) -> ResolveResult {
    let key = normalize_lookup_key(request.raw_text);

    // Step 3: exact concept ID matches
    let mut candidates: Vec<Candidate> = Vec::new();

    for c in &pack.concepts {
        if c.id == key {
            candidates.push(Candidate {
                concept_id: c.id.clone(),
                status: c.status,
                kind: c.kind,
                _match_type: MatchType::ExactId,
            });
        }
    }

    // Step 4: canonical name matches
    for c in &pack.concepts {
        if normalize_lookup_key(&c.canonical_name) == key {
            if !candidates.iter().any(|cand| cand.concept_id == c.id) {
                candidates.push(Candidate {
                    concept_id: c.id.clone(),
                    status: c.status,
                    kind: c.kind,
                    _match_type: MatchType::CanonicalName,
                });
            }
        }
    }

    // Step 5: alias matches — collect ALL aliases for this normalized key
    let alias_matches: Vec<&AliasDef> = pack
        .aliases
        .iter()
        .filter(|a| a.normalized_alias == key)
        .collect();

    for alias in &alias_matches {
        if let Some(concept) = pack
            .concepts
            .iter()
            .find(|c| c.id == alias.target_concept_id)
        {
            if !candidates.iter().any(|cand| cand.concept_id == concept.id) {
                candidates.push(Candidate {
                    concept_id: concept.id.clone(),
                    status: concept.status,
                    kind: concept.kind,
                    _match_type: MatchType::Alias,
                });
            }
        }
    }

    // Step 6: mapping rule matches
    for rule in &pack.mapping_rules {
        if normalize_lookup_key(&rule.source_term) == key {
            if let Some(concept) = pack
                .concepts
                .iter()
                .find(|c| c.id == rule.target_concept_id)
            {
                if !candidates.iter().any(|cand| cand.concept_id == concept.id) {
                    candidates.push(Candidate {
                        concept_id: concept.id.clone(),
                        status: concept.status,
                        kind: concept.kind,
                        _match_type: MatchType::MappingRule,
                    });
                }
            }
        }
    }

    // Step 7: filter by expected kind
    let mut kind_excluded = Vec::new();
    if let Some(expected) = request.expected_kind {
        for cand in &candidates {
            if cand.kind != expected {
                kind_excluded.push(cand.clone());
            }
        }
        candidates.retain(|c| c.kind == expected);
    }

    // Step 8: no candidates
    if candidates.is_empty() {
        let suggestions = kind_excluded
            .iter()
            .map(|c| super::schema::Suggestion {
                label: c.concept_id.clone(),
                rank: 10,
            })
            .collect();
        return ResolveResult {
            resolved_concept_id: None,
            semantic_truth: SemanticTruth::Unknown,
            diagnostic_code: Some(SemanticDiagnosticCode::UnknownConcept),
            diagnostic_severity: unknown_severity(options),
            message: format!("Unknown concept: '{}'", request.raw_text),
            suggestions,
        };
    }

    // Step 9: rejected/blocked
    for cand in &candidates {
        if cand.status == ConceptStatus::Rejected {
            return ResolveResult {
                resolved_concept_id: None,
                semantic_truth: SemanticTruth::Invalid,
                diagnostic_code: Some(SemanticDiagnosticCode::RejectedConcept),
                diagnostic_severity: DiagnosticSeverity::Error,
                message: format!("Rejected concept: '{}'", request.raw_text),
                suggestions: vec![],
            };
        }
    }
    for alias in &alias_matches {
        if alias.status == AliasStatus::Blocked {
            return ResolveResult {
                resolved_concept_id: None,
                semantic_truth: SemanticTruth::Invalid,
                diagnostic_code: Some(SemanticDiagnosticCode::AliasConflict),
                diagnostic_severity: DiagnosticSeverity::Error,
                message: format!("Blocked alias: '{}'", request.raw_text),
                suggestions: vec![],
            };
        }
    }

    // Step 10: group by target concept
    let mut groups: std::collections::HashMap<String, Vec<&Candidate>> =
        std::collections::HashMap::new();
    for cand in &candidates {
        groups
            .entry(cand.concept_id.clone())
            .or_default()
            .push(cand);
    }

    if groups.len() > 1 {
        // Check if this is an explicit ambiguous-only group
        let non_blocked_aliases: Vec<&AliasDef> = alias_matches
            .iter()
            .filter(|a| a.status != AliasStatus::Blocked)
            .copied()
            .collect();

        let all_ambiguous = non_blocked_aliases
            .iter()
            .all(|a| a.status == AliasStatus::Ambiguous);
        if all_ambiguous && !non_blocked_aliases.is_empty() {
            return ResolveResult {
                resolved_concept_id: None,
                semantic_truth: SemanticTruth::Unknown,
                diagnostic_code: Some(SemanticDiagnosticCode::AmbiguousAlias),
                diagnostic_severity: ambiguous_severity(options),
                message: format!(
                    "Ambiguous alias '{}' targets multiple concepts: {}",
                    request.raw_text,
                    groups.keys().cloned().collect::<Vec<_>>().join(", ")
                ),
                suggestions: groups
                    .keys()
                    .map(|k| super::schema::Suggestion {
                        label: k.clone(),
                        rank: 5,
                    })
                    .collect(),
            };
        }

        return ResolveResult {
            resolved_concept_id: None,
            semantic_truth: SemanticTruth::Unknown,
            diagnostic_code: Some(SemanticDiagnosticCode::AmbiguousConcept),
            diagnostic_severity: ambiguous_severity(options),
            message: format!(
                "Ambiguous concept '{}' resolves to: {}",
                request.raw_text,
                groups.keys().cloned().collect::<Vec<_>>().join(", ")
            ),
            suggestions: groups
                .keys()
                .map(|k| super::schema::Suggestion {
                    label: k.clone(),
                    rank: 5,
                })
                .collect(),
        };
    }

    // Step 12: single winner
    let winner = candidates.iter().next().unwrap();

    // Step 13: proposed
    if winner.status == ConceptStatus::Proposed {
        return ResolveResult {
            resolved_concept_id: Some(winner.concept_id.clone()),
            semantic_truth: SemanticTruth::Unknown,
            diagnostic_code: Some(SemanticDiagnosticCode::ProposedConcept),
            diagnostic_severity: unknown_severity(options),
            message: format!("Proposed concept: '{}'", request.raw_text),
            suggestions: vec![],
        };
    }

    // Step 14: deprecated
    if winner.status == ConceptStatus::Deprecated
        || alias_matches
            .iter()
            .any(|a| a.status == AliasStatus::Deprecated)
    {
        let severity = deprecated_severity(options);
        return ResolveResult {
            resolved_concept_id: Some(winner.concept_id.clone()),
            semantic_truth: SemanticTruth::Valid,
            diagnostic_code: Some(SemanticDiagnosticCode::DeprecatedConcept),
            diagnostic_severity: severity,
            message: format!("Deprecated concept: '{}'", request.raw_text),
            suggestions: vec![],
        };
    }

    // Step 15: active and valid
    ResolveResult::valid(winner.concept_id.clone())
}

// ---------------------------------------------------------------------------
// Alias conflict detection for builder (§5.1)
// ---------------------------------------------------------------------------
pub struct AliasConflictReport {
    pub has_conflict: bool,
    pub conflicting_keys: Vec<String>,
    pub ambiguous_only_keys: Vec<String>,
}

pub fn detect_alias_conflicts(aliases: &[AliasDef]) -> AliasConflictReport {
    let mut groups: std::collections::HashMap<String, Vec<&AliasDef>> =
        std::collections::HashMap::new();
    for a in aliases {
        groups
            .entry(a.normalized_alias.clone())
            .or_default()
            .push(a);
    }

    let mut conflicting = Vec::new();
    let mut ambiguous_only = Vec::new();

    for (key, entries) in &groups {
        let non_blocked: Vec<&AliasDef> = entries
            .iter()
            .filter(|a| a.status != AliasStatus::Blocked)
            .copied()
            .collect();
        if non_blocked.len() < 2 {
            continue;
        }

        let distinct_targets: std::collections::HashSet<&str> = non_blocked
            .iter()
            .map(|a| a.target_concept_id.as_str())
            .collect();
        if distinct_targets.len() < 2 {
            continue;
        }

        // Check if all non-blocked are ambiguous
        let all_ambiguous = non_blocked
            .iter()
            .all(|a| a.status == AliasStatus::Ambiguous);
        if all_ambiguous {
            ambiguous_only.push(key.clone());
        } else {
            conflicting.push(key.clone());
        }
    }

    AliasConflictReport {
        has_conflict: !conflicting.is_empty(),
        conflicting_keys: conflicting,
        ambiguous_only_keys: ambiguous_only,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
fn unknown_severity(options: &ValidationOptions) -> DiagnosticSeverity {
    match options.mode {
        ValidationMode::Strict => match options.unknown_concept_policy {
            super::diagnostics::UnknownConceptPolicy::Error => DiagnosticSeverity::Error,
            _ => DiagnosticSeverity::Warning,
        },
        _ => match options.unknown_concept_policy {
            super::diagnostics::UnknownConceptPolicy::Error => DiagnosticSeverity::Error,
            super::diagnostics::UnknownConceptPolicy::Warning => DiagnosticSeverity::Warning,
            _ => DiagnosticSeverity::Hint,
        },
    }
}

fn ambiguous_severity(options: &ValidationOptions) -> DiagnosticSeverity {
    match options.mode {
        ValidationMode::Strict => DiagnosticSeverity::Error,
        _ => DiagnosticSeverity::Warning,
    }
}

fn deprecated_severity(options: &ValidationOptions) -> DiagnosticSeverity {
    match options.deprecated_policy {
        super::diagnostics::DeprecatedPolicy::Allow => DiagnosticSeverity::Hint,
        super::diagnostics::DeprecatedPolicy::Warn => DiagnosticSeverity::Warning,
        super::diagnostics::DeprecatedPolicy::ErrorInStrict => match options.mode {
            ValidationMode::Strict => DiagnosticSeverity::Error,
            _ => DiagnosticSeverity::Warning,
        },
        super::diagnostics::DeprecatedPolicy::ErrorAlways => DiagnosticSeverity::Error,
    }
}

#[derive(Debug, Clone)]
struct Candidate {
    concept_id: String,
    status: ConceptStatus,
    kind: ConceptKind,
    _match_type: MatchType,
}

#[derive(Debug, Clone, Copy)]
enum MatchType {
    ExactId,
    CanonicalName,
    Alias,
    MappingRule,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_basic() {
        assert_eq!(normalize_lookup_key("  Hello   World  "), "hello world");
    }

    #[test]
    fn normalize_case_fold() {
        assert_eq!(normalize_lookup_key("Vendor"), "vendor");
    }

    #[test]
    fn normalize_unicode_nfc() {
        // e + combining accent -> precomposed é
        let input = "e\u{0301}cole";
        assert_eq!(normalize_lookup_key(input), "école");
    }

    #[test]
    fn valid_canonical_id() {
        assert!(is_valid_canonical_id("supplier"));
        assert!(is_valid_canonical_id("org.acme/supplier-v2"));
        assert!(!is_valid_canonical_id(""));
        assert!(!is_valid_canonical_id("has spaces"));
    }
}
