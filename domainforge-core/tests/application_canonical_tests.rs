//! §6 canonical independence proof (M0 gate finding 4): envelopes are
//! byte-identical under declaration/module reordering, import alias renames,
//! equivalent decimal spellings, and Unicode normalization; semantic changes
//! move the hashes.

use domainforge_core::application::resolve_semantic_envelope;
use serde_json::json;

fn envelope_bytes(entry: &str, sources: &str) -> (String, String) {
    let doc = resolve_semantic_envelope(entry, sources).expect("closure resolves");
    (
        serde_json::to_string(&doc.envelope).expect("envelope serializes"),
        doc.semantic_closure_hash,
    )
}

#[test]
fn declaration_reordering_does_not_change_the_envelope() {
    let a = json!({
        "main.sea": "@namespace \"t\"\nrole \"Approver\"\nentity \"Tank\" { key id: uuid }\npattern \"Email\" matches \"^[a-z]+$\"\n"
    })
    .to_string();
    let b = json!({
        "main.sea": "@namespace \"t\"\npattern \"Email\" matches \"^[a-z]+$\"\nentity \"Tank\" { key id: uuid }\nrole \"Approver\"\n"
    })
    .to_string();
    let (bytes_a, hash_a) = envelope_bytes("main.sea", &a);
    let (bytes_b, hash_b) = envelope_bytes("main.sea", &b);
    assert_eq!(
        bytes_a, bytes_b,
        "declaration order leaked into the envelope"
    );
    assert_eq!(hash_a, hash_b);
}

#[test]
fn source_map_supply_order_does_not_change_the_document() {
    let module_a = "@namespace \"alpha\"\nexport entity \"Tank\" { key id: uuid }\n";
    let module_b =
        "@namespace \"beta\"\nimport { Tank } from \"./a.sea\"\n\nentity \"Store\" {\n    key id: uuid\n    tank: ref<Tank>\n}\n";
    let one = json!({"b.sea": module_b, "a.sea": module_a}).to_string();
    let two = json!({"a.sea": module_a, "b.sea": module_b}).to_string();
    let doc_one = resolve_semantic_envelope("b.sea", &one).expect("resolves");
    let doc_two = resolve_semantic_envelope("b.sea", &two).expect("resolves");
    assert_eq!(
        serde_json::to_string(&doc_one).unwrap(),
        serde_json::to_string(&doc_two).unwrap(),
        "source map supply order leaked into the document"
    );
}

#[test]
fn import_alias_renames_do_not_change_the_envelope() {
    let target = "@namespace \"alpha\"\nexport entity \"Tank\" { key id: uuid }\n";
    let plain = json!({
        "a.sea": target,
        "main.sea": "@namespace \"m\"\nimport { Tank } from \"./a.sea\"\n\nentity \"Store\" {\n    key id: uuid\n    tank: ref<Tank>\n}\n",
    })
    .to_string();
    let aliased = json!({
        "a.sea": target,
        "main.sea": "@namespace \"m\"\nimport { Tank as Vessel } from \"./a.sea\"\n\nentity \"Store\" {\n    key id: uuid\n    tank: ref<Vessel>\n}\n",
    })
    .to_string();
    let (bytes_plain, hash_plain) = envelope_bytes("main.sea", &plain);
    let (bytes_aliased, hash_aliased) = envelope_bytes("main.sea", &aliased);
    assert_eq!(
        bytes_plain, bytes_aliased,
        "local alias spelling leaked into the envelope"
    );
    assert_eq!(hash_plain, hash_aliased);
}

#[test]
fn equivalent_decimal_spellings_share_one_canonical_form() {
    let spell = |factor: &str, default: &str| {
        json!({
            "main.sea": format!(
                "@namespace \"t\"\ndimension \"Mass\"\nunit \"g\" of \"Mass\" factor 1 base \"g\"\nunit \"kg\" of \"Mass\" factor {factor} base \"g\"\nentity \"Tank\" {{\n    key id: uuid\n    weight: quantity<kg> (min 0.50) default {default}\n}}\n"
            )
        })
        .to_string()
    };
    let (bytes_a, hash_a) = envelope_bytes("main.sea", &spell("1000", "2.5"));
    let (bytes_b, hash_b) = envelope_bytes("main.sea", &spell("1000.000", "2.50"));
    assert_eq!(
        bytes_a, bytes_b,
        "decimal spelling leaked into the envelope"
    );
    assert_eq!(hash_a, hash_b);
    // A different value is a semantic change.
    let (_, hash_c) = envelope_bytes("main.sea", &spell("1000", "2.51"));
    assert_ne!(hash_a, hash_c);
}

#[test]
fn unicode_normalization_is_applied_to_application_strings() {
    let with_intent = |intent: &str| {
        json!({
            "main.sea": format!(
                "@namespace \"t\"\nrecord In {{ id: uuid }}\nrecord Out {{ id: uuid }}\nentity \"Doc\" {{\n    key id: uuid\n}}\noperation read_doc {{\n    intent \"{intent}\"\n    direction inbound\n    actor anonymous\n    access public\n    input In\n    output Out\n    state Doc\n    effect reads Doc\n    transaction read_only\n    failure missing_doc for missing_state \"no such doc\"\n    idempotency inherent\n    concurrency read_snapshot\n    evidence operation_trace\n    lifecycle synchronous_request_response\n}}\n"
            )
        })
        .to_string()
    };
    // "café" precomposed (U+00E9) vs decomposed (e + U+0301).
    let (bytes_nfc, hash_nfc) = envelope_bytes("main.sea", &with_intent("read caf\u{e9}"));
    let (bytes_nfd, hash_nfd) = envelope_bytes("main.sea", &with_intent("read cafe\u{301}"));
    assert_eq!(
        bytes_nfc, bytes_nfd,
        "Unicode normalization form leaked into the envelope"
    );
    assert_eq!(hash_nfc, hash_nfd);
    // Different intent text is a semantic change.
    let (_, hash_other) = envelope_bytes("main.sea", &with_intent("read cafe"));
    assert_ne!(hash_nfc, hash_other);
}
