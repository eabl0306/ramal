//! La disposicion contra los dos ejemplos de referencia del analisis de diseno:
//! los 19 commits de `git-rudder-data.js` (tabla 3.6) y el ejemplo pequeno (3.5).
//! Los carriles guardados a mano en el prototipo no sirven de oraculo; estas
//! tablas si.

#![allow(
    unused_crate_dependencies,
    reason = "integration tests only use the crate under test"
)]
#![allow(
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; an out of bounds row is a failed assertion"
)]

mod support;

use git_rudder_core_domain::graph::{EdgeKind, layout};
use support::{lanes, nodes, sha};

/// Los 19 commits de muestra, del mas nuevo al mas viejo: `cN` es el id `N`.
fn sample() -> Vec<git_rudder_core_domain::CommitNode> {
    nodes(&[
        (1, &[3]),
        (2, &[4, 5]),
        (3, &[6]),
        (4, &[7]),
        (5, &[8]),
        (6, &[7]),
        (7, &[9]),
        (8, &[9]),
        (9, &[11]),
        (10, &[14, 12]),
        (11, &[13]),
        (12, &[14]),
        (13, &[14]),
        (14, &[15]),
        (15, &[16, 17]),
        (16, &[18]),
        (17, &[18]),
        (18, &[19]),
        (19, &[]),
    ])
}

#[test]
fn the_sample_without_pins_matches_the_recommended_column() {
    let result = layout(&sample(), &[]);

    assert_eq!(
        lanes(&result),
        vec![0, 1, 0, 1, 2, 0, 0, 2, 0, 1, 0, 2, 0, 0, 0, 0, 1, 0, 0]
    );
    assert_eq!(result.width, 3);
}

#[test]
fn the_sample_with_main_and_develop_pinned_matches_the_design() {
    let result = layout(&sample(), &[sha(10), sha(2)]);

    assert_eq!(
        lanes(&result),
        vec![2, 1, 2, 1, 3, 2, 1, 3, 1, 0, 1, 2, 1, 0, 0, 0, 1, 0, 0]
    );
    assert_eq!(result.width, 4);
}

#[test]
fn the_sample_merge_of_hotfix_reuses_a_free_lane() {
    let result = layout(&sample(), &[sha(10), sha(2)]);
    let merge = &result.rows[9];

    assert_eq!(merge.edges.len(), 2);
    assert_eq!(merge.edges[1].kind, EdgeKind::Merge);
    assert_eq!(merge.edges[1].parent, sha(12));
    assert_eq!(merge.edges[1].parent_row, Some(11));
    assert_eq!(merge.edges[1].travel_lane, 2);
    assert_eq!(merge.edges[1].to_lane, 2);
}

/// Ejemplo 3.5: M, F, A, E, H, B, X, N, Q, R como 1..=10.
fn worked_example() -> Vec<git_rudder_core_domain::CommitNode> {
    nodes(&[
        (1, &[3, 2]),
        (2, &[4]),
        (3, &[6]),
        (4, &[6]),
        (5, &[6]),
        (6, &[10]),
        (7, &[10]),
        (8, &[10, 9]),
        (9, &[10]),
        (10, &[]),
    ])
}

#[test]
fn the_worked_example_frees_converging_lanes() {
    let result = layout(&worked_example(), &[]);

    assert_eq!(lanes(&result), vec![0, 1, 0, 1, 2, 0, 1, 2, 3, 0]);
    assert_eq!(result.width, 4);
}

#[test]
fn the_worked_example_edges_match_the_drawn_paths() {
    let result = layout(&worked_example(), &[]);

    let merge = &result.rows[0].edges[1];
    assert_eq!(merge.kind, EdgeKind::Merge);
    assert_eq!(
        (merge.from_lane, merge.travel_lane, merge.to_lane),
        (0, 1, 1)
    );
    assert_eq!(merge.parent_row, Some(1));

    let same_lane = &result.rows[1].edges[0];
    assert_eq!(
        (
            same_lane.from_lane,
            same_lane.travel_lane,
            same_lane.to_lane
        ),
        (1, 1, 1)
    );
    assert_eq!(same_lane.parent_row, Some(3));

    let branch_off = &result.rows[3].edges[0];
    assert_eq!(branch_off.kind, EdgeKind::FirstParent);
    assert_eq!(
        (
            branch_off.from_lane,
            branch_off.travel_lane,
            branch_off.to_lane
        ),
        (1, 1, 0)
    );
    assert_eq!(branch_off.parent_row, Some(5));
}
