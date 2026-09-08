//! Admin template / CSS agreement gate, ported from
//! scripts/check-admin-css-classes.sh.
//!
//! Admin pages are server-rendered from Handlebars templates; the CSS ships
//! separately as a bundle. Nothing links the two, so a renamed or deleted
//! rule leaves the markup referencing a class that no longer styles
//! anything. This test reads every `class="..."` in the templates and
//! partials and fails if a token has no matching `.token` rule anywhere in
//! the admin or core CSS sources.
//!
//! Exemption: list a class (one per line, `#` comments) in
//! scripts/admin-css-class-exemptions.txt. Reserve it for classes toggled
//! or generated at runtime by JS, never for a rule that is simply missing.


use crate::support::{repo_root, walk};

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn exemptions(root: &Path) -> BTreeSet<String> {
    let path = root.join("scripts/admin-css-class-exemptions.txt");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return BTreeSet::new();
    };
    content
        .lines()
        .filter_map(|line| {
            let entry = line.split('#').next().unwrap_or("").trim();
            (!entry.is_empty()).then(|| entry.to_owned())
        })
        .collect()
}

// Replace every `{{...}}` expression (possibly spanning lines) with one
// space BEFORE finding attributes: an expression can contain a `"` (e.g.
// `{{#if (eq x "active")}}`) that would otherwise terminate the
// `class="..."` match early and spill junk tokens. Braces do not nest.
fn strip_handlebars(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find("{{") {
        out.push_str(&rest[..open]);
        if let Some(close) = rest[open..].find("}}") {
            out.push(' ');
            rest = &rest[open + close + 2..];
        } else {
            // Why: unbalanced open brace — keep the tail verbatim, matching
            // the shell gate's non-greedy regex.
            rest = &rest[open..];
            break;
        }
    }
    out.push_str(rest);
    out
}

// Class tokens from a template. Dynamic or too-generic tokens are
// discarded: anything still holding braces, shorter than three
// characters, starting with a non-letter, or ending in `-` (the stump of
// a stripped dynamic modifier such as `cc-bp-item--{{status}}`).
fn classes_in(template: &str) -> BTreeSet<String> {
    let text = strip_handlebars(template);
    let mut found = BTreeSet::new();
    for chunk in text.split("class").skip(1) {
        let after_eq = chunk.trim_start();
        let Some(after_eq) = after_eq.strip_prefix('=') else {
            continue;
        };
        let Some(quoted) = after_eq.trim_start().strip_prefix('"') else {
            continue;
        };
        let Some(value) = quoted.split('"').next() else {
            continue;
        };
        for tok in value.split_whitespace() {
            if tok.contains("{{") || tok.contains("}}") {
                continue;
            }
            if tok.chars().count() < 3 {
                continue;
            }
            if !tok.chars().next().is_some_and(char::is_alphabetic) {
                continue;
            }
            if tok.ends_with('-') {
                continue;
            }
            found.insert(tok.to_owned());
        }
    }
    found
}

// A class `foo` is satisfied when `.foo` appears in the CSS corpus not
// immediately followed by another class-name character.
fn has_rule(css: &str, class: &str) -> bool {
    let needle = format!(".{class}");
    for (idx, _) in css.match_indices(&needle) {
        let next = css[idx + needle.len()..].chars().next();
        if !next.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return true;
        }
    }
    false
}

