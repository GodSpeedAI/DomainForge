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

/// The timestamp did not match the RFC 3339 syntax accepted by stored records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseTimestampError {
    /// A date, time or offset component was outside its allowed range.
    OutOfRange,
    /// The input ended before the timestamp was complete.
    Incomplete,
    /// Unexpected characters appeared in the timestamp.
    Invalid,
    /// Extra characters followed a complete timestamp.
    Trailing,
    /// The local date and offset fall outside the UTC calendar.
    Impossible,
}

impl std::fmt::Display for ParseTimestampError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reason = match self {
            Self::OutOfRange => "input is out of range",
            Self::Incomplete => "premature end of input",
            Self::Invalid => "input contains invalid characters",
            Self::Trailing => "trailing input",
            Self::Impossible => "no possible date and time matching input",
        };
        f.write_str(reason)
    }
}

impl std::error::Error for ParseTimestampError {}

impl Timestamp {
    /// Current UTC timestamp.
    pub fn now_utc() -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            utc_anchor: now.replace_nanosecond(0).expect("valid nanoseconds"),
            nanos: now.nanosecond(),
            leap: false,
            offset_seconds: 0,
        }
    }

    /// Construct an instant within Chrono's historical calendar range.
    pub fn from_unix_seconds(seconds: i64) -> Option<Self> {
        let utc_anchor = OffsetDateTime::from_unix_timestamp(seconds).ok()?;
        if !(-262_143..=262_142).contains(&utc_anchor.year()) {
            return None;
        }
        Some(Self {
            utc_anchor,
            nanos: 0,
            leap: false,
            offset_seconds: 0,
        })
    }

    /// Add a signed duration, retaining Chrono's leap-second arithmetic.
    pub fn checked_add(self, duration: Duration) -> Option<Self> {
        let mut secs = self.utc_anchor.unix_timestamp() as i128;
        let mut frac = self.nanos as i128 + if self.leap { 1_000_000_000 } else { 0 };
        let ds = duration.0 / 1_000_000_000;
        let dn = duration.0 % 1_000_000_000;
        if self.leap {
            if ds > 0 || (dn > 0 && frac + dn >= 2_000_000_000) {
                frac -= 1_000_000_000;
            } else if ds < 0 {
                frac -= 1_000_000_000;
                secs += 1;
            } else {
                frac += dn;
                let leap = frac >= 1_000_000_000;
                return Some(Self {
                    utc_anchor: self.utc_anchor,
                    nanos: (frac % 1_000_000_000) as u32,
                    leap,
                    offset_seconds: 0,
                });
            }
        }
        secs += ds;
        frac += dn;
        secs += frac.div_euclid(1_000_000_000);
        frac = frac.rem_euclid(1_000_000_000);
        let mut result = Self::from_unix_seconds(i64::try_from(secs).ok()?)?;
        result.nanos = frac as u32;
        Some(result)
    }

    /// Subtract a signed duration.
    pub fn checked_sub(self, duration: Duration) -> Option<Self> {
        self.checked_add(-duration)
    }
    /// Parse the strict RFC 3339 grammar and report the historical diagnostic.
    pub fn parse_rfc3339(input: &str) -> Result<Self, ParseTimestampError> {
        detailed_parse_rfc3339(input)
    }

    /// Parse the RFC 3339 forms accepted by Chrono's RFC 3339 parser.
    pub fn parse(input: &str) -> Option<Self> {
        Self::parse_rfc3339(input).ok()
    }

    // Persisted timestamps used the relaxed grammar and normalized to UTC.
    fn parse_wire(input: &str) -> Result<Self, ParseTimestampError> {
        use ParseTimestampError::OutOfRange;

        let input = input.trim_start();
        let sign = usize::from(input.starts_with(['+', '-']));
        let width = if sign == 0 { 4 } else { usize::MAX };
        let (year_digits, rest) = split_digits(&input[sign..], width);
        wire_separator(&input[sign..], (!year_digits.is_empty()).then_some(rest))?;
        let year = input[..sign + year_digits.len()]
            .parse::<i32>()
            .map_err(|_| OutOfRange)?;

        let rest = rest.trim_start();
        let (month, rest) = wire_component(wire_separator(rest, rest.strip_prefix('-'))?)?;
        let rest = rest.trim_start();
        let (day, rest) = wire_component(wire_separator(rest, rest.strip_prefix('-'))?)?;
        let (hour, rest) =
            wire_component(wire_separator(rest, rest.strip_prefix(['T', 't', ' ']))?)?;
        let rest = rest.trim_start();
        let (minute, rest) = wire_component(wire_separator(rest, rest.strip_prefix(':'))?)?;
        let rest = rest.trim_start();
        let (second, rest) = wire_component(wire_separator(rest, rest.strip_prefix(':'))?)?;
        let (nanos, zone) = detailed_fraction_and_zone(rest)?;
        let offset = detailed_parse_offset(zone, true)?;
        let date = detailed_calendar_date(year, month, day)?;
        let (local, fraction) = detailed_local_time(date, [hour, minute, second], nanos)?;
        let mut timestamp = detailed_from_local(local, fraction, offset)?;
        timestamp.offset_seconds = 0;
        Ok(timestamp)
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

/// Signed nanosecond duration with Chrono's representable range and wire format.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Duration(i128);

impl Duration {
    const LIMIT: i128 = i64::MAX as i128 * 1_000_000;
    pub const fn zero() -> Self {
        Self(0)
    }
    pub fn try_seconds(value: i64) -> Option<Self> {
        Self::from_nanos(value as i128 * 1_000_000_000)
    }
    pub fn seconds(value: i64) -> Self {
        Self::try_seconds(value).expect("duration out of bounds")
    }
    pub fn milliseconds(value: i64) -> Self {
        Self::from_nanos(value as i128 * 1_000_000).expect("duration out of bounds")
    }
    pub fn minutes(value: i64) -> Self {
        Self::from_nanos(value as i128 * 60 * 1_000_000_000).expect("duration out of bounds")
    }
    pub fn hours(value: i64) -> Self {
        Self::from_nanos(value as i128 * 3600 * 1_000_000_000).expect("duration out of bounds")
    }
    pub fn days(value: i64) -> Self {
        Self::from_nanos(value as i128 * 86400 * 1_000_000_000).expect("duration out of bounds")
    }
    pub fn num_seconds(self) -> i64 {
        (self.0 / 1_000_000_000) as i64
    }
    pub fn num_milliseconds(self) -> i64 {
        (self.0 / 1_000_000) as i64
    }
    fn from_nanos(value: i128) -> Option<Self> {
        (-Self::LIMIT..=Self::LIMIT)
            .contains(&value)
            .then_some(Self(value))
    }
}
impl std::ops::Neg for Duration {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
    }
}
impl serde::Serialize for Duration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serde::Serialize::serialize(
            &(
                (self.0.div_euclid(1_000_000_000)) as i64,
                self.0.rem_euclid(1_000_000_000) as i32,
            ),
            serializer,
        )
    }
}
impl<'de> serde::Deserialize<'de> for Duration {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (seconds, nanos) = <(i64, i32) as serde::Deserialize>::deserialize(deserializer)?;
        if !(0..1_000_000_000).contains(&nanos) {
            return Err(serde::de::Error::custom("TimeDelta out of bounds"));
        }
        Self::from_nanos(seconds as i128 * 1_000_000_000 + nanos as i128)
            .ok_or_else(|| serde::de::Error::custom("TimeDelta out of bounds"))
    }
}
impl std::ops::Add<Duration> for Timestamp {
    type Output = Self;
    fn add(self, rhs: Duration) -> Self {
        self.checked_add(rhs).expect("timestamp out of bounds")
    }
}
impl std::ops::Sub<Duration> for Timestamp {
    type Output = Self;
    fn sub(self, rhs: Duration) -> Self {
        self.checked_sub(rhs).expect("timestamp out of bounds")
    }
}

