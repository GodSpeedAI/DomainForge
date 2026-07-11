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

/// Lowercase ASCII slug for URI segments, event-type fragments, channel
/// addresses, and Python function-name fragments: keep ASCII alphanumerics,
/// replace every other character with `_`, fold to lowercase. Empty input
/// yields `_`. This is the single blessed slug function every projection
/// family MUST use (ADR-011 §2) instead of private reimplementations.
pub fn slug(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push('_');
    }
    out
}

/// PascalCase (UpperCamelCase) identifier for type names (DDD entity classes,
/// aggregates, commands, events, errors, value objects): every maximal run of
/// ASCII alphanumerics becomes a capitalized word; non-alphanumeric chars act
/// as word boundaries and are dropped. The first alpha of each run is
/// upper-cased; the rest are preserved as-is. Empty input yields `Unknown`.
/// Use this for type-name positions only — never for slugs or display text.
pub fn pascal(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut cap_next = true;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            if cap_next {
                out.push(ch.to_ascii_uppercase());
                cap_next = false;
            } else {
                out.push(ch);
            }
        } else {
            cap_next = true;
        }
    }
    if out.is_empty() {
        out.push_str("Unknown");
    }
    out
}

/// Identifier sanitizer for code-like contexts (TLA+, Alloy, Cedar entity
/// types, Python identifiers): keep ASCII alphanumerics and `_`, replace
/// everything else with `_`, prefix a leading digit with `_`. Empty input
/// yields `Unknown`. Use this for identifier positions only — never for
/// interpolated display text (escape that per-format instead).
pub fn ident(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for (i, ch) in raw.chars().enumerate() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            if i == 0 && ch.is_ascii_digit() {
                out.push('_');
            }
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push_str("Unknown");
    }
    out
}

/// Filesystem-safe basename sanitizer: keep ASCII alphanumerics and `-`, `_`,
/// `.`; replace everything else with `_`. Empty input yields `default`.
pub fn sanitize_filename(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push_str("default");
    }
    out
}

/// First 8 hex chars of the content hash of `raw` — the collision suffix the
/// [`NameRegistrar`] appends when two distinct originals collapse to the same
/// sanitized form.
pub fn collision_suffix(raw: &str) -> String {
    content_hash(&[raw])[..8].to_string()
}

/// Collision-safe name registrar. Per emission run, tracks the sanitized →
/// original mapping; on collision (two distinct originals produce the same
/// sanitized identifier) appends `_<collision_suffix(original)>` so every
/// emitted name is unique and deterministic. ADR-011 §2 requires this so
/// sanitizers can never silently merge or overwrite distinct model names.
///
/// Usage: `let mut reg = NameRegistrar::new(); let id = reg.register("sig", name);`
/// where `"sig"` is the sanitizer key. The registrar owns its own map, so the
/// same raw name can be registered independently under different sanitizer
/// contexts.
pub struct NameRegistrar {
    taken: std::collections::BTreeMap<String, String>,
}

impl NameRegistrar {
    pub fn new() -> Self {
        Self {
            taken: std::collections::BTreeMap::new(),
        }
    }

    /// Register `original` under the given `sanitizer` and return the
    /// sanitized, collision-safe identifier. `sanitizer` is one of `"slug"`,
    /// `"ident"`, `"filename"`. On collision, appends `_<hash8(original)>`.
    pub fn register(&mut self, sanitizer: &str, original: &str) -> String {
        let base = match sanitizer {
            "slug" => slug(original),
            "ident" => ident(original),
            "filename" => sanitize_filename(original),
            _ => ident(original),
        };
        match self.taken.get(&base) {
            None => {
                self.taken.insert(base.clone(), original.to_string());
                base
            }
            Some(prev) if prev == original => base,
            Some(_) => {
                // Collision: two distinct originals map to the same sanitized
                // form. Append a deterministic hash suffix of the new original.
                let suffix = collision_suffix(original);
                let unique = format!("{base}_{suffix}");
                self.taken.insert(unique.clone(), original.to_string());
                unique
            }
        }
    }
}

impl Default for NameRegistrar {
    fn default() -> Self {
        Self::new()
    }
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

    #[test]
    fn slug_rules() {
        assert_eq!(slug("PurchaseOrder"), "purchaseorder");
        assert_eq!(slug("Order Line"), "order_line");
        assert_eq!(slug("supply chain"), "supply_chain");
        assert_eq!(slug(""), "_");
    }

    #[test]
    fn ident_rules() {
        assert_eq!(ident("PurchaseOrder"), "PurchaseOrder");
        assert_eq!(ident("Order Line"), "Order_Line");
        assert_eq!(ident("2fast"), "_2fast");
        assert_eq!(ident(""), "Unknown");
    }

    #[test]
    fn pascal_rules() {
        assert_eq!(pascal("require_approval"), "RequireApproval");
        assert_eq!(pascal("PurchaseOrder"), "PurchaseOrder");
        assert_eq!(pascal("order_count"), "OrderCount");
        assert_eq!(pascal("order line"), "OrderLine");
        assert_eq!(pascal("2fast"), "2fast");
        assert_eq!(pascal(""), "Unknown");
    }

    #[test]
    fn sanitize_filename_rules() {
        assert_eq!(sanitize_filename("supply chain"), "supply_chain");
        assert_eq!(sanitize_filename(""), "default");
    }

    #[test]
    fn name_registrar_no_collision() {
        let mut reg = NameRegistrar::new();
        assert_eq!(reg.register("slug", "PurchaseOrder"), "purchaseorder");
        assert_eq!(reg.register("slug", "Payment"), "payment");
    }

    #[test]
    fn name_registrar_collision_appends_hash() {
        // "Order Line" and "Order_Line" both slug to "order_line".
        let mut reg = NameRegistrar::new();
        let a = reg.register("slug", "Order Line");
        let b = reg.register("slug", "Order_Line");
        assert_eq!(a, "order_line");
        assert_ne!(a, b, "collision must produce a distinct id");
        assert!(
            b.starts_with("order_line_"),
            "collision suffix must be appended: {b}"
        );
        // Same original re-registered returns the same id (no double-suffix).
        assert_eq!(reg.register("slug", "Order Line"), a);
    }
}
