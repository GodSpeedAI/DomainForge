//! Golden tests that pin the exact bytes DomainForge emits and the exact
//! inputs it accepts for every timestamp and duration it serializes, signs or
//! parses. They were recorded from the `chrono` implementation (RUSTSEC-2026-0543
//! migration) and must keep passing, unedited, on any replacement.
//!
//! The corpora in `tests/fixtures/timestamp_golden/` hold the recorded chrono
//! outputs. These tests deliberately use only type-agnostic surfaces (JSON in,
//! JSON/bytes out) so they do not name the underlying time library.
//!
//! Leap-second inputs (`:60`) are excluded: that edge is covered by a separate
//! documented-divergence test, because `time` cannot represent a leap second.

use domainforge_core::application::canonical::canonical_typed_value;
use domainforge_core::application::contract::TypedValue;
use domainforge_core::authority::FactEnvelope;
use domainforge_core::policy::{BinaryOp, Expression, Policy};
use domainforge_core::Graph;
use serde_json::{json, Value};

fn corpus(name: &str) -> Vec<Value> {
    let path = format!(
        "{}/tests/fixtures/timestamp_golden/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap()
}

fn is_leap(text: &str) -> bool {
    text.contains(":60")
}

fn envelope_json(observed: &str, expires: Option<&str>) -> Value {
    let mut v = json!({
        "path": "customer.credit_hold",
        "value": {"a": [1, 2], "b": "x"},
        "source_class": "system_of_record",
        "source_id": "erp-1",
        "observed_at": observed,
    });
    if let Some(e) = expires {
        v["expires_at"] = json!(e);
    }
    v
}

fn timestamp_value(text: &str) -> Value {
    json!({"kind": "timestamp", "data": text})
}

// ---------------------------------------------------------------------------
// Formats: FactEnvelope serde (AutoSi + Z), signing payload (to_rfc3339 +00:00),
// canonical typed value (AutoSi + Z)
// ---------------------------------------------------------------------------

#[test]
fn fact_envelope_serde_bytes_are_pinned_for_every_fraction() {
    let mut checked = 0;
    for entry in corpus("format_corpus") {
        if entry.get("unrepresentable").is_some() {
            continue;
        }
        let autosi = entry["autosi_z"].as_str().unwrap();
        let expected = entry["serde"].as_str().unwrap();
        let env: FactEnvelope =
            serde_json::from_value(envelope_json(autosi, Some(autosi))).unwrap();
        let out = serde_json::to_value(&env).unwrap();
        assert_eq!(out["observed_at"], expected, "observed_at for {entry}");
        assert_eq!(out["expires_at"], expected, "expires_at for {entry}");
        // byte-level: the serialized envelope text embeds exactly this token
        let text = serde_json::to_string(&env).unwrap();
        assert!(text.contains(&format!("\"observed_at\":\"{expected}\"")));
        // serde round trip is a fixed point
        let again: FactEnvelope = serde_json::from_str(&text).unwrap();
        assert_eq!(serde_json::to_string(&again).unwrap(), text);
        checked += 1;
    }
    assert!(checked >= 100, "corpus unexpectedly small: {checked}");
}

#[cfg(feature = "signing")]
#[test]
fn signing_payload_bytes_are_pinned_for_every_fraction() {
    use domainforge_core::authority::signing::build_signing_payload;
    for entry in corpus("format_corpus") {
        if entry.get("unrepresentable").is_some() {
            continue;
        }
        let autosi = entry["autosi_z"].as_str().unwrap();
        let rfc = entry["to_rfc3339"].as_str().unwrap();
        let env: FactEnvelope = serde_json::from_value(envelope_json(autosi, None)).unwrap();
        let payload = build_signing_payload(&env);
        let expected = format!(
            "DOMAINFORGE_FACT_V1\ncustomer.credit_hold\n{}\nerp-1\n{rfc}\n",
            serde_json::to_string(&json!({"a": [1, 2], "b": "x"})).unwrap()
        );
        assert_eq!(
            payload,
            expected.into_bytes(),
            "signing payload for {entry}"
        );
    }
}

#[test]
fn canonical_typed_timestamp_is_autosi_with_z() {
    for entry in corpus("format_corpus") {
        if entry.get("unrepresentable").is_some() {
            continue;
        }
        let autosi = entry["autosi_z"].as_str().unwrap();
        let value: TypedValue = serde_json::from_value(timestamp_value(autosi)).unwrap();
        let canonical = canonical_typed_value(&value);
        assert_eq!(
            canonical,
            json!({"kind": "timestamp", "data": autosi}),
            "canonical for {entry}"
        );
        // and the typed value itself serializes to the same token
        assert_eq!(
            serde_json::to_value(&value).unwrap(),
            timestamp_value(autosi)
        );
    }
}

#[test]
fn authority_request_requested_at_serde_is_pinned() {
    for entry in corpus("format_corpus").iter().take(40) {
        if entry.get("unrepresentable").is_some() {
            continue;
        }
        let autosi = entry["autosi_z"].as_str().unwrap();
        let req = json!({
            "request_id": "r1",
            "actor": {"id": "a"},
            "operation": "op",
            "resource": {"id": "i", "type": "t"},
            "requested_at": autosi,
        });
        let parsed: domainforge_core::authority::AuthorityRequest =
            serde_json::from_value(req).unwrap();
        assert_eq!(
            serde_json::to_value(&parsed).unwrap()["requested_at"],
            entry["serde"]
        );
    }
}

// ---------------------------------------------------------------------------
// Parsing: relaxed (serde / FromStr) and strict (parse_from_rfc3339)
// ---------------------------------------------------------------------------

#[test]
fn relaxed_serde_parsing_matches_recorded_behaviour() {
    let (mut ok, mut err) = (0, 0);
    for entry in corpus("parse_corpus") {
        let input = entry["input"].as_str().unwrap();
        let relaxed = &entry["relaxed"];
        if let Some(expected) = relaxed.get("ok") {
            if is_leap(expected.as_str().unwrap()) {
                continue;
            }
            let value: TypedValue = serde_json::from_value(timestamp_value(input))
                .unwrap_or_else(|e| panic!("{input:?} should parse, got {e}"));
            assert_eq!(
                serde_json::to_value(&value).unwrap()["data"],
                *expected,
                "instant for {input:?}"
            );
            // FactEnvelope uses the same adapter
            let env: FactEnvelope =
                serde_json::from_value(envelope_json(input, Some(input))).unwrap();
            assert_eq!(
                serde_json::to_value(&env).unwrap()["observed_at"],
                *expected
            );
            ok += 1;
        } else {
            let expected = relaxed["err"].as_str().unwrap();
            let got = serde_json::from_value::<TypedValue>(timestamp_value(input))
                .err()
                .unwrap_or_else(|| panic!("{input:?} should be rejected"));
            assert_eq!(got.to_string(), expected, "error text for {input:?}");
            assert!(
                serde_json::from_value::<FactEnvelope>(envelope_json(input, None)).is_err(),
                "{input:?} should be rejected for FactEnvelope"
            );
            err += 1;
        }
    }
    assert!(ok > 300 && err > 1000, "ok={ok} err={err}");
}

fn eval(op: BinaryOp, left: &str, right: &str, three_valued: bool) -> Result<Option<bool>, String> {
    let policy = Policy::new(
        "t",
        Expression::binary(op, Expression::literal(left), Expression::literal(right)),
    );
    policy
        .evaluate_with_mode(&Graph::new(), three_valued)
        .map(|r| r.is_satisfied_tristate)
}

#[test]
fn strict_rfc3339_parsing_matches_recorded_behaviour() {
    let (mut ok, mut equal_checked, mut err) = (0, 0, 0);
    for entry in corpus("parse_corpus") {
        let input = entry["input"].as_str().unwrap();
        let strict = &entry["strict"];
        if let Some(expected) = strict.get("ok") {
            let utc = expected["utc"].as_str().unwrap();
            if is_leap(utc) {
                continue;
            }
            ok += 1;
            // accepted by both the boolean and three-valued temporal operators
            let b = eval(BinaryOp::Before, input, "9999-12-31T23:59:59Z", false);
            assert!(b.is_ok(), "{input:?} must be accepted: {b:?}");
            let t = eval(BinaryOp::Before, input, "9999-12-31T23:59:59Z", true).unwrap();
            assert!(t.is_some(), "{input:?} must not be Null in 3VL");
            // when the UTC instant is itself expressible in strict form, the
            // input and that instant must compare equal (offset handled)
            let year_ok = utc.len() == 20 || utc.contains('.');
            let plain_year =
                utc.chars().next().is_some_and(|c| c.is_ascii_digit()) && utc.find('-') == Some(4);
            if year_ok && plain_year {
                for (op, want) in [(BinaryOp::Before, false), (BinaryOp::After, false)] {
                    assert_eq!(
                        eval(op.clone(), input, utc, false).unwrap(),
                        Some(want),
                        "{input:?} vs {utc}"
                    );
                    assert_eq!(
                        eval(op.clone(), utc, input, false).unwrap(),
                        Some(want),
                        "{utc} vs {input:?}"
                    );
                }
                equal_checked += 1;
            }
        } else {
            let msg = strict["err"].as_str().unwrap();
            assert_eq!(
                eval(BinaryOp::Before, input, "2024-01-01T00:00:00Z", false),
                Err(format!("Failed to parse left timestamp '{input}': {msg}"))
            );
            assert_eq!(
                eval(BinaryOp::After, "2024-01-01T00:00:00Z", input, false),
                Err(format!("Failed to parse right timestamp '{input}': {msg}"))
            );
            // three-valued logic: invalid timestamp is Null
            assert_eq!(
                eval(BinaryOp::Before, input, "2024-01-01T00:00:00Z", true),
                Ok(None)
            );
            err += 1;
        }
    }
    assert!(
        ok > 100 && equal_checked > 100 && err > 1500,
        "ok={ok} eq={equal_checked} err={err}"
    );
}

#[test]
fn strict_ordering_is_by_instant_not_text() {
    // +05:30 offset: 03:00+05:30 is 21:30Z the previous day
    assert_eq!(
        eval(
            BinaryOp::Before,
            "2024-01-02T03:00:00+05:30",
            "2024-01-01T22:00:00Z",
            false
        ),
        Ok(Some(true))
    );
    assert_eq!(
        eval(
            BinaryOp::After,
            "2024-01-02T03:00:00+05:30",
            "2024-01-01T21:00:00Z",
            false
        ),
        Ok(Some(true))
    );
    // sub-second ordering
    assert_eq!(
        eval(
            BinaryOp::Before,
            "2024-01-01T00:00:00.999999999Z",
            "2024-01-01T00:00:01Z",
            false
        ),
        Ok(Some(true))
    );
    assert_eq!(
        eval(
            BinaryOp::After,
            "2024-01-01T00:00:00.000000001Z",
            "2024-01-01T00:00:00Z",
            false
        ),
        Ok(Some(true))
    );
}

// ---------------------------------------------------------------------------
// Durations: serde shape and arithmetic-visible behaviour
// ---------------------------------------------------------------------------

#[test]
fn fact_requirement_max_age_serde_shape_is_pinned() {
    use domainforge_core::authority::FactRequirement;
    let cases = [
        (json!([0, 0]), json!([0, 0])),
        (json!([3600, 0]), json!([3600, 0])),
        (json!([1, 500_000_000]), json!([1, 500_000_000])),
        (json!([-90, 0]), json!([-90, 0])),
        (json!([0, 1]), json!([0, 1])),
        (json!([-2, 500_000_000]), json!([-2, 500_000_000])),
        (json!([86_400, 999_999_999]), json!([86_400, 999_999_999])),
    ];
    for (input, expected) in cases {
        let req: FactRequirement = serde_json::from_value(json!({
            "fact_path": "p",
            "allowed_source_classes": ["system_of_record"],
            "max_age": input,
        }))
        .unwrap_or_else(|e| panic!("{input}: {e}"));
        assert_eq!(serde_json::to_value(&req).unwrap()["max_age"], expected);
    }
}

#[test]
fn fact_requirement_freshness_boundary_is_pinned() {
    use domainforge_core::authority::FactRequirement;
    let req: FactRequirement = serde_json::from_value(json!({
        "fact_path": "customer.credit_hold",
        "allowed_source_classes": ["system_of_record"],
        "max_age": [60, 0],
    }))
    .unwrap();
    let env: FactEnvelope =
        serde_json::from_value(envelope_json("2024-01-01T00:00:00Z", None)).unwrap();
    let at = |s: &str| -> bool {
        // obtain a "now" of the crate's own timestamp type via serde
        let now_env: FactEnvelope = serde_json::from_value(envelope_json(s, None)).unwrap();
        req.is_satisfied_by(&env, &now_env.observed_at)
    };
    // exactly max_age old: still satisfied, one nanosecond older: not
    assert!(at("2024-01-01T00:00:59.999999999Z"));
    assert!(at("2024-01-01T00:01:00Z"));
    assert!(!at("2024-01-01T00:01:00.000000001Z"));
}

#[test]
fn metric_duration_serde_bytes_are_pinned() {
    use domainforge_core::parse_to_graph;
    use domainforge_core::primitives::Metric;
    let graph = parse_to_graph(
        r#"
        Metric "serde_metric" as: count(flows)
            @refresh_interval 90 "seconds"
            @window 2 "hours"
    "#,
    )
    .unwrap();
    let metric = graph
        .get_metric(&domainforge_core::ConceptId::from_concept(
            "default",
            "serde_metric",
        ))
        .unwrap()
        .clone();
    let mut value = serde_json::to_value(&metric).unwrap();
    assert_eq!(value["refresh_interval"], json!([90, 0]));
    assert_eq!(value["window"], json!([7200, 0]));
    // sub-second and negative durations round-trip through the same shape
    value["refresh_interval"] = json!([1, 500_000_000]);
    value["window"] = json!([-2, 500_000_000]);
    let restored: Metric = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(&restored).unwrap(), value);
    // whole-second views used by the printers and CALM export
    let calm = domainforge_core::calm::export(&graph).unwrap();
    let text = calm.to_string();
    assert!(text.contains("\"sea:refresh_interval\":90"), "{text}");
    assert!(text.contains("\"sea:window\":7200"), "{text}");
}

