//! Ramas con upstream, remotos, stashes, submodulos, conflictos y la inspeccion
//! de confianza contra repositorios reales de prueba.

#![allow(
    unused_crate_dependencies,
    reason = "the lint runs per target and this test does not use every dependency"
)]
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

mod support;

use git_rudder_core_application::{
    ConflictReader, GitReadError, GitReader, InventoryReader, LogQuery, RepoInspector,
};
use git_rudder_core_domain::conflict::ConflictSegment;
use git_rudder_core_domain::status::{AheadBehind, ConflictCode};
use git_rudder_core_domain::submodule::SubmoduleState;
use git_rudder_core_domain::trust::TrustFinding;
use git_rudder_core_domain::{RepoPath, RepoRoot, Sha, TrustLevel};
use git_rudder_git::{GitCliConfig, GitCliReader};
use pretty_assertions::assert_eq;
use support::FixtureRepo;

fn reader() -> GitCliReader {
    GitCliReader::new(GitCliConfig::default())
}

fn root(repo: &FixtureRepo) -> RepoRoot {
    RepoRoot::new(repo.path())
}

#[tokio::test]
async fn branches_know_their_upstream_and_remotes_their_web_page() {
    let upstream = FixtureRepo::with_merged_feature();
    let repo = FixtureRepo::new();
    repo.config("remote.origin.url", &upstream.path().to_string_lossy());
    repo.config("remote.origin.fetch", "+refs/heads/*:refs/remotes/origin/*");
    repo.config("remote.mirror.url", "git@github.com:acme/rudder.git");
    repo.git(&["fetch", "--quiet", "origin"]);
    repo.git(&[
        "checkout",
        "--quiet",
        "-b",
        "main",
        "--track",
        "origin/main",
    ]);
    repo.commit("Ahead one");
    repo.git(&["branch", "a,b", "HEAD~1"]);
    repo.git(&["branch", "--set-upstream-to=origin/feature", "a,b"]);
    repo.git(&["branch", "lonely"]);

    let listing = reader().list_refs(&root(&repo)).await.unwrap().sorted();

    let names: Vec<&str> = listing.local.iter().map(|b| b.name.short_name()).collect();
    assert_eq!(names, vec!["a,b", "lonely", "main"]);
    let main = &listing.local[2];
    assert!(main.is_head);
    let tracking = main.tracking.as_ref().unwrap();
    assert_eq!(tracking.upstream, "refs/remotes/origin/main");
    assert_eq!(
        tracking.distance,
        Some(AheadBehind {
            ahead: 1,
            behind: 0
        })
    );
    assert_eq!(listing.local[1].tracking, None, "no upstream is not 0/0");
    assert!(
        listing.local[0]
            .tracking
            .as_ref()
            .unwrap()
            .distance
            .is_some()
    );
    let remotes: Vec<(&str, Option<String>)> = listing
        .remotes
        .iter()
        .map(|r| (r.name.as_str(), r.web_url()))
        .collect();
    assert_eq!(
        remotes,
        vec![
            ("mirror", Some("https://github.com/acme/rudder".to_owned())),
            ("origin", None),
        ]
    );
    assert!(
        listing
            .remote
            .iter()
            .all(|b| b.name.short_name() != "origin/HEAD")
    );
}

#[tokio::test]
async fn a_stash_shows_its_files_and_untracked_ones_but_not_in_the_graph() {
    let repo = FixtureRepo::new();
    repo.commit_files(&[("a.txt", "a\n")], "Initial");
    repo.write("a.txt", "changed\n");
    repo.write("new file.txt", "untracked\n");
    repo.stash("half done");

    let stashes = reader().list_stashes(&root(&repo)).await.unwrap();
    let contents = reader()
        .stash_contents(&root(&repo), TrustLevel::Untrusted, stashes[0].sha)
        .await
        .unwrap();
    let log = reader()
        .read_log(&root(&repo), &LogQuery::first(50))
        .await
        .unwrap();

    assert_eq!(stashes.len(), 1);
    assert_eq!(stashes[0].index, 0);
    assert_eq!(stashes[0].message, "half done");
    assert_eq!(stashes[0].branch.as_deref(), Some("main"));
    let tracked: Vec<&str> = contents.tracked.iter().map(|d| d.path.as_str()).collect();
    let untracked: Vec<&str> = contents.untracked.iter().map(|d| d.path.as_str()).collect();
    assert_eq!(tracked, vec!["a.txt"]);
    assert_eq!(untracked, vec!["new file.txt"]);
    assert_eq!(log.len(), 1, "the stash commits are not project history");
    let empty = FixtureRepo::new();
    assert_eq!(
        reader().list_stashes(&root(&empty)).await.unwrap(),
        Vec::new()
    );
}

