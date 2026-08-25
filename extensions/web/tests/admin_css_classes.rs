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

mod support;

use support::{repo_root, walk};

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
    if !tpl_dir.is_dir() {
        return;
    }
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
        !css.is_empty(),
        "no admin/core CSS sources found under {}",
        root.display()
    );

    let exempt = exemptions(&root);
    let mut cache: BTreeMap<String, bool> = BTreeMap::new();
    let mut violations = Vec::new();
    for template in &templates {
        let Ok(content) = std::fs::read_to_string(template) else {
            continue;
        };
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
