use super::parse_stash_subject;

#[test]
fn a_stash_with_a_message_keeps_branch_and_message() {
    assert_eq!(
        parse_stash_subject("On main: half of the login form"),
        (Some("main".to_owned()), "half of the login form".to_owned())
    );
}

#[test]
fn a_stash_without_a_message_drops_the_short_sha() {
    assert_eq!(
        parse_stash_subject("WIP on feature/x: 1a2b3c4 Fix: the login"),
        (Some("feature/x".to_owned()), "Fix: the login".to_owned())
    );
}

#[test]
fn a_stash_from_a_detached_head_has_no_branch() {
    assert_eq!(
        parse_stash_subject("WIP on (no branch): 1a2b3c4 Try"),
        (None, "Try".to_owned())
    );
}

#[test]
fn an_unknown_subject_is_kept_whole() {
    assert_eq!(
        parse_stash_subject("autostash"),
        (None, "autostash".to_owned())
    );
}
