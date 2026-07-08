//! Deterministic identity for projection elements.
//!
//! Every projection family mints element ids and IRI/QName fragments through
//! this module so identity is uniform across targets and stable run-to-run.
//! Hashing reuses `authority::compute_deterministic_hash` (xxh64 seed 42,
//! 16 hex chars); the U+0001 separator matches the ai-learning record-id
//! convention so ids never collide across otherwise-equal part tuples.

use crate::authority::compute_deterministic_hash;

/// U+0001 field separator: illegal in the human-authored names it joins, so it
/// cannot appear inside a part and force a collision.
const SEP: char = '\u{1}';

/// Deterministic content hash of a U+0001-joined tuple of `parts`
/// (xxh64 seed 42, 16 hex chars). The lowest-level primitive the other
/// helpers build on.
pub fn content_hash(parts: &[&str]) -> String {
    compute_deterministic_hash(&join(parts))
}

/// Deterministic element id for a projection `family`. The family tag is the
/// first hashed field, so the same part tuple yields distinct ids under
/// different families (an RDF class and a BPMN task never share an id).
pub fn element_id(family: &str, parts: &[&str]) -> String {
    let mut all = Vec::with_capacity(parts.len() + 1);
    all.push(family);
    all.extend_from_slice(parts);
    content_hash(&all)
}

/// Map an arbitrary string to a safe XML QName / IRI fragment: keep ASCII
/// alphanumerics and `_` / `-` / `.`, replace every other character with `_`.
/// A leading digit (illegal in an XML NCName) is prefixed with `_`; empty
/// input yields `_`. Pure and deterministic — no locale or Unicode folding.
pub fn sanitize_qname(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        return "_".to_string();
    }
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

fn join(parts: &[&str]) -> String {
    let mut joined = String::new();
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            joined.push(SEP);
        }
        joined.push_str(part);
    }
    joined
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_hash_is_stable_and_16_hex() {
        let id = content_hash(&["a", "b"]);
        assert_eq!(id.len(), 16);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(id, content_hash(&["a", "b"]));
    }

    #[test]
    fn separator_prevents_collision() {
        assert_ne!(content_hash(&["ab", "c"]), content_hash(&["a", "bc"]));
    }

    #[test]
    fn element_id_namespaces_by_family() {
        assert_ne!(element_id("rdf", &["x"]), element_id("bpmn", &["x"]));
    }

    #[test]
    fn sanitize_qname_rules() {
        assert_eq!(sanitize_qname("Order Line/2"), "Order_Line_2");
        assert_eq!(sanitize_qname("2fast"), "_2fast");
        assert_eq!(sanitize_qname(""), "_");
        assert_eq!(sanitize_qname("keep_-.ok"), "keep_-.ok");
    }
}
