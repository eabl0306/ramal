#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use git_rudder_core_domain::Sha;
use git_rudder_core_domain::status::{
    AheadBehind, ChangeKind, ConflictCode, EntryState, SubmoduleChange, WorkingTreeStatus,
};
use pretty_assertions::assert_eq;

use super::parse_status;

const OID: &str = "e19fdb16479b57ca73ded9a8bbe042b638bff6e2";
const BLOB: &str = "78981922613b2afb6025042ff6bd878ac1994e85";

fn nul(fields: &[&str]) -> Vec<u8> {
    let mut out = fields.join("\0").into_bytes();
    out.push(0);
    out
}

fn sample() -> WorkingTreeStatus {
    let ordinary = format!("1 .M N... 100644 100644 100644 {BLOB} {BLOB} a file.txt");
    let rename = format!("2 R. N... 100644 100644 100644 {BLOB} {BLOB} R100 c d.txt");
    let conflict = format!("u UU N... 100644 100644 100644 100644 {BLOB} {BLOB} {BLOB} clash.txt");
    let stdout = nul(&[
        &format!("# branch.oid {OID}"),
        "# branch.head feature/ñ",
        "# branch.upstream origin/feature/ñ",
        "# branch.ab +2 -3",
        &ordinary,
        &rename,
        "b.txt",
        &conflict,
        "? nuevo fichero.md",
        "! target/",
    ]);
    parse_status(&stdout).unwrap()
}

#[test]
fn branch_headers_are_read() {
    let status = sample();

    assert_eq!(status.branch.head_sha, Some(Sha::parse(OID).unwrap()));
    assert_eq!(status.branch.branch.as_deref(), Some("feature/ñ"));
    assert_eq!(status.branch.upstream.as_deref(), Some("origin/feature/ñ"));
    assert_eq!(
        status.branch.ahead_behind,
        Some(AheadBehind {
            ahead: 2,
            behind: 3
        })
    );
}

#[test]
fn every_kind_of_entry_is_read() {
    let status = sample();

    let paths: Vec<&str> = status.entries.iter().map(|e| e.path.as_str()).collect();
    assert_eq!(
        paths,
        vec![
            "a file.txt",
            "c d.txt",
            "clash.txt",
            "nuevo fichero.md",
            "target/"
        ]
    );
    assert_eq!(status.entries[1].original_path.as_deref(), Some("b.txt"));
    assert_eq!(
        status.entries[1].state,
        EntryState::Changed {
            index: ChangeKind::Renamed,
            worktree: ChangeKind::Unmodified,
            submodule: None,
        }
    );
    assert_eq!(
        status.entries[2].state,
        EntryState::Unmerged {
            conflict: ConflictCode::BothModified
        }
    );
    assert_eq!(status.entries[3].state, EntryState::Untracked);
    assert_eq!(status.entries[4].state, EntryState::Ignored);
}

#[test]
fn an_unborn_branch_and_a_detached_head_have_no_sha_or_branch() {
    let unborn = parse_status(&nul(&["# branch.oid (initial)", "# branch.head main"])).unwrap();
    assert!(unborn.branch.is_unborn());
    assert_eq!(unborn.branch.branch.as_deref(), Some("main"));

    let detached = parse_status(&nul(&[
        &format!("# branch.oid {OID}"),
        "# branch.head (detached)",
    ]))
    .unwrap();
    assert!(detached.branch.is_detached());
    assert!(!detached.branch.is_unborn());
}

#[test]
fn a_dirty_submodule_reports_what_changed_inside() {
    let entry = format!("1 .M SCMU 160000 160000 160000 {BLOB} {BLOB} vendor/lib");

    let status = parse_status(&nul(&[&entry])).unwrap();

    assert_eq!(
        status.entries[0].state,
        EntryState::Changed {
            index: ChangeKind::Unmodified,
            worktree: ChangeKind::Modified,
            submodule: Some(SubmoduleChange {
                commit_changed: true,
                has_tracked_changes: true,
                has_untracked: true,
            }),
        }
    );
}

#[test]
fn every_conflict_code_is_known() {
    for (xy, code) in [
        ("DD", ConflictCode::BothDeleted),
        ("AU", ConflictCode::AddedByUs),
        ("UD", ConflictCode::DeletedByThem),
        ("UA", ConflictCode::AddedByThem),
        ("DU", ConflictCode::DeletedByUs),
        ("AA", ConflictCode::BothAdded),
    ] {
        let entry = format!("u {xy} N... 1 2 3 4 {BLOB} {BLOB} {BLOB} f");
        let status = parse_status(&nul(&[&entry])).unwrap();
        assert_eq!(
            status.entries[0].state,
            EntryState::Unmerged { conflict: code }
        );
    }
}

#[test]
fn broken_entries_are_errors() {
    for field in [
        "1 .M N... short",
        "2 R. N... 1 2 3 a b R100 path-without-origin",
        "u XX N... 1 2 3 4 a b c f",
        "1 .Z N... 1 2 3 a b f",
        "1 .M Q... 1 2 3 a b f",
        "# branch.ab 2 3",
        "# branch.oid nothex",
        "x what",
    ] {
        assert!(parse_status(&nul(&[field])).is_err(), "{field}");
    }
}
