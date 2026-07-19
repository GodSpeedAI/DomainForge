//! Reference §6 canonicalization for application artifacts.
//!
//! Byte serialization reuses the unchanged `semantic_pack::canonical_json`
//! helper; no application hash feeds semantic-pack content hashes (D10).

use crate::application::contract::{ApplicationContractDocument, TypedValue};
use crate::semantic_pack::{canonical_json, compute_sha256};
use indexmap::IndexMap;
use rust_decimal::Decimal;
use unicode_normalization::UnicodeNormalization;

/// NFC-normalize a canonical display/artifact string.
pub fn nfc(s: &str) -> String {
    s.nfc().collect()
}

/// Canonical decimal form: no exponent, no trailing fractional zeros,
/// `"0"` for zero. `125.00` fingerprints as `125`.
pub fn canonical_decimal(value: Decimal) -> String {
    let normalized = value.normalize();
    if normalized.is_zero() {
        "0".to_string()
    } else {
        normalized.to_string()
    }
}

/// Reference §6 source-set hash: NFC logical IDs sorted bytewise, exact
/// decoded source text, duplicate IDs rejected.
pub fn source_set_hash<'a, I>(sources: I) -> Result<String, String>
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let mut entries: Vec<(String, &str)> = sources
        .into_iter()
        .map(|(id, source)| (nfc(id), source))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    for pair in entries.windows(2) {
        if pair[0].0 == pair[1].0 {
            return Err(format!("duplicate logical id '{}'", pair[0].0));
        }
    }
    let value = serde_json::json!({
        "schema_version": "domainforge-source-set/v1",
        "sources": entries
            .iter()
            .map(|(id, source)| serde_json::json!({"logical_id": id, "source_utf8": source}))
            .collect::<Vec<_>>(),
    });
    Ok(compute_sha256(canonical_json(&value).as_bytes()))
}

/// Reference §6 semantic-pack-set hash over `{pack_id, content_hash}` entries.
/// The empty set hashes the fixed frame, never an empty string.
pub fn semantic_pack_set_hash(packs: &[(String, String)]) -> Result<String, String> {
    let mut entries: Vec<&(String, String)> = packs.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    for pair in entries.windows(2) {
        if pair[0].0 == pair[1].0 {
            return Err(format!("duplicate pack id '{}'", pair[0].0));
        }
    }
    let value = serde_json::json!({
        "schema_version": "domainforge-semantic-pack-set/v1",
        "packs": entries
            .iter()
            .map(|(id, hash)| serde_json::json!({"pack_id": id, "content_hash": hash}))
            .collect::<Vec<_>>(),
    });
    Ok(compute_sha256(canonical_json(&value).as_bytes()))
}

fn tagged(kind: &str, data: serde_json::Value) -> serde_json::Value {
    serde_json::json!({"kind": kind, "data": data})
}

/// Canonical wire value of one typed value: NFC strings, canonical decimals,
/// base-unit quantities, RFC 3339 UTC timestamps with a trailing `Z`.
pub fn canonical_typed_value(value: &TypedValue) -> serde_json::Value {
    match value {
        TypedValue::String(s) => tagged("string", serde_json::json!(nfc(s))),
        TypedValue::Int(i) => tagged("int", serde_json::json!(i)),
        TypedValue::Decimal(d) => tagged("decimal", serde_json::json!(canonical_decimal(*d))),
        TypedValue::Bool(b) => tagged("bool", serde_json::json!(b)),
        TypedValue::Timestamp(t) => tagged(
            "timestamp",
            serde_json::json!(t.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true)),
        ),
        TypedValue::Uuid(u) => tagged("uuid", serde_json::json!(u.to_string())),
        TypedValue::Quantity { base_value, unit } => tagged(
            "quantity",
            serde_json::json!({
                "base_value": canonical_decimal(*base_value),
                "unit": unit,
            }),
        ),
        TypedValue::EntityRef { entity, key } => tagged(
            "entity_ref",
            serde_json::json!({"entity": entity, "key": canonical_typed_value(key)}),
        ),
        TypedValue::Enum { symbol, wire } => tagged(
            "enum",
            serde_json::json!({"symbol": symbol, "wire": nfc(wire)}),
        ),
        TypedValue::List(items) => tagged(
            "list",
            serde_json::Value::Array(items.iter().map(canonical_typed_value).collect()),
        ),
    }
}

/// Reference §6 canonical input fingerprint: SHA-256 over canonical JSON of
/// the normalized input record. Keys sort bytewise after NFC normalization.
pub fn input_fingerprint(input: &IndexMap<String, TypedValue>) -> String {
    let object: serde_json::Map<String, serde_json::Value> = input
        .iter()
        .map(|(name, value)| (nfc(name), canonical_typed_value(value)))
        .collect();
    compute_sha256(canonical_json(&serde_json::Value::Object(object)).as_bytes())
}

/// Reference §6 artifact self-hash: canonical JSON with only `self_hash`
/// absent.
pub fn document_self_hash(doc: &ApplicationContractDocument) -> String {
    let mut value = serde_json::to_value(doc).expect("contract document serializes");
    value
        .as_object_mut()
        .expect("document is an object")
        .remove("self_hash");
    compute_sha256(canonical_json(&value).as_bytes())
}
