use serde::{Deserialize, Serialize};

use super::schema::{PackRef, SemanticPack};

// ---------------------------------------------------------------------------
// PackSet (10.1)
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackSet {
    pub packs: Vec<PackRef>,
    pub precedence: Vec<String>,
    pub merged_pack_hash: String,
    pub conflicts: Vec<PackConflict>,
}

// ---------------------------------------------------------------------------
// PackConflict (10.4)
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackConflict {
    pub conflict_type: ConflictType,
    pub key: String,
    pub pack_a_id: String,
    pub pack_b_id: String,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    SameConceptIdDifferentHash,
    SameCanonicalNameDifferentId,
    SameAliasKeyDifferentTarget,
    IncompatibleRelationCardinality,
    SameUnitSymbolDifferentDimension,
}

// ---------------------------------------------------------------------------
// Merge packs into a PackSet
// ---------------------------------------------------------------------------
pub fn merge_packs(packs: &[SemanticPack], priority_order: &[i32]) -> Result<PackSet, Vec<PackConflict>> {
    if !packs.is_empty() && priority_order.len() != packs.len() {
        return Err(vec![PackConflict {
            conflict_type: ConflictType::SameUnitSymbolDifferentDimension,
            key: String::new(),
            pack_a_id: String::new(),
            pack_b_id: String::new(),
            detail: format!(
                "priority_order length ({}) must match packs length ({})",
                priority_order.len(),
                packs.len()
            ),
        }]);
    }
    if packs.is_empty() {
        return Ok(PackSet {
            packs: vec![],
            precedence: vec![],
            merged_pack_hash: super::canonical_json::compute_sha256(b"empty_pack_set"),
            conflicts: vec![],
        });
    }

    // Build PackRefs sorted by canonical merge order (§10.2)
    let mut refs: Vec<(PackRef, usize)> = Vec::new();
    for (i, pack) in packs.iter().enumerate() {
        let content_hash = super::canonical_json::compute_pack_content_hash(pack);
        let priority = priority_order.get(i).copied().unwrap_or(0);
        refs.push((
            PackRef {
                pack_id: pack.pack_id.clone(),
                pack_content_hash: content_hash,
                path_or_uri: format!("pack://{}", pack.pack_id),
                priority,
            },
            i,
        ));
    }

    // Sort: explicit priority ascending, pack_id ascending, hash ascending
    refs.sort_by(|a, b| {
        a.0.priority
            .cmp(&b.0.priority)
            .then(a.0.pack_id.cmp(&b.0.pack_id))
            .then(a.0.pack_content_hash.cmp(&b.0.pack_content_hash))
    });

    // Detect conflicts (§10.4)
    let conflicts = detect_conflicts(packs, &refs);

    if !conflicts.is_empty() {
        return Err(conflicts);
    }

    let pack_refs: Vec<PackRef> = refs.into_iter().map(|(r, _)| r).collect();
    let merged_hash = compute_merged_hash(&pack_refs);
    let precedence: Vec<String> = pack_refs.iter().map(|r| r.pack_id.clone()).collect();

    Ok(PackSet {
        packs: pack_refs.clone(),
        precedence,
        merged_pack_hash: merged_hash,
        conflicts: vec![],
    })
}

