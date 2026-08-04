//! Textual front-end standards gate, ported from
//! scripts/check-frontend-standards.sh: banned constructs in JS,
//! centralisation of fetch/event registration, and CSS hygiene over
//! storage/files/{js,css}.
//!
//! Exemption: a `path:rule` pair (one per line, `#` comments) in
//! scripts/frontend-standards-exemptions.txt. Reserve it for cases with a
//! documented reason, never as a way to mute a fixable violation.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    // Why: extensions/web sits two levels below the repo root.
    let mut root = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    for _ in 0..2 {
        root.pop();
    }
    root
}

fn walk(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, ext, out);
        } else if path.extension().is_some_and(|e| e == ext) {
            out.push(path);
        }
    }
}

fn exemptions(root: &Path) -> BTreeSet<String> {
    let path = root.join("scripts/frontend-standards-exemptions.txt");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return BTreeSet::new();
    };
    content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

const fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Strip string literals ('…', "…", `…`, non-multiline) and a trailing
/// `//` comment, so `==` inside strings and URLs never fire on code rules.
/// Each removed literal becomes one space to preserve token boundaries.
fn strip_literals_and_comment(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\'' | '"' | '`' => {
                let quote = c;
                let mut closed = false;
                while let Some(inner) = chars.next() {
                    if inner == '\\' {
                        chars.next();
                    } else if inner == quote {
                        closed = true;
                        break;
                    }
                }
                out.push(' ');
                if !closed {
                    break;
                }
            },
            '/' if chars.peek() == Some(&'/') => break,
            _ => out.push(c),
        }
    }
    out
}

/// Occurrence of `needle` whose preceding character fails `prev_ok`
/// (start-of-line counts as ok unless `require_prev`).
fn has_call(hay: &str, needle: &str, require_prev: bool, prev_ok: impl Fn(char) -> bool) -> bool {
    for (idx, _) in hay.match_indices(needle) {
        match hay[..idx].chars().next_back() {
            Some(prev) => {
                if prev_ok(prev) {
                    return true;
                }
            },
            None => {
                if !require_prev {
                    return true;
                }
            },
        }
    }
    false
}

fn has_loose_equality(code: &str) -> bool {
    let bytes = code.as_bytes();
    let mut i = 1;
    while i + 2 < bytes.len() {
        if bytes[i] == b'=' && bytes[i + 1] == b'=' {
            let prev = bytes[i - 1];
            let next = bytes[i + 2];
            if !matches!(prev, b'=' | b'!' | b'<' | b'>') && next != b'=' {
                return true;
            }
            i += 2;
        } else {
            i += 1;
        }
    }
    false
}

fn has_token_fallback(line: &str) -> bool {
    for (idx, _) in line.match_indices("var(--sp-") {
        let rest = &line[idx + "var(--sp-".len()..];
        let name_len = rest
            .find(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'))
            .unwrap_or(rest.len());
        if name_len > 0 && rest[name_len..].starts_with(',') {
            return true;
        }
    }
    false
}

/// Files whose job is fetch/event wiring for the public site; mirrors the
/// path excludes in the shell gate.
fn is_site_entry(rel: &str) -> bool {
    const TOP_LEVEL: &[&str] = &["analytics", "homepage", "blog-list", "docs", "mobile-menu"];
    rel.contains("services/api.js")
        || rel.contains("site/")
        || TOP_LEVEL.iter().any(|name| {
            rel.strip_prefix("storage/files/js/")
                .is_some_and(|tail| tail.starts_with(name))
        })
}

struct Violations {
    exempt: BTreeSet<String>,
    found: Vec<String>,
}

impl Violations {
    fn report(&mut self, rel: &str, line_no: usize, rule: &str, line: &str) {
        if self.exempt.contains(&format!("{rel}:{rule}")) {
            return;
        }
        self.found
            .push(format!("FAIL[{rule}] {rel}:{line_no}: {}", line.trim()));
    }
}

fn source_files(root: &Path, subdir: &str, ext: &str) -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    walk(&root.join(subdir), ext, &mut files);
    files.sort();
    files
        .into_iter()
        .filter_map(|path| {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            (!rel.contains("admin-bundle")).then_some((path, rel))
        })
        .collect()
}

