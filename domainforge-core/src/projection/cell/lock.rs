//! `cell.lock`: composed, deterministic projection identity. Every emitted
//! file is hashed and bound to one `cell.lock`; `mod.rs::emit` fails closed
//! (`CELL030`) if a renderer produced a file the lock did not capture.

use super::ir::CellIr;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Build `cell.lock` from every other emitted file's content. `model_hash`
/// hashes the canonically formatted `.sea` source so whitespace-only edits
/// don't churn the lock.
pub fn build(
    ir: &CellIr,
    created_at: &str,
    canonical_source: &str,
    overrides_source: Option<&str>,
    files: &BTreeMap<String, String>,
) -> String {
    let model_hash = format!("sha256:{}", sha256_hex(canonical_source.as_bytes()));
    let profile_hash = format!(
        "sha256:{}",
        sha256_hex(format!("{}:{}", ir.profile_id, ir.profile_version).as_bytes())
    );

    let mut file_hashes = serde_json::Map::new();
    for (path, content) in files {
        file_hashes.insert(
            path.clone(),
            json!(format!("sha256:{}", sha256_hex(content.as_bytes()))),
        );
    }

    let overrides = match overrides_source {
        Some(src) => json!({
            "present": true,
            "unsafe": ir.unsafe_override.is_some(),
            "hash": format!("sha256:{}", sha256_hex(src.as_bytes())),
        }),
        None => json!({ "present": false, "unsafe": false }),
    };

    let doc = json!({
        "schema": "domainforge-cell-lock/v1",
        "cell_id": ir.identity.cell_id,
        "created_at": created_at,
        "model": { "version": ir.identity.model_version, "hash": model_hash },
        "profile": { "id": ir.profile_id, "version": ir.profile_version, "hash": profile_hash },
        "overrides": overrides,
        "files": file_hashes,
    });
    format!("{}\n", serde_json::to_string_pretty(&doc).unwrap())
}
