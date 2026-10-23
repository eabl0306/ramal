use git_rudder_core_application::GitVersion;

use super::{capabilities_for, parse_version};

#[test]
fn it_reads_the_usual_version_lines() {
    let cases = [
        ("git version 2.53.0\n", GitVersion::new(2, 53, 0)),
        ("git version 2.34.1", GitVersion::new(2, 34, 1)),
        (
            "git version 2.39.5 (Apple Git-154)",
            GitVersion::new(2, 39, 5),
        ),
        ("git version 2.45.1.windows.1", GitVersion::new(2, 45, 1)),
        ("git version 2.40", GitVersion::new(2, 40, 0)),
        ("git version 2.48.0-rc2", GitVersion::new(2, 48, 0)),
    ];
    for (line, expected) in cases {
        assert_eq!(parse_version(line), Some(expected), "{line}");
    }
}

#[test]
fn it_rejects_output_that_is_not_a_version() {
    for line in ["", "hub version 2.14", "git version", "git version abc"] {
        assert_eq!(parse_version(line), None, "{line}");
    }
}

#[test]
fn capabilities_arrive_with_git_2_38() {
    let old = capabilities_for(GitVersion::new(2, 37, 9));
    assert!(!old.merge_tree_write_tree);
    assert!(!old.rebase_update_refs);

    let modern = capabilities_for(GitVersion::new(2, 38, 0));
    assert!(modern.merge_tree_write_tree);
    assert!(modern.rebase_update_refs);
}
