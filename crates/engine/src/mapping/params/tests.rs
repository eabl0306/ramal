#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use git_rudder_core_domain::diff::{DiffTarget, Whitespace};
use git_rudder_protocol::ErrorKind;
use git_rudder_protocol::diff::{self as dto, DiffFileParams};
use git_rudder_protocol::graph::GraphFilter;

use super::{diff_request, graph_filter, path, sha};

fn params(target: dto::DiffTarget, file: &str) -> DiffFileParams {
    DiffFileParams {
        repo_id: "repo-1".to_owned(),
        path: file.to_owned(),
        old_path: None,
        target,
        options: None,
    }
}

#[test]
fn a_diff_request_takes_defaults_and_validated_values() {
    let request = diff_request(&DiffFileParams {
        options: Some(dto::DiffOptions {
            whitespace: Some(dto::Whitespace::IgnoreAll),
            ..dto::DiffOptions::default()
        }),
        ..params(
            dto::DiffTarget::Commit {
                sha: "a".repeat(40),
            },
            "src/a.rs",
        )
    })
    .unwrap();

    assert!(matches!(request.target, DiffTarget::Commit(_)));
    assert_eq!(request.options.whitespace, Whitespace::IgnoreAll);
    assert_eq!(request.options.context_lines, 3);
    assert!(!request.options.word_diff);
}

#[test]
fn paths_and_shas_from_a_webview_are_checked() {
    for bad in ["../etc/passwd", "/etc/passwd", ""] {
        let error = diff_request(&params(dto::DiffTarget::WorkingTree, bad)).unwrap_err();
        assert_eq!(error.data.unwrap().kind, ErrorKind::InvalidParams, "{bad}");
    }
    let range = dto::DiffTarget::Range {
        from: "HEAD".to_owned(),
        to: "b".repeat(40),
    };
    assert!(diff_request(&params(range, "a")).is_err());
    assert!(sha("-n").is_err());
    assert!(path("a/b").is_ok());
}

#[test]
fn a_filter_needs_full_ref_names_and_strict_dates() {
    let filter = graph_filter(Some(&GraphFilter {
        only: Some(vec!["refs/heads/main".to_owned()]),
        author: Some("  ".to_owned()),
        since: Some("2026-09-30".to_owned()),
        ..GraphFilter::default()
    }))
    .unwrap();
    assert_eq!(filter.only.len(), 1);
    assert_eq!(filter.author, None, "a blank author filters nothing");
    assert_eq!(filter.since, Some(1_790_726_400));

    let short_name = GraphFilter {
        hidden: Some(vec!["main".to_owned()]),
        ..GraphFilter::default()
    };
    assert!(graph_filter(Some(&short_name)).is_err());
    let bad_date = GraphFilter {
        until: Some("yesterday".to_owned()),
        ..GraphFilter::default()
    };
    assert!(graph_filter(Some(&bad_date)).is_err());
    assert!(graph_filter(None).unwrap().is_empty());
}