fn check_js_line(v: &mut Violations, rel: &str, line_no: usize, raw: &str) {
    let trimmed = raw.trim_start();
    if trimmed.starts_with("//") || trimmed.starts_with("/*") {
        v.report(rel, line_no, "comments", raw);
        return;
    }
    let code = strip_literals_and_comment(raw);

    if has_call(&code, "var ", false, |p| !is_word(p) && p != '$') {
        v.report(rel, line_no, "var", raw);
    }
    if has_loose_equality(&code) && !raw.contains("null") {
        v.report(rel, line_no, "loose-equality", raw);
    }
    if has_call(&code, "eval(", false, |p| !is_word(p)) {
        v.report(rel, line_no, "eval", raw);
    }
    if code.contains("export default") {
        v.report(rel, line_no, "default-export", raw);
    }
    let dialog_ok = ["showConfirm", "showPrompt", ".confirm(", ".prompt("]
        .iter()
        .any(|ok| raw.contains(ok));
    if !dialog_ok
        && ["alert(", "confirm(", "prompt("]
            .iter()
            .any(|call| has_call(&code, call, false, |p| !is_word(p)))
    {
        v.report(rel, line_no, "alert-confirm-prompt", raw);
    }
    if ["log", "debug", "info", "warn", "error"]
        .iter()
        .any(|level| code.contains(&format!("console.{level}")))
    {
        v.report(rel, line_no, "console", raw);
    }
    if !is_site_entry(rel)
        && has_call(&code, "fetch(", true, |p| {
            !p.is_ascii_alphabetic() && p != '.'
        })
    {
        v.report(rel, line_no, "raw-fetch", raw);
    }
    if !is_site_entry(rel)
        && !rel.contains("services/events.js")
        && raw.contains("document.addEventListener('click'")
    {
        v.report(rel, line_no, "document-click-listener", raw);
    }
    if code.contains(".catch(() => {})") || code.contains(".catch(() => ({}))") {
        v.report(rel, line_no, "empty-catch", raw);
    }
    if code.contains("JSON.parse(JSON.stringify") {
        v.report(rel, line_no, "json-clone", raw);
    }
    if (code.contains(".appendChild(") || code.contains(".removeChild("))
        && !raw.contains("cloneNode")
    {
        v.report(rel, line_no, "legacy-dom", raw);
    }
}

fn check_css_line(v: &mut Violations, rel: &str, line_no: usize, raw: &str) {
    const FALLBACK_OK: &[&str] = &[
        "var(--sp-fill",
        "var(--sp-progress",
        "var(--sp-section-color",
        "var(--sp-xp-pct",
    ];
    const IMPORTANT_OK: &[&str] = &[
        "prefers-reduced-motion",
        "animation-duration",
        "animation-iteration-count",
        "transition-duration",
        "scroll-behavior",
    ];
    if raw.contains("!important") && !IMPORTANT_OK.iter().any(|ok| raw.contains(ok)) {
        v.report(rel, line_no, "important", raw);
    }
    if raw.contains("@import") {
        v.report(rel, line_no, "at-import", raw);
    }
    let mut chars = raw.chars();
    if chars.next() == Some('#') && chars.next().is_some_and(|c| c.is_ascii_lowercase()) {
        v.report(rel, line_no, "id-selector", raw);
    }
    if has_token_fallback(raw) && !FALLBACK_OK.iter().any(|ok| raw.contains(ok)) {
        v.report(rel, line_no, "token-fallback", raw);
    }
    if raw.contains("/*") && !rel.contains("core/fonts.css") {
        v.report(rel, line_no, "css-comments", raw);
    }
}

#[test]
fn frontend_sources_meet_textual_standards() {
    let root = repo_root();
    let mut v = Violations {
        exempt: exemptions(&root),
        found: Vec::new(),
    };

    for (path, rel) in source_files(&root, "storage/files/js", "js") {
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (idx, line) in content.lines().enumerate() {
            check_js_line(&mut v, &rel, idx + 1, line);
        }
    }
    for (path, rel) in source_files(&root, "storage/files/css", "css") {
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (idx, line) in content.lines().enumerate() {
            check_css_line(&mut v, &rel, idx + 1, line);
        }
    }

    assert!(
        v.found.is_empty(),
        "front-end standards violations (exempt via scripts/frontend-standards-exemptions.txt only with a documented reason):\n{}",
        v.found.join("\n")
    );
}
