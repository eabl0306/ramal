#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use pretty_assertions::assert_eq;
use ramal_core_domain::{GitTime, Sha};

use super::parse_log;

const A: &str = "1111111111111111111111111111111111111111";
const B: &str = "2222222222222222222222222222222222222222";
const C: &str = "3333333333333333333333333333333333333333";

/// Un registro como lo escribe git: sha, sha corto, padres, fecha raw, autor,
/// correo y asunto, uno por linea, y NUL al final.
fn record(fields: &[&str]) -> String {
    format!("{}\0", fields.join("\n"))
}

#[test]
fn it_reads_commits_with_their_parents_in_order() {
    let stdout = [
        record(&[
            A,
            "1111111",
            &format!("{B} {C}"),
            "1790761320 +0200",
            "Ana",
            "ana@x.dev",
            "Merge branch 'x'",
        ]),
        record(&[
            B,
            "2222222",
            C,
            "1790676000 +0000",
            "Bea",
            "bea@x.dev",
            "Add graph",
        ]),
        record(&[
            C,
            "3333333",
            "",
            "1790596800 -0300",
            "Ana",
            "ana@x.dev",
            "Root",
        ]),
    ]
    .concat();

    let commits = parse_log(stdout.as_bytes()).unwrap();

    assert_eq!(commits.len(), 3);
    assert_eq!(commits[0].node.sha, Sha::parse(A).unwrap());
    assert_eq!(
        commits[0].node.parents,
        vec![Sha::parse(B).unwrap(), Sha::parse(C).unwrap()]
    );
    assert_eq!(commits[0].short_sha.as_str(), "1111111");
    assert_eq!(commits[0].author.name, "Ana");
    assert_eq!(commits[0].author.email, "ana@x.dev");
    assert_eq!(
        commits[0].authored_at.to_string(),
        "2026-09-30T11:42:00+02:00"
    );
    assert_eq!(commits[0].summary, "Merge branch 'x'");
    let roots = &commits[2].node.parents;
    assert!(roots.is_empty(), "{roots:?}");
    assert_eq!(commits[2].authored_at.offset_minutes(), -180);
}

#[test]
fn a_subject_may_contain_the_old_separator_and_unicode() {
    let stdout = record(&[
        A,
        "1111111",
        "",
        "1790761320 +0000",
        "Jose Nunez",
        "j@x.dev",
        "uno\u{1f}dos, tres",
    ]);

    let commits = parse_log(stdout.as_bytes()).unwrap();

    assert_eq!(commits[0].summary, "uno\u{1f}dos, tres");
}

#[test]
fn a_unit_separator_in_the_name_or_email_does_not_shift_the_fields() {
    // git acepta GIT_AUTHOR_NAME=$'Bad\x1fName' y fsck no lo marca.
    let stdout = record(&[
        A,
        "1111111",
        "",
        "1700000000 +0000",
        "Bad\u{1f}Name",
        "a\u{1f}b@x.dev",
        "s",
    ]);

    let commits = parse_log(stdout.as_bytes()).unwrap();

    assert_eq!(commits[0].author.name, "Bad\u{1f}Name");
    assert_eq!(commits[0].author.email, "a\u{1f}b@x.dev");
    assert_eq!(commits[0].summary, "s");
}

#[test]
fn empty_name_email_and_subject_are_kept_as_empty_fields() {
    let stdout = record(&[A, "1111111", "", "1700000000 -0130", "", "", ""]);

    let commits = parse_log(stdout.as_bytes()).unwrap();

    assert_eq!(commits[0].author.name, "");
    assert_eq!(commits[0].author.email, "");
    assert_eq!(commits[0].summary, "");
    assert_eq!(commits[0].authored_at.offset_minutes(), -90);
}

#[test]
fn odd_dates_that_git_accepts_are_read_like_git_does() {
    let stdout = [
        record(&[A, "1111111", "", "1700000000 +0090", "T", "t@t", "zone"]),
        record(&[B, "2222222", "", "99999999999999 +0000", "T", "t@t", "far"]),
    ]
    .concat();

    let commits = parse_log(stdout.as_bytes()).unwrap();

    assert_eq!(commits[0].authored_at.offset_minutes(), 90);
    assert_eq!(commits[1].authored_at.seconds(), 99_999_999_999_999);
}

#[test]
fn an_unreadable_date_falls_back_to_the_epoch_instead_of_failing() {
    // `author T <t@t> notadate` deja %ad vacio; una zona enorme no cabe en i16.
    let stdout = [
        record(&[A, "1111111", "", "", "T", "t@t", "no date"]),
        record(&[B, "2222222", "", "1700000000 +123456", "T", "t@t", "zone"]),
        record(&[C, "3333333", "", "yesterday", "T", "t@t", "words"]),
    ]
    .concat();

    let commits = parse_log(stdout.as_bytes()).unwrap();

    assert_eq!(commits.len(), 3);
    for commit in &commits {
        assert_eq!(commit.authored_at, GitTime::new(0, 0));
    }
}

#[test]
fn empty_output_is_an_empty_history() {
    let empty = parse_log(b"").unwrap();
    let only_nul = parse_log(b"\0").unwrap();

    assert!(empty.is_empty(), "{empty:?}");
    assert!(only_nul.is_empty(), "{only_nul:?}");
}

#[test]
fn a_short_record_is_an_error() {
    let error = parse_log(format!("{A}\n1111111\0").as_bytes()).unwrap_err();

    assert!(error.to_string().contains("2 fields"));
}

#[test]
fn a_bad_sha_short_sha_or_parent_is_still_an_error() {
    let bad_sha = record(&["xyz", "1111111", "", "0 +0000", "Ana", "a@x", "s"]);
    let bad_parent = record(&[A, "1111111", "nope", "0 +0000", "Ana", "a@x", "s"]);
    let bad_short = record(&[A, "zz", "", "0 +0000", "Ana", "a@x", "s"]);

    for stdout in [bad_sha, bad_parent, bad_short] {
        assert!(parse_log(stdout.as_bytes()).is_err(), "{stdout:?}");
    }
}
