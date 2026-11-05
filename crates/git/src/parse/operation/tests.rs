#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use git_rudder_core_domain::Sha;
use git_rudder_core_domain::operation::{Operation, RebaseProgress, RebaseState};
use pretty_assertions::assert_eq;

use super::{OperationFiles, RebaseFiles, detect_operation};

const A: &str = "1111111111111111111111111111111111111111";
const B: &str = "2222222222222222222222222222222222222222";

fn owned(text: &str) -> String {
    text.to_owned()
}

#[test]
fn an_interactive_rebase_reports_its_step_and_where_it_stopped() {
    let files = OperationFiles {
        rebase_merge: Some(RebaseFiles {
            head_name: Some(owned("refs/heads/main\n")),
            onto: Some(owned(&format!("{A}\n"))),
            step: Some(owned("3\n")),
            total: Some(owned("7\n")),
            stopped: Some(owned(B)),
            interactive: true,
        }),
        cherry_pick_head: Some(owned(B)),
        ..OperationFiles::default()
    };

    assert_eq!(
        detect_operation(&files),
        Some(Operation::Rebase(RebaseState {
            branch: Some("main".to_owned()),
            onto: Some(Sha::parse(A).unwrap()),
            progress: Some(RebaseProgress { step: 3, total: 7 }),
            stopped_at: Some(Sha::parse(B).unwrap()),
            interactive: true,
        }))
    );
}

#[test]
fn a_rebase_of_a_detached_head_has_no_branch() {
    let files = OperationFiles {
        rebase_apply: Some(RebaseFiles {
            head_name: Some(owned("detached HEAD")),
            step: Some(owned("x")),
            ..RebaseFiles::default()
        }),
        ..OperationFiles::default()
    };

    let Some(Operation::Rebase(state)) = detect_operation(&files) else {
        unreachable!("a rebase was in progress")
    };
    assert_eq!(state.branch, None);
    assert_eq!(state.progress, None, "unreadable numbers give no progress");
}

#[test]
fn a_merge_names_the_incoming_branch() {
    let files = OperationFiles {
        merge_head: Some(owned(&format!("{A}\n"))),
        merge_msg: Some(owned("Merge branch 'feature'\n\n# Conflicts:\n")),
        ..OperationFiles::default()
    };

    assert_eq!(
        detect_operation(&files),
        Some(Operation::Merge {
            incoming: Some(Sha::parse(A).unwrap()),
            incoming_name: Some("feature".to_owned()),
        })
    );
}

#[test]
fn cherry_pick_and_revert_carry_their_commit() {
    let pick = OperationFiles {
        cherry_pick_head: Some(owned(A)),
        ..OperationFiles::default()
    };
    let revert = OperationFiles {
        revert_head: Some(owned("garbage")),
        ..OperationFiles::default()
    };

    assert_eq!(
        detect_operation(&pick),
        Some(Operation::CherryPick {
            commit: Some(Sha::parse(A).unwrap())
        })
    );
    assert_eq!(
        detect_operation(&revert),
        Some(Operation::Revert { commit: None })
    );
    assert_eq!(detect_operation(&OperationFiles::default()), None);
}
