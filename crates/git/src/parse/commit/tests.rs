#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use git_rudder_core_domain::{GitTime, Sha};
use pretty_assertions::assert_eq;

use super::parse_commit_detail;

const A: &str = "1111111111111111111111111111111111111111";
const B: &str = "2222222222222222222222222222222222222222";
const C: &str = "3333333333333333333333333333333333333333";

#[test]
fn a_merge_keeps_every_parent_and_the_whole_message() {
    let stdout = format!(
        "{A}\n1111111\n{B} {C}\nAna Ruiz\nana@example.com\n1790761320 +0200\n\
         Luis\nluis@example.com\n1790761380 -0500\nMerge branch 'x'\n\nBody line\n\n"
    );

    let detail = parse_commit_detail(stdout.as_bytes()).unwrap();

    assert_eq!(detail.sha, Sha::parse(A).unwrap());
    assert_eq!(detail.parents.len(), 2);
    assert_eq!(detail.author.name, "Ana Ruiz");
    assert_eq!(detail.committer.email, "luis@example.com");
    assert_eq!(detail.committed_at, GitTime::new(1_790_761_380, -300));
    assert_eq!(detail.message, "Merge branch 'x'\n\nBody line");
    assert_eq!(detail.summary(), "Merge branch 'x'");
}

#[test]
fn a_root_commit_with_an_odd_date_still_parses() {
    let stdout = format!("{A}\n1111111\n\nA\na@x\nnot a date\nA\na@x\n0 +0000\nInit\n");

    let detail = parse_commit_detail(stdout.as_bytes()).unwrap();

    assert_eq!(detail.parents, Vec::new());
    assert_eq!(detail.authored_at, GitTime::new(0, 0));
}

#[test]
fn missing_fields_are_an_error() {
    assert!(parse_commit_detail(b"short\n").is_err());
    let bad_parent = format!("{A}\n1111111\nzz\nA\na\n0 +0000\nA\na\n0 +0000\nm");
    assert!(parse_commit_detail(bad_parent.as_bytes()).is_err());
}
