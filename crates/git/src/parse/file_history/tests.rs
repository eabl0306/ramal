#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use git_rudder_core_domain::diff::FileStatus;
use pretty_assertions::assert_eq;

use super::parse_file_history;

const A: &str = "1111111111111111111111111111111111111111";
const B: &str = "2222222222222222222222222222222222222222";
const C: &str = "3333333333333333333333333333333333333333";

fn commit(sha: &str, parents: &str, summary: &str) -> String {
    format!(
        "{sha}\n{}\n{parents}\n1790761320 +0200\nAna\nana@example.com\n{summary}",
        &sha[..7]
    )
}

#[test]
fn renames_are_followed_and_each_entry_has_its_path() {
    let stdout = format!(
        "{}\0\nR087\0old name.txt\0new name.txt\0{}\0\nM\0old name.txt\0{}\0\nA\0old name.txt\0",
        commit(A, B, "Rename"),
        commit(B, C, "Edit"),
        commit(C, "", "Add"),
    );

    let entries = parse_file_history(stdout.as_bytes(), "new name.txt").unwrap();

    let shape: Vec<(&str, Option<&str>, FileStatus)> = entries
        .iter()
        .map(|e| (e.path.as_str(), e.old_path.as_deref(), e.status))
        .collect();
    assert_eq!(
        shape,
        vec![
            ("new name.txt", Some("old name.txt"), FileStatus::Renamed),
            ("old name.txt", None, FileStatus::Modified),
            ("old name.txt", None, FileStatus::Added),
        ]
    );
    assert_eq!(entries[1].commit.summary, "Edit");
}

#[test]
fn a_merge_without_files_keeps_the_current_path() {
    let stdout = format!(
        "{}\0{}\0\nM\0a.txt\0",
        commit(A, &format!("{B} {C}"), "Merge"),
        commit(B, C, "Edit"),
    );

    let entries = parse_file_history(stdout.as_bytes(), "a.txt").unwrap();

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].path, "a.txt");
    assert_eq!(entries[0].commit.node.parents.len(), 2);
}

#[test]
fn broken_output_is_an_error() {
    assert!(parse_file_history(b"not a commit\0", "a").is_err());
    let missing_path = format!("{}\0\nM\0", commit(A, "", "x"));
    assert!(parse_file_history(missing_path.as_bytes(), "a").is_err());
    assert_eq!(parse_file_history(b"", "a").unwrap(), Vec::new());
}
