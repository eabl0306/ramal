//! Diffs de verdad: arbol de trabajo, indice, commits y rangos; renombrados,
//! binarios, imagenes, LFS, ficheros enormes, palabras y espacios.

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

use git_rudder_core_application::{DiffRequest, GitReadError, StatusReader};
use git_rudder_core_domain::diff::{
    DiffContent, DiffOptions, DiffTarget, FileDiff, FileStatus, Hunk, LineKind, Whitespace,
};
use git_rudder_core_domain::{RepoPath, RepoRoot, Sha, TrustLevel};
use git_rudder_git::{GitCliConfig, GitCliReader};
use pretty_assertions::assert_eq;
use support::FixtureRepo;

fn request(target: DiffTarget, path: &str) -> DiffRequest {
    DiffRequest {
        target,
        path: RepoPath::parse(path).unwrap(),
        old_path: None,
        options: DiffOptions::default(),
    }
}

async fn diff(repo: &FixtureRepo, request: &DiffRequest) -> Result<FileDiff, GitReadError> {
    GitCliReader::new(GitCliConfig::default())
        .read_diff(&RepoRoot::new(repo.path()), TrustLevel::Untrusted, request)
        .await
}

fn hunks(diff: &FileDiff) -> &[Hunk] {
    let DiffContent::Text { hunks } = &diff.content else {
        unreachable!("expected text, got {:?}", diff.content)
    };
    hunks
}

fn sha(text: &str) -> Sha {
    Sha::parse(text).unwrap()
}

#[tokio::test]
async fn working_tree_index_commit_and_range_diffs_are_read() {
    let repo = FixtureRepo::new();
    let first = repo.commit_files(&[("a.txt", "one\ntwo\nthree\n")], "Initial");
    let second = repo.commit_files(&[("a.txt", "one\n2\nthree\n")], "Edit");
    repo.write("a.txt", "one\n2\nthree\nfour\n");
    repo.add(&["a.txt"]);
    repo.write("a.txt", "zero\none\n2\nthree\nfour\n");

    let worktree = diff(&repo, &request(DiffTarget::WorkingTree, "a.txt"))
        .await
        .unwrap();
    let index = diff(&repo, &request(DiffTarget::Index, "a.txt"))
        .await
        .unwrap();
    let commit = diff(&repo, &request(DiffTarget::Commit(sha(&second)), "a.txt"))
        .await
        .unwrap();
    let range = DiffTarget::Range {
        from: sha(&first),
        to: sha(&second),
    };
    let between = diff(&repo, &request(range, "a.txt")).await.unwrap();

    assert_eq!((worktree.additions, worktree.deletions), (1, 0));
    assert_eq!(hunks(&worktree)[0].lines[0].text, "zero");
    assert_eq!((index.additions, index.deletions), (1, 0));
    assert_eq!(hunks(&index)[0].lines.last().unwrap().text, "four");
    assert_eq!((commit.additions, commit.deletions), (1, 1));
    assert_eq!(commit, between);
    let kinds: Vec<LineKind> = hunks(&commit)[0].lines.iter().map(|l| l.kind).collect();
    assert_eq!(
        kinds,
        vec![
            LineKind::Context,
            LineKind::Removed,
            LineKind::Added,
            LineKind::Context
        ]
    );
}

#[tokio::test]
async fn a_root_commit_and_a_merge_diff_against_their_first_parent() {
    let repo = FixtureRepo::new();
    let root = repo.commit_files(&[("a.txt", "a\n")], "Initial");
    repo.checkout_new("feature");
    repo.commit_files(&[("b.txt", "b\n")], "Feature");
    repo.checkout("main");
    let merge = repo.merge(&["feature"], "Merge feature");

    let added = diff(&repo, &request(DiffTarget::Commit(sha(&root)), "a.txt"))
        .await
        .unwrap();
    let merged = diff(&repo, &request(DiffTarget::Commit(sha(&merge)), "b.txt"))
        .await
        .unwrap();

    assert_eq!(added.status, FileStatus::Added);
    assert_eq!(
        merged.status,
        FileStatus::Added,
        "b.txt is new against main"
    );
}

#[tokio::test]
async fn a_rename_keeps_both_paths() {
    let repo = FixtureRepo::new();
    repo.commit_files(
        &[("old name.txt", "same text\nfor rename detection\n")],
        "Initial",
    );
    let renamed = repo.rename("old name.txt", "new name.txt", "Rename");
    let request = DiffRequest {
        old_path: Some(RepoPath::parse("old name.txt").unwrap()),
        ..request(DiffTarget::Commit(sha(&renamed)), "new name.txt")
    };

    let diff = diff(&repo, &request).await.unwrap();

    assert_eq!(diff.status, FileStatus::Renamed);
    assert_eq!(diff.old_path.as_deref(), Some("old name.txt"));
    assert_eq!(diff.path, "new name.txt");
    assert_eq!(hunks(&diff), &[]);
}

