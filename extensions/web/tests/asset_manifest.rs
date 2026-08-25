//! Build-time front-end standards gate.
//!
//! Validates the asset manifest and the front-end source tree against the
//! project's front-end coding standards: manifest/disk agreement in both
//! directions, template references resolving to published assets, inline
//! code kept out of templates, file-size ceilings, and a single-source
//! design-token invariant (`--sp-` prefix, one defining file per token per
//! CSS scope).

mod support;

use support::{repo_root, walk};

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use systemprompt::extension::AssetPaths;
use systemprompt::extension::prelude::Extension;
use systemprompt_web_extension::extension::WebExtension;

fn web_assets(paths: &dyn AssetPaths) -> Vec<systemprompt::extension::AssetDefinition> {
    WebExtension::new().required_assets(paths)
}

struct TestPaths {
    storage: PathBuf,
    dist: PathBuf,
}

impl AssetPaths for TestPaths {
    fn storage_files(&self) -> &Path {
        &self.storage
    }
    fn web_dist(&self) -> &Path {
        &self.dist
    }
}

fn test_paths() -> TestPaths {
    let root = repo_root();
    TestPaths {
        storage: root.join("storage/files"),
        dist: root.join("web/dist"),
    }
}

// Generated at publish time by `bundle_admin_css`; absent in a fresh checkout.
const GENERATED: &[&str] = &["css/admin-bundle.css"];

#[test]
fn every_registered_asset_exists_on_disk() {
    let paths = test_paths();
    let missing: Vec<String> = web_assets(&paths)
        .iter()
        .filter(|a| a.is_required() && !a.source().is_file())
        .map(|a| a.source().display().to_string())
        .collect();
    assert!(
        missing.is_empty(),
        "registered assets missing on disk:\n{}",
        missing.join("\n")
    );
}

#[test]
fn every_source_file_is_registered() {
    let paths = test_paths();
    let registered: BTreeSet<PathBuf> = web_assets(&paths)
        .iter()
        .map(|a| a.source().to_path_buf())
        .collect();

    let storage = paths.storage_files();
    let mut sources = Vec::new();
    walk(&storage.join("css"), "css", &mut sources);
    walk(&storage.join("js"), "js", &mut sources);

    let orphans: Vec<String> = sources
        .into_iter()
        .filter(|p| {
            let rel = p.strip_prefix(storage).unwrap_or(p);
            let rel_str = rel.to_string_lossy();
            let bundled_admin_css = rel_str.starts_with("css/admin/");
            let generated = GENERATED.iter().any(|g| rel_str == *g);
            !bundled_admin_css && !generated && !registered.contains(p)
        })
        .map(|p| p.display().to_string())
        .collect();

    assert!(
        orphans.is_empty(),
        "unregistered source files (register in extensions/web/site/src/assets/ or extensions/web/admin/src/assets.rs, or delete):\n{}",
        orphans.join("\n")
    );
}

fn template_files(root: &Path) -> Vec<PathBuf> {
    let mut templates = Vec::new();
    walk(&root.join("storage/files/admin"), "hbs", &mut templates);
    walk(&root.join("services/web/templates"), "html", &mut templates);
    templates
}

fn extract_asset_refs(content: &str) -> Vec<String> {
    let mut refs = Vec::new();
    for attr in ["src=\"", "href=\""] {
        for chunk in content.split(attr).skip(1) {
            let Some(value) = chunk.split('"').next() else {
                continue;
            };
            let value = value
                .replace("{{CSS_BASE_PATH}}", "/css")
                .replace("{{JS_BASE_PATH}}", "/js");
            let value = value.split('?').next().unwrap_or(&value).to_owned();
            let has_ext = |ext: &str| {
                Path::new(&value)
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case(ext))
            };
            if (value.starts_with("/css/") && has_ext("css"))
                || (value.starts_with("/js/") && has_ext("js"))
            {
                refs.push(value.trim_start_matches('/').to_owned());
            }
        }
    }
    refs
}

#[test]
fn every_template_asset_reference_resolves() {
    let root = repo_root();
    let paths = test_paths();
    let published: BTreeSet<String> = web_assets(&paths)
        .iter()
        .map(|a| a.destination().to_owned())
        .collect();

    let mut unresolved = Vec::new();
    for template in template_files(&root) {
        let Ok(content) = std::fs::read_to_string(&template) else {
            continue;
        };
        for reference in extract_asset_refs(&content) {
            if !published.contains(&reference) {
                unresolved.push(format!("{} -> {reference}", template.display()));
            }
        }
    }
    assert!(
        unresolved.is_empty(),
        "template asset references not in the manifest:\n{}",
        unresolved.join("\n")
    );
}

