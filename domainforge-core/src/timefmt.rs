//! Exact RFC 3339 formatting, parsing and serde adapters on top of the `time`
//! crate.
//!
//! DomainForge used to depend on `chrono` (RUSTSEC-2026-0543: unmaintained).
//! Several timestamps are signed, digested or placed on the wire, so the
//! replacement has to reproduce chrono's bytes exactly: `time`'s own defaults
//! (`Z` instead of `+00:00`, trimmed fractions, tuple serde) differ from every
//! format chrono produced. Everything that formats or parses a timestamp or a
//! duration therefore goes through this module.
//!
//! | function                  | chrono equivalent                              | example                          |
//! |---------------------------|------------------------------------------------|----------------------------------|
//! | [`to_rfc3339`]            | `DateTime<Utc>::to_rfc3339()`                  | `2024-01-02T03:04:05.5+00:00`    |
//! | [`to_rfc3339_z`]          | `to_rfc3339_opts(AutoSi, true)` and serde      | `2024-01-02T03:04:05.500Z`       |
//! | [`to_rfc3339_micros_z`]   | `to_rfc3339_opts(Micros, true)`                | `2024-01-02T03:04:05.500000Z`    |
//! | [`to_rfc3339_millis_z`]   | `to_rfc3339_opts(Millis, true)`                | `2024-01-02T03:04:05.500Z`       |
//! | [`to_rfc3339_secs_z`]     | `to_rfc3339_opts(Secs, true)`                  | `2024-01-02T03:04:05Z`           |
//! | [`parse_rfc3339`]         | `DateTime::parse_from_rfc3339`                 |                                  |
//! | [`parse_relaxed`]         | `str::parse::<DateTime<Utc>>` (serde input)    |                                  |
//!
//! The fraction of the "AutoSi" formats is 0, 3, 6 or 9 digits, whichever is
//! the shortest that is exact. Years outside `0..=9999` are written with an
//! explicit sign and at least four digits (`+10000`, `-0001`), as chrono does.
//!
//! # The one thing `time` cannot represent
//!
//! chrono keeps leap seconds (`23:59:60`) as an out-of-range nanosecond.
//! `time` has no such value. [`parse_rfc3339`] still accepts a `:60` second
//! (so comparisons keep working) and folds it into the next second, as POSIX
//! time does. Anything that *stores* a parsed value ([`parse_rfc3339_exact`],
//! [`parse_relaxed`], the serde adapters) rejects it with
//! [`ParseError::LeapSecond`] instead of silently emitting different bytes.

use std::fmt;
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

/// chrono's `NaiveDate` range: `-262143-01-01 ..= +262142-12-31`.
const MIN_YEAR: i32 = -262_143;
const MAX_YEAR: i32 = 262_142;

/// Why a timestamp failed to parse. The `Display` text of every variant but
/// [`ParseError::LeapSecond`] is identical to chrono's `ParseError`, because
/// those messages reach users through policy and CLI errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// A field is outside its range (month 13, hour 24, offset +24:00, ...).
    OutOfRange,
    /// Fields that cannot describe one instant.
    Impossible,
    /// Too little information to build an instant.
    NotEnough,
    /// An unexpected character.
    Invalid,
    /// The input ended early.
    TooShort,
    /// Characters remain after a complete timestamp.
    TooLong,
    /// A `:60` leap second, which `time` cannot represent.
    LeapSecond,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::OutOfRange => "input is out of range",
            Self::Impossible => "no possible date and time matching input",
            Self::NotEnough => "input is not enough for unique date and time",
            Self::Invalid => "input contains invalid characters",
            Self::TooShort => "premature end of input",
            Self::TooLong => "trailing input",
            Self::LeapSecond => "leap seconds (:60) are not representable",
        })
    }
}

impl std::error::Error for ParseError {}

type PResult<T> = Result<T, ParseError>;

// ---------------------------------------------------------------------------
// Clock
// ---------------------------------------------------------------------------

/// The current instant in UTC.
pub fn now() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

