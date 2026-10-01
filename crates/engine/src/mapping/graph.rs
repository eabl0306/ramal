//! Ventana del grafo del dominio -> DTO.

use ramal_core_domain::graph::{EdgeKind, GraphEdge, GraphWindow, WindowRow};
use ramal_core_domain::{RefKind, RefLabel, RepoId};
use ramal_protocol::graph as dto;

use super::to_u32;

/// Ventana lista para serializar.
#[must_use]
pub fn window_to_dto(repo_id: &RepoId, window: &GraphWindow) -> dto::GraphWindow {
    dto::GraphWindow {
        repo_id: repo_id.as_str().to_owned(),
        offset: to_u32(window.offset),
        rows: window.rows.iter().map(row_to_dto).collect(),
        has_more: window.has_more,
        lane_count: to_u32(window.lane_count),
    }
}

fn row_to_dto(row: &WindowRow) -> dto::GraphRow {
    let commit = &row.commit;
    dto::GraphRow {
        sha: commit.node.sha.to_hex(),
        short_sha: commit.short_sha.as_str().to_owned(),
        parents: commit.node.parents.iter().map(|sha| sha.to_hex()).collect(),
        lane: to_u32(row.lane),
        edges: row.edges.iter().map(edge_to_dto).collect(),
        summary: commit.summary.clone(),
        author_name: commit.author.name.clone(),
        author_email: commit.author.email.clone(),
        authored_at: commit.authored_at.to_iso8601(),
        refs: row.refs.iter().map(label_to_dto).collect(),
        is_head: row.is_head,
    }
}

fn edge_to_dto(edge: &GraphEdge) -> dto::GraphEdge {
    dto::GraphEdge {
        parent_sha: edge.parent.to_hex(),
        parent_row: edge.parent_row.map(to_u32),
        from_lane: to_u32(edge.from_lane),
        travel_lane: to_u32(edge.travel_lane),
        to_lane: to_u32(edge.to_lane),
        kind: match edge.kind {
            EdgeKind::FirstParent => dto::EdgeKind::FirstParent,
            EdgeKind::Merge => dto::EdgeKind::Merge,
        },
    }
}

fn label_to_dto(label: &RefLabel) -> dto::RefLabel {
    dto::RefLabel {
        name: label.name.short_name().to_owned(),
        full_name: label.name.as_str().to_owned(),
        kind: match label.name.kind() {
            RefKind::LocalBranch => dto::RefKind::LocalBranch,
            RefKind::RemoteBranch => dto::RefKind::RemoteBranch,
            RefKind::Tag => dto::RefKind::Tag,
        },
        is_head: label.is_head,
    }
}