#[tokio::test]
async fn binaries_images_and_lfs_pointers_are_marked_without_bytes() {
    let repo = FixtureRepo::new();
    repo.commit_files(&[("keep.txt", "x\n")], "Initial");
    repo.write_bytes("logo.png", &[0x89, b'P', b'N', b'G', 0, 1, 2, 3]);
    repo.write_bytes("data.bin", &[0, 159, 146, 150]);
    repo.write(
        "video.mp4",
        "version https://git-lfs.github.com/spec/v1\noid sha256:abc\nsize 1048576\n",
    );
    repo.add(&["logo.png", "data.bin", "video.mp4"]);

    let image = diff(&repo, &request(DiffTarget::Index, "logo.png"))
        .await
        .unwrap();
    let binary = diff(&repo, &request(DiffTarget::Index, "data.bin"))
        .await
        .unwrap();
    let lfs = diff(&repo, &request(DiffTarget::Index, "video.mp4"))
        .await
        .unwrap();

    assert_eq!(image.content, DiffContent::Binary { image: true });
    assert_eq!(binary.content, DiffContent::Binary { image: false });
    let DiffContent::Lfs { old, new } = lfs.content else {
        unreachable!("an LFS pointer is not text: {:?}", lfs.content)
    };
    assert_eq!(old, None);
    assert_eq!(new.unwrap().size, 1_048_576);
}

#[tokio::test]
async fn an_untracked_file_comes_whole_and_an_unchanged_one_is_empty() {
    let repo = FixtureRepo::new();
    repo.commit_files(&[("a.txt", "a\n")], "Initial");
    repo.write("new dir/nuevo ñ.txt", "uno\ndos\n");

    let untracked = diff(
        &repo,
        &request(DiffTarget::WorkingTree, "new dir/nuevo ñ.txt"),
    )
    .await
    .unwrap();
    let unchanged = diff(&repo, &request(DiffTarget::WorkingTree, "a.txt"))
        .await
        .unwrap();

    assert_eq!(untracked.status, FileStatus::Added);
    assert_eq!(untracked.additions, 2);
    assert_eq!(unchanged.content, DiffContent::Text { hunks: Vec::new() });
}

#[cfg(unix)]
#[tokio::test]
async fn an_untracked_path_through_a_symlink_out_of_the_tree_is_refused() {
    let repo = FixtureRepo::new();
    repo.commit_files(&[("a.txt", "a\n")], "Initial");
    let outside = repo.outside();
    std::fs::write(outside.join("secret.txt"), "secret\n").unwrap();
    std::os::unix::fs::symlink(&outside, repo.path().join("link")).unwrap();

    let result = diff(&repo, &request(DiffTarget::WorkingTree, "link/secret.txt")).await;

    assert!(
        matches!(
            result,
            Err(GitReadError::Missing { .. }) | Ok(FileDiff { additions: 0, .. })
        ),
        "the secret must not be read: {result:?}"
    );
}

#[tokio::test]
async fn a_huge_file_is_cut_with_a_warning() {
    let repo = FixtureRepo::new();
    repo.commit_files(&[("big.txt", "start\n")], "Initial");
    let line = "x".repeat(99);
    let big = (0..60_000).fold(String::new(), |mut big, n| {
        use std::fmt::Write as _;
        let _ = writeln!(big, "{n} {line}");
        big
    });
    repo.write("big.txt", &big);

    let diff = diff(&repo, &request(DiffTarget::WorkingTree, "big.txt"))
        .await
        .unwrap();

    assert!(diff.truncated);
    assert!(
        diff.additions > 1000 && diff.additions < 60_000,
        "{}",
        diff.additions
    );
}

#[tokio::test]
async fn word_diff_marks_the_changed_words_inside_a_line() {
    let repo = FixtureRepo::new();
    repo.commit_files(&[("w.txt", "the quick fox\n")], "Initial");
    repo.write("w.txt", "the slow fox\n");
    let request = DiffRequest {
        options: DiffOptions {
            word_diff: true,
            ..DiffOptions::default()
        },
        ..request(DiffTarget::WorkingTree, "w.txt")
    };

    let diff = diff(&repo, &request).await.unwrap();

    let line = &hunks(&diff)[0].lines[0];
    assert_eq!(line.text, "the slow fox");
    let changed: Vec<(LineKind, &str)> = line
        .words
        .iter()
        .filter(|w| w.kind != LineKind::Context)
        .map(|w| (w.kind, w.text.as_str()))
        .collect();
    assert_eq!(
        changed,
        vec![(LineKind::Removed, "quick"), (LineKind::Added, "slow")]
    );
}

async fn with_whitespace(repo: &FixtureRepo, whitespace: Whitespace) -> u32 {
    let request = DiffRequest {
        options: DiffOptions {
            whitespace,
            ..DiffOptions::default()
        },
        ..request(DiffTarget::WorkingTree, "s.txt")
    };
    let diff = diff(repo, &request).await.unwrap();
    diff.additions + diff.deletions
}

#[tokio::test]
async fn the_three_whitespace_modes_and_mixed_line_endings() {
    let repo = FixtureRepo::new();
    repo.commit_files(&[("s.txt", "a b\nc d\ne f\ng\n")], "Initial");
    // Linea 1: espacios de mas dentro; 2: espacios al final; 3: sin espacio; 4:
    // de LF a CRLF.
    repo.write_bytes("s.txt", b"a   b\nc d  \nef\ng\r\n");

    assert_eq!(with_whitespace(&repo, Whitespace::Exact).await, 8);
    assert_eq!(with_whitespace(&repo, Whitespace::IgnoreAtEol).await, 4);
    assert_eq!(with_whitespace(&repo, Whitespace::IgnoreChange).await, 2);
    assert_eq!(with_whitespace(&repo, Whitespace::IgnoreAll).await, 0);
}