fn detailed_parse_rfc3339(input: &str) -> Result<Timestamp, ParseTimestampError> {
    use ParseTimestampError::{Incomplete, Invalid};

    let bytes = input.as_bytes();
    if bytes.len() < 19 {
        return Err(Incomplete);
    }
    if bytes[4] != b'-' || bytes[7] != b'-' {
        return Err(Invalid);
    }
    let year = digits(&bytes[0..4]).ok_or(Invalid)? as i32;
    let month = digits(&bytes[5..7]).ok_or(Invalid)? as u8;
    let day = digits(&bytes[8..10]).ok_or(Invalid)? as u8;
    let date = detailed_calendar_date(year, month, day)?;
    if !matches!(bytes[10], b'T' | b't' | b' ') || bytes[13] != b':' || bytes[16] != b':' {
        return Err(Invalid);
    }
    let hour = digits(&bytes[11..13]).ok_or(Invalid)? as u8;
    let minute = digits(&bytes[14..16]).ok_or(Invalid)? as u8;
    let second = digits(&bytes[17..19]).ok_or(Invalid)? as u8;
    let (fraction, zone) = detailed_fraction_and_zone(&input[19..])?;
    let (local, fraction) = detailed_local_time(date, [hour, minute, second], fraction)?;
    let offset = detailed_parse_offset(zone, false)?;
    detailed_from_local(local, fraction, offset)
}

