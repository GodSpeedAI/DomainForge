#![cfg(feature = "cli")]

use sea_core::parser::parse_to_graph;
use sea_core::semantic_pack::canonical_json::compute_sha256;

#[test]
fn canonical_hash_reflects_parse_order() {
    let model_a = r#"
Entity "Alpha"
Entity "Beta"
Resource "Water" units
Flow "Water" from "Alpha" to "Beta" quantity 10
"#;
    let model_b = r#"
Entity "Beta"
Entity "Alpha"
Resource "Water" units
Flow "Water" from "Alpha" to "Beta" quantity 10
"#;

    let graph_a = parse_to_graph(model_a).expect("parsed model A");
    let graph_b = parse_to_graph(model_b).expect("parsed model B");

    let json_a = serde_json::to_string(&graph_a).expect("serialized A");
    let json_b = serde_json::to_string(&graph_b).expect("serialized B");

    let hash_a = compute_sha256(json_a.as_bytes());
    let hash_b = compute_sha256(json_b.as_bytes());

    assert_ne!(
        hash_a, hash_b,
        "canonical form is parse-order-dependent (IndexMap insertion order); \
         see docs/specs/canonical_entrypoints.md §4 (G2 caveat). \
         Two declaration-order permutations of the same model produce different \
         canonical JSON because entity insertion order differs."
    );
}

#[test]
fn document_order_sensitivity_g2_caveat() {
    let model = r#"
Entity "X"
Entity "Y"
"#;
    let reordered = r#"
Entity "Y"
Entity "X"
"#;

    let g1 = parse_to_graph(model).expect("parsed");
    let g2 = parse_to_graph(reordered).expect("parsed");

    let j1 = serde_json::to_string(&g1).expect("ser");
    let j2 = serde_json::to_string(&g2).expect("ser");

    assert_ne!(
        compute_sha256(j1.as_bytes()),
        compute_sha256(j2.as_bytes()),
        "G2 caveat: canonical form is parse-order-dependent. This test documents \
         that two semantically-equivalent models in different declaration order \
         are NOT hash-equal. Determinism holds for a FIXED parse order, not for \
         order-independence. Tracked by audit Phase 6."
    );
}
