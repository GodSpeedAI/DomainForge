//! Stage 2 `world_ref` contract: one immutable reference to one resolved
//! semantic-world revision, grounded exclusively in
//! `DomainModelIdentity::canonical_digest()`.

use domainforge_core::application::envelope::DomainModelIdentity;
use domainforge_core::application::resolve_semantic_envelope;
use domainforge_core::application::world::{
    WorldAlias, WorldCatalog, WorldLabel, WorldName, WorldRef, WorldRefError,
};
use serde_json::json;

const BASIC: &str = "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";

fn identity_for(source: &str) -> DomainModelIdentity {
    let sources = json!({ "main.sea": source }).to_string();
    let doc = resolve_semantic_envelope("main.sea", &sources).expect("model resolves");
    DomainModelIdentity::from_document(&doc, None)
}

fn name(s: &str) -> WorldName {
    s.parse().expect("valid world name")
}

// ---- format ----

#[test]
fn world_ref_round_trips_through_its_canonical_text() {
    let id = identity_for(BASIC);
    let world = WorldRef::from_identity(name("godspeed-corporate"), &id).unwrap();
    let text = world.to_string();
    assert!(
        text.starts_with("world:godspeed-corporate@sha256:"),
        "{text}"
    );
    assert_eq!(text.len(), "world:godspeed-corporate@".len() + 7 + 64);
    assert_eq!(text.parse::<WorldRef>().unwrap(), world);
    assert_eq!(world.digest(), id.canonical_digest());
}

#[test]
fn malformed_world_refs_are_rejected() {
    let hex = "a".repeat(64);
    let cases = [
        "",
        "world:",
        "world:corp",                                     // alias, not a ref
        "corp@sha256:aaaa",                               // no scheme
        &format!("world:Corp@sha256:{hex}"),              // uppercase name
        &format!("world:corp@sha1:{hex}"),                // wrong algorithm
        &format!("world:corp@sha256:{}", "a".repeat(63)), // short digest
        &format!("world:corp@sha256:{}", "A".repeat(64)), // uppercase hex
        &format!("world:corp@sha256:{}", "g".repeat(64)), // non-hex
        &format!(" world:corp@sha256:{hex}"),             // whitespace
        &format!("world:corp@sha256:{hex} "),
        &format!("world:corp@@sha256:{hex}"),
        &format!("world:-corp@sha256:{hex}"), // name grammar
        &format!("world:corp--x@sha256:{hex}"),
        &format!("world:{}@sha256:{hex}", "a".repeat(65)), // name too long
    ];
    for case in cases {
        assert!(case.parse::<WorldRef>().is_err(), "must reject {case:?}");
    }
}

#[test]
fn alias_is_a_distinct_type_and_never_parses_as_a_ref() {
    let alias: WorldAlias = "world:corporate".parse().unwrap();
    assert_eq!(alias.to_string(), "world:corporate");
    assert!("world:corporate@sha256:".parse::<WorldAlias>().is_err());
    let hex = "b".repeat(64);
    assert!(format!("world:corporate@sha256:{hex}")
        .parse::<WorldAlias>()
        .is_err());
}

#[test]
fn label_is_presentation_only() {
    let label = WorldLabel::new("  GodSpeed Corporate World ").unwrap();
    assert_eq!(label.as_str(), "GodSpeed Corporate World");
    assert!(WorldLabel::new("   ").is_err());
    assert!(WorldLabel::new("bad\u{0007}label").is_err());
}

// ---- grounding in DomainForge identity ----

#[test]
fn same_resolved_world_yields_the_same_world_ref() {
    let a = WorldRef::from_identity(name("t"), &identity_for(BASIC)).unwrap();
    let b = WorldRef::from_identity(name("t"), &identity_for(BASIC)).unwrap();
    assert_eq!(a, b);
}

#[test]
fn semantic_change_yields_a_new_world_ref() {
    let a = WorldRef::from_identity(name("t"), &identity_for(BASIC)).unwrap();
    let changed =
        "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\nentity \"Pump\" { key id: uuid }\n";
    let b = WorldRef::from_identity(name("t"), &identity_for(changed)).unwrap();
    assert_ne!(a.digest(), b.digest());
}

/// DomainModelIdentity binds the exact source text (`source_set_hash`) as well
/// as the semantic closure. A comment-only edit therefore mints a new
/// `world_ref`, while `semantic_closure_hash` proves the meaning is unchanged.
#[test]
fn comment_only_edit_is_a_new_world_with_the_same_semantic_closure() {
    let commented = "// documentation only\n@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";
    let a = identity_for(BASIC);
    let b = identity_for(commented);
    assert_eq!(a.semantic_closure_hash, b.semantic_closure_hash);
    assert_ne!(a.source_set_hash, b.source_set_hash);
    assert_ne!(a.canonical_digest(), b.canonical_digest());
}

#[test]
fn identity_construction_follows_the_documented_field_mapping() {
    let sources = json!({ "main.sea": BASIC }).to_string();
    let doc = resolve_semantic_envelope("main.sea", &sources).unwrap();
    let registry = format!("sha256:{}", "c".repeat(64));
    let id = DomainModelIdentity::from_document(&doc, Some(registry.as_str()));
    assert_eq!(id.registry_content_hash.as_deref(), Some(registry.as_str()));
    assert_eq!(id.identity_scheme_version, "v2-full-preimage");
    assert_eq!(id.producer, doc.producer.name);
    assert_eq!(id.producer_version, doc.producer.version);
    assert_eq!(id.content_hash, doc.self_hash);
    assert_eq!(id.semantic_closure_hash, doc.semantic_closure_hash);
    assert_eq!(id.source_set_hash, doc.inputs.source_set_hash);
    assert_eq!(
        id.semantic_pack_set_hash.as_deref(),
        Some(doc.inputs.semantic_pack_set_hash.as_str())
    );
}