fn detailed_calendar_date(
    year: i32,
    month: u8,
    day: u8,
) -> Result<time::Date, ParseTimestampError> {
    use ParseTimestampError::OutOfRange;

    if !(-262_143..=262_142).contains(&year) {
        return Err(OutOfRange);
    }
    let month = time::Month::try_from(month).map_err(|_| OutOfRange)?;
    time::Date::from_calendar_date(year, month, day).map_err(|_| OutOfRange)
}

fn detailed_local_time(
    date: time::Date,
    [hour, minute, second]: [u8; 3],
    fraction: u32,
) -> Result<(time::PrimitiveDateTime, u32), ParseTimestampError> {
    use ParseTimestampError::OutOfRange;

    if second > 60 {
        return Err(OutOfRange);
    }
    let local = date
        .with_hms(hour, minute, second.min(59))
        .map_err(|_| OutOfRange)?;
    Ok((local, fraction + u32::from(second == 60) * 1_000_000_000))
}

fn detailed_from_local(
    local: time::PrimitiveDateTime,
    fraction: u32,
    offset: time::UtcOffset,
) -> Result<Timestamp, ParseTimestampError> {
    let utc = local.assume_offset(offset).to_offset(time::UtcOffset::UTC);
    if !(-262_143..=262_142).contains(&utc.year()) {
        return Err(ParseTimestampError::Impossible);
    }
    Ok(Timestamp {
        utc_anchor: utc,
        nanos: fraction % 1_000_000_000,
        leap: fraction >= 1_000_000_000,
        offset_seconds: offset.whole_seconds(),
    })
}

fn detailed_fraction_and_zone(input: &str) -> Result<(u32, &str), ParseTimestampError> {
    let Some(input) = input.strip_prefix('.') else {
        return Ok((0, input));
    };
    let (numeral, rest) = split_digits(input, 9);
    if numeral.is_empty() {
        return Err(if input.is_empty() {
            ParseTimestampError::Incomplete
        } else {
            ParseTimestampError::Invalid
        });
    }
    let precision = numeral.len();
    let fraction =
        digits(numeral.as_bytes()).expect("ASCII digits") * 10_u32.pow((9 - precision) as u32);
    Ok((
        fraction,
        rest.trim_start_matches(|c: char| c.is_ascii_digit()),
    ))
}

