//! The parsed shape of a unified diff (files, hunks, numbered lines, stats) and
//! the folding of long unchanged runs.

use std::ops::Range;

/// What a diff line does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineKind {
    Context,
    Added,
    Removed,
    NoNewline,
}

/// One line of a hunk with its old and new line numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: LineKind,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
    pub text: String,
}

/// A hunk: its header ranges, the section heading after the `@@`, and its
/// lines.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Hunk {
    pub old_start: u32,
    pub old_len: u32,
    pub new_start: u32,
    pub new_len: u32,
    pub section: String,
    pub lines: Vec<DiffLine>,
}

/// How a file changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FileStatus {
    #[default]
    Modified,
    Added,
    Deleted,
    Renamed,
}

/// Added and removed line counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Stat {
    pub added: usize,
    pub removed: usize,
}

impl Stat {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.added == 0 && self.removed == 0
    }
}

impl std::ops::Add for Stat {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            added: self.added + other.added,
            removed: self.removed + other.removed,
        }
    }
}

/// The changes to one file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FilePatch {
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub status: FileStatus,
    pub binary: bool,
    pub hunks: Vec<Hunk>,
}

impl FilePatch {
    #[must_use]
    pub fn display_path(&self) -> &str {
        self.new_path
            .as_deref()
            .or(self.old_path.as_deref())
            .unwrap_or_default()
    }

    #[must_use]
    pub fn stat(&self) -> Stat {
        let mut stat = Stat::default();
        for line in self.hunks.iter().flat_map(|hunk| hunk.lines.iter()) {
            match line.kind {
                LineKind::Added => stat.added += 1,
                LineKind::Removed => stat.removed += 1,
                LineKind::Context | LineKind::NoNewline => {},
            }
        }
        stat
    }
}

/// A parsed unified diff (`git diff` or `diff -u` output), one entry per file.
/// Parsing skips lines it does not understand, so a partial or decorated patch
/// still shows what it can.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Patch {
    pub files: Vec<FilePatch>,
}

impl Patch {
    #[must_use]
    pub fn parse(text: &str) -> Self {
        super::parse::parse_patch(text)
    }

    #[must_use]
    pub fn stat(&self) -> Stat {
        self.files
            .iter()
            .fold(Stat::default(), |total, file| total + file.stat())
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

/// A visible run of lines or a folded run of unchanged ones, as indices into a
/// hunk's lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    Lines(Range<usize>),
    Folded(Range<usize>),
}

#[must_use]
pub fn fold_unchanged(lines: &[DiffLine], keep: usize, min_hidden: usize) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut visible_start = 0;
    let mut index = 0;
    while index < lines.len() {
        if lines[index].kind != LineKind::Context {
            index += 1;
            continue;
        }
        let run_start = index;
        while index < lines.len() && lines[index].kind == LineKind::Context {
            index += 1;
        }
        let run_end = index;
        let lead = if run_start == 0 { 0 } else { keep };
        let trail = if run_end == lines.len() { 0 } else { keep };
        let hide_start = run_start + lead;
        let hide_end = run_end.saturating_sub(trail);
        if hide_end > hide_start && hide_end - hide_start >= min_hidden.max(1) {
            if hide_start > visible_start {
                segments.push(Segment::Lines(visible_start..hide_start));
            }
            segments.push(Segment::Folded(hide_start..hide_end));
            visible_start = hide_end;
        }
    }
    if visible_start < lines.len() {
        segments.push(Segment::Lines(visible_start..lines.len()));
    }
    segments
}
