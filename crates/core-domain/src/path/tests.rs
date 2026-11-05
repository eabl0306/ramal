#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use super::{PathError, RepoPath};

#[test]
fn a_plain_relative_path_is_kept_as_written() {
    let path = RepoPath::parse("src/main file.rs").unwrap();

    assert_eq!(path.as_str(), "src/main file.rs");
    assert_eq!(path.to_string(), "src/main file.rs");
    assert_eq!(path.file_name(), "main file.rs");
    assert_eq!(path.extension().as_deref(), Some("rs"));
}

#[test]
fn accents_and_dots_inside_names_are_fine() {
    for text in ["canción.md", "a..b/c", ".gitignore", "dir.d/x.tar.GZ"] {
        assert!(RepoPath::parse(text).is_ok(), "{text} should be accepted");
    }
    let hidden = RepoPath::parse(".gitignore").unwrap();
    assert_eq!(hidden.extension(), None, "a dot file has no extension");
    let archive = RepoPath::parse("dir.d/x.tar.GZ").unwrap();
    assert_eq!(archive.extension().as_deref(), Some("gz"));
}

#[test]
fn paths_that_could_leave_the_work_tree_are_rejected() {
    let cases = [
        ("", PathError::Empty),
        ("/etc/passwd", PathError::Absolute),
        ("\\server\\share", PathError::Absolute),
        ("C:/Windows", PathError::Absolute),
        ("../outside", PathError::ParentComponent),
        ("a/../../b", PathError::ParentComponent),
        ("a\\..\\b", PathError::ParentComponent),
        ("a//b", PathError::EmptyComponent),
        ("a/./b", PathError::EmptyComponent),
        ("dir/", PathError::EmptyComponent),
        ("a\0b", PathError::Nul),
    ];
    for (text, expected) in cases {
        assert_eq!(RepoPath::parse(text), Err(expected), "{text:?}");
    }
}

#[test]
fn errors_explain_the_rule_in_english() {
    assert_eq!(
        PathError::ParentComponent.to_string(),
        "the path climbs out with .."
    );
}