// ---------------------------------------------------------------------------
// SOURCE_DATE_EPOCH-driven timestamps (CALM export and protobuf projection)
// ---------------------------------------------------------------------------

#[test]
fn source_date_epoch_timestamps_are_pinned() {
    use domainforge_core::projection::protobuf::ProtobufEngine;
    // (epoch, expected to_rfc3339); one test fn because it mutates the process env.
    let cases: &[(&str, &str)] = &[
        ("0", "1970-01-01T00:00:00+00:00"),
        ("1", "1970-01-01T00:00:01+00:00"),
        ("-1", "1969-12-31T23:59:59+00:00"),
        ("1700000000", "2023-11-14T22:13:20+00:00"),
        ("951782400", "2000-02-29T00:00:00+00:00"),
        ("253402300799", "9999-12-31T23:59:59+00:00"),
        ("253402300800", "+10000-01-01T00:00:00+00:00"),
        ("-62167219200", "0000-01-01T00:00:00+00:00"),
        ("-62167219201", "-0001-12-31T23:59:59+00:00"),
    ];
    let graph = Graph::new();
    for (epoch, expected) in cases {
        std::env::set_var("SOURCE_DATE_EPOCH", epoch);
        let calm = domainforge_core::calm::export(&graph).unwrap();
        assert_eq!(
            calm["metadata"]["sea:timestamp"],
            json!(expected),
            "calm export for epoch {epoch}"
        );
        let proto = ProtobufEngine::project(&graph, "", "pkg");
        assert_eq!(
            proto.metadata.generated_at, *expected,
            "protobuf generated_at for epoch {epoch}"
        );
    }
    // out of chrono's representable range: protobuf leaves it empty
    std::env::set_var("SOURCE_DATE_EPOCH", "99999999999999999");
    let proto = ProtobufEngine::project(&graph, "", "pkg");
    assert_eq!(proto.metadata.generated_at, "");
    // not a number: protobuf leaves it empty
    std::env::set_var("SOURCE_DATE_EPOCH", "soon");
    let proto = ProtobufEngine::project(&graph, "", "pkg");
    assert_eq!(proto.metadata.generated_at, "");
    std::env::remove_var("SOURCE_DATE_EPOCH");
    // without an epoch, CALM falls back to the wall clock: `+00:00` shape
    let calm = domainforge_core::calm::export(&graph).unwrap();
    assert_rfc3339_plus00(calm["metadata"]["sea:timestamp"].as_str().unwrap());
}

