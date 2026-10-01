//! Casos limite de la asignacion de carriles (tabla 3.4 del analisis de diseno).

#![allow(
    unused_crate_dependencies,
    reason = "integration tests only use the crate under test"
)]
#![allow(
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; an out of bounds row is a failed assertion"
)]

mod support;

use ramal_core_domain::graph::layout;
use support::{lanes, node, nodes, sha};

#[test]
fn empty_input_has_no_rows_and_the_width_of_the_pins() {
    assert_eq!(layout(&[], &[]).rows.len(), 0);
    assert_eq!(layout(&[], &[]).width, 0);
    assert_eq!(layout(&[], &[sha(1), sha(2)]).width, 2);
}

#[test]
fn a_single_root_commit_takes_lane_zero() {
    let result = layout(&[node(1, &[])], &[]);

    assert_eq!(lanes(&result), vec![0]);
    assert_eq!(result.width, 1);
    assert!(result.rows[0].edges.is_empty());
}

#[test]
fn linear_history_stays_in_lane_zero() {
    let result = layout(&nodes(&[(1, &[2]), (2, &[3]), (3, &[4]), (4, &[])]), &[]);

    assert_eq!(lanes(&result), vec![0, 0, 0, 0]);
    assert_eq!(result.width, 1);
}

#[test]
fn several_roots_each_keep_their_own_lane_while_open() {
    // Dos historias sin relacion (por ejemplo gh-pages) entrelazadas por fecha.
    let result = layout(&nodes(&[(1, &[3]), (2, &[4]), (3, &[]), (4, &[])]), &[]);

    assert_eq!(lanes(&result), vec![0, 1, 0, 1]);
    assert_eq!(result.width, 2);
}

#[test]
fn a_root_commit_frees_its_lane_for_the_next_tip() {
    let result = layout(&nodes(&[(1, &[]), (2, &[3]), (3, &[])]), &[]);

    assert_eq!(lanes(&result), vec![0, 0, 0]);
}

#[test]
fn an_octopus_merge_gives_each_extra_parent_a_free_lane() {
    let result = layout(
        &nodes(&[
            (1, &[2, 3, 4, 5]),
            (2, &[6]),
            (3, &[6]),
            (4, &[6]),
            (5, &[6]),
            (6, &[]),
        ]),
        &[],
    );

    assert_eq!(lanes(&result), vec![0, 0, 1, 2, 3, 0]);
    assert_eq!(result.width, 4);
    let travel: Vec<usize> = result.rows[0]
        .edges
        .iter()
        .map(|edge| edge.travel_lane)
        .collect();
    assert_eq!(travel, vec![0, 1, 2, 3]);
}

#[test]
fn a_merge_parent_already_reserved_does_not_take_a_second_lane() {
    // 1 espera a 3; el merge 2 tambien trae a 3 como padre extra.
    let result = layout(&nodes(&[(1, &[3]), (2, &[4, 3]), (3, &[]), (4, &[])]), &[]);

    assert_eq!(lanes(&result), vec![0, 1, 0, 1]);
    assert_eq!(result.width, 2);
    assert_eq!(result.rows[1].edges[1].travel_lane, 0);
}

#[test]
fn a_parent_outside_the_loaded_commits_keeps_its_lane_reserved() {
    let result = layout(&nodes(&[(1, &[99]), (2, &[3]), (3, &[])]), &[]);

    assert_eq!(lanes(&result), vec![0, 1, 1]);
    assert_eq!(result.width, 2);
    let edge = &result.rows[0].edges[0];
    assert_eq!(edge.parent_row, None);
    assert_eq!(edge.to_lane, edge.travel_lane);
}

#[test]
fn a_pinned_tip_keeps_its_lane_until_it_appears() {
    // La rama por defecto (3) va fijada en el carril 0 aunque 1 sea mas nuevo.
    let result = layout(&nodes(&[(1, &[2]), (2, &[3]), (3, &[])]), &[sha(3)]);

    assert_eq!(lanes(&result), vec![1, 1, 0]);
    assert_eq!(result.width, 2);
    let into_main = &result.rows[1].edges[0];
    assert_eq!(
        (
            into_main.from_lane,
            into_main.travel_lane,
            into_main.to_lane
        ),
        (1, 1, 0)
    );
}

#[test]
fn a_pinned_tip_that_never_appears_stays_reserved() {
    let result = layout(&nodes(&[(1, &[2]), (2, &[])]), &[sha(42)]);

    assert_eq!(lanes(&result), vec![1, 1]);
    assert_eq!(result.width, 2);
}

#[test]
fn many_tips_open_many_lanes_and_close_them_at_the_shared_root() {
    let tips: Vec<(u32, Vec<u32>)> = (1..=30).map(|id| (id, vec![100])).collect();
    let mut commits: Vec<_> = tips
        .iter()
        .map(|(id, parents)| node(*id, parents))
        .collect();
    commits.push(node(100, &[]));
    let result = layout(&commits, &[]);

    let expected: Vec<usize> = (0..30).chain([0]).collect();
    assert_eq!(lanes(&result), expected);
    assert_eq!(result.width, 30);
    assert_eq!(result.rows[29].edges[0].to_lane, 0);
}

#[test]
fn the_first_parent_never_lands_to_the_right_of_its_child() {
    let commits = nodes(&[
        (1, &[4, 2]),
        (2, &[3]),
        (5, &[3]),
        (3, &[4]),
        (6, &[4, 7]),
        (7, &[4]),
        (4, &[8]),
        (8, &[]),
    ]);
    let result = layout(&commits, &[sha(6)]);

    for row in &result.rows {
        for edge in row.edges.iter().filter(|edge| edge.parent_row.is_some()) {
            if edge.kind == ramal_core_domain::graph::EdgeKind::FirstParent {
                assert!(edge.to_lane <= edge.from_lane, "edge {edge:?}");
            }
        }
    }
}

#[test]
fn earlier_rows_do_not_change_when_more_commits_are_loaded() {
    let commits = nodes(&[
        (1, &[3, 2]),
        (2, &[4]),
        (3, &[5]),
        (4, &[5]),
        (5, &[6, 7]),
        (6, &[8]),
        (7, &[8]),
        (8, &[]),
    ]);
    let full = layout(&commits, &[sha(3)]);
    for end in 1..commits.len() {
        let prefix = layout(&commits[..end], &[sha(3)]);
        assert_eq!(lanes(&prefix), lanes(&full)[..end].to_vec(), "prefix {end}");
    }
}

#[test]
fn the_layout_is_deterministic() {
    let commits = nodes(&[(1, &[2, 3]), (2, &[4]), (3, &[4]), (4, &[])]);

    assert_eq!(layout(&commits, &[]), layout(&commits, &[]));
}
