#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use git_rudder_core_domain::status::AheadBehind;
use git_rudder_core_domain::submodule::SubmoduleState;
use pretty_assertions::assert_eq;

use super::{parse_listing, parse_remotes, parse_stashes, parse_submodules};

const A: &str = "1111111111111111111111111111111111111111";
const B: &str = "2222222222222222222222222222222222222222";

fn line(fields: &[&str]) -> String {
    fields.join("\0")
}

fn listing() -> git_rudder_core_domain::refs::RefListing {
    let stdout = [
        line(&[
            "refs/heads/main",
            A,
            "",
            "commit",
            "refs/remotes/origin/main",
            "ahead 1, behind 2",
            "Fix",
            "",
        ]),
        line(&[
            "refs/heads/a,b",
            A,
            "",
            "commit",
            "refs/remotes/origin/x",
            "gone",
            "",
            "",
        ]),
        line(&["refs/heads/local", A, "", "commit", "", "", "", ""]),
        line(&[
            "refs/heads/synced",
            A,
            "",
            "commit",
            "refs/remotes/origin/synced",
            "",
            "",
            "",
        ]),
        line(&[
            "refs/remotes/origin/HEAD",
            A,
            "",
            "commit",
            "",
            "",
            "",
            "refs/remotes/origin/main",
        ]),
        line(&["refs/remotes/origin/main", B, "", "commit", "", "", "", ""]),
        line(&["refs/tags/v1", B, A, "tag", "", "", "Release 1", ""]),
        line(&["refs/tags/light", A, "", "commit", "", "", "Fix", ""]),
    ]
    .join("\n");

    parse_listing(stdout.as_bytes(), Some("refs/heads/main")).unwrap()
}

#[test]
fn branches_carry_their_upstream_distance() {
    let listing = listing();

    assert_eq!(listing.local.len(), 4);
    assert!(listing.local[0].is_head);
    let main = listing.local[0].tracking.as_ref().unwrap();
    assert_eq!(
        main.distance,
        Some(AheadBehind {
            ahead: 1,
            behind: 2
        })
    );
    assert_eq!(listing.local[1].name.as_str(), "refs/heads/a,b");
    assert!(listing.local[1].tracking.as_ref().unwrap().is_gone());
    assert_eq!(listing.local[2].tracking, None);
    assert_eq!(
        listing.local[3].tracking.as_ref().unwrap().distance,
        Some(AheadBehind::default())
    );
}

#[test]
fn remote_heads_are_skipped_and_tags_keep_their_annotation() {
    let listing = listing();

    assert_eq!(listing.remote.len(), 1, "origin/HEAD is not a branch");
    assert_eq!(
        listing.tags[0].target.to_hex(),
        A,
        "annotated tags are peeled"
    );
    assert_eq!(listing.tags[0].annotation.as_deref(), Some("Release 1"));
    assert_eq!(listing.tags[1].annotation, None);
}

#[test]
fn a_broken_listing_is_an_error() {
    assert!(parse_listing(b"only\0two", None).is_err());
    let bad = line(&["refs/heads/x", A, "", "commit", "u", "sideways 3", "", ""]);
    assert!(parse_listing(bad.as_bytes(), None).is_err());
}

#[test]
fn remotes_with_dots_in_the_name_and_push_urls_are_read() {
    let stdout = "remote.origin.url\ngit@github.com:acme/r.git\0\
                  remote.my.fork.url\nhttps://example.com/me/r\0\
                  remote.my.fork.pushurl\nssh://git@example.com/me/r\0\
                  remote.origin.url\nsecond-url-ignored\0";

    let remotes = parse_remotes(stdout.as_bytes());

    assert_eq!(remotes.len(), 2);
    assert_eq!(
        remotes[0].fetch_url.as_deref(),
        Some("git@github.com:acme/r.git")
    );
    assert_eq!(remotes[1].name, "my.fork");
    assert_eq!(
        remotes[1].push_url.as_deref(),
        Some("ssh://git@example.com/me/r")
    );
}

#[test]
fn stashes_keep_their_index_branch_and_message() {
    let stdout = format!(
        "{A}\nstash@{{0}}\nOn main: half done\n1790761320\0\n{B}\nstash@{{1}}\nWIP on x: 1a2b3c4 Try\n1790761000\0"
    );

    let stashes = parse_stashes(stdout.as_bytes()).unwrap();

    assert_eq!(stashes.len(), 2);
    assert_eq!(stashes[0].index, 0);
    assert_eq!(stashes[0].branch.as_deref(), Some("main"));
    assert_eq!(stashes[0].message, "half done");
    assert_eq!(stashes[1].message, "Try");
    assert_eq!(stashes[1].created_at.seconds(), 1_790_761_000);
    assert!(parse_stashes(b"x\nnot-a-selector\ns\n1\0").is_err());
}

#[test]
fn submodule_states_paths_with_spaces_and_descriptions_are_read() {
    let stdout = format!(
        " {A} vendor/lib (v1.2-3-gabc1234)\n-{B} vendor/not yet\n+{A} vendor/lib/nested (heads/main)\nU{B} clash\n"
    );

    let submodules = parse_submodules(stdout.as_bytes()).unwrap();

    let shape: Vec<(&str, SubmoduleState, Option<&str>)> = submodules
        .iter()
        .map(|s| (s.path.as_str(), s.state, s.describe.as_deref()))
        .collect();
    assert_eq!(
        shape,
        vec![
            (
                "vendor/lib",
                SubmoduleState::UpToDate,
                Some("v1.2-3-gabc1234")
            ),
            ("vendor/not yet", SubmoduleState::Uninitialized, None),
            (
                "vendor/lib/nested",
                SubmoduleState::Modified,
                Some("heads/main")
            ),
            ("clash", SubmoduleState::Conflicted, None),
        ]
    );
    assert!(parse_submodules(b"?zzz path\n").is_err());
}
