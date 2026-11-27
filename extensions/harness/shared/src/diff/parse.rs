//! Unified diff parsing: files, hunks and numbered lines from `git diff` or
//! `diff -u` text.

use super::model::{DiffLine, FilePatch, FileStatus, Hunk, LineKind, Patch};

pub(super) fn parse_patch(text: &str) -> Patch {
    Parser::default().run(text)
}

#[derive(Default)]
struct Parser {
    files: Vec<FilePatch>,
    current: Option<FilePatch>,
    hunk: Option<Hunk>,
    old_next: u32,
    new_next: u32,
    old_left: u32,
    new_left: u32,
}

impl Parser {
    fn run(mut self, text: &str) -> Patch {
        for raw in text.lines() {
            let line = raw.strip_suffix('\r').unwrap_or(raw);
            let marker = self.hunk.is_some() && line.starts_with('\\');
            if (marker || self.in_hunk_body()) && self.hunk_line(line) {
                continue;
            }
            self.header_line(line);
        }
        self.finish_file();
        Patch { files: self.files }
    }

    const fn in_hunk_body(&self) -> bool {
        self.hunk.is_some() && (self.old_left > 0 || self.new_left > 0)
    }

    fn hunk_line(&mut self, line: &str) -> bool {
        let (kind, body) = match line.as_bytes().first() {
            Some(b'+') => (LineKind::Added, &line[1..]),
            Some(b'-') => (LineKind::Removed, &line[1..]),
            Some(b' ') => (LineKind::Context, &line[1..]),
            Some(b'\\') => (LineKind::NoNewline, &line[1..]),
            None => (LineKind::Context, ""),
            Some(_) => return false,
        };
        let (old_line, new_line) = match kind {
            LineKind::Context => {
                let numbers = (Some(self.old_next), Some(self.new_next));
                self.old_next += 1;
                self.new_next += 1;
                self.old_left = self.old_left.saturating_sub(1);
                self.new_left = self.new_left.saturating_sub(1);
                numbers
            },
            LineKind::Added => {
                let number = Some(self.new_next);
                self.new_next += 1;
                self.new_left = self.new_left.saturating_sub(1);
                (None, number)
            },
            LineKind::Removed => {
                let number = Some(self.old_next);
                self.old_next += 1;
                self.old_left = self.old_left.saturating_sub(1);
                (number, None)
            },
            LineKind::NoNewline => (None, None),
        };
        if let Some(hunk) = self.hunk.as_mut() {
            hunk.lines.push(DiffLine {
                kind,
                old_line,
                new_line,
                text: body.to_owned(),
            });
        }
        true
    }

    fn header_line(&mut self, line: &str) {
        if let Some(rest) = line.strip_prefix("diff --git ") {
            self.git_header(rest);
        } else if let Some(rest) = line.strip_prefix("--- ") {
            self.old_header(rest);
        } else if let Some(rest) = line.strip_prefix("+++ ") {
            self.new_header(rest);
        } else if line.starts_with("@@") {
            if let Some(header) = parse_hunk_header(line) {
                self.start_hunk(header);
            }
        } else if let Some(file) = self.current.as_mut() {
            metadata_line(file, line);
        }
    }

    fn git_header(&mut self, rest: &str) {
        self.start_file();
        if let Some((old, new)) = split_git_paths(rest)
            && let Some(file) = self.current.as_mut()
        {
            file.old_path = Some(old);
            file.new_path = Some(new);
        }
    }

    fn old_header(&mut self, rest: &str) {
        let has_hunks = self
            .current
            .as_ref()
            .is_some_and(|file| !file.hunks.is_empty())
            || self.hunk.is_some();
        if self.current.is_none() || has_hunks {
            self.start_file();
        }
        let path = header_path(rest);
        if let Some(file) = self.current.as_mut() {
            if path.is_none() {
                file.status = FileStatus::Added;
            }
            file.old_path = path;
        }
    }

    fn new_header(&mut self, rest: &str) {
        if self.current.is_none() {
            self.start_file();
        }
        let path = header_path(rest);
        if let Some(file) = self.current.as_mut() {
            if path.is_none() {
                file.status = FileStatus::Deleted;
            }
            file.new_path = path;
        }
    }

    fn start_file(&mut self) {
        self.finish_file();
        self.current = Some(FilePatch::default());
    }

    fn finish_hunk(&mut self) {
        if let Some(hunk) = self.hunk.take() {
            self.current
                .get_or_insert_with(FilePatch::default)
                .hunks
                .push(hunk);
        }
        self.old_left = 0;
        self.new_left = 0;
    }

    fn finish_file(&mut self) {
        self.finish_hunk();
        if let Some(file) = self.current.take()
            && (has_header(&file) || !file.hunks.is_empty())
        {
            self.files.push(file);
        }
    }

    fn start_hunk(&mut self, header: Hunk) {
        self.finish_hunk();
        self.old_next = header.old_start;
        self.new_next = header.new_start;
        self.old_left = header.old_len;
        self.new_left = header.new_len;
        self.hunk = Some(header);
    }
}

fn header_path(rest: &str) -> Option<String> {
    let path = rest.split('\t').next().unwrap_or(rest).trim_end();
    if path == "/dev/null" {
        return None;
    }
    let path = path
        .strip_prefix("a/")
        .or_else(|| path.strip_prefix("b/"))
        .unwrap_or(path);
    Some(path.to_owned())
}

fn split_git_paths(rest: &str) -> Option<(String, String)> {
    let rest = rest.strip_prefix("a/")?;
    let middle = rest.find(" b/")?;
    Some((rest[..middle].to_owned(), rest[middle + 3..].to_owned()))
}

fn parse_range(spec: &str) -> Option<(u32, u32)> {
    match spec.split_once(',') {
        Some((start, len)) => Some((start.parse().ok()?, len.parse().ok()?)),
        None => Some((spec.parse().ok()?, 1)),
    }
}

fn parse_hunk_header(line: &str) -> Option<Hunk> {
    let rest = line.strip_prefix("@@ ")?;
    let (ranges, section) = rest.split_once(" @@")?;
    let (old, new) = ranges.split_once(' ')?;
    let (old_start, old_len) = parse_range(old.strip_prefix('-')?)?;
    let (new_start, new_len) = parse_range(new.strip_prefix('+')?)?;
    Some(Hunk {
        old_start,
        old_len,
        new_start,
        new_len,
        section: section.trim().to_owned(),
        lines: Vec::new(),
    })
}

const fn has_header(file: &FilePatch) -> bool {
    file.old_path.is_some() || file.new_path.is_some() || file.binary
}

fn metadata_line(file: &mut FilePatch, line: &str) {
    if line.starts_with("new file mode") {
        file.status = FileStatus::Added;
    } else if line.starts_with("deleted file mode") {
        file.status = FileStatus::Deleted;
    } else if let Some(from) = line.strip_prefix("rename from ") {
        file.status = FileStatus::Renamed;
        file.old_path = Some(from.to_owned());
    } else if let Some(to) = line.strip_prefix("rename to ") {
        file.status = FileStatus::Renamed;
        file.new_path = Some(to.to_owned());
    } else if line.starts_with("Binary files ") || line.starts_with("GIT binary patch") {
        file.binary = true;
    }
}
