use super::{
    BranchStatus, ChangeKind, ConflictCode, EntryState, StatusCounts, StatusEntry, SubmoduleChange,
    WorkingTreeStatus,
};

fn changed(path: &str, index: ChangeKind, worktree: ChangeKind) -> StatusEntry {
    StatusEntry {
        path: path.to_owned(),
        original_path: None,
        state: EntryState::Changed {
            index,
            worktree,
            submodule: None,
        },
    }
}

fn other(path: &str, state: EntryState) -> StatusEntry {
    StatusEntry {
        path: path.to_owned(),
        original_path: None,
        state,
    }
}

#[test]
fn counts_put_each_file_in_its_groups() {
    let status = WorkingTreeStatus {
        entries: vec![
            changed("staged.txt", ChangeKind::Added, ChangeKind::Unmodified),
            changed("both.txt", ChangeKind::Modified, ChangeKind::Modified),
            changed("edited.txt", ChangeKind::Unmodified, ChangeKind::Deleted),
            other("new.txt", EntryState::Untracked),
            other("build/", EntryState::Ignored),
            other(
                "clash.txt",
                EntryState::Unmerged {
                    conflict: ConflictCode::BothModified,
                },
            ),
        ],
        ..WorkingTreeStatus::default()
    };

    assert_eq!(
        status.counts(),
        StatusCounts {
            staged: 2,
            unstaged: 2,
            untracked: 1,
            conflicted: 1,
        }
    );
    assert!(!status.is_clean());
}

#[test]
fn a_status_with_only_ignored_files_is_clean() {
    let status = WorkingTreeStatus {
        entries: vec![other("target/", EntryState::Ignored)],
        ..WorkingTreeStatus::default()
    };

    assert!(status.is_clean());
}

#[test]
fn a_submodule_with_dirty_contents_is_unstaged_even_on_the_same_commit() {
    let dirty = StatusEntry {
        path: "vendor/lib".to_owned(),
        original_path: None,
        state: EntryState::Changed {
            index: ChangeKind::Unmodified,
            worktree: ChangeKind::Unmodified,
            submodule: Some(SubmoduleChange {
                has_untracked: true,
                ..SubmoduleChange::default()
            }),
        },
    };

    assert!(dirty.is_unstaged());
    assert!(!dirty.is_staged());
    assert!(!other("x", EntryState::Untracked).is_unstaged());
}

#[test]
fn conflict_codes_tell_deletions_and_rename_sides_apart() {
    assert!(ConflictCode::DeletedByThem.is_deletion());
    assert!(ConflictCode::BothDeleted.is_deletion());
    assert!(!ConflictCode::BothModified.is_deletion());
    assert!(ConflictCode::AddedByUs.is_rename_side());
    assert!(!ConflictCode::BothAdded.is_rename_side());
}

#[test]
fn a_new_repository_is_unborn_and_a_detached_head_has_no_branch() {
    let unborn = BranchStatus {
        branch: Some("main".to_owned()),
        ..BranchStatus::default()
    };
    assert!(unborn.is_unborn());
    assert!(!unborn.is_detached());
    assert!(BranchStatus::default().is_detached());
}
