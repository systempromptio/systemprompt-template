use systemprompt_harness_shared::diff::{
    DiffLine, FileStatus, LineKind, Patch, Segment, Stat, fold_unchanged,
};

const GIT: &str = "diff --git a/src/lib.rs b/src/lib.rs
index 1111111..2222222 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,4 +1,5 @@ fn main() {
 use std::io;
-let a = 1;
+let a = 2;
+let b = 3;
 
 fn x() {}
diff --git a/new.txt b/new.txt
new file mode 100644
--- /dev/null
+++ b/new.txt
@@ -0,0 +1,2 @@
+hello
+world
\\ No newline at end of file
";

#[test]
fn parses_files_hunks_and_line_numbers() {
    let patch = Patch::parse(GIT);
    assert_eq!(patch.files.len(), 2);
    let first = &patch.files[0];
    assert_eq!(first.display_path(), "src/lib.rs");
    assert_eq!(first.status, FileStatus::Modified);
    assert_eq!(first.hunks.len(), 1);
    let hunk = &first.hunks[0];
    assert_eq!(
        (hunk.old_start, hunk.old_len, hunk.new_start, hunk.new_len),
        (1, 4, 1, 5)
    );
    assert_eq!(hunk.section, "fn main() {");
    let kinds: Vec<LineKind> = hunk.lines.iter().map(|line| line.kind).collect();
    assert_eq!(
        kinds,
        vec![
            LineKind::Context,
            LineKind::Removed,
            LineKind::Added,
            LineKind::Added,
            LineKind::Context,
            LineKind::Context,
        ]
    );
    assert_eq!(hunk.lines[1].old_line, Some(2));
    assert_eq!(hunk.lines[1].new_line, None);
    assert_eq!(hunk.lines[3].new_line, Some(3));
    assert_eq!(hunk.lines[5].old_line, Some(4));
    assert_eq!(hunk.lines[5].new_line, Some(5));
    assert_eq!(
        first.stat(),
        Stat {
            added: 2,
            removed: 1
        }
    );
}

#[test]
fn new_files_and_no_newline_markers() {
    let patch = Patch::parse(GIT);
    let second = &patch.files[1];
    assert_eq!(second.status, FileStatus::Added);
    assert_eq!(second.old_path, None);
    assert_eq!(second.display_path(), "new.txt");
    let lines = &second.hunks[0].lines;
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[2].kind, LineKind::NoNewline);
    assert_eq!(
        patch.stat(),
        Stat {
            added: 4,
            removed: 1
        }
    );
}

#[test]
fn content_that_looks_like_a_header_stays_content() {
    let text =
        "--- a/notes.md\n+++ b/notes.md\n@@ -1,2 +1,2 @@\n--- old rule\n+++ new rule\n context\n";
    let patch = Patch::parse(text);
    assert_eq!(patch.files.len(), 1);
    let lines = &patch.files[0].hunks[0].lines;
    assert_eq!(lines[0].kind, LineKind::Removed);
    assert_eq!(lines[0].text, "-- old rule");
    assert_eq!(lines[1].kind, LineKind::Added);
    assert_eq!(lines[1].text, "++ new rule");
}

#[test]
fn plain_unified_diff_and_deletion() {
    let text = "--- old.txt\t2024-01-01\n+++ /dev/null\n@@ -1 +0,0 @@\n-gone\n";
    let patch = Patch::parse(text);
    let file = &patch.files[0];
    assert_eq!(file.status, FileStatus::Deleted);
    assert_eq!(file.old_path.as_deref(), Some("old.txt"));
    assert_eq!(file.display_path(), "old.txt");
    assert_eq!(
        file.stat(),
        Stat {
            added: 0,
            removed: 1
        }
    );
}

#[test]
fn renames_and_binaries() {
    let text = "diff --git a/a.png b/b.png\nsimilarity index 90%\nrename from a.png\nrename to b.png\nBinary files a/a.png and b/b.png differ\n";
    let patch = Patch::parse(text);
    let file = &patch.files[0];
    assert_eq!(file.status, FileStatus::Renamed);
    assert!(file.binary);
    assert_eq!(file.old_path.as_deref(), Some("a.png"));
    assert_eq!(file.new_path.as_deref(), Some("b.png"));
}

#[test]
fn bare_hunks_without_headers_are_kept() {
    let patch = Patch::parse("@@ -3,2 +3,2 @@\n-a\n+b\n same\n");
    assert_eq!(patch.files.len(), 1);
    assert_eq!(patch.files[0].display_path(), "");
    assert_eq!(patch.files[0].hunks[0].lines.len(), 3);
}

#[test]
fn crlf_and_garbage_are_tolerated() {
    let patch = Patch::parse("noise\r\n--- a/x\r\n+++ b/x\r\n@@ -1 +1 @@\r\n-a\r\n+b\r\n");
    assert_eq!(patch.files[0].hunks[0].lines[1].text, "b");
    assert!(Patch::parse("not a diff at all").is_empty());
}

fn context(count: usize) -> Vec<DiffLine> {
    (0..count)
        .map(|index| DiffLine {
            kind: LineKind::Context,
            old_line: Some(index as u32 + 1),
            new_line: Some(index as u32 + 1),
            text: String::new(),
        })
        .collect()
}

fn change() -> DiffLine {
    DiffLine {
        kind: LineKind::Added,
        old_line: None,
        new_line: Some(1),
        text: String::new(),
    }
}

#[test]
fn long_unchanged_runs_fold_to_their_context() {
    let mut lines = vec![change()];
    lines.extend(context(20));
    lines.push(change());
    let segments = fold_unchanged(&lines, 3, 4);
    assert_eq!(
        segments,
        vec![
            Segment::Lines(0..4),
            Segment::Folded(4..18),
            Segment::Lines(18..22)
        ]
    );
}

#[test]
fn runs_at_the_edges_keep_context_on_one_side() {
    let mut lines = context(10);
    lines.push(change());
    lines.extend(context(10));
    let segments = fold_unchanged(&lines, 3, 4);
    assert_eq!(
        segments,
        vec![
            Segment::Folded(0..7),
            Segment::Lines(7..14),
            Segment::Folded(14..21),
        ]
    );
}

#[test]
fn short_runs_stay_visible() {
    let mut lines = vec![change()];
    lines.extend(context(8));
    lines.push(change());
    assert_eq!(fold_unchanged(&lines, 3, 4), vec![Segment::Lines(0..10)]);
}
