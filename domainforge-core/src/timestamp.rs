use time::OffsetDateTime;

fn format_utc(timestamp: OffsetDateTime, micros: bool) -> String {
    let timestamp = timestamp.to_offset(time::UtcOffset::UTC);
    let nanos = timestamp.nanosecond();
    let mut result = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        timestamp.year(),
        u8::from(timestamp.month()),
        timestamp.day(),
        timestamp.hour(),
        timestamp.minute(),
        timestamp.second(),
    );
    if micros {
        result.push_str(&format!(".{:06}Z", nanos / 1_000));
    } else {
        if nanos != 0 {
            if nanos.is_multiple_of(1_000_000) {
                result.push_str(&format!(".{:03}", nanos / 1_000_000));
            } else if nanos.is_multiple_of(1_000) {
                result.push_str(&format!(".{:06}", nanos / 1_000));
            } else {
                result.push_str(&format!(".{nanos:09}"));
            }
        }
        result.push_str("+00:00");
    }
    result
}

pub(crate) fn now_auto_si() -> String {
    format_utc(OffsetDateTime::now_utc(), false)
}

#[cfg(feature = "cli")]
pub(crate) fn now_micros_z() -> String {
    format_utc(OffsetDateTime::now_utc(), true)
}

// Chrono represents a leap second as second 59 with nanoseconds >= 1e9.
// Keep the anchor and leap flag separate: POSIX timestamps alone alias the
// leap second with the following second or lose its fractional precision.
/// A UTC-normalized timestamp preserving Chrono-compatible leap seconds.
///
/// Strict parsing retains the input offset for inspection. Deserialization,
/// equality, ordering, and serialization use the normalized UTC value.
#[derive(Clone, Copy, Debug)]
pub struct Timestamp {
    utc_anchor: OffsetDateTime,
    nanos: u32,
    leap: bool,
    offset_seconds: i32,
}

impl Timestamp {
    /// Parse the RFC 3339 forms accepted by Chrono's RFC 3339 parser.
    pub fn parse(input: &str) -> Option<Self> {
        let b = input.as_bytes();
        if b.len() < 20
            || b.get(4) != Some(&b'-')
            || b.get(7) != Some(&b'-')
            || !matches!(b.get(10), Some(b'T' | b't' | b' '))
            || b.get(13) != Some(&b':')
            || b.get(16) != Some(&b':')
        {
            return None;
        }
        let year = digits(&b[0..4])? as i32;
        let month = time::Month::try_from(digits(&b[5..7])? as u8).ok()?;
        let day = digits(&b[8..10])? as u8;
        let hour = digits(&b[11..13])? as u8;
        let minute = digits(&b[14..16])? as u8;
        let second = digits(&b[17..19])? as u8;
        let (nanos, zone) = fraction_and_zone(&input[19..])?;
        let offset = parse_offset(zone, false)?;
        let date = time::Date::from_calendar_date(year, month, day).ok()?;
        Self::from_local(date, [hour, minute, second], nanos, offset)
    }

    fn from_local(
        date: time::Date,
        [hour, minute, second]: [u8; 3],
        nanos: u32,
        offset_seconds: i32,
    ) -> Option<Self> {
        if second > 60 {
            return None;
        }
        let leap = second == 60;
        let local = date.with_hms(hour, minute, second.min(59)).ok()?;
        let offset = time::UtcOffset::from_whole_seconds(offset_seconds).ok()?;
        Some(Self {
            utc_anchor: local.assume_offset(offset).to_offset(time::UtcOffset::UTC),
            nanos,
            leap,
            offset_seconds,
        })
    }

    fn parse_wire(input: &str) -> Option<Self> {
        let mut timestamp = Self::parse(input).or_else(|| Self::parse_relaxed_wire(input))?;
        timestamp.offset_seconds = 0;
        Some(timestamp)
    }

