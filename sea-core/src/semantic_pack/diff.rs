use serde::{Deserialize, Serialize};

use super::schema::{ConceptDef, SemanticPack};

/// Diff classification (12.4)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiffClassification {
    Additive,
    DefinitionalChange,
    Deprecating,
    Breaking,
    GovernanceCritical,
    SignatureOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEntry {
    pub classification: DiffClassification,
    pub subject_type: String,
    pub subject_id: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackDiff {
    pub old_pack_id: String,
    pub new_pack_id: String,
    pub entries: Vec<DiffEntry>,
    pub summary: DiffSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub additive_count: usize,
    pub definitional_change_count: usize,
    pub deprecating_count: usize,
    pub breaking_count: usize,
    pub governance_critical_count: usize,
    pub signature_only_count: usize,
}

/// Compute the diff between two semantic packs.
pub fn diff_packs(old: &SemanticPack, new: &SemanticPack) -> PackDiff {
    let mut entries = Vec::new();

    let old_concepts: std::collections::HashMap<&str, &ConceptDef> =
        old.concepts.iter().map(|c| (c.id.as_str(), c)).collect();
    let new_concepts: std::collections::HashMap<&str, &ConceptDef> =
        new.concepts.iter().map(|c| (c.id.as_str(), c)).collect();

    // Added concepts
    for (id, concept) in &new_concepts {
        if !old_concepts.contains_key(id) {
            entries.push(DiffEntry {
                classification: DiffClassification::Additive,
                subject_type: "concept".to_string(),
                subject_id: id.to_string(),
                detail: format!(
                    "Added concept '{}' ({})",
                    concept.canonical_name,
                    concept.status.as_str()
                ),
            });
        }
    }

    // Removed concepts (breaking if active)
    for (id, concept) in &old_concepts {
        if !new_concepts.contains_key(id) {
            let is_breaking = matches!(concept.status, super::schema::ConceptStatus::Active);
            entries.push(DiffEntry {
                classification: if is_breaking {
                    DiffClassification::Breaking
                } else {
                    DiffClassification::Deprecating
                },
                subject_type: "concept".to_string(),
                subject_id: id.to_string(),
                detail: format!(
                    "Removed concept '{}' (was {})",
                    concept.canonical_name,
                    concept.status.as_str()
                ),
            });
        }
    }

    // Changed concepts
    for (id, new_concept) in &new_concepts {
        if let Some(old_concept) = old_concepts.get(id) {
            // Definition hash change
            if old_concept.definition.definition_hash != new_concept.definition.definition_hash {
                let is_breaking = old.meaning_fingerprint != new.meaning_fingerprint
                    && !version_increased(&old.meaning_version, &new.meaning_version);
                entries.push(DiffEntry {
                    classification: if is_breaking {
                        DiffClassification::Breaking
                    } else {
                        DiffClassification::DefinitionalChange
                    },
                    subject_type: "concept".to_string(),
                    subject_id: id.to_string(),
                    detail: format!(
                        "Definition changed for '{}' (hash {} -> {})",
                        id,
                        old_concept.definition.definition_hash,
                        new_concept.definition.definition_hash
                    ),
                });
            }

            // Status transition
            if old_concept.status != new_concept.status {
                let classification = match new_concept.status {
                    super::schema::ConceptStatus::Deprecated => DiffClassification::Deprecating,
                    super::schema::ConceptStatus::Rejected => DiffClassification::Breaking,
                    _ => DiffClassification::GovernanceCritical,
                };
                entries.push(DiffEntry {
                    classification,
                    subject_type: "concept".to_string(),
                    subject_id: id.to_string(),
                    detail: format!(
                        "Status changed: {} -> {}",
                        old_concept.status.as_str(),
                        new_concept.status.as_str()
                    ),
                });
            }
        }
    }

    // Meaning fingerprint change without version bump
    if old.meaning_fingerprint != new.meaning_fingerprint
        && !version_increased(&old.meaning_version, &new.meaning_version)
    {
        entries.push(DiffEntry {
            classification: DiffClassification::Breaking,
            subject_type: "pack".to_string(),
            subject_id: "meaning_fingerprint".to_string(),
            detail: format!(
                "meaning_fingerprint changed without meaning_version increase ({} vs {})",
                old.meaning_version, new.meaning_version
            ),
        });
    }

    // Sort entries by classification priority then subject_id
    entries.sort_by(|a, b| {
        classification_order(a.classification)
            .cmp(&classification_order(b.classification))
            .then(a.subject_id.cmp(&b.subject_id))
    });

    let summary = DiffSummary {
        additive_count: entries
            .iter()
            .filter(|e| e.classification == DiffClassification::Additive)
            .count(),
        definitional_change_count: entries
            .iter()
            .filter(|e| e.classification == DiffClassification::DefinitionalChange)
            .count(),
        deprecating_count: entries
            .iter()
            .filter(|e| e.classification == DiffClassification::Deprecating)
            .count(),
        breaking_count: entries
            .iter()
            .filter(|e| e.classification == DiffClassification::Breaking)
            .count(),
        governance_critical_count: entries
            .iter()
            .filter(|e| e.classification == DiffClassification::GovernanceCritical)
            .count(),
        signature_only_count: entries
            .iter()
            .filter(|e| e.classification == DiffClassification::SignatureOnly)
            .count(),
    };

    PackDiff {
        old_pack_id: old.pack_id.clone(),
        new_pack_id: new.pack_id.clone(),
        entries,
        summary,
    }
}

fn classification_order(c: DiffClassification) -> u8 {
    match c {
        DiffClassification::Breaking => 0,
        DiffClassification::GovernanceCritical => 1,
        DiffClassification::DefinitionalChange => 2,
        DiffClassification::Deprecating => 3,
        DiffClassification::Additive => 4,
        DiffClassification::SignatureOnly => 5,
    }
}

fn version_increased(old: &str, new: &str) -> bool {
    super::builder::version_increased(old, new).unwrap_or(false)
}

impl super::schema::ConceptStatus {
    fn as_str(&self) -> &'static str {
        match self {
            super::schema::ConceptStatus::Active => "active",
            super::schema::ConceptStatus::Proposed => "proposed",
            super::schema::ConceptStatus::Deprecated => "deprecated",
            super::schema::ConceptStatus::Rejected => "rejected",
            super::schema::ConceptStatus::ExternalOnly => "external_only",
        }
    }
}
