//! TEMPORARY (removed in the chrono -> time swap commit): regenerates the
//! timestamp golden corpora from the real chrono implementation.
//! `cargo test --features cli --test timestamp_golden_gen -- --ignored`
use chrono::{DateTime, FixedOffset, SecondsFormat, Utc};
use serde_json::{json, Value};

fn inputs() -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    let dates = [
        "2024-01-02",
        "2024-02-29",
        "2023-02-29",
        "0000-01-01",
        "9999-12-31",
        "1969-12-31",
        "1970-01-01",
        "2024-1-2",
        "2024-13-01",
        "2024-00-10",
        "2024-04-31",
        "+12345-06-07",
        "-0001-01-01",
        "10000-01-01",
        "24-01-02",
        "2024/01/02",
        "2024-01-02 ",
    ];
    let seps = ["T", "t", " ", "_", "", "TT", "T "];
    let times = [
        "03:04:05",
        "00:00:00",
        "23:59:59",
        "23:59:60",
        "23:59:61",
        "24:00:00",
        "12:60:00",
        "12:00:61",
        "3:4:5",
        "03:04",
        "03:04:05.",
        "03:04:05,5",
        " 03:04:05",
        "03 : 04 : 05",
    ];
    let fracs = [
        "",
        ".5",
        ".123",
        ".123456",
        ".123456789",
        ".1234567890",
        ".000000001",
        ".999999999",
        ".9999999999999",
        ".0",
        ".000",
        ".001",
        ".000001",
        ".12",
    ];
    let offs = [
        "Z",
        "z",
        "+00:00",
        "-00:00",
        "+05:30",
        "-08:00",
        "+23:59",
        "-23:59",
        "+24:00",
        "+00:60",
        "+99:00",
        "+0530",
        "+05",
        "+05:3",
        "+5:30",
        "+05 30",
        "\u{2212}05:30",
        " Z",
        "UTC",
        "utc",
        " UTC",
        "Z ",
        "GMT",
        "",
        "+05:30:00",
        "+05:30Z",
    ];
    for d in dates {
        for t in times {
            v.push(format!("{d}T{t}Z"));
        }
    }
    for s in seps {
        v.push(format!("2024-01-02{s}03:04:05Z"));
    }
    for f in fracs {
        for o in offs {
            v.push(format!("2024-01-02T03:04:05{f}{o}"));
        }
    }
    // a deterministic pseudo-random mixture of all the pieces
    let mut x: u64 = 0x9E3779B97F4A7C15;
    let mut next = |n: usize| {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        (x % n as u64) as usize
    };
    for _ in 0..1500 {
        let d = dates[next(dates.len())];
        let s = ["T", "t", " ", "T", "T", "T"][next(6)];
        let t = times[next(times.len())];
        let f = fracs[next(fracs.len())];
        let o = offs[next(offs.len())];
        // only attach a fraction to a time that ends in seconds
        v.push(format!("{d}{s}{t}{f}{o}"));
    }
    for s in [
        "",
        " ",
        "x",
        "2024",
        "2024-01-02",
        "2024-01-02T",
        "2024-01-02T03:04:05",
        "not a date",
        "0",
        "1700000000",
        "2024-01-02T03:04:05Zjunk",
        "2024-01-02T03:04:05+05:30 ",
        "  2024-01-02T03:04:05Z",
        "2024-01-02T03:04:05Z\n",
        "2024-01-02T03:04:05.123456789123Z",
        "\u{ff12}024-01-02T03:04:05Z",
        "2024-01-02T03:04:05\u{ff3a}",
        "99999999999999999999-01-02T03:04:05Z",
        "+262143-12-31T23:59:59Z",
        "+262144-01-01T00:00:00Z",
        "-262143-01-01T00:00:00Z",
        "-262144-01-01T00:00:00Z",
        "+999999-01-01T00:00:00Z",
    ] {
        v.push(s.to_string());
    }
    v.sort();
    v.dedup();
    v
}

fn fmt_ok(dt: &DateTime<FixedOffset>) -> Value {
    let u = dt.with_timezone(&Utc);
    json!({
        "utc": u.to_rfc3339_opts(SecondsFormat::AutoSi, true),
        "offset_secs": dt.offset().local_minus_utc(),
    })
}

#[test]
#[ignore]
fn generate() {
    let mut parse = Vec::new();
    for s in inputs() {
        let strict = match DateTime::parse_from_rfc3339(&s) {
            Ok(dt) => json!({"ok": fmt_ok(&dt)}),
            Err(e) => json!({"err": e.to_string()}),
        };
        let relaxed = match s.parse::<DateTime<Utc>>() {
            Ok(dt) => json!({"ok": serde_json::to_value(dt).unwrap()}),
            Err(e) => json!({"err": e.to_string()}),
        };
        // the serde route (what FactEnvelope uses) must agree with FromStr
        let via_serde: Result<DateTime<Utc>, _> = serde_json::from_value(Value::String(s.clone()));
        assert_eq!(via_serde.is_ok(), relaxed.get("ok").is_some(), "{s:?}");
        parse.push(json!({"input": s, "strict": strict, "relaxed": relaxed}));
    }
    // formats of fixed instants
    let mut fmts = Vec::new();
    let secs = [
        0i64,
        1,
        -1,
        1_700_000_000,
        951_782_400,
        253_402_300_799,
        253_402_300_800,
        -62_167_219_200,
        -62_167_219_201,
        1_234_567_890,
        68_000_000_000_000,
    ];
    let nanos = [
        0u32,
        1_000_000,
        1,
        1_000,
        999_999_999,
        500_000_000,
        120_000_000,
        123_456_000,
        123_456_789,
        10,
        100,
    ];
    for sec in secs {
        for ns in nanos {
            if let Some(dt) = DateTime::<Utc>::from_timestamp(sec, ns) {
                fmts.push(json!({
                    "epoch_secs": sec, "nanos": ns,
                    "to_rfc3339": dt.to_rfc3339(),
                    "autosi_z": dt.to_rfc3339_opts(SecondsFormat::AutoSi, true),
                    "micros_z": dt.to_rfc3339_opts(SecondsFormat::Micros, true),
                    "millis_z": dt.to_rfc3339_opts(SecondsFormat::Millis, true),
                    "secs_z": dt.to_rfc3339_opts(SecondsFormat::Secs, true),
                    "serde": serde_json::to_value(dt).unwrap(),
                }));
            } else {
                fmts.push(json!({"epoch_secs": sec, "nanos": ns, "unrepresentable": true}));
            }
        }
    }
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/timestamp_golden"
    );
    std::fs::write(
        format!("{dir}/parse_corpus.json"),
        serde_json::to_string_pretty(&parse).unwrap() + "\n",
    )
    .unwrap();
    std::fs::write(
        format!("{dir}/format_corpus.json"),
        serde_json::to_string_pretty(&fmts).unwrap() + "\n",
    )
    .unwrap();
    // Duration serde shapes
    for d in [
        chrono::Duration::zero(),
        chrono::Duration::seconds(3600),
        chrono::Duration::milliseconds(1500),
        chrono::Duration::seconds(-90),
        chrono::Duration::nanoseconds(1),
    ] {
        println!("DURATION {} => {}", d, serde_json::to_string(&d).unwrap());
    }
    println!("count parse={} fmts={}", parse.len(), fmts.len());
}