fn detailed_parse_offset(
    input: &str,
    relaxed: bool,
) -> Result<time::UtcOffset, ParseTimestampError> {
    use ParseTimestampError::{Incomplete, Invalid, OutOfRange, Trailing};

    let input = if relaxed { input.trim() } else { input };
    if relaxed
        && input
            .as_bytes()
            .get(..3)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"UTC"))
    {
        return if input.len() == 3 {
            Ok(time::UtcOffset::UTC)
        } else {
            Err(Trailing)
        };
    }
    if let Some(rest) = input.strip_prefix(['Z', 'z']) {
        return if rest.is_empty() {
            Ok(time::UtcOffset::UTC)
        } else {
            Err(Trailing)
        };
    }
    let (sign, offset) = match input.chars().next() {
        Some('+') => (1, &input[1..]),
        Some('-') => (-1, &input[1..]),
        Some('−') => (-1, &input['−'.len_utf8()..]),
        Some(_) => return Err(Invalid),
        None => return Err(Incomplete),
    };
    let hour = digits(offset.as_bytes().get(..2).ok_or(Incomplete)?).ok_or(Invalid)? as i8;
    let remainder = &offset[2..];
    let minutes = if relaxed {
        remainder.trim_start_matches(|c: char| c == ':' || c.is_whitespace())
    } else {
        remainder.strip_prefix(':').ok_or(if remainder.is_empty() {
            Incomplete
        } else {
            Invalid
        })?
    };
    let minute = digits(minutes.as_bytes().get(..2).ok_or(Incomplete)?).ok_or(Invalid)? as i8;
    if minute > 59 {
        return Err(OutOfRange);
    }
    if minutes.len() > 2 {
        return Err(Trailing);
    }
    if hour > 23 {
        return Err(OutOfRange);
    }
    time::UtcOffset::from_hms(sign * hour, sign * minute, 0).map_err(|_| OutOfRange)
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

fn wire_separator<'a>(input: &str, rest: Option<&'a str>) -> Result<&'a str, ParseTimestampError> {
    rest.ok_or(if input.is_empty() {
        ParseTimestampError::Incomplete
    } else {
        ParseTimestampError::Invalid
    })
}