// ---- verification ----

#[test]
fn tampered_identity_does_not_verify_against_the_original_ref() {
    let id = identity_for(BASIC);
    let world = WorldRef::from_identity(name("t"), &id).unwrap();
    assert!(world.verify_identity(&id).is_ok());

    let mut forged = id.clone();
    forged.semantic_closure_hash = format!("sha256:{}", "0".repeat(64));
    assert!(matches!(
        world.verify_identity(&forged),
        Err(WorldRefError::DigestMismatch { .. })
    ));

    let mut downgraded = id.clone();
    downgraded.producer_version = "0.0.1".to_string();
    assert!(world.verify_identity(&downgraded).is_err());
}

#[test]
fn unsupported_identity_scheme_and_malformed_hashes_fail_closed() {
    let mut id = identity_for(BASIC);
    id.identity_scheme_version = "v1-legacy".to_string();
    assert!(matches!(
        WorldRef::from_identity(name("t"), &id),
        Err(WorldRefError::UnsupportedIdentityScheme(_))
    ));

    let mut id = identity_for(BASIC);
    id.content_hash = "not-a-hash".to_string();
    assert!(matches!(
        WorldRef::from_identity(name("t"), &id),
        Err(WorldRefError::MalformedIdentity(_))
    ));
}

// ---- catalog: lookup, aliases, history ----

#[test]
fn catalog_registers_verifies_and_distinguishes_worlds() {
    let mut catalog = WorldCatalog::default();
    let a_id = identity_for(BASIC);
    let b_id = identity_for("@namespace \"t\"\nentity \"Pump\" { key id: uuid }\n");
    let a = catalog.register(name("t"), a_id.clone()).unwrap();
    let b = catalog.register(name("t"), b_id.clone()).unwrap();
    assert_ne!(a, b);
    assert!(catalog.verify(&a, &a_id).is_ok());
    assert!(catalog.verify(&b, &b_id).is_ok());
    assert!(catalog.verify(&a, &b_id).is_err());
    assert_eq!(catalog.lookup(&a).unwrap(), &a_id);

    // idempotent
    assert_eq!(catalog.register(name("t"), a_id).unwrap(), a);
}

#[test]
fn unknown_world_fails_closed_even_if_well_formed() {
    let catalog = WorldCatalog::default();
    let ghost = WorldRef::from_identity(name("t"), &identity_for(BASIC)).unwrap();
    assert!(matches!(
        catalog.lookup(&ghost),
        Err(WorldRefError::UnknownWorld(_))
    ));
    assert!(matches!(
        catalog.pin(&ghost.to_string()),
        Err(WorldRefError::UnknownWorld(_))
    ));
}

#[test]
fn a_digest_has_exactly_one_canonical_name() {
    let mut catalog = WorldCatalog::default();
    let id = identity_for(BASIC);
    catalog.register(name("alpha"), id.clone()).unwrap();
    assert!(matches!(
        catalog.register(name("beta"), id),
        Err(WorldRefError::NameConflict { .. })
    ));
}

#[test]
fn alias_retarget_never_rewrites_a_historical_world_ref() {
    let mut catalog = WorldCatalog::default();
    let a = catalog
        .register(name("corporate"), identity_for(BASIC))
        .unwrap();
    let b_id = identity_for("@namespace \"t\"\nentity \"Pump\" { key id: uuid }\n");
    let b = catalog.register(name("corporate"), b_id).unwrap();
    let alias: WorldAlias = "world:corporate".parse().unwrap();

    catalog.retarget_alias(&alias, &a).unwrap();
    // A run begins: the alias is pinned to an immutable ref.
    let pinned_for_run = catalog.pin("world:corporate").unwrap();
    assert_eq!(pinned_for_run, a);

    catalog.retarget_alias(&alias, &b).unwrap();
    assert_eq!(catalog.resolve_alias(&alias).unwrap(), b);
    assert_eq!(catalog.pin("world:corporate").unwrap(), b);

    // The run's pinned reference is unchanged and still verifies.
    assert_eq!(pinned_for_run, a);
    assert!(catalog.lookup(&pinned_for_run).is_ok());
    assert_eq!(catalog.alias_history(&alias), vec![a, b]);
}

#[test]
fn pin_requires_a_known_alias_or_a_known_ref() {
    let mut catalog = WorldCatalog::default();
    let a = catalog
        .register(name("corporate"), identity_for(BASIC))
        .unwrap();
    assert!(matches!(
        catalog.pin("world:corporate"),
        Err(WorldRefError::UnknownAlias(_))
    ));
    assert_eq!(catalog.pin(&a.to_string()).unwrap(), a);
    assert!(catalog.pin("corporate").is_err());
    assert!(catalog.pin("").is_err());
}

#[test]
fn retargeting_an_alias_to_an_unregistered_world_is_rejected() {
    let mut catalog = WorldCatalog::default();
    let ghost = WorldRef::from_identity(name("t"), &identity_for(BASIC)).unwrap();
    let alias: WorldAlias = "world:corporate".parse().unwrap();
    assert!(matches!(
        catalog.retarget_alias(&alias, &ghost),
        Err(WorldRefError::UnknownWorld(_))
    ));
}
