//! The round-trip proof: re-import the exported tree with core's strict
//! importer and compare what comes back against `services/`.
//!
//! A value that differs is a failure. A key `services/` carries that the
//! import did not reproduce is reported as *lost* and is a failure too — the
//! point of the exporter is that nothing about the marketplace changes when
//! it moves repositories. Keys the import adds (defaults it fills in) are
//! not differences: they are the same declaration written out fully.

use std::fmt;
use std::path::Path;

use anyhow::{Context, Result};
use serde_yaml::Value;
use systemprompt::manifest::services::split_frontmatter;
use systemprompt::marketplace::{ImportOptions, import_anthropic_tree};

use crate::ExportReport;

// Why: Anthropic's marketplace.json has no slot for these and core's sidecar
// refuses them, so they are reported nowhere rather than as a loss.
const NOT_IN_KIT_FORM: &[&str] = &["marketplace.keywords", "marketplace.license"];

#[derive(Debug)]
pub enum Diff {
    Value {
        file: String,
        path: String,
        services: String,
        imported: String,
    },
    Lost {
        file: String,
        path: String,
    },
    Missing {
        file: String,
    },
}

impl fmt::Display for Diff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value {
                file,
                path,
                services,
                imported,
            } => write!(f, "{file}: {path}: services={services} imported={imported}"),
            Self::Lost { file, path } => write!(f, "{file}: {path}: lost on import"),
            Self::Missing { file } => write!(f, "{file}: not produced by the import"),
        }
    }
}

pub fn round_trip(kit: &Path, services_root: &Path, report: &ExportReport) -> Result<Vec<Diff>> {
    let scratch = tempfile::tempdir().context("creating a scratch directory")?;
    let into = scratch.path().join("services");
    import_anthropic_tree(
        kit,
        &into,
        &ImportOptions {
            strict: true,
            dry_run: false,
        },
    )
    .map_err(|e| anyhow::anyhow!("strict re-import failed: {e}"))?;

    let mut diffs = Vec::new();
    let mut files = vec![format!("marketplaces/{}/config.yaml", report.marketplace)];
    files.extend(
        report
            .plugins
            .iter()
            .map(|p| format!("plugins/{p}/config.yaml")),
    );
    for skill in &report.skills {
        files.push(format!("skills/{skill}/config.yaml"));
        files.push(format!("skills/{skill}/SKILL.md"));
    }
    for rel in files {
        compare_file(services_root, &into, &rel, &mut diffs)?;
    }
    Ok(diffs)
}

fn compare_file(
    services_root: &Path,
    imported_root: &Path,
    rel: &str,
    out: &mut Vec<Diff>,
) -> Result<()> {
    let a = services_root.join(rel);
    let b = imported_root.join(rel);
    if !b.is_file() {
        out.push(Diff::Missing {
            file: rel.to_owned(),
        });
        return Ok(());
    }
    let a_text = std::fs::read_to_string(&a).with_context(|| a.display().to_string())?;
    let b_text = std::fs::read_to_string(&b).with_context(|| b.display().to_string())?;
    // Why: the kit's SKILL.md carries config.yaml merged into its frontmatter,
    // so the frontmatter is expected to differ; the instructions must not.
    if Path::new(rel)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        if body_of(&a_text) != body_of(&b_text) {
            out.push(Diff::Value {
                file: rel.to_owned(),
                path: "(body)".to_owned(),
                services: format!("{} bytes", a_text.len()),
                imported: format!("{} bytes", b_text.len()),
            });
        }
        return Ok(());
    }
    let a_doc: Value = serde_yaml::from_str(&a_text).with_context(|| a.display().to_string())?;
    let b_doc: Value = serde_yaml::from_str(&b_text).with_context(|| b.display().to_string())?;
    diff_values(rel, "", &a_doc, &b_doc, out);
    Ok(())
}

fn diff_values(file: &str, path: &str, services: &Value, imported: &Value, out: &mut Vec<Diff>) {
    match (services, imported) {
        (Value::Mapping(a), Value::Mapping(b)) => {
            for (k, av) in a {
                let key = k.as_str().map_or_else(|| format!("{k:?}"), str::to_owned);
                let sub = if path.is_empty() {
                    key
                } else {
                    format!("{path}.{key}")
                };
                match b.get(k) {
                    Some(bv) => diff_values(file, &sub, av, bv, out),
                    None if is_empty(av) || NOT_IN_KIT_FORM.contains(&sub.as_str()) => {},
                    None => out.push(Diff::Lost {
                        file: file.to_owned(),
                        path: sub,
                    }),
                }
            }
        },
        (a, b) if a == b => {},
        (_, b) if NOT_IN_KIT_FORM.contains(&path) && is_empty(b) => {},
        (a, b) => out.push(Diff::Value {
            file: file.to_owned(),
            path: path.to_owned(),
            services: render(a),
            imported: render(b),
        }),
    }
}

// Why: an empty list, map or null on the instance side is nothing to lose;
// the import writes the same declaration without spelling out the absence.
fn is_empty(v: &Value) -> bool {
    match v {
        Value::Null => true,
        Value::Sequence(s) => s.is_empty(),
        Value::Mapping(m) => m.is_empty(),
        Value::String(s) => s.is_empty(),
        _ => false,
    }
}

fn body_of(md: &str) -> &str {
    split_frontmatter(md).map_or(md, |f| f.body).trim()
}

fn render(v: &Value) -> String {
    serde_yaml::to_string(v).map_or_else(|_| format!("{v:?}"), |s| s.trim().replace('\n', " "))
}
