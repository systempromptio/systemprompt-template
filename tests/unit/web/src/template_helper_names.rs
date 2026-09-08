//! Gate: no template may name a variable or a partial parameter after a
//! registered Handlebars helper.
//!
//! A helper wins the name. `sub` is registered as a two-argument subtraction,
//! so a bare `sub` mustache called it with no arguments and rendered "0" under
//! every KPI value and beside every section heading, on every page, for as long
//! as the partials named their supporting line `sub`. Nothing failed: the page
//! rendered, the gates passed, and the console just quietly showed a zero.
//!
//! The helper list is read from the registration site rather than restated
//! here, so a helper added later is covered without anyone remembering to come
//! back. A field that genuinely has to carry such a name is readable as
//! `this.<name>`, which is a path and cannot be taken for a helper call.

use crate::support::{repo_root, walk};

use std::collections::BTreeSet;
use std::path::Path;

// Every name passed to register_helper in the admin crate's registration site.
fn registered_helpers(root: &Path) -> BTreeSet<String> {
    let path = root.join("extensions/web/admin/src/templates/helpers/mod.rs");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("gate cannot read {}: {e}", path.display()));
    let mut names = BTreeSet::new();
    for line in source.lines() {
        let Some(rest) = line.split_once("register_helper(\"") else {
            continue;
        };
        if let Some((name, _)) = rest.1.split_once('"') {
            names.insert(name.to_owned());
        }
    }
    assert!(
        names.len() > 5,
        "found only {} helper registrations in {} — the gate is reading the wrong file",
        names.len(),
        path.display()
    );
    names
}

// `css_version` is a helper the layout is meant to call: it stamps the asset
// query string, and there is no field of that name for it to shadow.
const CALLED_ON_PURPOSE: [&str; 1] = ["css_version"];

fn identifier_at(text: &str, start: usize) -> Option<(String, usize)> {
    let bytes = text.as_bytes();
    let mut end = start;
    while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
    }
    (end > start).then(|| (text[start..end].to_owned(), end))
}

// A bare `{{name}}`, `{{#if name}}` or `{{#unless name}}` — no path, no
// arguments, so the helper resolves ahead of any field or parameter.
fn bare_mustaches(line: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find("{{") {
        let after = &rest[open + 2..];
        let Some(close) = after.find("}}") else { break };
        let body = after[..close].trim();
        let body = body
            .trim_start_matches('#')
            .trim_start_matches('/')
            .trim_start_matches('>')
            .trim_start_matches('&')
            .trim();
        let body = body
            .strip_prefix("if ")
            .or_else(|| body.strip_prefix("unless "))
            .unwrap_or(body)
            .trim();
        if let Some((name, end)) = identifier_at(body, 0) {
            if body[end..].trim().is_empty() {
                found.push(name);
            }
        }
        rest = &after[close + 2..];
    }
    found
}

// A partial parameter, `{{> partial name=value}}`.
fn parameter_names(line: &str) -> Vec<String> {
    let mut found = Vec::new();
    let bytes = line.as_bytes();
    for (idx, _) in line.match_indices('=') {
        if idx == 0 || bytes.get(idx + 1) == Some(&b'=') || bytes[idx - 1] == b'=' {
            continue;
        }
        let mut start = idx;
        while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
            start -= 1;
        }
        if start < idx && bytes[start].is_ascii_alphabetic() {
            found.push(line[start..idx].to_owned());
        }
    }
    found
}

#[test]
fn no_template_names_a_value_after_a_registered_helper() {
    let root = repo_root();
    let helpers = registered_helpers(&root);

    let mut files = Vec::new();
    walk(&root.join("storage/files/admin"), "hbs", &mut files);
    files.sort();
    assert!(
        !files.is_empty(),
        "no admin templates under {}/storage/files/admin",
        root.display()
    );

    let mut violations = Vec::new();
    for file in &files {
        let content = std::fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("read {}: {e}", file.display()));
        let rel = file
            .strip_prefix(&root)
            .unwrap_or(file)
            .display()
            .to_string();
        let mut in_comment = false;
        for (idx, line) in content.lines().enumerate() {
            // Handlebars comments illustrate the rule; they do not render.
            if line.contains("{{!--") {
                in_comment = true;
            }
            let comment_ends = line.contains("--}}");
            let skip = in_comment;
            if comment_ends {
                in_comment = false;
            }
            if skip {
                continue;
            }
            for name in bare_mustaches(line) {
                if helpers.contains(&name) && !CALLED_ON_PURPOSE.contains(&name.as_str()) {
                    violations.push(format!("{rel}:{}: {{{{{name}}}}}", idx + 1));
                }
            }
            for name in parameter_names(line) {
                if helpers.contains(&name) && !CALLED_ON_PURPOSE.contains(&name.as_str()) {
                    violations.push(format!("{rel}:{}: parameter {name}=", idx + 1));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "template value(s) named after a registered helper — the helper wins the name, so this renders the helper's result instead of the value. Read a field as `this.{{name}}`, or rename a partial parameter:\n{}",
        violations.join("\n")
    );
}