    // Chrono's serde path uses its relaxed parser and bounds both the local
    // date and the resulting UTC date to the Chrono calendar's year range.
    fn parse_relaxed_wire(input: &str) -> Option<Self> {
        const WIRE_YEARS: std::ops::RangeInclusive<i32> = -262_143..=262_142;

        let input = input.trim_start();
        let sign = usize::from(input.starts_with(['+', '-']));
        let width = if sign == 0 { 4 } else { usize::MAX };
        let (year_digits, rest) = split_digits(&input[sign..], width);
        if year_digits.is_empty() {
            return None;
        }
        let year = input[..sign + year_digits.len()].parse::<i32>().ok()?;
        if !WIRE_YEARS.contains(&year) {
            return None;
        }

        let (month, rest) = wire_component(rest.trim_start().strip_prefix('-')?)?;
        let (day, rest) = wire_component(rest.trim_start().strip_prefix('-')?)?;
        let (hour, rest) = wire_component(rest.strip_prefix(['T', 't', ' '])?)?;
        let (minute, rest) = wire_component(rest.trim_start().strip_prefix(':')?)?;
        let (second, rest) = wire_component(rest.trim_start().strip_prefix(':')?)?;
        let (nanos, zone) = fraction_and_zone(rest)?;
        let offset = parse_offset(zone, true)?;
        let date =
            time::Date::from_calendar_date(year, time::Month::try_from(month).ok()?, day).ok()?;
        let timestamp = Self::from_local(date, [hour, minute, second], nanos, offset)?;
        WIRE_YEARS
            .contains(&timestamp.utc_anchor.year())
            .then_some(timestamp)
    }

    /// Return the input offset in seconds east of UTC.
    pub fn offset_seconds(self) -> i32 {
        self.offset_seconds
    }

    /// Format in UTC using Chrono AutoSi precision and a trailing `Z`.
    pub fn to_utc_rfc3339(self) -> String {
        let t = self.utc_anchor;
        let year = t.year();
        let year = if (0..=9999).contains(&year) {
            format!("{year:04}")
        } else {
            format!("{year:+05}")
        };
        let mut out = format!(
            "{}-{:02}-{:02}T{:02}:{:02}:{:02}",
            year,
            u8::from(t.month()),
            t.day(),
            t.hour(),
            t.minute(),
            t.second() + u8::from(self.leap),
        );
        if self.nanos != 0 {
            if self.nanos.is_multiple_of(1_000_000) {
                out.push_str(&format!(".{:03}", self.nanos / 1_000_000));
            } else if self.nanos.is_multiple_of(1_000) {
                out.push_str(&format!(".{:06}", self.nanos / 1_000));
            } else {
                out.push_str(&format!(".{:09}", self.nanos));
            }
        }
        out.push('Z');
        out
    }

    /// Format in UTC with the `+00:00` suffix used by Chrono's `to_rfc3339`.
    pub fn to_rfc3339(self) -> String {
        let mut value = self.to_utc_rfc3339();
        value.pop();
        value.push_str("+00:00");
        value
    }

    fn ordering_key(self) -> (i64, bool, u32) {
        (self.utc_anchor.unix_timestamp(), self.leap, self.nanos)
    }
}

// Only used for bounded slices of at most nine ASCII digits.
fn digits(bytes: &[u8]) -> Option<u32> {
    bytes.iter().try_fold(0, |value, digit| {
        digit
            .is_ascii_digit()
            .then(|| value * 10 + u32::from(digit - b'0'))
    })
}

fn split_digits(input: &str, max: usize) -> (&str, &str) {
    let len = input
        .bytes()
        .take(max)
        .take_while(u8::is_ascii_digit)
        .count();
    input.split_at(len)
}

fn wire_component(input: &str) -> Option<(u8, &str)> {
    let (value, rest) = split_digits(input.trim_start(), 2);
    Some((value.parse().ok()?, rest))
}

fn fraction_and_zone(input: &str) -> Option<(u32, &str)> {
    let Some(input) = input.strip_prefix('.') else {
        return Some((0, input));
    };
    let (fraction, rest) = split_digits(input, 9);
    if fraction.is_empty() {
        return None;
    }
    let nanos = digits(fraction.as_bytes())? * 10_u32.pow((9 - fraction.len()) as u32);
    Some((nanos, rest.trim_start_matches(|c: char| c.is_ascii_digit())))
}

fn parse_offset(input: &str, relaxed: bool) -> Option<i32> {
    let input = if relaxed { input.trim() } else { input };
    if matches!(input, "Z" | "z") || (relaxed && input.eq_ignore_ascii_case("UTC")) {
        return Some(0);
    }
    let (sign, offset) = if let Some(offset) = input.strip_prefix('+') {
        (1, offset)
    } else {
        (-1, input.strip_prefix(['-', '−'])?)
    };
    let hours = digits(offset.as_bytes().get(..2)?)?;
    let minutes = if relaxed {
        offset[2..].trim_start_matches(|c: char| c == ':' || c.is_whitespace())
    } else {
        offset[2..].strip_prefix(':')?
    };
    if minutes.len() != 2 {
        return None;
    }
    let minutes = digits(minutes.as_bytes())?;
    if hours > 23 || minutes > 59 {
        return None;
    }
    Some(sign * (hours * 3600 + minutes * 60) as i32)
}