fn detect_conflicts(
    packs: &[SemanticPack],
    refs: &[(PackRef, usize)],
) -> Vec<PackConflict> {
    let mut conflicts = Vec::new();

    // Same concept ID, different definition hash
    for i in 0..refs.len() {
        for j in (i + 1)..refs.len() {
            let pack_a = &packs[refs[i].1];
            let pack_b = &packs[refs[j].1];

            // Concept conflicts
            for ca in &pack_a.concepts {
                for cb in &pack_b.concepts {
                    if ca.id == cb.id && ca.definition.definition_hash != cb.definition.definition_hash {
                        conflicts.push(PackConflict {
                            conflict_type: ConflictType::SameConceptIdDifferentHash,
                            key: ca.id.clone(),
                            pack_a_id: pack_a.pack_id.clone(),
                            pack_b_id: pack_b.pack_id.clone(),
                            detail: format!(
                                "Concept '{}' has different definition hashes: {} vs {}",
                                ca.id, ca.definition.definition_hash, cb.definition.definition_hash
                            ),
                        });
                    }
                }
            }

            // Canonical name conflicts
            use super::resolver::normalize_lookup_key;
            let mut names_a: std::collections::HashMap<String, &str> = std::collections::HashMap::new();
            for c in &pack_a.concepts {
                let norm = normalize_lookup_key(&c.canonical_name);
                names_a.insert(norm, &c.id);
            }
            for c in &pack_b.concepts {
                let norm = normalize_lookup_key(&c.canonical_name);
                if let Some(existing_id) = names_a.get(&norm) {
                    if *existing_id != c.id {
                        conflicts.push(PackConflict {
                            conflict_type: ConflictType::SameCanonicalNameDifferentId,
                            key: norm.clone(),
                            pack_a_id: pack_a.pack_id.clone(),
                            pack_b_id: pack_b.pack_id.clone(),
                            detail: format!(
                                "Canonical name '{}' maps to '{}' and '{}'",
                                norm, existing_id, c.id
                            ),
                        });
                    }
                }
            }

            // Alias conflicts
            let mut aliases_a: std::collections::HashMap<String, &str> = std::collections::HashMap::new();
            for a in &pack_a.aliases {
                aliases_a.insert(a.normalized_alias.clone(), &a.target_concept_id);
            }
            for a in &pack_b.aliases {
                if let Some(existing_target) = aliases_a.get(&a.normalized_alias) {
                    if *existing_target != a.target_concept_id {
                        conflicts.push(PackConflict {
                            conflict_type: ConflictType::SameAliasKeyDifferentTarget,
                            key: a.normalized_alias.clone(),
                            pack_a_id: pack_a.pack_id.clone(),
                            pack_b_id: pack_b.pack_id.clone(),
                            detail: format!(
                                "Alias '{}' targets '{}' and '{}'",
                                a.normalized_alias, existing_target, a.target_concept_id
                            ),
                        });
                    }
                }
            }

            // Unit symbol conflicts
            let mut units_a: std::collections::HashMap<String, &str> = std::collections::HashMap::new();
            for u in &pack_a.units {
                units_a.insert(u.symbol.clone(), &u.dimension_id);
            }
            for u in &pack_b.units {
                if let Some(existing_dim) = units_a.get(&u.symbol) {
                    if *existing_dim != u.dimension_id {
                        conflicts.push(PackConflict {
                            conflict_type: ConflictType::SameUnitSymbolDifferentDimension,
                            key: u.symbol.clone(),
                            pack_a_id: pack_a.pack_id.clone(),
                            pack_b_id: pack_b.pack_id.clone(),
                            detail: format!(
                                "Unit '{}' has different dimensions: {} vs {}",
                                u.symbol, existing_dim, u.dimension_id
                            ),
                        });
                    }
                }
            }
        }
    }

    conflicts
}

fn compute_merged_hash(refs: &[PackRef]) -> String {
    let mut sorted_refs: Vec<serde_json::Value> = refs
        .iter()
        .map(|r| serde_json::to_value(r).expect("failed to serialize PackRef for hash computation"))
        .collect();
    sorted_refs.sort_by(|a, b| {
        let a_id = a.get("pack_id").and_then(|v| v.as_str()).unwrap_or("");
        let b_id = b.get("pack_id").and_then(|v| v.as_str()).unwrap_or("");
        a_id.cmp(b_id)
    });
    let input = serde_json::json!({
        "pack_refs": sorted_refs,
        "conflict_policy": "error_on_conflict"
    });
    super::canonical_json::hash_canonical_json(&input)
}
