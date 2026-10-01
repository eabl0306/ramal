//! Tiempo de git: lectura de `--date=raw` y salida ISO 8601 que la interfaz
//! siempre puede leer, aunque el commit traiga una fecha que git acepta y nadie
//! mas (zonas como `+0090`, anios de mas de cuatro cifras).

#![allow(
    unused_crate_dependencies,
    reason = "integration tests only use the crate under test"
)]
#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use ramal_core_domain::{GitTime, TimeError};

#[test]
fn git_time_reads_the_raw_format_of_git() {
    let time = GitTime::parse_raw("1790761320 +0200").unwrap();

    assert_eq!(time.seconds(), 1_790_761_320);
    assert_eq!(time.offset_minutes(), 120);
    assert_eq!(time.to_iso8601(), "2026-09-30T11:42:00+02:00");
}

#[test]
fn git_time_accepts_zero_and_negative_offsets() {
    let utc = GitTime::parse_raw("0 +0000").unwrap();
    assert_eq!(utc.seconds(), 0);
    assert_eq!(utc.to_string(), "1970-01-01T00:00:00+00:00");

    let west = GitTime::parse_raw("951886815 -0530").unwrap();
    assert_eq!(west.offset_minutes(), -330);
    assert_eq!(west.to_string(), "2000-02-29T23:30:15-05:30");
}

#[test]
fn minutes_of_sixty_or_more_count_as_minutes_like_git_does() {
    // Commit real que git fsck no marca: `author T <t@t> 1700000000 +0090`.
    let odd = GitTime::parse_raw("1700000000 +0090").unwrap();
    assert_eq!(odd.offset_minutes(), 90);
    assert_eq!(odd.to_string(), "2023-11-14T23:43:20+01:30");

    let widest = GitTime::parse_raw("1700000000 -9999").unwrap();
    assert_eq!(widest.offset_minutes(), -(99 * 60 + 99));
}

#[test]
fn git_time_formats_dates_before_the_epoch_and_far_ahead() {
    assert_eq!(GitTime::new(-1, 0).to_string(), "1969-12-31T23:59:59+00:00");
    assert_eq!(
        GitTime::new(-2_203_891_200, 60).to_string(),
        "1900-03-01T01:00:00+01:00"
    );
    assert_eq!(
        GitTime::new(13_574_602_800, -660).to_string(),
        "2400-02-29T00:00:00-11:00"
    );
    assert_eq!(
        GitTime::new(253_402_250_399, 840).to_string(),
        "9999-12-31T23:59:59+14:00"
    );
}

#[test]
fn an_instant_outside_four_digit_years_is_clamped_to_the_edge() {
    // `author T <t@t> 99999999999999 +0000`: %aI daba 3170843-11-07T09:46:39Z.
    let far = GitTime::parse_raw("99999999999999 +0000").unwrap();
    assert_eq!(far.seconds(), 99_999_999_999_999);
    assert_eq!(far.to_string(), "9999-12-31T23:59:59+00:00");

    assert_eq!(
        GitTime::new(i64::MIN, -60).to_string(),
        "0000-01-01T00:00:00-01:00"
    );
    assert_eq!(
        GitTime::new(i64::MAX, 60).to_string(),
        "9999-12-31T23:59:59+01:00"
    );
}

#[test]
fn an_offset_of_a_day_or_more_is_shown_in_utc() {
    let time = GitTime::new(1_700_000_000, 99 * 60 + 99);

    assert_eq!(time.offset_minutes(), 6039);
    assert_eq!(time.to_string(), "2023-11-14T22:13:20+00:00");
}

#[test]
fn git_time_rejects_malformed_raw_text() {
    for text in [
        "",
        "1700000000",
        "1700000000 0200",
        "1700000000 +200",
        "1700000000  +0200",
        "-5 +0000",
        "+5 +0000",
        "17x0 +0000",
        "1700000000 +02a0",
        "1700000000 +-200",
        "2026-09-30T11:42:00+02:00",
        "1700000000 \u{e9}0200",
    ] {
        assert!(
            matches!(GitTime::parse_raw(text), Err(TimeError::Malformed(_))),
            "{text}"
        );
    }
}

#[test]
fn git_time_rejects_values_that_do_not_fit() {
    for text in [
        "99999999999999999999 +0000",
        "1700000000 +123456",
        "1700000000 +99999999999999999999",
    ] {
        let error = GitTime::parse_raw(text).unwrap_err();
        assert!(matches!(error, TimeError::OutOfRange(_)), "{text}");
        assert!(error.to_string().contains(text));
    }
}
