#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use pretty_assertions::assert_eq;
use ramal_core_domain::{GitRef, RefName, Sha};

use super::parse_refs;

const A: &str = "1111111111111111111111111111111111111111";
const T: &str = "4444444444444444444444444444444444444444";
const TREE: &str = "5555555555555555555555555555555555555555";

fn line(fields: [&str; 6]) -> String {
    format!("{}\n", fields.join("\0"))
}

fn git_ref(name: &str, target: &str) -> GitRef {
    GitRef {
        name: RefName::parse(name).unwrap(),
        target: Sha::parse(target).unwrap(),
    }
}

#[test]
fn branches_remotes_and_tags_point_at_commits() {
    let stdout = [
        line(["refs/heads/feature/a,b", "commit", A, "", "", ""]),
        line(["refs/remotes/origin/main", "commit", A, "", "", ""]),
        line(["refs/tags/light", "commit", A, "", "", ""]),
        line(["refs/tags/v1.0", "tag", T, "commit", A, ""]),
    ]
    .concat();

    let parsed = parse_refs(stdout.as_bytes()).unwrap();

    assert_eq!(
        parsed.refs,
        vec![
            git_ref("refs/heads/feature/a,b", A),
            git_ref("refs/remotes/origin/main", A),
            git_ref("refs/tags/light", A),
            git_ref("refs/tags/v1.0", A),
        ]
    );
    assert_eq!(parsed.remote_default, None);
}

#[test]
fn origin_head_names_the_default_branch_and_is_not_a_ref() {
    let stdout = [
        line([
            "refs/remotes/origin/HEAD",
            "commit",
            A,
            "",
            "",
            "refs/remotes/origin/main",
        ]),
        line([
            "refs/remotes/upstream/HEAD",
            "commit",
            A,
            "",
            "",
            "refs/remotes/upstream/x",
        ]),
        line(["refs/remotes/origin/main", "commit", A, "", "", ""]),
    ]
    .concat();

    let parsed = parse_refs(stdout.as_bytes()).unwrap();

    assert_eq!(parsed.refs, vec![git_ref("refs/remotes/origin/main", A)]);
    assert_eq!(
        parsed.remote_default,
        Some(RefName::parse("refs/remotes/origin/main").unwrap())
    );
}

#[test]
fn tags_that_do_not_end_in_a_commit_are_skipped() {
    let stdout = [
        line(["refs/tags/tree-tag", "tag", T, "tree", TREE, ""]),
        line(["refs/tags/raw-tree", "tree", TREE, "", "", ""]),
    ]
    .concat();

    let snapshot = parse_refs(stdout.as_bytes()).unwrap();

    assert!(snapshot.refs.is_empty(), "{:?}", snapshot.refs);
}

#[test]
fn malformed_lines_are_errors() {
    assert!(parse_refs(b"refs/heads/main\0commit\n").is_err());
    assert!(
        parse_refs(line(["refs/heads/main", "commit", "nope", "", "", ""]).as_bytes()).is_err()
    );
    assert!(parse_refs(line(["refs/stash", "commit", A, "", "", ""]).as_bytes()).is_err());
    let empty = parse_refs(b"").unwrap();
    assert!(empty.refs.is_empty(), "{:?}", empty.refs);
}
