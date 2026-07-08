//! Deterministic split assignment: a pure function of
//! (record_id ‖ source_model_hash ‖ seed). No RNG state, no shuffle order.

use super::recipe::SplitRatios;
use xxhash_rust::xxh64::xxh64;

pub const TRAIN: &str = "train";
pub const VALIDATION: &str = "validation";
pub const TEST: &str = "test";

pub fn assign_split(
    record_id: &str,
    source_model_hash: &str,
    seed: u64,
    ratios: &SplitRatios,
) -> &'static str {
    let key = format!("{record_id}\u{1}{source_model_hash}\u{1}{seed}");
    let bucket = xxh64(key.as_bytes(), seed) as f64 / u64::MAX as f64;
    if bucket < ratios.train {
        TRAIN
    } else if bucket < ratios.train + ratios.validation {
        VALIDATION
    } else {
        TEST
    }
}

/// Fixed rule for tiny datasets: every non-empty dataset places at least one
/// record in train. Returns the index that was reassigned, if any.
pub fn ensure_train_nonempty(splits: &mut [&'static str]) -> Option<usize> {
    if splits.is_empty() || splits.contains(&TRAIN) {
        return None;
    }
    splits[0] = TRAIN;
    Some(0)
}

/// Deterministic content hash used for record ids (xxh64, seed 42, 16-hex).
/// Routes through the shared projection identity kernel so ai-learning record
/// ids and every other family's element ids share one convention.
pub fn record_id(parts: &[&str]) -> String {
    crate::projection::ids::content_hash(parts)
}