#[test]
fn every_admin_template_class_has_a_css_rule() {
    let root = repo_root();

    let tpl_dir = root.join("storage/files/admin/templates");
    // Why: returning here would report the same green as a run that checked
    // every template, which is exactly the failure a renamed directory causes.
    assert!(
        tpl_dir.is_dir(),
        "no admin templates directory at {}",
        tpl_dir.display()
    );
    let mut templates = Vec::new();
    walk(&tpl_dir, "hbs", &mut templates);
    walk(
        &root.join("storage/files/admin/partials"),
        "hbs",
        &mut templates,
    );
    templates.sort();

    let mut css_files = Vec::new();
    walk(&root.join("storage/files/css/admin"), "css", &mut css_files);
    walk(&root.join("storage/files/css/core"), "css", &mut css_files);
    css_files.sort();
    let mut css = String::new();
    for file in &css_files {
        if let Ok(content) = std::fs::read_to_string(file) {
            css.push_str(&content);
            css.push('\n');
        }
    }
    assert!(
        !templates.is_empty(),
        "no admin templates found under {}",
        tpl_dir.display()
    );
    assert!(
        !css.is_empty(),
        "no admin/core CSS sources found under {}",
        root.display()
    );

    let exempt = exemptions(&root);
    let mut cache: BTreeMap<String, bool> = BTreeMap::new();
    let mut violations = Vec::new();
    for template in &templates {
        let content = std::fs::read_to_string(template)
            .unwrap_or_else(|e| panic!("read {}: {e}", template.display()));
        let missing: Vec<String> = classes_in(&content)
            .into_iter()
            .filter(|class| {
                !exempt.contains(class)
                    && !*cache
                        .entry(class.clone())
                        .or_insert_with(|| has_rule(&css, class))
            })
            .collect();
        if !missing.is_empty() {
            let rel = template.strip_prefix(&root).unwrap_or(template);
            violations.push(format!(
                "{}\n    .{}",
                rel.display(),
                missing.join("\n    .")
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "admin template class(es) with no matching CSS rule (add the rule to storage/files/css/, or if the class is toggled by JS, list it in scripts/admin-css-class-exemptions.txt with a reason):\n{}",
        violations.join("\n")
    );
}

// The forward gate above proves markup has CSS. These two prove the reverse and
// the naming, which is what keeps one design language from drifting back into
// four: a `.sp-` rule nobody renders is dead weight the next reader has to
// disprove, and an unprefixed class is a rule outside the system.

fn admin_css_sources(root: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    walk(&root.join("storage/files/css/admin"), "css", &mut files);
    files.sort();
    assert!(!files.is_empty(), "no admin CSS under {}", root.display());
    files
}

fn admin_consumer_text(root: &Path) -> String {
    let mut files = Vec::new();
    walk(&root.join("storage/files/admin"), "hbs", &mut files);
    for dir in ["services", "components", "pages"] {
        walk(&root.join("storage/files/js").join(dir), "js", &mut files);
    }
    let mut text = String::new();
    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            text.push_str(&content);
            text.push('\n');
        }
    }
    assert!(
        !text.is_empty(),
        "no admin consumers under {}",
        root.display()
    );
    text
}

fn declared_classes(css: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let bytes: Vec<char> = css.chars().collect();
    let mut idx = 0;
    let mut in_block = false;
    while idx < bytes.len() {
        let c = bytes[idx];
        if c == '{' {
            in_block = true;
        } else if c == '}' {
            in_block = false;
        } else if c == '.' && !in_block {
            let start = idx + 1;
            let mut end = start;
            while end < bytes.len()
                && (bytes[end].is_ascii_alphanumeric() || bytes[end] == '-' || bytes[end] == '_')
            {
                end += 1;
            }
            if end > start && bytes[start].is_ascii_alphabetic() {
                out.insert(bytes[start..end].iter().collect::<String>());
            }
            idx = end;
            continue;
        }
        idx += 1;
    }
    out
}

#[test]
fn admin_css_classes_are_all_namespaced() {
    let root = repo_root();
    let exempt = exemptions(&root);
    let mut bad = Vec::new();
    for file in admin_css_sources(&root) {
        let css = std::fs::read_to_string(&file).unwrap();
        let rel = file
            .strip_prefix(&root)
            .unwrap_or(&file)
            .display()
            .to_string();
        for class in declared_classes(&css) {
            let ok = class.starts_with("sp-") || class.starts_with("is-");
            if !ok && !exempt.contains(&class) {
                bad.push(format!("{rel}: .{class}"));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "admin CSS class(es) outside the sp- / sp-u- / is- namespaces:\n{}",
        bad.join("\n")
    );
}

fn orphan_exemptions(root: &Path) -> BTreeSet<String> {
    let path = root.join("scripts/admin-css-orphan-exemptions.txt");
    let content =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

fn dynamic_stems(corpus: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for opener in ["{{", "${"] {
        let mut rest = corpus;
        while let Some(pos) = rest.find(opener) {
            let head = &rest[..pos];
            let stem: String = head
                .chars()
                .rev()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            if stem.len() > 3 && stem.ends_with('-') {
                out.insert(stem);
            }
            rest = &rest[pos + opener.len()..];
        }
    }
    out
}

#[test]
fn every_admin_css_class_has_a_consumer() {
    let root = repo_root();
    let mut exempt = exemptions(&root);
    exempt.extend(orphan_exemptions(&root));
    let consumers = admin_consumer_text(&root);
    let mut declared = BTreeSet::new();
    for file in admin_css_sources(&root) {
        declared.extend(declared_classes(&std::fs::read_to_string(&file).unwrap()));
    }
    // A class assembled at render time (`class="sp-badge sp-badge--{{tone}}"`,
    // or a JS template literal) never appears whole in the corpus. Collect the
    // literal stems that precede an interpolation and treat any class they
    // prefix as consumed.
    let stems = dynamic_stems(&consumers);
    let orphans: Vec<String> = declared
        .into_iter()
        .filter(|class| {
            !exempt.contains(class)
                && !consumers.contains(class.as_str())
                && !stems.iter().any(|stem| class.starts_with(stem))
        })
        .collect();
    assert!(
        orphans.is_empty(),
        "admin CSS rule(s) with no template, partial or JS consumer (delete the rule, or if it is generated at runtime list it in scripts/admin-css-class-exemptions.txt):\n    .{}",
        orphans.join("\n    .")
    );
}
