//! Direct tests of `timefmt` against the corpora recorded from chrono
//! (see `timestamp_golden_tests.rs`), plus the one documented divergence:
//! leap seconds, which `time` cannot represent.

use domainforge_core::timefmt::{self, ParseError};
use serde_json::Value;
use time::{OffsetDateTime, UtcOffset};

fn corpus(name: &str) -> Vec<Value> {
    let path = format!(
        "{}/tests/fixtures/timestamp_golden/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn every_recorded_format_matches_chrono_byte_for_byte() {
    let mut n = 0;
    for e in corpus("format_corpus") {
        if e.get("unrepresentable").is_some() {
            // chrono could not represent it; timefmt must not either
            assert!(timefmt::from_unix_timestamp(e["epoch_secs"].as_i64().unwrap()).is_none());
            continue;
        }
        let secs = e["epoch_secs"].as_i64().unwrap();
        let nanos = e["nanos"].as_u64().unwrap() as i64;
        let dt = timefmt::from_unix_timestamp(secs).unwrap() + time::Duration::nanoseconds(nanos);
        assert_eq!(timefmt::to_rfc3339(dt), e["to_rfc3339"].as_str().unwrap());
        assert_eq!(timefmt::to_rfc3339_z(dt), e["autosi_z"].as_str().unwrap());
        assert_eq!(
            timefmt::to_rfc3339_micros_z(dt),
            e["micros_z"].as_str().unwrap()
        );
        assert_eq!(
            timefmt::to_rfc3339_millis_z(dt),
            e["millis_z"].as_str().unwrap()
        );
        assert_eq!(
            timefmt::to_rfc3339_secs_z(dt),
            e["secs_z"].as_str().unwrap()
        );
        n += 1;
    }
    assert!(n >= 100);
}

#[test]
fn non_utc_offsets_format_as_utc() {
    let dt = timefmt::parse_rfc3339("2024-01-02T03:04:05.5+05:30").unwrap();
    assert_eq!(timefmt::to_rfc3339(dt), "2024-01-01T21:34:05.500+00:00");
    assert_eq!(timefmt::to_rfc3339_z(dt), "2024-01-01T21:34:05.500Z");
}

#[test]
fn every_recorded_parse_matches_chrono() {
    let mut leap = 0;
    for e in corpus("parse_corpus") {
        let input = e["input"].as_str().unwrap();
        // strict
        let strict = timefmt::parse_rfc3339(input);
        match e["strict"].get("ok") {
            Some(ok) => {
                let expected = ok["utc"].as_str().unwrap();
                let dt = strict.unwrap_or_else(|er| panic!("{input:?}: {er}"));
                assert_eq!(
                    dt.offset().whole_seconds() as i64,
                    ok["offset_secs"].as_i64().unwrap()
                );
                if expected.contains(":60") {
                    leap += 1;
                    assert_eq!(
                        timefmt::parse_rfc3339_exact(input),
                        Err(ParseError::LeapSecond)
                    );
                } else {
                    assert_eq!(timefmt::to_rfc3339_z(dt), expected, "{input:?}");
                    assert_eq!(timefmt::parse_rfc3339_exact(input).unwrap(), dt);
                }
            }
            None => assert_eq!(
                strict.unwrap_err().to_string(),
                e["strict"]["err"].as_str().unwrap(),
                "{input:?}"
            ),
        }
        // relaxed
        let relaxed = timefmt::parse_relaxed(input);
        match e["relaxed"].get("ok") {
            Some(ok) if ok.as_str().unwrap().contains(":60") => {
                assert_eq!(relaxed, Err(ParseError::LeapSecond), "{input:?}");
            }
            Some(ok) => assert_eq!(
                timefmt::to_rfc3339_z(relaxed.unwrap_or_else(|er| panic!("{input:?}: {er}"))),
                ok.as_str().unwrap(),
                "{input:?}"
            ),
            None => assert_eq!(
                relaxed.unwrap_err().to_string(),
                e["relaxed"]["err"].as_str().unwrap(),
                "{input:?}"
            ),
        }
    }
    assert!(leap > 0, "corpus must contain leap-second inputs");
}

/// DOCUMENTED DIVERGENCE: chrono kept `23:59:60` as a leap second and wrote
/// `:60` back out. `time` cannot hold it. Comparison parsing folds it into the
/// next second; anything stored rejects it instead of emitting different bytes.
#[test]
fn leap_second_is_folded_for_comparison_and_rejected_for_storage() {
    let leap = timefmt::parse_rfc3339("2016-12-31T23:59:60Z").unwrap();
    assert_eq!(timefmt::to_rfc3339_z(leap), "2017-01-01T00:00:00Z");
    assert_eq!(
        timefmt::parse_rfc3339_exact("2016-12-31T23:59:60Z"),
        Err(ParseError::LeapSecond)
    );
    assert_eq!(
        timefmt::parse_relaxed("2016-12-31T23:59:60Z"),
        Err(ParseError::LeapSecond)
    );
    // typed values and fact envelopes reject it with a clear message
    let err = serde_json::from_value::<domainforge_core::application::contract::TypedValue>(
        serde_json::json!({"kind": "timestamp", "data": "2016-12-31T23:59:60Z"}),
    )
    .unwrap_err();
    assert!(err.to_string().contains("leap seconds"), "{err}");
    // a leap second compares as the next second in policy temporal operators
    use domainforge_core::policy::{BinaryOp, Expression, Policy};
    let p = Policy::new(
        "t",
        Expression::binary(
            BinaryOp::After,
            Expression::literal("2016-12-31T23:59:60Z"),
            Expression::literal("2016-12-31T23:59:59Z"),
        ),
    );
    let r = p
        .evaluate_with_mode(&domainforge_core::Graph::new(), false)
        .unwrap();
    assert_eq!(r.is_satisfied_tristate, Some(true));
}

#[test]
fn duration_serde_bounds_match_chrono() {
    use domainforge_core::timefmt::serde_duration_option;
    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct D(#[serde(with = "serde_duration_option")] Option<time::Duration>);
    for ok in [
        "[0,0]",
        "[9223372036854775,807000000]",
        "[-9223372036854776,193000000]",
    ] {
        let d: D = serde_json::from_str(ok).unwrap();
        assert_eq!(serde_json::to_string(&d).unwrap(), ok);
    }
    for bad in [
        "[9223372036854775,807000001]",
        "[9223372036854776,0]",
        "[-9223372036854776,192999999]",
        "[0,1000000000]",
        "[0,-1]",
    ] {
        let e = serde_json::from_str::<D>(bad).unwrap_err().to_string();
        assert!(e.contains("TimeDelta out of bounds"), "{bad}: {e}");
    }
}

#[test]
fn now_helpers_are_utc_and_well_shaped() {
    let before = OffsetDateTime::now_utc();
    let now = timefmt::now();
    assert_eq!(now.offset(), UtcOffset::UTC);
    assert!(now >= before);
    assert!(timefmt::now_rfc3339().ends_with("+00:00"));
    let m = timefmt::now_micros_z();
    assert!(m.ends_with('Z') && m.len() == 27, "{m}");
}
