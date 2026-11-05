//! La historia dispuesta por trozos (la cache del grafo) da las mismas ventanas
//! que disponerlo todo de una vez, y los filtros reescriben los padres.

#![allow(
    unused_crate_dependencies,
    reason = "integration tests only use the crate under test"
)]
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

mod support;

use git_rudder_core_domain::graph::{
    GraphFilter, LaidOutHistory, RefSelection, WindowRange, filter_history, graph_window,
    pinned_tips,
};
use git_rudder_core_domain::{CommitRecord, GitTime, RefName, RefSnapshot};
use support::{git_ref, record, sha, snapshot_on_branch};

/// Dos ramas que se juntan con un merge y una tercera que sigue abierta.
fn history() -> Vec<CommitRecord> {
    vec![
        record(1, &[2, 5]),
        record(7, &[6]),
        record(2, &[3]),
        record(5, &[3]),
        record(6, &[3]),
        record(3, &[4]),
        record(4, &[]),
    ]
}

fn refs() -> RefSnapshot {
    snapshot_on_branch(
        vec![
            git_ref("refs/heads/main", 1),
            git_ref("refs/heads/topic", 7),
        ],
        "refs/heads/main",
        1,
    )
}

fn laid_out_in_pieces(sizes: &[usize]) -> LaidOutHistory {
    let refs = refs();
    let mut history_left = history();
    let mut laid = LaidOutHistory::new(&pinned_tips(&refs));
    for size in sizes {
        let rest = history_left.split_off((*size).min(history_left.len()));
        let piece = std::mem::replace(&mut history_left, rest);
        laid.extend(piece, history_left.is_empty());
    }
    laid
}

#[test]
fn windows_from_a_history_laid_out_in_pieces_match_a_single_layout() {
    let refs = refs();
    for (offset, limit) in [(0, 2), (2, 2), (3, 3), (0, 7), (5, 10)] {
        let range = WindowRange::new(offset, limit).unwrap();
        let read: Vec<CommitRecord> = history()
            .into_iter()
            .take(range.commits_to_read())
            .collect();
        let expected = graph_window(range, read, &refs);

        for pieces in [&[7][..], &[1, 1, 5], &[3, 4], &[2, 2, 2, 2]] {
            let laid = laid_out_in_pieces(pieces);
            assert_eq!(
                laid.window(range, &refs),
                expected,
                "window {offset}+{limit} with pieces {pieces:?}"
            );
        }
    }
}

#[test]
fn coverage_tells_when_more_commits_must_be_read() {
    let laid = laid_out_in_pieces(&[3]);
    assert_eq!(laid.len(), 3);
    assert!(!laid.is_complete());
    assert!(laid.covers(WindowRange::new(0, 2).unwrap()));
    assert!(!laid.covers(WindowRange::new(0, 3).unwrap()));
    assert_eq!(laid.row_of(sha(2)), Some(2));
    assert_eq!(laid.row_of(sha(4)), None);

    let all = laid_out_in_pieces(&[7]);
    assert!(all.is_complete());
    assert!(all.covers(WindowRange::new(100, 10).unwrap()));
}

#[test]
fn an_empty_history_gives_an_empty_window_with_the_pinned_lane() {
    let refs = refs();
    let laid = LaidOutHistory::new(&[sha(1)]);

    let window = laid.window(WindowRange::new(0, 10).unwrap(), &refs);

    assert!(laid.is_empty());
    assert_eq!(window.rows, Vec::new());
    assert!(!window.has_more);
    assert_eq!(window.lane_count, 1);
}

fn dated(id: u32, parents: &[u32], author: &str, seconds: i64) -> CommitRecord {
    let mut commit = record(id, parents);
    author.clone_into(&mut commit.author.name);
    commit.author.email = format!("{}@example.com", author.to_lowercase());
    commit.authored_at = GitTime::new(seconds, 0);
    commit
}

#[test]
fn an_author_filter_links_each_row_to_its_nearest_visible_ancestor() {
    // 1 (ana) -> 2 (luis) -> 3 (ana) -> 4 (luis) -> 5 (ana)
    let commits = vec![
        dated(1, &[2], "Ana", 50),
        dated(2, &[3], "Luis", 40),
        dated(3, &[4], "Ana", 30),
        dated(4, &[5], "Luis", 20),
        dated(5, &[], "Ana", 10),
    ];
    let filter = GraphFilter {
        author: Some("ana".to_owned()),
        ..GraphFilter::default()
    };

    let kept = filter_history(commits, &filter);

    let shape: Vec<(
        git_rudder_core_domain::Sha,
        Vec<git_rudder_core_domain::Sha>,
    )> = kept
        .iter()
        .map(|commit| (commit.node.sha, commit.node.parents.clone()))
        .collect();
    assert_eq!(
        shape,
        vec![
            (sha(1), vec![sha(3)]),
            (sha(3), vec![sha(5)]),
            (sha(5), vec![]),
        ]
    );
}

#[test]
fn a_hidden_merge_parent_side_collapses_without_duplicates() {
    // 1 = merge(2, 3); 2 y 3 salen de 4. Sin 2 ni 3, 1 queda con un solo padre 4.
    let commits = vec![
        dated(1, &[2, 3], "Ana", 40),
        dated(2, &[4], "Luis", 30),
        dated(3, &[4], "Luis", 20),
        dated(4, &[9], "Ana", 10),
    ];
    let filter = GraphFilter {
        author: Some("ANA".to_owned()),
        ..GraphFilter::default()
    };

    let kept = filter_history(commits, &filter);

    assert_eq!(kept[0].node.parents, vec![sha(4)]);
    assert_eq!(
        kept[1].node.parents,
        vec![sha(9)],
        "a parent outside the read history is kept as it is"
    );
}

#[test]
fn author_and_dates_combine() {
    let commits = vec![
        dated(1, &[2], "Ana", 300),
        dated(2, &[3], "Ana", 200),
        dated(3, &[], "Luis", 100),
    ];
    let filter = GraphFilter {
        author: Some("example.com".to_owned()),
        since: Some(150),
        until: Some(250),
        ..GraphFilter::default()
    };

    let kept = filter_history(commits, &filter);

    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].node.sha, sha(2));
    assert!(filter.filters_rows());
}

#[test]
fn isolating_wins_over_hiding_and_an_empty_filter_selects_everything() {
    let main = RefName::parse("refs/heads/main").unwrap();
    let topic = RefName::parse("refs/heads/topic").unwrap();
    let both = GraphFilter {
        only: vec![main.clone()],
        hidden: vec![topic.clone()],
        ..GraphFilter::default()
    };
    let hiding = GraphFilter {
        hidden: vec![topic.clone()],
        ..GraphFilter::default()
    };

    assert_eq!(both.selection(), RefSelection::Only(vec![main]));
    assert_eq!(hiding.selection(), RefSelection::Excluding(vec![topic]));
    assert_eq!(GraphFilter::default().selection(), RefSelection::All);
    assert!(GraphFilter::default().is_empty());
    assert!(!hiding.is_empty());
    assert!(!hiding.filters_rows());
}
