#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use super::{ConflictLabels, ConflictSegment, MarkerError, parse_conflict_markers};

fn lines(items: &[&str]) -> Vec<String> {
    items.iter().map(|line| (*line).to_owned()).collect()
}

#[test]
fn a_zdiff3_block_keeps_ours_base_and_theirs() {
    let text =
        "intro\n<<<<<<< HEAD\nours\n||||||| base\nold\n=======\ntheirs\n>>>>>>> feature\nend";

    let parsed = parse_conflict_markers(text).unwrap();

    assert_eq!(
        parsed.segments,
        vec![
            ConflictSegment::Context {
                lines: lines(&["intro"])
            },
            ConflictSegment::Hunk {
                ours: lines(&["ours"]),
                base: Some(lines(&["old"])),
                theirs: lines(&["theirs"]),
            },
            ConflictSegment::Context {
                lines: lines(&["end"])
            },
        ]
    );
    assert_eq!(
        parsed.labels,
        Some(ConflictLabels {
            ours: "HEAD".to_owned(),
            base: Some("base".to_owned()),
            theirs: "feature".to_owned(),
        })
    );
}

#[test]
fn a_merge_style_block_has_no_base() {
    let parsed = parse_conflict_markers("<<<<<<<\na\n=======\nb\n>>>>>>>\n").unwrap();

    assert_eq!(
        parsed.segments.first(),
        Some(&ConflictSegment::Hunk {
            ours: lines(&["a"]),
            base: None,
            theirs: lines(&["b"]),
        })
    );
    assert_eq!(
        parsed.segments.last(),
        Some(&ConflictSegment::Context {
            lines: lines(&[""])
        }),
        "the trailing newline survives as an empty last line"
    );
}

#[test]
fn a_markdown_underline_outside_a_block_is_plain_text() {
    let text = "Title\n=======\n\n||||||| not a base\nbody";

    let parsed = parse_conflict_markers(text).unwrap();

    assert_eq!(parsed.segments.len(), 1);
    assert_eq!(parsed.labels, None);
}

#[test]
fn lines_that_only_start_like_a_marker_are_text() {
    let text = "<<<<<<<<< nine\n=======x\n>>>>>>>no-space";

    let parsed = parse_conflict_markers(text).unwrap();

    assert_eq!(
        parsed.segments,
        vec![ConflictSegment::Context {
            lines: lines(&["<<<<<<<<< nine", "=======x", ">>>>>>>no-space"])
        }]
    );
}

#[test]
fn crlf_markers_are_recognized_and_lines_keep_their_carriage_return() {
    let text = "<<<<<<< HEAD\r\na\r\n=======\r\nb\r\n>>>>>>> x\r\n";

    let parsed = parse_conflict_markers(text).unwrap();

    assert_eq!(
        parsed.segments.first(),
        Some(&ConflictSegment::Hunk {
            ours: lines(&["a\r"]),
            base: None,
            theirs: lines(&["b\r"]),
        })
    );
    assert_eq!(
        parsed.labels.map(|labels| labels.theirs).as_deref(),
        Some("x")
    );
}

#[test]
fn broken_markers_are_errors_with_their_line() {
    assert_eq!(
        parse_conflict_markers("<<<<<<< HEAD\na\n=======\nb"),
        Err(MarkerError::Unterminated { line: 4 })
    );
    assert_eq!(
        parse_conflict_markers("a\n>>>>>>> stray"),
        Err(MarkerError::UnexpectedMarker { line: 2 })
    );
    assert_eq!(
        MarkerError::UnexpectedMarker { line: 2 }.to_string(),
        "unexpected conflict marker at line 2"
    );
}
