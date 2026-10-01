//! `OpenedRepository` -> `RepoHandle`.

use ramal_core_application::OpenedRepository;
use ramal_core_domain::Sha;
use ramal_protocol::repo as dto;

/// Repositorio abierto tal como lo ve el host.
#[must_use]
pub fn opened_to_dto(opened: &OpenedRepository) -> dto::RepoHandle {
    dto::RepoHandle {
        repo_id: opened.id.as_str().to_owned(),
        root: opened.root.as_path().to_string_lossy().into_owned(),
        head_ref: opened
            .refs
            .head_branch()
            .map(|branch| branch.short_name().to_owned()),
        head_sha: opened.refs.head_target().map(Sha::to_hex),
        is_detached: opened.refs.is_detached(),
        is_empty: opened.refs.is_empty(),
    }
}
