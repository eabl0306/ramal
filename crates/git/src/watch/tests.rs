use std::path::Path;

use notify::EventKind;
use notify::event::{AccessKind, CreateKind, DataChange, MetadataKind, ModifyKind};

use super::{is_change, is_relevant_git_path};

#[test]
fn only_head_index_refs_and_operations_count_inside_git() {
    let git = Path::new("/repo/.git");
    for relevant in [
        "HEAD",
        "index",
        "packed-refs",
        "MERGE_HEAD",
        "refs/heads/main",
        "refs/tags/v1",
        "rebase-merge/msgnum",
    ] {
        assert!(is_relevant_git_path(git, &git.join(relevant)), "{relevant}");
    }
    for ignored in [
        "index.lock",
        "refs/heads/main.lock",
        "objects/ab/cdef",
        "logs/HEAD",
        "COMMIT_EDITMSG",
        "HEAD/odd",
    ] {
        assert!(!is_relevant_git_path(git, &git.join(ignored)), "{ignored}");
    }
    assert!(!is_relevant_git_path(git, Path::new("/elsewhere/HEAD")));
}

#[test]
fn reading_or_touching_metadata_is_not_a_change() {
    assert!(is_change(EventKind::Create(CreateKind::File)));
    assert!(is_change(EventKind::Modify(ModifyKind::Data(
        DataChange::Content
    ))));
    assert!(!is_change(EventKind::Access(AccessKind::Read)));
    assert!(!is_change(EventKind::Modify(ModifyKind::Metadata(
        MetadataKind::AccessTime
    ))));
    assert!(!is_change(EventKind::Other));
}
