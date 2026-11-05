#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use git_rudder_core_domain::{GitTime, Sha};
use pretty_assertions::assert_eq;

use super::parse_blame;

const A: &str = "1111111111111111111111111111111111111111";
const ZERO: &str = "0000000000000000000000000000000000000000";

#[test]
fn commit_data_is_kept_once_and_every_line_points_at_it() {
    let stdout = format!(
        "{A} 1 1 2\nauthor Ana Ruiz\nauthor-mail <ana@example.com>\nauthor-time 1790761320\n\
         author-tz +0200\nsummary Fix login\nfilename src/a.rs\n\tfirst\n{A} 2 2\n\tsecond\n\
         {ZERO} 3 3 1\nauthor Not Committed Yet\nauthor-mail <not.committed.yet>\n\
         author-time 1790761400\nauthor-tz +0000\nsummary Version of src/a.rs from src/a.rs\n\
         filename src/a.rs\n\tthird\n"
    );

    let blame = parse_blame(stdout.as_bytes()).unwrap();

    assert_eq!(blame.lines.len(), 3);
    assert_eq!(blame.commits.len(), 2);
    let first = &blame.commits[0];
    assert_eq!(first.sha, Sha::parse(A).unwrap());
    assert_eq!(first.author.name, "Ana Ruiz");
    assert_eq!(first.author.email, "ana@example.com");
    assert_eq!(first.authored_at, GitTime::new(1_790_761_320, 120));
    assert_eq!(first.summary, "Fix login");
    assert_eq!(first.path, "src/a.rs");
    assert!(!first.is_uncommitted());
    assert!(blame.commits[1].is_uncommitted());
    let lines: Vec<(u32, u32, &str)> = blame
        .lines
        .iter()
        .map(|l| (l.line, l.original_line, l.text.as_str()))
        .collect();
    assert_eq!(
        lines,
        vec![(1, 1, "first"), (2, 2, "second"), (3, 3, "third")]
    );
}

#[test]
fn a_line_without_header_or_a_bad_header_is_an_error() {
    assert!(parse_blame(b"\torphan\n").is_err());
    assert!(parse_blame(b"nothex 1 1\n").is_err());
    assert!(parse_blame(format!("{A} x 1\n").as_bytes()).is_err());
    assert_eq!(parse_blame(b"").unwrap().lines, Vec::new());
}
