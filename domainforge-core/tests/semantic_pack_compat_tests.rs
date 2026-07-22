//! D10 guard (plan Task 11): the shared `semantic_pack::canonical_json`
//! serializer must stay byte-identical while application canonicalization
//! reuses it. Golden vectors captured before the application packets landed.

use domainforge_core::semantic_pack::{canonical_json, compute_sha256};
use serde_json::json;

#[test]
fn canonical_json_bytes_are_unchanged() {
    let value = json!({"c": {"y": true}, "a": "x", "b": [2, 1]});
    // Keys sort bytewise, arrays keep authored order, no whitespace.
    assert_eq!(
        canonical_json(&value),
        r#"{"a":"x","b":[2,1],"c":{"y":true}}"#
    );
    assert_eq!(
        compute_sha256(canonical_json(&value).as_bytes()),
        "sha256:aa651abb9cf26447f1e097abebda14b989f16d90ef4272d6c0a584f89ca33ea8"
    );
}

#[test]
fn application_hashes_do_not_feed_pack_content_hashes() {
    // The empty semantic-pack-set golden vector is fixed and framed; it is
    // never the hash of an empty string.
    assert_eq!(
        domainforge_core::application::semantic_pack_set_hash(&[]).unwrap(),
        "sha256:6e5312099a8d89e1d271bd89d9fcb36b031069bd8dad72be397af6b331d85c40"
    );
    assert_ne!(
        domainforge_core::application::semantic_pack_set_hash(&[]).unwrap(),
        compute_sha256(b"")
    );
}
