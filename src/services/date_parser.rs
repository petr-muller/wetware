//! Date parsing service - turns user-supplied date strings into calendar dates.
//!
//! Shared by `wet add --date`, `wet edit --date`, and the interactive composer's
//! date field, so all three accept exactly the same forms.

use crate::errors::ThoughtError;
use chrono::{Datelike, Days, Months, NaiveDate, Utc, Weekday};

/// Human-readable summary of the accepted forms, used in error messages and
/// in the composer's help footer.
pub const ACCEPTED_FORMS: &str = "YYYY-MM-DD, today, yesterday, tomorrow, -3d/-2w/-1m, or a weekday name";

/// Parse a user-supplied date string relative to `today`.
///
/// Accepted forms (case-insensitive, surrounding whitespace ignored):
///
/// - `2026-08-01` - an absolute ISO date
/// - `today` / `t`, `yesterday` / `y`, `tomorrow`
/// - `-3d`, `-2w`, `-1m` - that many days, weeks, or months back. A leading `+`
///   (or no sign) moves forward instead: `+3d`, `3d`.
/// - `mon`..`sun` or `monday`..`sunday` - the most recent occurrence of that
///   weekday, which is `today` when today is that weekday.
///
/// Month arithmetic clamps to the end of the target month, so `-1m` from
/// March 31 is the last day of February.
pub fn parse_date_from(input: &str, today: NaiveDate) -> Result<NaiveDate, ThoughtError> {
    let raw = input.trim();
    if raw.is_empty() {
        return Err(invalid(input));
    }
    let lower = raw.to_lowercase();

    if let Ok(date) = NaiveDate::parse_from_str(&lower, "%Y-%m-%d") {
        return Ok(date);
    }

    match lower.as_str() {
        "today" | "t" => return Ok(today),
        "yesterday" | "y" => return today.checked_sub_days(Days::new(1)).ok_or_else(|| invalid(input)),
        "tomorrow" => return today.checked_add_days(Days::new(1)).ok_or_else(|| invalid(input)),
        _ => {}
    }

    if let Some(weekday) = parse_weekday(&lower) {
        return Ok(most_recent_weekday(today, weekday));
    }

    if let Some(date) = parse_offset(&lower, today) {
        return Ok(date);
    }

    Err(invalid(input))
}

/// Parse a user-supplied date string relative to the current UTC date.
pub fn parse_date(input: &str) -> Result<NaiveDate, ThoughtError> {
    parse_date_from(input, Utc::now().date_naive())
}

fn invalid(input: &str) -> ThoughtError {
    ThoughtError::InvalidInput(format!("Invalid date '{}'. Expected {}.", input.trim(), ACCEPTED_FORMS))
}

/// Match full or three-letter weekday names.
fn parse_weekday(lower: &str) -> Option<Weekday> {
    match lower {
        "mon" | "monday" => Some(Weekday::Mon),
        "tue" | "tuesday" => Some(Weekday::Tue),
        "wed" | "wednesday" => Some(Weekday::Wed),
        "thu" | "thursday" => Some(Weekday::Thu),
        "fri" | "friday" => Some(Weekday::Fri),
        "sat" | "saturday" => Some(Weekday::Sat),
        "sun" | "sunday" => Some(Weekday::Sun),
        _ => None,
    }
}

/// The most recent occurrence of `weekday` at or before `today`.
fn most_recent_weekday(today: NaiveDate, weekday: Weekday) -> NaiveDate {
    let back = today.weekday().num_days_from_monday() as i64 - weekday.num_days_from_monday() as i64;
    let back = back.rem_euclid(7) as u64;
    today
        .checked_sub_days(Days::new(back))
        .expect("subtracting at most 6 days from a valid date cannot overflow")
}

/// Parse a signed offset like `-3d`, `+2w`, `1m`.
fn parse_offset(lower: &str, today: NaiveDate) -> Option<NaiveDate> {
    let (negative, rest) = match lower.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, lower.strip_prefix('+').unwrap_or(lower)),
    };

    let unit = rest.chars().last()?;
    let digits = &rest[..rest.len() - unit.len_utf8()];
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let amount: u32 = digits.parse().ok()?;

    match unit {
        'd' => shift_days(today, amount as u64, negative),
        'w' => shift_days(today, amount as u64 * 7, negative),
        'm' => {
            let months = Months::new(amount);
            if negative {
                today.checked_sub_months(months)
            } else {
                today.checked_add_months(months)
            }
        }
        _ => None,
    }
}