#[test]
fn templates_contain_no_inline_code() {
    let root = repo_root();
    let mut violations = Vec::new();
    for template in template_files(&root) {
        let Ok(content) = std::fs::read_to_string(&template) else {
            continue;
        };
        let name = template.display().to_string();
        let is_fouc_shell = name.ends_with("partials/layout.hbs");
        let is_critical_css_shell = name.ends_with("partials/head-assets.html");
        for (idx, line) in content.lines().enumerate() {
            let has_open = line.contains("<script") && !line.contains("src=");
            let is_data = line.contains("application/ld+json") || line.contains("application/json");
            let is_theme_snippet = line.contains("</script>") && line.contains("colorScheme");
            if has_open && !is_data && !is_fouc_shell && !is_theme_snippet {
                violations.push(format!("{name}:{} inline <script>", idx + 1));
            }
            if line.contains("<style") && !is_critical_css_shell {
                violations.push(format!("{name}:{} inline <style>", idx + 1));
            }
            if line.contains("style=\"") && !line.contains("style=\"--") {
                violations.push(format!("{name}:{} inline style attribute", idx + 1));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "inline code in templates (extract to registered CSS/JS):\n{}",
        violations.join("\n")
    );
}

#[test]
fn source_files_respect_line_limits() {
    let root = repo_root();
    let mut js = Vec::new();
    let mut css = Vec::new();
    walk(&root.join("storage/files/js"), "js", &mut js);
    walk(&root.join("storage/files/css"), "css", &mut css);

    let mut oversized = Vec::new();
    for (files, limit) in [(&js, 250usize), (&css, 400usize)] {
        for file in files {
            let rel = file.strip_prefix(&root).unwrap_or(file);
            if GENERATED.iter().any(|g| rel.to_string_lossy().ends_with(g)) {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(file) else {
                continue;
            };
            let lines = content.lines().count();
            if lines > limit {
                oversized.push(format!("{} ({lines} > {limit})", rel.display()));
            }
        }
    }
    assert!(
        oversized.is_empty(),
        "files over the line limit (split by component/concern):\n{}",
        oversized.join("\n")
    );
}

fn token_definitions(file: &Path) -> Vec<String> {
    let Ok(content) = std::fs::read_to_string(file) else {
        return Vec::new();
    };
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("--") {
                trimmed.split(':').next().map(str::to_owned)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn custom_properties_use_sp_prefix() {
    let root = repo_root();
    let mut css = Vec::new();
    walk(&root.join("storage/files/css"), "css", &mut css);
    let mut violations = Vec::new();
    for file in &css {
        let rel = file
            .strip_prefix(&root)
            .unwrap_or(file)
            .to_string_lossy()
            .to_string();
        if GENERATED.iter().any(|g| rel.ends_with(g)) {
            continue;
        }
        for token in token_definitions(file) {
            if !token.starts_with("--sp-") {
                violations.push(format!(
                    "{}: {token}",
                    file.strip_prefix(&root).unwrap_or(file).display()
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "custom properties without the --sp- prefix:\n{}",
        violations.join("\n")
    );
}

// Scoped custom properties assigned contextually (per-section accents,
// per-element fills, responsive gutter overrides), not design tokens.
const SCOPED_PROPERTIES: &[&str] = &[
    "--sp-section-color",
    "--sp-fill",
    "--sp-gutter",
    "--sp-header-height",
];

#[test]
fn design_tokens_have_a_single_defining_file_per_scope() {
    let root = repo_root();
    let css_root = root.join("storage/files/css");
    let mut all = Vec::new();
    walk(&css_root, "css", &mut all);

    let admin_scope: Vec<&PathBuf> = all
        .iter()
        .filter(|p| {
            p.strip_prefix(&css_root)
                .is_ok_and(|r| r.starts_with("admin/"))
        })
        .collect();
    let site_scope: Vec<&PathBuf> = all
        .iter()
        .filter(|p| {
            p.strip_prefix(&css_root).is_ok_and(|r| {
                !r.starts_with("admin") && !r.to_string_lossy().contains("admin-bundle")
            })
        })
        .collect();

    let mut conflicts = Vec::new();
    for scope in [admin_scope, site_scope] {
        let mut owners: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for file in scope {
            let rel = file
                .strip_prefix(&root)
                .unwrap_or(file)
                .display()
                .to_string();
            for token in token_definitions(file) {
                if SCOPED_PROPERTIES.contains(&token.as_str()) {
                    continue;
                }
                owners.entry(token).or_default().insert(rel.clone());
            }
        }
        for (token, files) in owners {
            if files.len() > 1 {
                conflicts.push(format!(
                    "{token} defined in {}",
                    files.into_iter().collect::<Vec<_>>().join(", ")
                ));
            }
        }
    }
    assert!(
        conflicts.is_empty(),
        "design tokens defined in more than one file per scope:\n{}",
        conflicts.join("\n")
    );
}
