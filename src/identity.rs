//! Who makes a commit, and when: author and committer from environment
//! variables

use crate::{Error, Result, Signature};

/// Moment in time: seconds since 1970 (Unix time, and the time zone as
/// minutes away from UTC.
pub type Time = (i64, i32);

/// The current time with the time zone offset of this computer.
pub fn now() -> Time {
    let now = jiff::ZonedDateTime::now();
    (now.timestamp().as_second(), now.offset().seconds() / 60)
}

/// Reads a date in Git format: `<seconds> <+hhmm>` (like `1700000000 +0100`).
///
/// # Errors
///
/// Returns [`Error::InvalidDate`] if `date` is not in this format.
pub fn parse_date(date: &str) -> Result<Time> {
    let invalid = || Error::InvalidDate {
        date: date.to_string(),
    };

    // "1700000000 +0100" -> "1700000000" and "+0100"
    let (seconds, offset) = date.split_once(' ').ok_or_else(invalid)?;
    let seconds: i64 = seconds.parse().map_err(|_| invalid())?;

    // "+0100" -> sign "+", hours "01", minutes "00"
    let (sign, digits) = match offset.as_bytes() {
        [b'+', rest @ ..] => (1, rest),
        [b'-', rest @ ..] => (-1, rest),
        _ => return Err(invalid()),
    };
    let [h1, h2, m1, m2] = digits else {
        return Err(invalid());
    };
    if !digits.iter().all(u8::is_ascii_digit) {
        return Err(invalid());
    }
    let hours = i32::from((h1 - b'0') * 10 + (h2 - b'0'));
    let minutes = i32::from((m1 - b'0') * 10 + (m2 - b'0'));
    if minutes >= 60 {
        return Err(invalid());
    }

    Ok((seconds, sign * (hours * 60 + minutes)))
}

/// Builds the author and the committer of a new commit. Uses Git environment
/// variables, without config file:
/// - `GIT_AUTHOR_NAME` and `GIT_AUTHOR_EMAIL` are required
/// - `GIT_COMMITTER_NAME` and `GIT_COMMITTER_EMAIL` default to the author
/// - `GIT_AUTHOR_DATE` and `GIT_COMMITTER_DATE` default to `now`
///
/// `var` looks up one environment variable. The program passes
/// `|name| std::env::var(name).ok()` and tests pass their own values.
///
/// # Errors
///
/// Returns [`Error::MissingVariable`] if the author's name or email is not
/// set and [`Error::InvalidDate`] if a date is in the wrong format.
pub fn author_and_committer(
    var: impl Fn(&str) -> Option<String>,
    now: Time,
) -> Result<(Signature, Signature)> {
    let author_name = required(&var, "GIT_AUTHOR_NAME")?;
    let author_email = required(&var, "GIT_AUTHOR_EMAIL")?;
    let (time, offset_minutes) = date_or_now(&var, "GIT_AUTHOR_DATE", now)?;
    let author = Signature {
        name: author_name.clone(),
        email: author_email.clone(),
        time,
        offset_minutes,
    };

    // The committer name and email default to the author
    let committer_name = lookup(&var, "GIT_COMMITTER_NAME").unwrap_or(author_name);
    let committer_email = lookup(&var, "GIT_COMMITTER_EMAIL").unwrap_or(author_email);
    let (time, offset_minutes) = date_or_now(&var, "GIT_COMMITTER_DATE", now)?;
    let committer = Signature {
        name: committer_name,
        email: committer_email,
        time,
        offset_minutes,
    };

    Ok((author, committer))
}

/// Lookup value of the varaible `name`. If not set it gives `None`.
/// An empty value counts as not set
fn lookup(var: &impl Fn(&str) -> Option<String>, name: &str) -> Option<String> {
    let value = var(name)?;
    if value.is_empty() { None } else { Some(value) }
}

