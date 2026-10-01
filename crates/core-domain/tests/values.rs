//! Tipos de valor del dominio: sha, forma corta, tiempo de git, refs y repos.

#![allow(
    unused_crate_dependencies,
    reason = "integration tests only use the crate under test"
)]
#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

mod support;

use ramal_core_domain::{
    CommitNode, GitTime, Head, RefError, RefKind, RefName, RefSnapshot, RepoId, RepoRoot, Sha,
    ShaError, ShortSha, TimeError,
};
use support::{git_ref, sha};

const HEX: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn a_sha_round_trips_through_lowercase_hex() {
    let parsed = Sha::parse(HEX).unwrap();

    assert_eq!(parsed.to_hex(), HEX);
    assert_eq!(parsed.to_string(), HEX);
    assert_eq!(format!("{parsed:?}"), format!("Sha({HEX})"));
    assert_eq!(HEX.to_uppercase().parse::<Sha>().unwrap(), parsed);
}

#[test]
fn a_sha_rejects_bad_length_and_characters() {
    assert_eq!(
        Sha::parse("abc"),
        Err(ShaError::InvalidLength {
            expected: "40",
            found: 3
        })
    );
    let bad = format!("{}g", &HEX[..39]);
    assert_eq!(Sha::parse(&bad), Err(ShaError::InvalidCharacter));
    assert!(ShaError::InvalidCharacter.to_string().contains("hex"));
}

#[test]
fn a_short_sha_accepts_four_to_forty_hex_characters() {
    assert_eq!(ShortSha::parse("A41F9C2").unwrap().as_str(), "a41f9c2");
    assert_eq!(ShortSha::parse("abcd").unwrap().to_string(), "abcd");
    assert!(ShortSha::parse(HEX).is_ok());
    assert!(ShortSha::parse("abc").is_err());
    assert_eq!(ShortSha::parse("abcz"), Err(ShaError::InvalidCharacter));
}

#[test]
fn git_time_reads_the_strict_iso_format_of_git() {
    let time = GitTime::parse_iso8601("2026-09-30T11:42:00+02:00").unwrap();

    assert_eq!(time.seconds(), 1_790_761_320);
    assert_eq!(time.offset_minutes(), 120);
    assert_eq!(time.to_iso8601(), "2026-09-30T11:42:00+02:00");
}

#[test]
fn git_time_accepts_z_and_negative_offsets() {
    let utc = GitTime::parse_iso8601("1970-01-01T00:00:00Z").unwrap();
    assert_eq!(utc.seconds(), 0);
    assert_eq!(utc.to_string(), "1970-01-01T00:00:00+00:00");

    let west = GitTime::parse_iso8601("2000-02-29T23:30:15-05:30").unwrap();
    assert_eq!(west.offset_minutes(), -330);
    assert_eq!(west.seconds(), 951_886_815);
    assert_eq!(west.to_string(), "2000-02-29T23:30:15-05:30");
}

#[test]
fn git_time_round_trips_dates_before_the_epoch_and_far_ahead() {
    for text in [
        "1969-12-31T23:59:59+00:00",
        "1900-03-01T12:00:00+01:00",
        "2400-02-29T00:00:00-11:00",
        "9999-12-31T23:59:59+14:00",
    ] {
        assert_eq!(GitTime::parse_iso8601(text).unwrap().to_string(), text);
    }
    assert_eq!(GitTime::new(-1, 0).to_string(), "1969-12-31T23:59:59+00:00");
}

#[test]
fn git_time_rejects_malformed_text() {
    for text in [
        "",
        "2026-09-30",
        "2026-09-30 11:42:00+02:00",
        "2026-09-30T11:42:00",
        "2026-09-30T11:42:00+0200",
        "2026-09-30T11:42:00+02-00",
        "+026-09-30T11:42:00Z",
        "2026-09-30T11:42:00Y",
        "2026-09-30T11:42:00+02:60",
    ] {
        assert!(
            matches!(GitTime::parse_iso8601(text), Err(TimeError::Malformed(_))),
            "{text}"
        );
    }
}

#[test]
fn git_time_rejects_fields_out_of_range() {
    for text in [
        "2026-13-01T00:00:00Z",
        "2026-04-31T00:00:00Z",
        "2026-02-29T00:00:00Z",
        "2026-01-01T24:00:00Z",
        "2026-01-01T00:60:00Z",
        "2026-01-01T00:00:60Z",
    ] {
        let error = GitTime::parse_iso8601(text).unwrap_err();
        assert!(matches!(error, TimeError::OutOfRange(_)), "{text}");
        assert!(error.to_string().contains(text));
    }
}

