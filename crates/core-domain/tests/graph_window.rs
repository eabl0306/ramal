//! Recorte de la ventana, etiquetas de refs por fila y validacion del rango.

#![allow(
    unused_crate_dependencies,
    reason = "integration tests only use the crate under test"
)]
#![allow(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

mod support;

use git_rudder_core_domain::graph::{
    GraphWindow, MAX_WINDOW_LIMIT, WindowError, WindowRange, graph_window, layout,
};
use git_rudder_core_domain::{Head, RefSnapshot};
use support::{git_ref, record, sha, snapshot_on_branch};

fn history() -> Vec<git_rudder_core_domain::CommitRecord> {
    vec![
        record(1, &[2]),
        record(2, &[3]),
        record(3, &[4]),
        record(4, &[5]),
        record(5, &[6]),
        record(6, &[]),
    ]
}

#[test]
fn the_range_accepts_one_to_a_thousand_rows() {
    assert!(WindowRange::new(0, 1).is_ok());
    assert!(WindowRange::new(5, MAX_WINDOW_LIMIT).is_ok());
    assert_eq!(
        WindowRange::new(0, 0),
        Err(WindowError::LimitOutOfRange {
            limit: 0,
            max: MAX_WINDOW_LIMIT
        })
    );
    assert!(WindowRange::new(0, MAX_WINDOW_LIMIT + 1).is_err());
    assert_eq!(
        WindowRange::new(usize::MAX, 10),
        Err(WindowError::OffsetTooLarge { offset: usize::MAX })
    );
}

#[test]
fn the_range_reads_one_commit_past_its_end() {
    let range = WindowRange::new(200, 100).unwrap();

    assert_eq!(range.offset(), 200);
    assert_eq!(range.limit(), 100);
    assert_eq!(range.end(), 300);
    assert_eq!(range.commits_to_read(), 301);
}

#[test]
fn a_window_in_the_middle_keeps_absolute_rows_and_says_there_is_more() {
    let refs = snapshot_on_branch(vec![git_ref("refs/heads/main", 1)], "refs/heads/main", 1);
    let window = graph_window(WindowRange::new(2, 3).unwrap(), history(), &refs);

    assert_eq!(window.offset, 2);
    let rows: Vec<usize> = window.rows.iter().map(|row| row.row).collect();
    assert_eq!(rows, vec![2, 3, 4]);
    assert!(window.has_more);
    assert_eq!(window.lane_count, 1);
    assert_eq!(window.rows[0].commit.node.sha, sha(3));
    assert_eq!(window.rows[0].edges[0].parent_row, Some(3));
    assert_eq!(window.snapshot, refs.fingerprint());
}

#[test]
fn the_last_window_is_short_and_has_no_more() {
    let refs = snapshot_on_branch(vec![git_ref("refs/heads/main", 1)], "refs/heads/main", 1);
    let window = graph_window(WindowRange::new(4, 10).unwrap(), history(), &refs);

    assert_eq!(window.rows.len(), 2);
    assert!(!window.has_more);
}

#[test]
fn a_window_past_the_end_is_empty() {
    let refs = snapshot_on_branch(vec![git_ref("refs/heads/main", 1)], "refs/heads/main", 1);
    let window = graph_window(WindowRange::new(50, 10).unwrap(), history(), &refs);

    assert!(window.rows.is_empty(), "{:?}", window.rows);
    assert!(!window.has_more);
}

#[test]
fn has_more_is_true_only_when_a_commit_follows_the_window() {
    let refs = snapshot_on_branch(vec![git_ref("refs/heads/main", 1)], "refs/heads/main", 1);

    let exact = graph_window(WindowRange::new(0, 6).unwrap(), history(), &refs);
    assert!(!exact.has_more);
    let short = graph_window(WindowRange::new(0, 5).unwrap(), history(), &refs);
    assert!(short.has_more);
}

#[test]
fn rows_carry_their_refs_with_the_head_branch_first() {
    let refs = snapshot_on_branch(
        vec![
            git_ref("refs/tags/v1", 1),
            git_ref("refs/remotes/origin/main", 1),
            git_ref("refs/heads/zeta", 1),
            git_ref("refs/heads/main", 1),
            git_ref("refs/heads/feature", 3),
        ],
        "refs/heads/zeta",
        1,
    );
    let window = graph_window(WindowRange::new(0, 4).unwrap(), history(), &refs);

    let names: Vec<&str> = window.rows[0]
        .refs
        .iter()
        .map(|label| label.name.as_str())
        .collect();
    assert_eq!(
        names,
        vec![
            "refs/heads/zeta",
            "refs/heads/main",
            "refs/remotes/origin/main",
            "refs/tags/v1"
        ]
    );
    assert!(window.rows[0].refs[0].is_head);
    assert!(!window.rows[0].refs[1].is_head);
    assert!(window.rows[0].is_head);
    assert!(!window.rows[2].is_head);
    assert_eq!(window.rows[2].refs.len(), 1);
    assert!(window.rows[1].refs.is_empty(), "{:?}", window.rows[1].refs);
}

#[test]
fn a_detached_head_marks_the_row_but_no_ref() {
    let refs = RefSnapshot::new(
        vec![git_ref("refs/heads/main", 1)],
        Head::Detached { target: sha(3) },
        None,
    );
    let window = graph_window(WindowRange::new(0, 6).unwrap(), history(), &refs);

    assert!(window.rows[2].is_head);
    assert!(!window.rows[0].is_head);
    assert!(!window.rows[0].refs[0].is_head);
}

#[test]
fn the_default_branch_is_pinned_to_the_left() {
    // feature (1 -> 2) es mas nueva que main (3), pero main va en el carril 0.
    let commits = vec![record(1, &[2]), record(2, &[3]), record(3, &[])];
    let refs = snapshot_on_branch(
        vec![
            git_ref("refs/heads/feature", 1),
            git_ref("refs/heads/main", 3),
        ],
        "refs/heads/feature",
        1,
    );
    let window = graph_window(WindowRange::new(0, 3).unwrap(), commits, &refs);

    let lanes: Vec<usize> = window.rows.iter().map(|row| row.lane).collect();
    assert_eq!(lanes, vec![1, 1, 0]);
    assert_eq!(window.lane_count, 2);
}

#[test]
fn cut_takes_any_layout_of_the_same_commits() {
    let commits = history();
    let nodes: Vec<_> = commits.iter().map(|record| record.node.clone()).collect();
    let refs = snapshot_on_branch(vec![], "refs/heads/main", 1);
    let window = GraphWindow::cut(
        WindowRange::new(0, 2).unwrap(),
        commits,
        layout(&nodes, &[]),
        &refs,
    );

    assert_eq!(window.rows.len(), 2);
    assert_eq!(window.rows[1].lane, 0);
}
