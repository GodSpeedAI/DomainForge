//! YAML emission shared by the projections that write YAML documents.

use serde_saphyr::SerializerOptions;

/// Serialize `value` to YAML with the projection options. Block scalars (`|`/`>`)
/// are disabled so every string stays a single scalar with no folding; the default
/// YAML 1.1 quoting rules quote values that any YAML 1.1/1.2 reader could type as a
/// bool, number, null, or timestamp. Anchors are never generated.
pub(crate) fn to_yaml<T: serde::Serialize>(value: &T) -> Result<String, serde_saphyr::ser::Error> {
    let mut options = SerializerOptions::default();
    options.prefer_block_scalars = false;
    serde_saphyr::to_string_with_options(value, options)
}
