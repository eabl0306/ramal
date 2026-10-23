//! Aristas: carril de viaje, destino y filas de los padres.

#![allow(
    unused_crate_dependencies,
    reason = "integration tests only use the crate under test"
)]
#![allow(
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; an out of bounds row is a failed assertion"
)]

mod support;

use git_rudder_core_domain::graph::{EdgeKind, GraphEdge, layout};
use support::{nodes, sha};

const fn route(edge: &GraphEdge) -> (usize, usize, usize) {
    (edge.from_lane, edge.travel_lane, edge.to_lane)
}

#[test]
fn a_merge_edge_travels_in_its_reservation_and_curves_into_a_lower_parent() {
    // 1 abre el carril 0 esperando a 2. El merge 3 abre el 1 y reserva 4 en el 2.
    // Despues 2 hereda el carril 0 esperando a 4, asi que 4 cae en el 0: la arista
    // del merge baja por el carril 2 (libre de nodos) y entra en el 0 al final.
    let result = layout(
        &nodes(&[(1, &[2]), (3, &[5, 4]), (2, &[4]), (5, &[4]), (4, &[])]),
        &[],
    );

    let merge = &result.rows[1].edges[1];
    assert_eq!(merge.kind, EdgeKind::Merge);
    assert_eq!(merge.parent, sha(4));
    assert_eq!(merge.parent_row, Some(4));
    assert_eq!(route(merge), (1, 2, 0));
    assert_eq!(result.rows[4].lane, 0);
}

#[test]
fn no_commit_sits_on_a_travel_lane_between_child_and_parent() {
    let commits = nodes(&[
        (1, &[2]),
        (3, &[5, 4]),
        (2, &[4]),
        (6, &[7]),
        (5, &[4]),
        (7, &[4]),
        (4, &[]),
    ]);
    let result = layout(&commits, &[]);

    for (child_row, row) in result.rows.iter().enumerate() {
        for edge in &row.edges {
            let parent_row = edge.parent_row.unwrap_or(result.rows.len());
            for between in (child_row + 1)..parent_row {
                assert_ne!(
                    result.rows[between].lane, edge.travel_lane,
                    "row {between} sits on the travel lane of {edge:?}"
                );
            }
        }
    }
}

#[test]
fn first_parent_edges_travel_in_the_child_lane() {
    let result = layout(&nodes(&[(1, &[3]), (2, &[3]), (3, &[4]), (4, &[])]), &[]);

    assert_eq!(route(&result.rows[0].edges[0]), (0, 0, 0));
    assert_eq!(route(&result.rows[1].edges[0]), (1, 1, 0));
    assert_eq!(result.rows[1].edges[0].kind, EdgeKind::FirstParent);
}

#[test]
fn edges_keep_the_order_of_the_parents() {
    let result = layout(&nodes(&[(1, &[2, 3]), (2, &[]), (3, &[])]), &[]);
    let parents: Vec<_> = result.rows[0]
        .edges
        .iter()
        .map(|edge| edge.parent)
        .collect();

    assert_eq!(parents, vec![sha(2), sha(3)]);
    assert_eq!(result.rows[0].edges[0].kind, EdgeKind::FirstParent);
    assert_eq!(result.rows[0].edges[1].kind, EdgeKind::Merge);
    assert_eq!(result.rows[0].edges[1].parent_row, Some(2));
}

#[test]
fn a_repeated_parent_does_not_open_a_second_reservation() {
    let result = layout(&nodes(&[(1, &[2, 2]), (2, &[])]), &[]);

    assert_eq!(result.width, 1);
    assert_eq!(route(&result.rows[0].edges[1]), (0, 0, 0));
}

#[test]
fn a_duplicated_commit_points_its_children_at_the_first_occurrence() {
    let result = layout(&nodes(&[(1, &[2]), (2, &[]), (2, &[])]), &[]);

    assert_eq!(result.rows[0].edges[0].parent_row, Some(1));
}