#[test]
fn ref_names_know_their_kind_and_short_name() {
    let local = RefName::parse("refs/heads/feature/graph").unwrap();
    assert_eq!(local.kind(), RefKind::LocalBranch);
    assert_eq!(local.short_name(), "feature/graph");

    let remote = RefName::parse("refs/remotes/origin/main").unwrap();
    assert_eq!(remote.kind(), RefKind::RemoteBranch);
    assert_eq!(remote.short_name(), "origin/main");

    let tag = RefName::parse("refs/tags/v2.3,1").unwrap();
    assert_eq!(tag.kind(), RefKind::Tag);
    assert_eq!(tag.short_name(), "v2.3,1");
    assert_eq!(tag.as_str(), "refs/tags/v2.3,1");
}

#[test]
fn ref_names_reject_other_namespaces() {
    for name in ["refs/stash", "refs/heads/", "HEAD", "refs/notes/commits"] {
        assert_eq!(
            RefName::parse(name),
            Err(RefError::Unsupported(name.to_owned()))
        );
    }
}

#[test]
fn a_snapshot_reports_head_and_emptiness() {
    let unborn = RefSnapshot::new(
        vec![],
        Head::Branch {
            name: RefName::parse("refs/heads/main").unwrap(),
            target: None,
        },
        None,
    );
    assert!(unborn.is_empty());
    assert_eq!(unborn.head_target(), None);
    assert_eq!(unborn.head_branch().unwrap().short_name(), "main");
    assert!(!unborn.is_detached());

    let detached = RefSnapshot::new(
        vec![git_ref("refs/heads/main", 1)],
        Head::Detached { target: sha(2) },
        None,
    );
    assert!(!detached.is_empty());
    assert!(detached.is_detached());
    assert_eq!(detached.head_branch(), None);
    assert_eq!(detached.head_target(), Some(sha(2)));
    assert_eq!(detached.head(), &Head::Detached { target: sha(2) });
}

#[test]
fn a_snapshot_sorts_refs_by_kind_and_name() {
    let snapshot = RefSnapshot::new(
        vec![
            git_ref("refs/tags/a", 1),
            git_ref("refs/heads/b", 1),
            git_ref("refs/remotes/origin/a", 1),
            git_ref("refs/heads/a", 1),
        ],
        Head::Detached { target: sha(1) },
        None,
    );
    let names: Vec<&str> = snapshot.refs().iter().map(|r| r.name.as_str()).collect();

    assert_eq!(
        names,
        vec![
            "refs/heads/a",
            "refs/heads/b",
            "refs/remotes/origin/a",
            "refs/tags/a"
        ]
    );
}

fn on_main(refs: Vec<ramal_core_domain::GitRef>, remote: Option<&str>) -> RefSnapshot {
    RefSnapshot::new(
        refs,
        Head::Detached { target: sha(9) },
        remote.map(|name| RefName::parse(name).unwrap()),
    )
}

#[test]
fn the_default_branch_follows_origin_head_and_prefers_the_local_branch() {
    let refs = vec![
        git_ref("refs/heads/trunk", 1),
        git_ref("refs/remotes/origin/trunk", 2),
        git_ref("refs/heads/main", 3),
    ];
    assert_eq!(
        on_main(refs, Some("refs/remotes/origin/trunk")).default_branch_tip(),
        Some(sha(1))
    );

    let only_remote = vec![
        git_ref("refs/remotes/origin/trunk", 2),
        git_ref("refs/heads/main", 3),
    ];
    assert_eq!(
        on_main(only_remote, Some("refs/remotes/origin/trunk")).default_branch_tip(),
        Some(sha(2))
    );
}

#[test]
fn the_default_branch_falls_back_to_main_then_master() {
    let both = vec![
        git_ref("refs/heads/master", 1),
        git_ref("refs/heads/main", 2),
    ];
    assert_eq!(on_main(both, None).default_branch_tip(), Some(sha(2)));

    let master = vec![git_ref("refs/heads/master", 1)];
    assert_eq!(on_main(master, None).default_branch_tip(), Some(sha(1)));

    let stale_remote = vec![git_ref("refs/heads/master", 1)];
    assert_eq!(
        on_main(stale_remote, Some("refs/remotes/origin/gone")).default_branch_tip(),
        Some(sha(1))
    );

    assert_eq!(
        on_main(vec![git_ref("refs/heads/dev", 1)], None).default_branch_tip(),
        None
    );
}

#[test]
fn repo_identities_wrap_their_values() {
    let root = RepoRoot::new("/work/ramal");
    assert_eq!(root.as_path(), std::path::Path::new("/work/ramal"));

    let id = RepoId::new("repo-1");
    assert_eq!(id.as_str(), "repo-1");
    assert_eq!(id.to_string(), "repo-1");
}

#[test]
fn a_commit_with_two_parents_is_a_merge() {
    assert!(CommitNode::new(sha(1), vec![sha(2), sha(3)]).is_merge());
    assert!(!CommitNode::new(sha(1), vec![sha(2)]).is_merge());
}