fn shift_days(today: NaiveDate, amount: u64, negative: bool) -> Option<NaiveDate> {
    let days = Days::new(amount);
    if negative {
        today.checked_sub_days(days)
    } else {
        today.checked_add_days(days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Friday, 2026-08-07.
    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, 7).unwrap()
    }

    fn parse(input: &str) -> NaiveDate {
        parse_date_from(input, today()).unwrap()
    }

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn test_parse_absolute_iso_date() {
        assert_eq!(parse("2026-08-01"), ymd(2026, 8, 1));
        assert_eq!(parse("1999-12-31"), ymd(1999, 12, 31));
    }

    #[test]
    fn test_parse_today_yesterday_tomorrow() {
        assert_eq!(parse("today"), today());
        assert_eq!(parse("t"), today());
        assert_eq!(parse("yesterday"), ymd(2026, 8, 6));
        assert_eq!(parse("y"), ymd(2026, 8, 6));
        assert_eq!(parse("tomorrow"), ymd(2026, 8, 8));
    }

    #[test]
    fn test_parse_day_and_week_offsets() {
        assert_eq!(parse("-3d"), ymd(2026, 8, 4));
        assert_eq!(parse("-2w"), ymd(2026, 7, 24));
        assert_eq!(parse("-0d"), today());
        assert_eq!(parse("+3d"), ymd(2026, 8, 10));
        assert_eq!(parse("3d"), ymd(2026, 8, 10));
    }

    #[test]
    fn test_parse_month_offset() {
        assert_eq!(parse("-1m"), ymd(2026, 7, 7));
        assert_eq!(parse("-12m"), ymd(2025, 8, 7));
        assert_eq!(parse("+1m"), ymd(2026, 9, 7));
    }

    #[test]
    fn test_month_offset_clamps_to_end_of_shorter_month() {
        // March 31 minus one month has no March-31 equivalent in February.
        let march31 = ymd(2026, 3, 31);
        assert_eq!(parse_date_from("-1m", march31).unwrap(), ymd(2026, 2, 28));
    }

    #[test]
    fn test_parse_weekday_returns_most_recent_past_occurrence() {
        // today() is a Friday.
        assert_eq!(parse("fri"), today());
        assert_eq!(parse("friday"), today());
        assert_eq!(parse("thu"), ymd(2026, 8, 6));
        assert_eq!(parse("mon"), ymd(2026, 8, 3));
        // Saturday is in the future, so it resolves to the previous Saturday.
        assert_eq!(parse("sat"), ymd(2026, 8, 1));
        assert_eq!(parse("sun"), ymd(2026, 8, 2));
    }

    #[test]
    fn test_parse_is_case_insensitive_and_trims() {
        assert_eq!(parse("  Yesterday  "), ymd(2026, 8, 6));
        assert_eq!(parse("TODAY"), today());
        assert_eq!(parse(" -3D "), ymd(2026, 8, 4));
        assert_eq!(parse("Mon"), ymd(2026, 8, 3));
    }

    #[test]
    fn test_parse_rejects_invalid_input() {
        for bad in [
            "",
            "   ",
            "not-a-date",
            "2026-13-01",
            "2026/08/01",
            "-d",
            "-3x",
            "3",
            "--3d",
            "yester",
        ] {
            assert!(
                parse_date_from(bad, today()).is_err(),
                "expected {:?} to be rejected",
                bad
            );
        }
    }

    #[test]
    fn test_error_message_lists_accepted_forms() {
        let err = parse_date_from("gibberish", today()).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("gibberish"), "{}", message);
        assert!(message.contains("YYYY-MM-DD"), "{}", message);
        assert!(message.contains("yesterday"), "{}", message);
    }

    #[test]
    fn test_parse_date_uses_current_date() {
        assert_eq!(parse_date("today").unwrap(), Utc::now().date_naive());
    }
}
