#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use git_rudder_core_domain::diff::FileStatus;
use git_rudder_core_domain::{ChangedFile, LineCounts};
use pretty_assertions::assert_eq;

use super::{parse_changes, status_of};

#[test]
fn name_status_and_numstat_are_joined_file_by_file() {
    let name_status = b"M\0src/a b.rs\0R087\0old.txt\0new.txt\0A\0logo.png\0";
    let numstat = b"3\t1\tsrc/a b.rs\x002\t2\t\x00old.txt\x00new.txt\x00-\t-\tlogo.png\x00";

    let files = parse_changes(name_status, numstat).unwrap();

    assert_eq!(
        files,
        vec![
            ChangedFile {
                path: "src/a b.rs".to_owned(),
                old_path: None,
                status: FileStatus::Modified,
                lines: Some(LineCounts {
                    additions: 3,
                    deletions: 1
                }),
            },
            ChangedFile {
                path: "new.txt".to_owned(),
                old_path: Some("old.txt".to_owned()),
                status: FileStatus::Renamed,
                lines: Some(LineCounts {
                    additions: 2,
                    deletions: 2
                }),
            },
            ChangedFile {
                path: "logo.png".to_owned(),
                old_path: None,
                status: FileStatus::Added,
                lines: None,
            },
        ]
    );
}

#[test]
fn every_status_letter_is_known_and_others_are_errors() {
    for (code, status) in [
        ("A", FileStatus::Added),
        ("M", FileStatus::Modified),
        ("U", FileStatus::Modified),
        ("D", FileStatus::Deleted),
        ("T", FileStatus::TypeChanged),
        ("R100", FileStatus::Renamed),
        ("C075", FileStatus::Copied),
    ] {
        assert_eq!(status_of(code).unwrap(), status, "{code}");
    }
    for code in ["X", "Rxx", ""] {
        assert!(status_of(code).is_err(), "{code:?}");
    }
}

#[test]
fn mismatched_outputs_and_missing_paths_are_errors() {
    assert!(parse_changes(b"M\0a\0", b"").is_err());
    assert!(parse_changes(b"R100\0only-one\0", b"1\t1\t\0a\0b\0").is_err());
    assert!(parse_changes(b"", b"bad\0").is_err());
    assert_eq!(parse_changes(b"", b"").unwrap(), Vec::new());
}