/// `Utc::now().to_rfc3339()`
pub fn now_rfc3339() -> String {
    to_rfc3339(now())
}

/// `Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true)`
pub fn now_micros_z() -> String {
    to_rfc3339_micros_z(now())
}

/// `DateTime::from_timestamp(secs, 0)`: `None` outside chrono's range.
pub fn from_unix_timestamp(secs: i64) -> Option<OffsetDateTime> {
    let dt = OffsetDateTime::from_unix_timestamp(secs).ok()?;
    year_in_range(dt.year()).then_some(dt)
}

fn year_in_range(year: i32) -> bool {
    (MIN_YEAR..=MAX_YEAR).contains(&year)
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Fraction {
    Auto,
    Millis,
    Micros,
    Secs,
}

/// `DateTime<Utc>::to_rfc3339()`: `+00:00`, AutoSi fraction.
pub fn to_rfc3339(dt: OffsetDateTime) -> String {
    format_utc(dt, Fraction::Auto, false)
}

/// `to_rfc3339_opts(AutoSi, true)`; also chrono's serde output.
pub fn to_rfc3339_z(dt: OffsetDateTime) -> String {
    format_utc(dt, Fraction::Auto, true)
}

/// `to_rfc3339_opts(Micros, true)`: always six fraction digits.
pub fn to_rfc3339_micros_z(dt: OffsetDateTime) -> String {
    format_utc(dt, Fraction::Micros, true)
}

/// `to_rfc3339_opts(Millis, true)`: always three fraction digits.
pub fn to_rfc3339_millis_z(dt: OffsetDateTime) -> String {
    format_utc(dt, Fraction::Millis, true)
}

/// `to_rfc3339_opts(Secs, true)`: no fraction.
pub fn to_rfc3339_secs_z(dt: OffsetDateTime) -> String {
    format_utc(dt, Fraction::Secs, true)
}

fn format_utc(dt: OffsetDateTime, fraction: Fraction, zulu: bool) -> String {
    let dt = dt.to_offset(UtcOffset::UTC);
    let year = dt.year();
    let nanos = dt.nanosecond();
    let mut out = String::with_capacity(35);
    if (0..=9999).contains(&year) {
        out.push_str(&format!("{year:04}"));
    } else {
        out.push_str(&format!("{year:+05}"));
    }
    out.push_str(&format!(
        "-{:02}-{:02}T{:02}:{:02}:{:02}",
        u8::from(dt.month()),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second()
    ));
    match fraction {
        Fraction::Auto => {
            if nanos == 0 {
            } else if nanos.is_multiple_of(1_000_000) {
                out.push_str(&format!(".{:03}", nanos / 1_000_000));
            } else if nanos.is_multiple_of(1_000) {
                out.push_str(&format!(".{:06}", nanos / 1_000));
            } else {
                out.push_str(&format!(".{nanos:09}"));
            }
        }
        Fraction::Millis => out.push_str(&format!(".{:03}", nanos / 1_000_000)),
        Fraction::Micros => out.push_str(&format!(".{:06}", nanos / 1_000)),
        Fraction::Secs => {}
    }
    out.push_str(if zulu { "Z" } else { "+00:00" });
    out
}

// ---------------------------------------------------------------------------
// Scanning (a port of chrono's `format::scan`, enough for RFC 3339)
// ---------------------------------------------------------------------------

/// Non-negative number of `min..=max` digits; more than `max` digits are left
/// in the remainder.
fn number(s: &str, min: usize, max: usize) -> PResult<(&str, i64)> {
    let bytes = s.as_bytes();
    if bytes.len() < min {
        return Err(ParseError::TooShort);
    }
    let mut n = 0i64;
    for (i, &c) in bytes.iter().take(max).enumerate() {
        if !c.is_ascii_digit() {
            if i < min {
                return Err(ParseError::Invalid);
            }
            return Ok((&s[i..], n));
        }
        n = n
            .checked_mul(10)
            .and_then(|n| n.checked_add(i64::from(c - b'0')))
            .ok_or(ParseError::OutOfRange)?;
    }
    Ok((&s[max.min(bytes.len())..], n))
}

/// Fractional seconds (at least one digit) as nanoseconds; digits past the
/// ninth are skipped.
fn nanosecond(s: &str) -> PResult<(&str, u32)> {
    let orig = s.len();
    let (s, v) = number(s, 1, 9)?;
    let consumed = orig - s.len();
    let v = u32::try_from(v).map_err(|_| ParseError::OutOfRange)?;
    let v = v * 10u32.pow(9 - consumed as u32);
    Ok((s.trim_start_matches(|c: char| c.is_ascii_digit()), v))
}

fn char(s: &str, c1: u8) -> PResult<&str> {
    match s.as_bytes().first() {
        Some(&c) if c == c1 => Ok(&s[1..]),
        Some(_) => Err(ParseError::Invalid),
        None => Err(ParseError::TooShort),
    }
}

fn colon(s: &str) -> PResult<&str> {
    char(s, b':')
}

fn colon_or_space(s: &str) -> PResult<&str> {
    Ok(s.trim_start_matches(|c: char| c == ':' || c.is_whitespace()))
}

/// `Z`/`z` or `+hh<sep>mm` (also `-` and U+2212). Returns seconds east.
fn timezone_offset(mut s: &str, consume_colon: fn(&str) -> PResult<&str>) -> PResult<(&str, i32)> {
    if let Some(&b'Z' | &b'z') = s.as_bytes().first() {
        return Ok((&s[1..], 0));
    }
    let digits = |s: &str| -> PResult<(u8, u8)> {
        let b = s.as_bytes();
        if b.len() < 2 {
            Err(ParseError::TooShort)
        } else {
            Ok((b[0], b[1]))
        }
    };
    let negative = match s.chars().next() {
        Some('+') => {
            s = &s[1..];
            false
        }
        Some('-') => {
            s = &s[1..];
            true
        }
        Some('\u{2212}') => {
            s = &s['\u{2212}'.len_utf8()..];
            true
        }
        Some(_) => return Err(ParseError::Invalid),
        None => return Err(ParseError::TooShort),
    };
    let hours = match digits(s)? {
        (h1 @ b'0'..=b'9', h2 @ b'0'..=b'9') => i32::from((h1 - b'0') * 10 + (h2 - b'0')),
        _ => return Err(ParseError::Invalid),
    };
    s = &s[2..];
    s = consume_colon(s)?;
    let minutes = if let Ok(ds) = digits(s) {
        match ds {
            (m1 @ b'0'..=b'5', m2 @ b'0'..=b'9') => i32::from((m1 - b'0') * 10 + (m2 - b'0')),
            (b'6'..=b'9', b'0'..=b'9') => return Err(ParseError::OutOfRange),
            _ => return Err(ParseError::Invalid),
        }
    } else {
        return Err(ParseError::TooShort);
    };
    s = match s.len() {
        len if len >= 2 => &s[2..],
        0 => s,
        _ => return Err(ParseError::TooShort),
    };
    let seconds = hours * 3600 + minutes * 60;
    Ok((s, if negative { -seconds } else { seconds }))
}

// ---------------------------------------------------------------------------
// Building an instant
// ---------------------------------------------------------------------------

struct Fields {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    /// 0..=60
    second: u32,
    nano: u32,
    offset_secs: i32,
}

/// Returns the instant (carrying its offset) and whether the input was a
/// `:60` leap second (folded into the following second).
fn build(f: &Fields) -> PResult<(OffsetDateTime, bool)> {
    if !year_in_range(f.year) {
        return Err(ParseError::OutOfRange);
    }
    let month = u8::try_from(f.month)
        .ok()
        .and_then(|m| Month::try_from(m).ok())
        .ok_or(ParseError::OutOfRange)?;
    let date =
        Date::from_calendar_date(f.year, month, f.day as u8).map_err(|_| ParseError::OutOfRange)?;
    let leap = f.second == 60;
    let second = if leap { 59 } else { f.second };
    let time = Time::from_hms_nano(f.hour as u8, f.minute as u8, second as u8, f.nano)
        .map_err(|_| ParseError::OutOfRange)?;
    // chrono rejects offsets of a day or more; `time` allows up to 25:59:59
    if f.offset_secs.abs() >= 86_400 {
        return Err(ParseError::OutOfRange);
    }
    let offset =
        UtcOffset::from_whole_seconds(f.offset_secs).map_err(|_| ParseError::OutOfRange)?;
    let mut dt = PrimitiveDateTime::new(date, time).assume_offset(offset);
    if leap {
        dt = dt
            .checked_add(time::Duration::SECOND)
            .ok_or(ParseError::OutOfRange)?;
    }
    // the UTC instant must also fit chrono's range
    dt.checked_to_offset(UtcOffset::UTC)
        .filter(|u| year_in_range(u.year()))
        .ok_or(ParseError::Impossible)?;
    Ok((dt, leap))
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

fn digit(bytes: &[u8], index: usize) -> PResult<u32> {
    if bytes[index].is_ascii_digit() {
        Ok(u32::from(bytes[index] - b'0'))
    } else {
        Err(ParseError::Invalid)
    }
}

fn parse_strict_fields(mut s: &str) -> PResult<Fields> {
    let bytes = s.as_bytes();
    if bytes.len() < 19 {
        return Err(ParseError::TooShort);
    }
    let year =
        digit(bytes, 0)? * 1000 + digit(bytes, 1)? * 100 + digit(bytes, 2)? * 10 + digit(bytes, 3)?;
    if bytes.get(4) != Some(&b'-') {
        return Err(ParseError::Invalid);
    }
    let month = digit(bytes, 5)? * 10 + digit(bytes, 6)?;
    if bytes.get(7) != Some(&b'-') {
        return Err(ParseError::Invalid);
    }
    let day = digit(bytes, 8)? * 10 + digit(bytes, 9)?;
    // the date must exist before the rest is looked at (chrono's order)
    let month_enum = u8::try_from(month)
        .ok()
        .and_then(|m| Month::try_from(m).ok())
        .ok_or(ParseError::OutOfRange)?;
    Date::from_calendar_date(year as i32, month_enum, day as u8)
        .map_err(|_| ParseError::OutOfRange)?;
    if !matches!(bytes.get(10), Some(&b't' | &b'T' | &b' ')) {
        return Err(ParseError::Invalid);
    }
    let hour = digit(bytes, 11)? * 10 + digit(bytes, 12)?;
    if bytes.get(13) != Some(&b':') {
        return Err(ParseError::Invalid);
    }
    let minute = digit(bytes, 14)? * 10 + digit(bytes, 15)?;
    if bytes.get(16) != Some(&b':') {
        return Err(ParseError::Invalid);
    }
    let second = digit(bytes, 17)? * 10 + digit(bytes, 18)?;
    let nano = if bytes.get(19) == Some(&b'.') {
        let (rest, nano) = nanosecond(&s[20..])?;
        s = rest;
        nano
    } else {
        s = &s[19..];
        0
    };
    if hour >= 24 || minute >= 60 || second > 60 {
        return Err(ParseError::OutOfRange);
    }
    let (rest, offset_secs) = timezone_offset(s, colon)?;
    if !rest.is_empty() {
        return Err(ParseError::TooLong);
    }
    Ok(Fields {
        year: year as i32,
        month,
        day,
        hour,
        minute,
        second,
        nano,
        offset_secs,
    })
}

/// `DateTime::parse_from_rfc3339`: strict RFC 3339 (`T`, `t` or a space between
/// date and time, `Z`/`z`/`±hh:mm`, any number of fraction digits). The result
/// keeps the input's offset; instants compare and order correctly regardless.
///
/// A `:60` leap second is accepted and folded into the next second (see the
/// module docs); use [`parse_rfc3339_exact`] where the value will be stored.
pub fn parse_rfc3339(s: &str) -> Result<OffsetDateTime, ParseError> {
    build(&parse_strict_fields(s)?).map(|(dt, _)| dt)
}

/// [`parse_rfc3339`], but rejects a leap second with [`ParseError::LeapSecond`].
pub fn parse_rfc3339_exact(s: &str) -> Result<OffsetDateTime, ParseError> {
    match build(&parse_strict_fields(s)?)? {
        (_, true) => Err(ParseError::LeapSecond),
        (dt, false) => Ok(dt),
    }
}

/// Relaxed RFC 3339, exactly what chrono's `FromStr`/serde accepted: values
/// need no zero padding, years outside `0..=9999` take a sign, whitespace may
/// separate components, `UTC` is a valid zone, and the offset colon may be a
/// space or missing. Returns the instant in UTC. A leap second is rejected.
pub fn parse_relaxed(s: &str) -> Result<OffsetDateTime, ParseError> {
    let fields = parse_relaxed_fields(s)?;
    match build(&fields)? {
        (_, true) => Err(ParseError::LeapSecond),
        (dt, false) => Ok(dt.to_offset(UtcOffset::UTC)),
    }
}

fn signed_or_width(s: &str, width: usize) -> PResult<(&str, i64)> {
    if let Some(rest) = s.strip_prefix('-') {
        let (rest, v) = number(rest, 1, usize::MAX)?;
        Ok((rest, 0i64.checked_sub(v).ok_or(ParseError::OutOfRange)?))
    } else if let Some(rest) = s.strip_prefix('+') {
        number(rest, 1, usize::MAX)
    } else {
        number(s, 1, width)
    }
}

fn parse_relaxed_fields(s: &str) -> PResult<Fields> {
    let literal = |s: &str, lit: &str| -> PResult<()> {
        if s.len() < lit.len() {
            return Err(ParseError::TooShort);
        }
        if !s.starts_with(lit) {
            return Err(ParseError::Invalid);
        }
        Ok(())
    };
    // year
    let s = s.trim_start();
    let (s, year) = signed_or_width(s, 4)?;
    let year = i32::try_from(year).map_err(|_| ParseError::OutOfRange)?;
    let s = s.trim_start();
    literal(s, "-")?;
    let s = &s[1..];
    // month
    let s = s.trim_start();
    let (s, month) = number(s, 1, 2)?;
    if !(1..=12).contains(&month) {
        return Err(ParseError::OutOfRange);
    }
    let s = s.trim_start();
    literal(s, "-")?;
    let s = &s[1..];
    // day
    let s = s.trim_start();
    let (s, day) = number(s, 1, 2)?;
    if !(1..=31).contains(&day) {
        return Err(ParseError::OutOfRange);
    }
    // separator
    let s = match s.as_bytes().first() {
        Some(&b't' | &b'T' | &b' ') => &s[1..],
        Some(_) => return Err(ParseError::Invalid),
        None => return Err(ParseError::TooShort),
    };
    // hour
    let s = s.trim_start();
    let (s, hour) = number(s, 1, 2)?;
    if !(0..=23).contains(&hour) {
        return Err(ParseError::OutOfRange);
    }
    let s = s.trim_start();
    literal(s, ":")?;
    let s = &s[1..];
    // minute
    let s = s.trim_start();
    let (s, minute) = number(s, 1, 2)?;
    if !(0..=59).contains(&minute) {
        return Err(ParseError::OutOfRange);
    }
    let s = s.trim_start();
    literal(s, ":")?;
    let s = &s[1..];
    // second
    let s = s.trim_start();
    let (s, second) = number(s, 1, 2)?;
    if !(0..=60).contains(&second) {
        return Err(ParseError::OutOfRange);
    }
    // optional fraction
    let (s, nano) = if let Some(rest) = s.strip_prefix('.') {
        nanosecond(rest)?
    } else {
        (s, 0)
    };
    // offset
    let s = s.trim_start();
    let (s, offset_secs) = if s.len() >= 3 && s.as_bytes()[..3].eq_ignore_ascii_case(b"UTC") {
        (&s[3..], 0)
    } else {
        timezone_offset(s, colon_or_space)?
    };
    if !s.trim_start().is_empty() {
        return Err(ParseError::TooLong);
    }
    Ok(Fields {
        year,
        month: month as u32,
        day: day as u32,
        hour: hour as u32,
        minute: minute as u32,
        second: second as u32,
        nano,
        offset_secs,
    })
}

// ---------------------------------------------------------------------------
// serde adapters
// ---------------------------------------------------------------------------

/// `#[serde(with = "crate::timefmt::serde_ts")]` for `OffsetDateTime`:
/// serializes as [`to_rfc3339_z`], deserializes with [`parse_relaxed`], like
/// chrono's `DateTime<Utc>` serde.
pub mod serde_ts {
    use super::*;
    use serde::{de, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(dt: &OffsetDateTime, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&to_rfc3339_z(*dt))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<OffsetDateTime, D::Error> {
        struct Visitor;
        impl de::Visitor<'_> for Visitor {
            type Value = OffsetDateTime;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a formatted date and time string")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                parse_relaxed(v).map_err(E::custom)
            }
        }
        d.deserialize_str(Visitor)
    }
}

/// [`serde_ts`] for `Option<OffsetDateTime>` (`null` / missing is `None`).
pub mod serde_ts_option {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(dt: &Option<OffsetDateTime>, s: S) -> Result<S::Ok, S::Error> {
        dt.map(to_rfc3339_z).serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<OffsetDateTime>, D::Error> {
        use serde::de::Error;
        Option::<String>::deserialize(d)?
            .map(|v| parse_relaxed(&v).map_err(D::Error::custom))
            .transpose()
    }
}

/// chrono serialized a `TimeDelta` as the tuple `(secs, nanos)` with the
/// seconds rounded toward negative infinity and `0 <= nanos < 1e9`.
pub mod serde_duration {
    use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

    /// chrono's `TimeDelta` holds at most `i64::MAX` milliseconds either way.
    const LIMIT_NANOS: i128 = (i64::MAX as i128) * 1_000_000;

    pub(super) fn to_parts(d: time::Duration) -> (i64, i32) {
        let mut secs = d.whole_seconds();
        let mut nanos = d.subsec_nanoseconds();
        if nanos < 0 {
            secs -= 1;
            nanos += 1_000_000_000;
        }
        (secs, nanos)
    }

    pub(super) fn from_parts(secs: i64, nanos: i32) -> Option<time::Duration> {
        let nanos = u32::try_from(nanos).ok().filter(|n| *n < 1_000_000_000)?;
        let total = i128::from(secs) * 1_000_000_000 + i128::from(nanos);
        if !(-LIMIT_NANOS..=LIMIT_NANOS).contains(&total) {
            return None;
        }
        Some(time::Duration::new(secs, nanos as i32))
    }

    pub fn serialize<S: Serializer>(d: &time::Duration, s: S) -> Result<S::Ok, S::Error> {
        to_parts(*d).serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<time::Duration, D::Error> {
        let (secs, nanos) = <(i64, i32)>::deserialize(d)?;
        from_parts(secs, nanos).ok_or_else(|| D::Error::custom("TimeDelta out of bounds"))
    }
}

/// [`serde_duration`] for `Option<time::Duration>`.
pub mod serde_duration_option {
    use super::*;
    use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(d: &Option<time::Duration>, s: S) -> Result<S::Ok, S::Error> {
        d.map(serde_duration::to_parts).serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<time::Duration>, D::Error> {
        Option::<(i64, i32)>::deserialize(d)?
            .map(|(secs, nanos)| {
                serde_duration::from_parts(secs, nanos)
                    .ok_or_else(|| D::Error::custom("TimeDelta out of bounds"))
            })
            .transpose()
    }
}
