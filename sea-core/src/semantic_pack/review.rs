use serde::{Deserialize, Serialize};

use super::schema::ReviewRecord;

/// Parse a JSONL review manifest into review records.
pub fn parse_review_manifest(jsonl: &str) -> Result<Vec<ReviewRecord>, ReviewError> {
    let mut records = Vec::new();
    for (i, line) in jsonl.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let record: ReviewRecord =
            serde_json::from_str(line).map_err(|e| ReviewError::ParseError {
                line: i + 1,
                message: e.to_string(),
            })?;
        records.push(record);
    }
    Ok(records)
}

/// Validate that all active concepts in the pack have matching review decisions.
pub fn validate_review_coverage(
    concept_ids: &[String],
    records: &[ReviewRecord],
) -> Vec<ReviewGap> {
    let reviewed: std::collections::HashSet<&str> = records
        .iter()
        .filter(|r| {
            matches!(
                r.decision,
                super::schema::ReviewDecision::Approve
                    | super::schema::ReviewDecision::MinorAmendmentNoSemanticChange
            )
        })
        .map(|r| r.subject_id.as_str())
        .collect();

    let mut gaps = Vec::new();
    for id in concept_ids {
        if !reviewed.contains(id.as_str()) {
            gaps.push(ReviewGap {
                subject_id: id.clone(),
                reason: "no_matching_approve_decision".to_string(),
            });
        }
    }
    gaps
}

/// Validate definition hashes match review manifest.
pub fn validate_definition_hashes(
    concept_id: &str,
    current_hash: &str,
    records: &[ReviewRecord],
) -> DefinitionHashResult {
    let matching: Vec<&ReviewRecord> = records
        .iter()
        .filter(|r| r.subject_id == concept_id)
        .collect();

    if matching.is_empty() {
        return DefinitionHashResult::NoReview;
    }

    let latest = matching.last().unwrap();
    if latest.definition_hash == current_hash {
        return DefinitionHashResult::Match;
    }

    // Check for minor amendment
    if matches!(
        latest.decision,
        super::schema::ReviewDecision::MinorAmendmentNoSemanticChange
    ) {
        if let (Some(prev), Some(new)) =
            (&latest.previous_definition_hash, &latest.new_definition_hash)
        {
            if prev == &latest.definition_hash && new == current_hash {
                return DefinitionHashResult::MinorAmendment;
            }
        }
    }

    DefinitionHashResult::Mismatch {
        reviewed_hash: latest.definition_hash.clone(),
        current_hash: current_hash.to_string(),
    }
}

/// Compute hash of the review manifest for deterministic tracking.
pub fn compute_review_manifest_hash(records: &[ReviewRecord]) -> String {
    let mut sorted = records.to_vec();
    sorted.sort_by(|a, b| a.decision_id.cmp(&b.decision_id));
    let json = serde_json::to_string(&sorted).expect("failed to serialize review records for manifest hash");
    super::canonical_json::compute_sha256(json.as_bytes())
}

#[derive(Debug, Clone)]
pub struct ReviewGap {
    pub subject_id: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub enum DefinitionHashResult {
    Match,
    MinorAmendment,
    NoReview,
    Mismatch { reviewed_hash: String, current_hash: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewError {
    ParseError { line: usize, message: String },
    IoError { message: String },
}