#[tokio::test]
async fn submodules_report_uninitialized_modified_up_to_date_and_nested() {
    let leaf = FixtureRepo::new();
    leaf.commit("Leaf");
    let middle = FixtureRepo::new();
    middle.commit("Middle");
    middle.add_submodule(&leaf, "leaf");
    let repo = FixtureRepo::new();
    repo.commit("Top");
    repo.add_submodule(&middle, "libs/middle");
    repo.git(&[
        "-c",
        "protocol.file.allow=always",
        "submodule",
        "update",
        "--init",
        "--recursive",
        "--quiet",
    ]);
    let other = FixtureRepo::new();
    other.commit("Other");
    repo.add_submodule(&other, "other");
    std::fs::remove_dir_all(repo.path().join("other")).unwrap();
    std::fs::create_dir(repo.path().join("other")).unwrap();
    repo.git(&[
        "-C",
        "libs/middle",
        "commit",
        "--quiet",
        "--allow-empty",
        "-m",
        "moved",
    ]);

    let submodules = reader().list_submodules(&root(&repo)).await.unwrap();

    let shape: Vec<(&str, SubmoduleState)> = submodules
        .iter()
        .map(|s| (s.path.as_str(), s.state))
        .collect();
    assert_eq!(
        shape,
        vec![
            ("libs/middle", SubmoduleState::Modified),
            ("libs/middle/leaf", SubmoduleState::UpToDate),
            ("other", SubmoduleState::Uninitialized),
        ]
    );
    assert!(submodules[1].can_open());
    assert!(!submodules[2].can_open());
}

fn conflicted(repo: &FixtureRepo, base: &str, ours: &str, theirs: &str) {
    repo.commit_files(&[("a.txt", base)], "Base");
    repo.checkout_new("feature");
    repo.commit_files(&[("a.txt", theirs)], "Theirs");
    repo.checkout("main");
    repo.commit_files(&[("a.txt", ours)], "Ours");
    assert!(!repo.try_git(&["merge", "--quiet", "feature"]));
}

#[tokio::test]
async fn a_text_conflict_comes_in_zdiff3_blocks_with_its_base() {
    let repo = FixtureRepo::new();
    conflicted(
        &repo,
        "keep\nold\nend\n",
        "keep\nours\nend\n",
        "keep\ntheirs\nend\n",
    );

    let file = reader()
        .read_conflict(&root(&repo), &RepoPath::parse("a.txt").unwrap())
        .await
        .unwrap();

    assert_eq!(file.code, ConflictCode::BothModified);
    assert!(!file.binary);
    let segments = file.content.unwrap().segments;
    assert_eq!(
        segments[1],
        ConflictSegment::Hunk {
            ours: vec!["ours".to_owned()],
            base: Some(vec!["old".to_owned()]),
            theirs: vec!["theirs".to_owned()],
        }
    );
    let not_conflicted = reader()
        .read_conflict(&root(&repo), &RepoPath::parse("nope.txt").unwrap())
        .await;
    assert!(matches!(not_conflicted, Err(GitReadError::Missing { .. })));
}

#[tokio::test]
async fn binary_and_deleted_conflicts_are_chosen_whole() {
    let repo = FixtureRepo::new();
    repo.commit_files(&[("doc.txt", "v1\n"), ("bin.dat", "x")], "Base");
    repo.checkout_new("feature");
    repo.git(&["rm", "--quiet", "doc.txt"]);
    repo.write_bytes("bin.dat", &[0, 1, 2, 3]);
    repo.add(&["bin.dat"]);
    repo.commit("Theirs");
    repo.checkout("main");
    repo.write("doc.txt", "v2\n");
    repo.write_bytes("bin.dat", &[0, 9, 9, 9]);
    repo.add(&["doc.txt", "bin.dat"]);
    repo.commit("Ours");
    assert!(!repo.try_git(&["merge", "--quiet", "feature"]));

    let deleted = reader()
        .read_conflict(&root(&repo), &RepoPath::parse("doc.txt").unwrap())
        .await
        .unwrap();
    let binary = reader()
        .read_conflict(&root(&repo), &RepoPath::parse("bin.dat").unwrap())
        .await
        .unwrap();

    assert_eq!(deleted.code, ConflictCode::DeletedByThem);
    assert_eq!(deleted.content, None);
    assert!(binary.binary);
    assert_eq!(binary.content, None);
}

#[tokio::test]
async fn the_inspection_finds_risky_keys_and_active_hooks_without_running_them() {
    let repo = FixtureRepo::new();
    repo.config("core.fsmonitor", "touch /tmp/should-not-run");
    repo.config("include.path", "/etc/evil.config");
    std::fs::write(repo.path().join(".git/hooks/post-checkout"), "#!/bin/sh\n").unwrap();

    let inspection = reader().inspect(&root(&repo)).await.unwrap();
    let findings = inspection.findings();

    assert!(findings.contains(&TrustFinding::ConfigKey {
        key: "core.fsmonitor".to_owned(),
        origin: "file:.git/config".to_owned(),
    }));
    assert!(
        findings
            .iter()
            .any(|f| matches!(f, TrustFinding::ConfigKey { key, .. } if key == "include.path"))
    );
    assert!(findings.contains(&TrustFinding::Hook {
        name: "post-checkout".to_owned()
    }));
    assert!(
        !findings
            .iter()
            .any(|f| matches!(f, TrustFinding::Hook { name } if name.ends_with(".sample"))),
        "sample hooks are not active"
    );
    let clean = FixtureRepo::new();
    assert_eq!(
        reader().inspect(&root(&clean)).await.unwrap().findings(),
        Vec::new()
    );
    let head = reader().read_refs(&root(&repo)).await.unwrap();
    assert_eq!(head.head_target(), None::<Sha>);
}