/// Lookup value of the variable `name`. If not set it gives error.
fn required(var: &impl Fn(&str) -> Option<String>, name: &'static str) -> Result<String> {
    match lookup(var, name) {
        Some(value) => Ok(value),
        None => Err(Error::MissingVariable { name }),
    }
}

/// Lookup date in the variable `name`. If not set it uses `now`.
fn date_or_now(var: &impl Fn(&str) -> Option<String>, name: &str, now: Time) -> Result<Time> {
    match lookup(var, name) {
        Some(value) => parse_date(&value),
        None => Ok(now),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    /// A fixed "now" so tests do not depend on the clock
    // (hopefully this thesis will be done until 15 of Jan hihi)
    const NOW: Time = (1_800_000_000, 60);

    /// Builds the author and committer from `variables` instead of the
    /// real environment
    fn with(variables: &[(&str, &str)]) -> Result<(Signature, Signature)> {
        let variables: HashMap<&str, &str> = variables.iter().copied().collect();
        author_and_committer(
            |name| variables.get(name).map(|value| value.to_string()),
            NOW,
        )
    }

    #[test]
    fn parses_gits_date_format() {
        assert_eq!(parse_date("1700000000 +0100").unwrap(), (1_700_000_000, 60));
        assert_eq!(
            parse_date("1700000000 -0530").unwrap(),
            (1_700_000_000, -330)
        );
        assert_eq!(parse_date("0 +0000").unwrap(), (0, 0));
    }

    #[test]
    fn rejects_other_date_formats() {
        let bad = [
            "yesterday",
            "1700000000",
            "1700000000 0100",
            "1700000000 +01",
            "1700000000 +01a0",
            "1700000000 +0175",
            "2023-11-14 +0100",
        ];

        for date in bad {
            assert!(
                matches!(parse_date(date), Err(Error::InvalidDate { .. })),
                "{date:?} was accepted"
            );
        }
    }

    #[test]
    fn uses_all_six_variables() {
        let (author, committer) = with(&[
            ("GIT_AUTHOR_NAME", "Wiki"),
            ("GIT_AUTHOR_EMAIL", "wiki@example.com"),
            ("GIT_AUTHOR_DATE", "1700000000 +0100"),
            ("GIT_COMMITTER_NAME", "Bongo"),
            ("GIT_COMMITTER_EMAIL", "bongo@example.com"),
            ("GIT_COMMITTER_DATE", "1700003600 -0530"),
        ])
        .unwrap();

        assert_eq!(
            author.to_string(),
            "Wiki <wiki@example.com> 1700000000 +0100"
        );
        assert_eq!(
            committer.to_string(),
            "Bongo <bongo@example.com> 1700003600 -0530"
        );
    }

    #[test]
    fn committer_defaults_to_the_author_and_dates_to_now() {
        let (author, committer) = with(&[
            ("GIT_AUTHOR_NAME", "Wiki"),
            ("GIT_AUTHOR_EMAIL", "wiki@example.com"),
        ])
        .unwrap();

        assert_eq!(
            author.to_string(),
            "Wiki <wiki@example.com> 1800000000 +0100"
        );
        assert_eq!(committer, author);
    }

    #[test]
    fn missing_or_empty_author_is_an_error() {
        let no_name = with(&[("GIT_AUTHOR_EMAIL", "wiki@example.com")]);
        let empty_email = with(&[("GIT_AUTHOR_NAME", "Wiki"), ("GIT_AUTHOR_EMAIL", "")]);

        assert!(matches!(
            no_name,
            Err(Error::MissingVariable {
                name: "GIT_AUTHOR_NAME"
            })
        ));
        assert!(matches!(
            empty_email,
            Err(Error::MissingVariable {
                name: "GIT_AUTHOR_EMAIL"
            })
        ));
    }

    #[test]
    fn now_has_a_real_time_zone() {
        let (seconds, offset_minutes) = now();

        // After 2020 and offset that exists somewhere in the world
        assert!(seconds > 1_577_836_800);
        assert!((-12 * 60..=14 * 60).contains(&offset_minutes));
    }
}
