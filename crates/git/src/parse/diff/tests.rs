#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use git_rudder_core_domain::diff::{FileStatus, LineKind};
use pretty_assertions::assert_eq;

use super::parse_diff;

const MODIFIED: &str = "diff --git a/src/a file.rs b/src/a file.rs
index 1111111..2222222 100644
--- a/src/a file.rs
+++ b/src/a file.rs
@@ -1,4 +1,4 @@ fn main() {
 keep
-old one
-old two
+new one
+new two
 end
";

#[test]
fn a_hunk_keeps_removed_lines_before_added_ones_with_their_numbers() {
    let sections = parse_diff(MODIFIED, false);

    assert_eq!(sections.len(), 1);
    let section = &sections[0];
    assert!(section.is_about("src/a file.rs"));
    assert_eq!(section.status, FileStatus::Modified);
    let hunk = &section.hunks[0];
    assert_eq!(
        (
            hunk.old_start,
            hunk.old_lines,
            hunk.new_start,
            hunk.new_lines
        ),
        (1, 4, 1, 4)
    );
    assert_eq!(hunk.section, "fn main() {");
    let kinds: Vec<LineKind> = hunk.lines.iter().map(|line| line.kind).collect();
    assert_eq!(
        kinds,
        vec![
            LineKind::Context,
            LineKind::Removed,
            LineKind::Removed,
            LineKind::Added,
            LineKind::Added,
            LineKind::Context
        ]
    );
    let numbers: Vec<(Option<u32>, Option<u32>)> = hunk
        .lines
        .iter()
        .map(|line| (line.old_line, line.new_line))
        .collect();
    assert_eq!(
        numbers,
        vec![
            (Some(1), Some(1)),
            (Some(2), None),
            (Some(3), None),
            (None, Some(2)),
            (None, Some(3)),
            (Some(4), Some(4))
        ]
    );
}

#[test]
fn renames_binaries_new_and_deleted_files_are_recognized() {
    let text = "diff --git a/old.txt b/new.txt
similarity index 100%
rename from old.txt
rename to new.txt
diff --git a/logo.png b/logo.png
new file mode 100644
index 0000000..3333333
Binary files /dev/null and b/logo.png differ
diff --git a/gone.txt b/gone.txt
deleted file mode 100644
--- a/gone.txt
+++ /dev/null
@@ -1 +0,0 @@
-bye
\\ No newline at end of file
diff --git a/link b/link
old mode 100644
new mode 120000
";
    let sections = parse_diff(text, false);

    assert_eq!(sections[0].status, FileStatus::Renamed);
    assert_eq!(sections[0].old_path.as_deref(), Some("old.txt"));
    assert!(sections[0].is_about("new.txt"));
    assert_eq!(sections[0].hunks, Vec::new());
    assert_eq!(sections[1].status, FileStatus::Added);
    assert!(sections[1].binary);
    assert_eq!(sections[2].status, FileStatus::Deleted);
    assert!(sections[2].hunks[0].lines[0].no_newline_at_end);
    assert_eq!(sections[2].hunks[0].lines[0].old_line, Some(1));
    assert_eq!(sections[3].status, FileStatus::TypeChanged);
}

#[test]
fn a_carriage_return_change_stays_visible() {
    let text = "diff --git a/w.txt b/w.txt\n--- a/w.txt\n+++ b/w.txt\n@@ -1 +1 @@\n-a\n+a\r\n";

    let sections = parse_diff(text, false);

    let lines = &sections[0].hunks[0].lines;
    assert_eq!(lines[0].text, "a");
    assert_eq!(lines[1].text, "a\r");
}

#[test]
fn word_diff_porcelain_becomes_lines_with_segments() {
    let text = "diff --git a/w.txt b/w.txt\n--- a/w.txt\n+++ b/w.txt\n@@ -1,3 +1,3 @@\n \
                the \n-quick\n+slow\n  fox\n~\n-gone line\n~\n+new line\n~\n";
    let sections = parse_diff(text, true);

    let lines = &sections[0].hunks[0].lines;
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].kind, LineKind::Context);
    assert_eq!(lines[0].text, "the slow fox");
    assert_eq!(lines[0].words.len(), 4);
    assert_eq!(lines[0].words[1].kind, LineKind::Removed);
    assert_eq!(lines[0].words[1].text, "quick");
    assert_eq!((lines[0].old_line, lines[0].new_line), (Some(1), Some(1)));
    assert_eq!(lines[1].kind, LineKind::Removed);
    assert_eq!(lines[1].text, "gone line");
    assert_eq!((lines[1].old_line, lines[1].new_line), (Some(2), None));
    assert_eq!(lines[2].kind, LineKind::Added);
    assert_eq!((lines[2].old_line, lines[2].new_line), (None, Some(2)));
}

#[test]
fn quoted_rename_paths_are_decoded_and_garbage_is_ignored() {
    let text = "warning: something\ndiff --git \"a/t\\tb\" \"b/t\\tc\"\nrename from \"t\\tb\"\nrename to \"t\\tc\"\n@@ broken header\n";

    let sections = parse_diff(text, false);

    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].old_path.as_deref(), Some("t\tb"));
    assert!(sections[0].is_about("t\tc"));
}