// ---------------------------------------------------------------------------
// `now()` defaults: exact shape of the wall-clock formats
// ---------------------------------------------------------------------------

fn assert_rfc3339_plus00(text: &str) {
    // chrono `to_rfc3339()`: +00:00 offset, AutoSi fraction of 0, 3, 6 or 9 digits
    let re = regex::Regex::new(r"^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(\.(\d{3}|\d{6}|\d{9}))?\+00:00$")
        .unwrap();
    assert!(re.is_match(text), "unexpected wall-clock shape: {text}");
}

#[test]
fn semantic_pack_builder_created_at_shape() {
    use domainforge_core::semantic_pack::*;
    let input = PackBuildInput {
        org_id: "o".into(),
        domain_id: "d".into(),
        pack_version: "1.0.0".into(),
        meaning_version: "1.0.0".into(),
        approval: ApprovalState::Candidate,
        concepts: vec![],
        relations: vec![],
        metrics: vec![],
        dimensions: vec![],
        units: vec![],
        aliases: vec![],
        mapping_rules: vec![],
        review_records: vec![],
        previous_pack: None,
        allow_first_approved_version: false,
        source_graph_hash: "sha256:test".into(),
    };
    let out = build_semantic_pack(input).unwrap_or_else(|d| panic!("{d:?}"));
    assert_rfc3339_plus00(&out.pack.created_at);
}

#[cfg(feature = "cli")]
#[test]
fn cli_envelope_created_at_is_micros_with_z() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/envelope/valid-basic.sea");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_domainforge"))
        .args(["envelope", "--emit", "cep"])
        .arg(&path)
        .output()
        .unwrap();
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    let created = envelope["created_at"].as_str().unwrap();
    let re = regex::Regex::new(r"^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d{6}Z$").unwrap();
    assert!(re.is_match(created), "unexpected created_at: {created}");
}