fn wire_component(input: &str) -> Result<(u8, &str), ParseTimestampError> {
    let input = input.trim_start();
    let (value, rest) = split_digits(input, 2);
    wire_separator(input, (!value.is_empty()).then_some(rest))?;
    Ok((value.parse().expect("one or two decimal digits"), rest))
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
                Timestamp::parse_wire(input).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(TimestampVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_diagnostics_keep_the_historical_messages() {
        for (input, message) in [
            ("", "premature end of input"),
            ("2024-01-01X00:00:00Z", "input contains invalid characters"),
            ("2024-02-30T00:00:00Z", "input is out of range"),
            ("2024-01-01T00:00:00Zextra", "trailing input"),
        ] {
            assert_eq!(
                Timestamp::parse_rfc3339(input).unwrap_err().to_string(),
                message
            );
            assert!(Timestamp::parse(input).is_none());
        }
    }

    #[test]
    fn signed_duration_wire_and_bounds() {
        for wire in [
            "[0,0]",
            "[1,250000000]",
            "[-2,750000000]",
            "[9223372036854775,807000000]",
            "[-9223372036854776,193000000]",
        ] {
            let value: Duration = serde_json::from_str(wire).unwrap();
            assert_eq!(serde_json::to_string(&value).unwrap(), wire);
        }
        for wire in [
            "[9223372036854775,807000001]",
            "[-9223372036854776,192999999]",
            "[0,-1]",
            "[0,1000000000]",
        ] {
            assert!(serde_json::from_str::<Duration>(wire).is_err());
        }
    }

    #[test]
    fn leap_second_arithmetic_preserves_chrono_edges() {
        let leap = Timestamp::parse("2016-12-31T23:59:60.5Z").unwrap();
        assert_eq!(
            (leap - Duration::seconds(1)).to_utc_rfc3339(),
            "2016-12-31T23:59:59.500Z"
        );
        assert_eq!(
            (leap + Duration::seconds(1)).to_utc_rfc3339(),
            "2017-01-01T00:00:00.500Z"
        );
        assert_eq!(
            (leap - Duration::milliseconds(750)).to_utc_rfc3339(),
            "2016-12-31T23:59:59.750Z"
        );
        assert_eq!(
            (leap + Duration::milliseconds(500)).to_utc_rfc3339(),
            "2017-01-01T00:00:00Z"
        );
        assert_eq!(
            Timestamp::from_unix_seconds(0).unwrap().to_utc_rfc3339(),
            "1970-01-01T00:00:00Z"
        );
        assert!(Timestamp::from_unix_seconds(i64::MAX).is_none());
    }

    #[test]
    fn timestamp_strict_baseline_values_and_order() {
        let cases = [
            (
                "1969-12-31T23:59:59.999999999Z",
                0,
                "1969-12-31T23:59:59.999999999Z",
            ),
            ("1970-01-01T01:30:00+01:30", 5400, "1970-01-01T00:00:00Z"),
            (
                "2000-02-29t23:59:59.000001z",
                0,
                "2000-02-29T23:59:59.000001Z",
            ),
            (
                "2024-01-15 12:34:60.123456789987−05:30",
                -19800,
                "2024-01-15T18:04:60.123456789Z",
            ),
            (
                "2024-01-15T18:04:60.123456789Z",
                0,
                "2024-01-15T18:04:60.123456789Z",
            ),
            (
                "2016-12-31T23:59:59.999999999Z",
                0,
                "2016-12-31T23:59:59.999999999Z",
            ),
            ("2016-12-31T23:59:60Z", 0, "2016-12-31T23:59:60Z"),
            (
                "2017-01-01T05:29:60.123456789+05:30",
                19800,
                "2016-12-31T23:59:60.123456789Z",
            ),
            ("2017-01-01T00:00:00Z", 0, "2017-01-01T00:00:00Z"),
            ("0000-01-01T00:00:00+23:59", 86340, "-0001-12-31T00:01:00Z"),
            (
                "9999-12-31T23:59:59-23:59",
                -86340,
                "+10000-01-01T23:58:59Z",
            ),
        ];
        for (input, offset, expected) in cases {
            let value = Timestamp::parse(input).unwrap();
            assert_eq!(value.offset_seconds(), offset, "{input}");
            assert_eq!(value.to_utc_rfc3339(), expected, "{input}");
        }
        let t = |s| Timestamp::parse(s).unwrap();
        assert!(t("2016-12-31T23:59:59.999999999Z") < t("2016-12-31T23:59:60Z"));
        assert!(t("2016-12-31T23:59:60Z") < t("2017-01-01T00:00:00Z"));
        assert_eq!(
            t("2024-01-15 12:34:60.123456789987−05:30"),
            t("2024-01-15T18:04:60.123456789Z")
        );
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
            assert!(Timestamp::parse(input).is_none(), "{input}");
        }
    }

    #[test]
    fn timestamp_serde_baseline_wire_values() {
        let cases = [
            (
                "2017-01-01T05:29:60.123456789+05:30",
                "2016-12-31T23:59:60.123456789Z",
            ),
            ("1969-12-31T23:59:59.001Z", "1969-12-31T23:59:59.001Z"),
            ("0000-01-01T00:00:00+23:59", "-0001-12-31T00:01:00Z"),
            ("9999-12-31T23:59:59-23:59", "+10000-01-01T23:58:59Z"),
            ("+10000-01-01T00:00:00Z", "+10000-01-01T00:00:00Z"),
            ("-0001-12-31T00:01:00Z", "-0001-12-31T00:01:00Z"),
            (
                " \u{2003}+82701 - 5 - 6t 15 : 9 : 60.898989898989 \u{2003}uTc\t",
                "+82701-05-06T15:09:60.898989898Z",
            ),
            ("1-2-3  4:5:6z", "0001-02-03T04:05:06Z"),
            (
                "+000000000000000000000000000001-2-3T4:5:6 UTC",
                "0001-02-03T04:05:06Z",
            ),
            ("-0-2-29T4:5:6Z", "0000-02-29T04:05:06Z"),
            ("2024-1-2T3:4:5+0130", "2024-01-02T01:34:05Z"),
            (
                "2024-1-2T3:4:60.1−01 :\u{2003}: 30",
                "2024-01-02T04:34:60.100Z",
            ),
            ("2024-01-01T00:00:00z\u{2003}", "2024-01-01T00:00:00Z"),
            ("-262143-01-01T00:00:00Z", "-262143-01-01T00:00:00Z"),
            ("-262143-01-01T23:59:60+23:59", "-262143-01-01T00:00:60Z"),
            (
                "+262142-12-31T23:59:60.999999999Z",
                "+262142-12-31T23:59:60.999999999Z",
            ),
        ];
        for (input, expected) in cases {
            let value: Timestamp = serde_json::from_value(serde_json::json!(input)).unwrap();
            assert_eq!(value.offset_seconds(), 0, "{input}");
            assert_eq!(serde_json::to_value(value).unwrap(), expected, "{input}");
            assert_eq!(
                serde_json::from_value::<Timestamp>(serde_json::json!(expected)).unwrap(),
                value
            );
        }
    }

    #[test]
    fn strict_and_wire_timestamp_diagnostics_match_chrono_0_4_45() {
        assert_eq!(
            Timestamp::parse_rfc3339("bad").unwrap_err().to_string(),
            "premature end of input",
        );
        for (input, diagnostic) in [
            ("bad", "input contains invalid characters"),
            ("2026-11-19", "premature end of input"),
            ("2026-02-30T00:00:00Z", "input is out of range"),
            ("2026-11-19T00:00:00Z junk", "trailing input"),
            (
                "-262143-01-01T00:00:00+00:01",
                "no possible date and time matching input",
            ),
        ] {
            let encoded = serde_json::to_string(input).unwrap();
            let error = serde_json::from_str::<Timestamp>(&encoded).unwrap_err();
            assert!(
                error.to_string().starts_with(diagnostic),
                "{input}: {error}"
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
                serde_json::from_str::<Timestamp>(&wire).is_err(),
                "{input:?}"
            );
        }
        for wire in ["123", "null", "[]"] {
            assert!(serde_json::from_str::<Timestamp>(wire).is_err());
        }
    }

    #[test]
    fn formatting_fractional_precision_and_pre_epoch_baselines() {
        for (seconds, date) in [
            (-1, "1969-12-31T23:59:59"),
            (0, "1970-01-01T00:00:00"),
            (1, "1970-01-01T00:00:01"),
            (1_700_000_000, "2023-11-14T22:13:20"),
        ] {
            for (nanos, fraction) in [
                (0, ""),
                (1, ".000000001"),
                (1_000, ".000001"),
                (1_000_000, ".001"),
                (123_000_000, ".123"),
                (123_456_000, ".123456"),
                (123_456_789, ".123456789"),
                (999_999_999, ".999999999"),
            ] {
                let value = OffsetDateTime::from_unix_timestamp(seconds)
                    .unwrap()
                    .replace_nanosecond(nanos)
                    .unwrap();
                assert_eq!(format_utc(value, false), format!("{date}{fraction}+00:00"));
                assert_eq!(
                    format_utc(value, true),
                    format!("{date}.{:06}Z", nanos / 1000)
                );
            }
        }
    }
}
