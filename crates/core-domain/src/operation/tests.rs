use super::merge_source_name;

#[test]
fn the_incoming_branch_comes_from_every_message_git_writes() {
    let cases = [
        ("Merge branch 'feature'\n", "feature"),
        ("Merge branch 'feature' into main", "feature"),
        ("Merge branch 'fix/a' of https://example.com/r.git", "fix/a"),
        ("Merge remote-tracking branch 'origin/main'", "origin/main"),
        ("Merge tag 'v1.0'", "v1.0"),
        (
            "Merge commit 'abc1234'\n\n# Conflicts:\n#\ta.txt",
            "abc1234",
        ),
    ];
    for (message, expected) in cases {
        assert_eq!(
            merge_source_name(message).as_deref(),
            Some(expected),
            "{message}"
        );
    }
}

#[test]
fn a_hand_written_message_has_no_incoming_name() {
    for message in [
        "Join the two lines",
        "",
        "Merge branch ''",
        "Merge branch 'open",
    ] {
        assert_eq!(merge_source_name(message), None, "{message:?}");
    }
}
