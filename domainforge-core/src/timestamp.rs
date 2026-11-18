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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::SecondsFormat;

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