impl PartialEq for Timestamp {
    fn eq(&self, other: &Self) -> bool {
        self.ordering_key() == other.ordering_key()
    }
}
impl Eq for Timestamp {}
impl PartialOrd for Timestamp {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Timestamp {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ordering_key().cmp(&other.ordering_key())
    }
}

impl serde::Serialize for Timestamp {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_utc_rfc3339())
    }
}

impl<'de> serde::Deserialize<'de> for Timestamp {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct TimestampVisitor;

        impl serde::de::Visitor<'_> for TimestampVisitor {
            type Value = Timestamp;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an RFC 3339 formatted date and time string")
            }

            fn visit_str<E: serde::de::Error>(self, input: &str) -> Result<Timestamp, E> {
                Timestamp::parse_wire(input).ok_or_else(|| E::custom("invalid timestamp"))
            }
        }

        deserializer.deserialize_str(TimestampVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::SecondsFormat;

    #[test]
    fn adapter_matches_chrono_parsing_values_and_order() {
        let inputs = [
            "1969-12-31T23:59:59.999999999Z",
            "1970-01-01T01:30:00+01:30",
            "2000-02-29t23:59:59.000001z",
            "2024-01-15 12:34:60.123456789987−05:30",
            "2024-01-15T18:04:60.123456789Z",
            "2016-12-31T23:59:59.999999999Z",
            "2016-12-31T23:59:60Z",
            "2017-01-01T05:29:60.123456789+05:30",
            "2017-01-01T00:00:00Z",
            "0000-01-01T00:00:00+23:59",
            "9999-12-31T23:59:59-23:59",
        ];
        let mut values = Vec::new();
        for input in inputs {
            let expected = chrono::DateTime::parse_from_rfc3339(input).unwrap();
            let actual = Timestamp::parse(input).unwrap();
            assert_eq!(
                actual.offset_seconds(),
                expected.offset().local_minus_utc(),
                "{input}"
            );
            assert_eq!(
                actual.utc_anchor.unix_timestamp(),
                expected.timestamp(),
                "{input}"
            );
            assert_eq!(
                actual.nanos + if actual.leap { 1_000_000_000 } else { 0 },
                expected.timestamp_subsec_nanos(),
                "{input}"
            );
            assert_eq!(
                actual.to_utc_rfc3339(),
                expected
                    .with_timezone(&chrono::Utc)
                    .to_rfc3339_opts(SecondsFormat::AutoSi, true),
                "{input}"
            );
            values.push((actual, expected));
        }
        for (actual_a, expected_a) in &values {
            for (actual_b, expected_b) in &values {
                assert_eq!(actual_a.cmp(actual_b), expected_a.cmp(expected_b));
            }
        }
    }

    #[test]
    fn adapter_rejects_the_same_malformed_inputs() {
        for input in [
            "",
            "2024-01-01X00:00:00Z",
            "2024-02-30T00:00:00Z",
            "2024-01-01T24:00:00Z",
            "2024-01-01T00:00:61Z",
            "2024-01-01T00:00:00.Z",
            "2024-01-01T00:00:00+24:00",
            "2024-01-01T00:00:00+00:60",
            "2024-01-01T00:00:00Zextra",
        ] {
            assert_eq!(
                Timestamp::parse(input).is_some(),
                chrono::DateTime::parse_from_rfc3339(input).is_ok(),
                "{input}"
            );
        }
    }

    #[test]
    fn timestamp_serde_matches_chrono_utc_wire_format() {
        let relaxed = [
            "+10000-01-01T00:00:00Z",
            "-0001-12-31T00:01:00Z",
            " \u{2003}+82701 - 5 - 6t 15 : 9 : 60.898989898989 \u{2003}uTc\t",
            "1-2-3  4:5:6z",
            "+000000000000000000000000000001-2-3T4:5:6 UTC",
            "-0-2-29T4:5:6Z",
            "2024-1-2T3:4:5+0130",
            "2024-1-2T3:4:60.1−01 :\u{2003}: 30",
            "2024-01-01T00:00:00z\u{2003}",
            "-262143-01-01T00:00:00Z",
            "-262143-01-01T23:59:60+23:59",
            "+262142-12-31T23:59:60.999999999Z",
        ];
        for input in relaxed {
            assert!(
                Timestamp::parse(input).is_none(),
                "strict parser: {input:?}"
            );
        }
        for input in [
            "2017-01-01T05:29:60.123456789+05:30",
            "1969-12-31T23:59:59.001Z",
            "0000-01-01T00:00:00+23:59",
            "9999-12-31T23:59:59-23:59",
        ]
        .into_iter()
        .chain(relaxed)
        {
            let wire = serde_json::to_string(input).unwrap();
            let expected: chrono::DateTime<chrono::Utc> = serde_json::from_str(&wire).unwrap();
            let actual: Timestamp = serde_json::from_str(&wire).unwrap();
            assert_eq!(actual.offset_seconds(), 0, "{input:?}");
            assert_eq!(
                actual.utc_anchor.unix_timestamp(),
                expected.timestamp(),
                "{input:?}"
            );
            assert_eq!(
                actual.nanos + u32::from(actual.leap) * 1_000_000_000,
                expected.timestamp_subsec_nanos(),
                "{input:?}"
            );
            let serialized = serde_json::to_string(&actual).unwrap();
            assert_eq!(
                serialized,
                serde_json::to_string(&expected).unwrap(),
                "{input:?}"
            );
            assert_eq!(actual.to_rfc3339(), expected.to_rfc3339(), "{input:?}");
            assert_eq!(
                serde_json::from_str::<Timestamp>(&serialized).unwrap(),
                actual
            );
            assert_eq!(
                serde_json::from_str::<chrono::DateTime<chrono::Utc>>(&serialized).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn timestamp_serde_rejects_the_same_out_of_grammar_inputs() {
        for input in [
            "not a timestamp",
            "2024-02-30T00:00:00Z",
            "2024-1-1 T1:2:3Z",
            "2024-1-1\t1:2:3Z",
            "2024-1-1T1:2:3 .1Z",
            "2024-1-1T1:2:3.Z",
            "2024-1-1T1:2:3,1Z",
            "2024-1-1T1:2:3GMT",
            "2024-1-1T1:2:3UTCx",
            "2024-1-1T1:2:3+01",
            "2024-1-1T1:2:3+1:30",
            "2024-1-1T1:2:3+ 01:30",
            "2024-1-1T1:2:3+01:3",
            "2024-1-1T1:2:3+24:00",
            "2024-1-1T1:2:3+00:60",
            "10000-1-1T1:2:3Z",
            "+ 10000-1-1T1:2:3Z",
            "−1-1-1T1:2:3Z",
            "+9223372036854775808-1-1T1:2:3Z",
            "-262144-12-31T23:59:59-00:01",
            "+262143-01-01T00:00:00+00:01",
            "-262143-01-01T00:00:00+00:01",
            "+262142-12-31T23:59:60-00:01",
        ] {
            let wire = serde_json::to_string(input).unwrap();
            assert!(
                serde_json::from_str::<chrono::DateTime<chrono::Utc>>(&wire).is_err(),
                "{input:?}"
            );
            assert!(
                serde_json::from_str::<Timestamp>(&wire).is_err(),
                "{input:?}"
            );
        }
        for wire in ["123", "null", "[]"] {
            assert!(serde_json::from_str::<chrono::DateTime<chrono::Utc>>(wire).is_err());
            assert!(serde_json::from_str::<Timestamp>(wire).is_err());
        }
    }

    #[test]
    fn matches_chrono_for_fractional_precision_and_pre_epoch() {
        for seconds in [-1, 0, 1, 1_700_000_000] {
            for nanos in [
                0,
                1,
                1_000,
                1_000_000,
                123_000_000,
                123_456_000,
                123_456_789,
                999_999_999,
            ] {
                let time = OffsetDateTime::from_unix_timestamp(seconds)
                    .unwrap()
                    .replace_nanosecond(nanos)
                    .unwrap();
                let chrono = chrono::DateTime::from_timestamp(seconds, nanos).unwrap();
                assert_eq!(format_utc(time, false), chrono.to_rfc3339());
                assert_eq!(
                    format_utc(time, true),
                    chrono.to_rfc3339_opts(SecondsFormat::Micros, true)
                );
            }
        }
    }
}
