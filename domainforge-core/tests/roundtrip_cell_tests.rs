//! Roundtrip-cell verification: proves the projection-cell fixture
//! round-trips through CALM (export -> import) with its representable
//! primitives intact. Gate: scripts/verify/projection-targets/roundtrip-cell.sh.

use domainforge_core::calm::{export, import};
use domainforge_core::parser::parse_to_graph;
use std::fs;

fn fixture_source() -> String {
    let path = format!(
        "{}/../fixtures/projection_cell/basic/model.sea",
        env!("CARGO_MANIFEST_DIR")
    );
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read projection-cell fixture at {path}: {e}"))
}

#[test]
fn projection_cell_round_trips_through_calm() {
    let source = fixture_source();
    let original = parse_to_graph(&source).expect("projection-cell fixture parses");

    let calm_json = export(&original).expect("CALM export succeeds");
    let round_tripped = import(calm_json).expect("CALM import succeeds");

    // Structural parity: the primitives CALM represents must survive.
    assert_eq!(
        original.entity_count(),
        round_tripped.entity_count(),
        "entity count changed across the CALM round-trip"
    );
    assert_eq!(
        original.resource_count(),
        round_tripped.resource_count(),
        "resource count changed across the CALM round-trip"
    );
    assert_eq!(
        original.flow_count(),
        round_tripped.flow_count(),
        "flow count changed across the CALM round-trip"
    );
}

#[test]
fn projection_cell_round_trip_preserves_names() {
    let source = fixture_source();
    let original = parse_to_graph(&source).expect("fixture parses");
    let calm_json = export(&original).expect("export succeeds");
    let round_tripped = import(calm_json).expect("import succeeds");

    let orig_entities: std::collections::BTreeSet<String> = original
        .all_entities()
        .iter()
        .map(|e| e.name().to_string())
        .collect();
    let rt_entities: std::collections::BTreeSet<String> = round_tripped
        .all_entities()
        .iter()
        .map(|e| e.name().to_string())
        .collect();
    assert_eq!(orig_entities, rt_entities, "entity name set diverged");

    let orig_resources: std::collections::BTreeSet<String> = original
        .all_resources()
        .iter()
        .map(|r| r.name().to_string())
        .collect();
    let rt_resources: std::collections::BTreeSet<String> = round_tripped
        .all_resources()
        .iter()
        .map(|r| r.name().to_string())
        .collect();
    assert_eq!(orig_resources, rt_resources, "resource name set diverged");
}
