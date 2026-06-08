use sha2::{Digest, Sha256};

use super::schema::{AliasStatus, SemanticPack, SourceRef};

/// Canonical JSON: UTF-8, sorted keys, no whitespace, RFC3339 UTC timestamps.
/// Arrays follow the explicit ordering table (§6.1.1).
pub fn canonical_json(value: &serde_json::Value) -> String {
    canonicalize_value(value).to_string()
}

fn canonicalize_value(v: &serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let ordered: serde_json::Map<String, serde_json::Value> = keys
                .into_iter()
                .map(|k| (k.clone(), canonicalize_value(&map[k])))
                .collect();
            serde_json::Value::Object(ordered)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(canonicalize_value).collect())
        }
        other => other.clone(),
    }
}

/// Compute sha256 of canonical JSON bytes.
pub fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("sha256:{:x}", result)
}

/// Compute sha256 of canonical JSON string.
pub fn hash_canonical_json(value: &serde_json::Value) -> String {
    let canonical = canonical_json(value);
    compute_sha256(canonical.as_bytes())
}

/// Sort a pack's vectors for deterministic serialization (§6.1.1).
pub fn sort_pack_for_canonicalization(pack: &mut SemanticPack) {
    pack.concepts.sort_by(|a, b| a.id.cmp(&b.id));
    pack.relations
        .sort_by(|a, b| a.id.cmp(&b.id).then(a.predicate.cmp(&b.predicate)));
    pack.metrics.sort_by(|a, b| a.id.cmp(&b.id));
    pack.dimensions.sort_by(|a, b| a.id.cmp(&b.id));
    pack.units
        .sort_by(|a, b| a.id.cmp(&b.id).then(a.symbol.cmp(&b.symbol)));
    pack.aliases.sort_by(|a, b| {
        a.normalized_alias
            .cmp(&b.normalized_alias)
            .then(a.target_concept_id.cmp(&b.target_concept_id))
            .then(alias_status_order(&a.status).cmp(&alias_status_order(&b.status)))
    });
    pack.mapping_rules.sort_by(|a, b| a.id.cmp(&b.id));

    for concept in &mut pack.concepts {
        sort_string_vec(&mut concept.examples);
        sort_string_vec(&mut concept.counterexamples);
        sort_string_vec(&mut concept.allowed_predicates);
        sort_string_vec(&mut concept.valid_contexts);
        sort_source_refs(&mut concept.source_refs);
    }
    for relation in &mut pack.relations {
        sort_source_refs(&mut relation.source_refs);
    }
    for metric in &mut pack.metrics {
        sort_source_refs(&mut metric.source_refs);
    }
    for dim in &mut pack.dimensions {
        sort_source_refs(&mut dim.source_refs);
    }
    for unit in &mut pack.units {
        sort_source_refs(&mut unit.source_refs);
    }
}

/// Sort diagnostics for deterministic output.
pub fn sort_diagnostics(diags: &mut Vec<super::diagnostics::SemanticDiagnostic>) {
    diags.sort_by(|a, b| {
        a.source_ref
            .uri
            .cmp(&b.source_ref.uri)
            .then(a.source_ref.start_byte.cmp(&b.source_ref.start_byte))
            .then(a.code.as_str().cmp(b.code.as_str()))
            .then(a.message.cmp(&b.message))
    });
    for d in diags.iter_mut() {
        d.suggestions
            .sort_by(|a, b| b.rank.cmp(&a.rank).then(a.label.cmp(&b.label)));
    }
}

/// Compute pack content hash excluding signature fields (§6.2).
pub fn compute_pack_content_hash(pack: &SemanticPack) -> String {
    let mut pack_for_hash = pack.clone();

    // Clear non-semantic / build-variant and signature metadata fields for hash computation
    pack_for_hash.created_at = String::new();
    pack_for_hash.trust.signed_by = None;
    pack_for_hash.trust.signature_alg = None;
    pack_for_hash.trust.signature = None;
    pack_for_hash.trust.signature_state = super::schema::SignatureState::Unsigned;

    sort_pack_for_canonicalization(&mut pack_for_hash);

    let json = serde_json::to_value(&pack_for_hash)
        .expect("failed to serialize pack for hash computation");
    hash_canonical_json(&json)
}

/// Compute meaning fingerprint from definition records.
pub fn compute_meaning_fingerprint(pack: &SemanticPack) -> String {
    let mut records: Vec<serde_json::Value> = Vec::new();

    for c in &pack.concepts {
        records.push(serde_json::json!({
            "subject_type": "concept",
            "subject_id": c.id,
            "definition_hash": c.definition.definition_hash,
            "status": serde_json::to_value(&c.status).unwrap_or_default(),
            "decision_ref": c.definition.decision_ref,
        }));
    }
    records.sort_by(|a, b| {
        let at = a.get("subject_type").and_then(|v| v.as_str()).unwrap_or("");
        let bt = b.get("subject_type").and_then(|v| v.as_str()).unwrap_or("");
        at.cmp(bt).then(
            a.get("subject_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .cmp(b.get("subject_id").and_then(|v| v.as_str()).unwrap_or("")),
        )
    });

    hash_canonical_json(&serde_json::Value::Array(records))
}

/// Compute definition hash for a concept from its text, examples, counterexamples, and status.
pub fn compute_definition_hash(
    text: &str,
    examples: &[String],
    counterexamples: &[String],
    status: &str,
) -> String {
    let mut norm_examples: Vec<String> = examples.to_vec();
    sort_string_vec(&mut norm_examples);
    let mut norm_counter: Vec<String> = counterexamples.to_vec();
    sort_string_vec(&mut norm_counter);

    let input = serde_json::json!({
        "text": text,
        "examples": norm_examples,
        "counterexamples": norm_counter,
        "status": status,
    });
    hash_canonical_json(&input)
}

fn alias_status_order(s: &AliasStatus) -> u8 {
    match s {
        AliasStatus::Approved => 0,
        AliasStatus::Deprecated => 1,
        AliasStatus::Ambiguous => 2,
        AliasStatus::Blocked => 3,
    }
}

fn sort_string_vec(v: &mut Vec<String>) {
    v.sort();
}

fn sort_source_refs(v: &mut Vec<SourceRef>) {
    v.sort_by(|a, b| {
        a.uri
            .cmp(&b.uri)
            .then(a.start_byte.cmp(&b.start_byte))
            .then(a.end_byte.cmp(&b.end_byte))
    });
}
