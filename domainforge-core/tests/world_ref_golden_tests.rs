//! Cross-language golden vectors for `world_ref`. The vectors are generated
//! and self-checked by an independent stdlib-only Python reference
//! (`tests/fixtures/world_ref/world_ref_reference.py`); this suite proves the
//! Rust core reproduces every digest and accepts/rejects exactly the same
//! strings. Do not regenerate the vectors to make this pass.

use domainforge_core::application::envelope::DomainModelIdentity;
use domainforge_core::application::world::{WorldAlias, WorldName, WorldRef};
use serde_json::Value;

fn vectors() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/world_ref/golden-vectors.json"
    );
    serde_json::from_str(&std::fs::read_to_string(path).expect("vectors exist")).expect("json")
}

#[test]
fn rust_reproduces_every_independent_digest_and_world_ref() {
    let v = vectors();
    for case in v["valid"].as_array().unwrap() {
        let label = case["case"].as_str().unwrap();
        let identity: DomainModelIdentity =
            serde_json::from_value(case["identity"].clone()).expect("identity deserializes");
        assert_eq!(
            identity.canonical_digest(),
            case["canonical_digest"].as_str().unwrap(),
            "{label}: digest"
        );
        let name: WorldName = case["name"].as_str().unwrap().parse().unwrap();
        let world = WorldRef::from_identity(name, &identity).unwrap();
        assert_eq!(
            world.to_string(),
            case["world_ref"].as_str().unwrap(),
            "{label}"
        );
        let parsed: WorldRef = case["world_ref"].as_str().unwrap().parse().unwrap();
        assert_eq!(parsed, world, "{label}: parse");
        assert!(parsed.verify_identity(&identity).is_ok(), "{label}: verify");
    }
}

#[test]
fn rust_accepts_and_rejects_exactly_what_the_reference_does() {
    let v = vectors();
    for bad in v["invalid_refs"].as_array().unwrap() {
        let s = bad.as_str().unwrap();
        assert!(s.parse::<WorldRef>().is_err(), "accepted invalid ref {s:?}");
    }
    for ok in v["valid_aliases"].as_array().unwrap() {
        let s = ok.as_str().unwrap();
        let alias: WorldAlias = s.parse().unwrap_or_else(|e| panic!("rejected {s:?}: {e}"));
        assert_eq!(alias.to_string(), s);
    }
    for bad in v["invalid_aliases"].as_array().unwrap() {
        let s = bad.as_str().unwrap();
        assert!(
            s.parse::<WorldAlias>().is_err(),
            "accepted invalid alias {s:?}"
        );
    }
}

#[test]
fn same_closure_but_different_source_or_compiler_is_a_different_world() {
    let v = vectors();
    let by_case = |name: &str| -> DomainModelIdentity {
        let c = v["valid"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["case"] == name)
            .unwrap();
        serde_json::from_value(c["identity"].clone()).unwrap()
    };
    let base = by_case("minimal");
    for variant in [
        "source-change-same-closure",
        "compiler-upgrade-same-closure",
    ] {
        let other = by_case(variant);
        assert_eq!(base.semantic_closure_hash, other.semantic_closure_hash);
        assert_ne!(
            base.canonical_digest(),
            other.canonical_digest(),
            "{variant}"
        );
    }
}
