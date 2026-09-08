//! Textual front-end standards gate, ported from
//! scripts/check-frontend-standards.sh: banned constructs in JS,
//! centralisation of fetch/event registration, and CSS hygiene over
//! storage/files/{js,css}.
//!
//! Exemption: a `path:rule` pair (one per line, `#` comments) in
//! scripts/frontend-standards-exemptions.txt. Reserve it for cases with a
//! documented reason, never as a way to mute a fixable violation.


use crate::support::{repo_root, walk};

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

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

// Strip string literals ('…', "…", `…`, non-multiline) and a trailing
// `//` comment, so `==` inside strings and URLs never fire on code rules.
// Each removed literal becomes one space to preserve token boundaries.
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

// Occurrence of `needle` whose preceding character fails `prev_ok`
// (start-of-line counts as ok unless `require_prev`).
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

// Files whose job is fetch/event wiring for the public site; mirrors the
// path excludes in the shell gate.
fn is_site_entry(rel: &str) -> bool {
    const TOP_LEVEL: &[&str] = &["analytics", "homepage", "docs", "mobile-menu"];
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

// A divider banner (`/* --- Layout --- */`) or a comment with too few words to
// carry a reason.
fn is_banner_or_stub(raw: &str) -> bool {
    let Some(rest) = raw.split_once("/*").map(|(_, r)| r) else {
        return false;
    };
    let body = rest.trim_start_matches('*').trim_end_matches("*/").trim();
    if body.is_empty() {
        return true;
    }
    let banner = body
        .chars()
        .all(|c| c == '-' || c == '=' || c == '*' || c == ' ')
        || (body.starts_with('-') && body.ends_with('-'));
    // A continuation line of a multi-line comment carries no `/*` of its own, so
    // only the opening line is judged, and it is judged on its own length.
    banner || body.split_whitespace().count() < 4
}

fn check_js_line(v: &mut Violations, rel: &str, line_no: usize, raw: &str) {
    let trimmed = raw.trim_start();
    // Same reasoning as the CSS rule: javascript-coding-standards asks for a WHY
    // on a non-obvious decision, so what is banned is the divider banner, the
    // one-liner too short to be a reason, and commented-out code — not the
    // reason itself.
    if trimmed.starts_with("/*") && is_banner_or_stub(trimmed) {
        v.report(rel, line_no, "comments", raw);
        return;
    }
    if let Some(body) = trimmed.strip_prefix("//") {
        let body = body.trim();
        let commented_code = body.ends_with(';') || body.ends_with('{') || body.ends_with('}');
        if body.is_empty() || commented_code || body.split_whitespace().count() < 4 {
            v.report(rel, line_no, "comments", raw);
            return;
        }
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
    // css-coding-standards requires a WHY comment on an invariant a file exists
    // to protect and on a contrast-critical token, so a blanket ban on comments
    // would forbid the one comment the standard asks for. What stays banned is
    // the section-divider banner — a file that needs sections needs splitting —
    // and the one-liner too short to be a reason.
    if raw.contains("/*") && !rel.contains("core/fonts.css") && is_banner_or_stub(raw) {
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

    // Why: the assertion below is "no violations", which an empty corpus
    // satisfies. Renaming either source directory would otherwise turn this
    // gate green by giving it nothing to read.
    let js = source_files(&root, "storage/files/js", "js");
    let css = source_files(&root, "storage/files/css", "css");
    assert!(
        !js.is_empty(),
        "no JavaScript sources under {}/storage/files/js",
        root.display()
    );
    assert!(
        !css.is_empty(),
        "no CSS sources under {}/storage/files/css",
        root.display()
    );

    for (path, rel) in js {
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        for (idx, line) in content.lines().enumerate() {
            check_js_line(&mut v, &rel, idx + 1, line);
        }
    }
    for (path, rel) in css {
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
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

// Design-system hygiene over storage/files/css/admin. These four are what make
// the token layer the only place a value is decided, so a later dark theme is a
// change to 01-tokens-*.css and nothing else.

fn admin_css(root: &Path) -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    walk(&root.join("storage/files/css/admin"), "css", &mut files);
    files.sort();
    assert!(!files.is_empty(), "no admin CSS under {}", root.display());
    files
        .into_iter()
        .map(|p| {
            let rel = p.strip_prefix(root).unwrap_or(&p).display().to_string();
            (p, rel)
        })
        .collect()
}

// Custom properties do not resolve in print engines, so @media print is the one
// place a literal colour and an !important are the only mechanism available.
fn print_block_lines(css: &str) -> BTreeSet<usize> {
    let mut out = BTreeSet::new();
    let mut depth: i32 = 0;
    let mut start_depth: Option<i32> = None;
    for (idx, line) in css.lines().enumerate() {
        if line.contains("@media print") {
            start_depth = Some(depth);
        }
        if start_depth.is_some() {
            out.insert(idx + 1);
        }
        depth += i32::try_from(line.matches('{').count()).unwrap_or(0);
        depth -= i32::try_from(line.matches('}').count()).unwrap_or(0);
        if let Some(sd) = start_depth {
            if depth <= sd {
                start_depth = None;
            }
        }
    }
    out
}

#[test]
fn admin_css_files_stay_under_the_line_limit() {
    let root = repo_root();
    let over: Vec<String> = admin_css(&root)
        .into_iter()
        .filter_map(|(path, rel)| {
            let lines = std::fs::read_to_string(&path).unwrap().lines().count();
            (lines > 200).then(|| format!("{rel}: {lines} lines"))
        })
        .collect();
    assert!(
        over.is_empty(),
        "admin CSS file(s) over 200 lines — split by component:\n{}",
        over.join("\n")
    );
}

#[test]
fn admin_css_holds_no_colour_literal_outside_the_token_files() {
    let root = repo_root();
    let mut bad = Vec::new();
    for (path, rel) in admin_css(&root) {
        if rel.contains("01-tokens-") {
            continue;
        }
        let css = std::fs::read_to_string(&path).unwrap();
        let print = print_block_lines(&css);
        for (idx, line) in css.lines().enumerate() {
            if print.contains(&(idx + 1)) {
                continue;
            }
            let code = line.split("/*").next().unwrap_or(line);
            let hex = code.split('#').skip(1).any(|tail| {
                let run = tail.chars().take_while(char::is_ascii_hexdigit).count();
                matches!(run, 3 | 4 | 6 | 8)
            });
            if hex || code.contains("rgb(") || code.contains("rgba(") {
                bad.push(format!("{rel}:{}: {}", idx + 1, code.trim()));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "colour literal(s) outside storage/files/css/admin/01-tokens-*.css — add a token instead:\n{}",
        bad.join("\n")
    );
}

#[test]
fn admin_css_uses_important_only_in_the_reset() {
    let root = repo_root();
    let mut bad = Vec::new();
    for (path, rel) in admin_css(&root) {
        if rel.ends_with("02-reset.css") {
            continue;
        }
        let css = std::fs::read_to_string(&path).unwrap();
        let print = print_block_lines(&css);
        for (idx, line) in css.lines().enumerate() {
            if line.contains("!important") && !print.contains(&(idx + 1)) {
                bad.push(format!("{rel}:{}: {}", idx + 1, line.trim()));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "!important outside the reduced-motion reset — restructure the selector:\n{}",
        bad.join("\n")
    );
}

#[test]
fn admin_css_spaces_in_tokens_not_pixels() {
    let root = repo_root();
    let props = [
        "padding",
        "margin",
        "gap",
        "row-gap",
        "column-gap",
        "padding-top",
        "padding-bottom",
        "padding-left",
        "padding-right",
        "padding-inline",
        "padding-block",
        "margin-top",
        "margin-bottom",
        "margin-left",
        "margin-right",
        "margin-inline",
        "margin-block",
    ];
    let mut bad = Vec::new();
    for (path, rel) in admin_css(&root) {
        let css = std::fs::read_to_string(&path).unwrap();
        for (idx, line) in css.lines().enumerate() {
            let code = line.split("/*").next().unwrap_or(line).trim();
            let Some((name, value)) = code.split_once(':') else {
                continue;
            };
            let name = name.trim();
            if !props.contains(&name) {
                continue;
            }
            // 0px and 1px are hairlines and offsets, not spacing steps.
            let offending = value.split_whitespace().any(|tok| {
                let tok = tok.trim_start_matches('-').trim_end_matches([';', ')']);
                tok.ends_with("px") && tok != "0px" && tok != "1px" && !tok.contains("var(")
            });
            if offending {
                bad.push(format!("{rel}:{}: {code}", idx + 1));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "pixel spacing in admin CSS — use a --sp-space-* or --sp-density-* token:\n{}",
        bad.join("\n")
    );
}
