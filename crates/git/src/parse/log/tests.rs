#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use pretty_assertions::assert_eq;
use ramal_core_domain::Sha;

use super::parse_log;

const A: &str = "1111111111111111111111111111111111111111";
const B: &str = "2222222222222222222222222222222222222222";
const C: &str = "3333333333333333333333333333333333333333";

fn record(fields: &[&str]) -> String {
    format!("{}\0", fields.join("\u{1f}"))
}

#[test]
fn it_reads_commits_with_their_parents_in_order() {
    let stdout = [
        record(&[
            A,
            "1111111",
            &format!("{B} {C}"),
            "Ana",
            "ana@x.dev",
            "2026-09-30T11:42:00+02:00",
            "Merge branch 'x'",
        ]),
        record(&[
            B,
            "2222222",
            C,
            "Bea",
            "bea@x.dev",
            "2026-09-29T10:00:00Z",
            "Add graph",
        ]),
        record(&[
            C,
            "3333333",
            "",
            "Ana",
            "ana@x.dev",
            "2026-09-28T09:00:00-03:00",
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
    assert!(commits[2].node.parents.is_empty());
}

#[test]
fn a_subject_may_contain_the_field_separator_and_unicode() {
    let stdout = record(&[
        A,
        "1111111",
        "",
        "Jose Nunez",
        "j@x.dev",
        "2026-09-30T11:42:00Z",
        "uno\u{1f}dos, tres",
    ]);

    let commits = parse_log(stdout.as_bytes()).unwrap();

    assert_eq!(commits[0].summary, "uno\u{1f}dos, tres");
}

#[test]
fn empty_output_is_an_empty_history() {
    assert!(parse_log(b"").unwrap().is_empty());
    assert!(parse_log(b"\0").unwrap().is_empty());
}

#[test]
fn a_short_record_is_an_error() {
    let error = parse_log(format!("{A}\u{1f}1111111\0").as_bytes()).unwrap_err();

    assert!(error.to_string().contains("2 fields"));
}

#[test]
fn a_bad_sha_or_date_is_an_error() {
    let bad_sha = record(&[
        "xyz",
        "1111111",
        "",
        "Ana",
        "a@x",
        "2026-09-30T11:42:00Z",
        "s",
    ]);
    let bad_parent = record(&[
        A,
        "1111111",
        "nope",
        "Ana",
        "a@x",
        "2026-09-30T11:42:00Z",
        "s",
    ]);
    let bad_short = record(&[A, "zz", "", "Ana", "a@x", "2026-09-30T11:42:00Z", "s"]);
    let bad_date = record(&[A, "1111111", "", "Ana", "a@x", "yesterday", "s"]);

    for stdout in [bad_sha, bad_parent, bad_short, bad_date] {
        assert!(parse_log(stdout.as_bytes()).is_err(), "{stdout:?}");
    }
}
