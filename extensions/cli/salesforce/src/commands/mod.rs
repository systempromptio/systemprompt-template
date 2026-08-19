//! Subcommand execution: connect to the target org, then dispatch.
//!
//! `export` and `diff` are read-only and live here; `apply` is the write path
//! and lives in [`apply`].

pub mod apply;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use systemprompt_web_admin::salesforce_org::diff::{self, ChangeKind};
use systemprompt_web_admin::salesforce_org::{Connection, OrgSpec, TargetOrg, export};

use crate::cli::{Cli, Command};

pub(crate) fn stage_err<E: std::fmt::Display>(stage: impl Into<String>) -> impl Fn(E) -> String {
    let stage = stage.into();
    move |e| format!("{stage}: {e}")
}

pub async fn run(cli: Cli) -> Result<ExitCode, String> {
    let spec_path = PathBuf::from(&cli.spec);
    let target = TargetOrg::from_env().map_err(|e| e.to_string())?;
    let auth_stage = format!("could not authenticate to {}", target.my_domain);
    let conn = Connection::connect(&target)
        .await
        .map_err(stage_err(auth_stage))?;

    match cli.command {
        Command::Export { out } => run_export(&conn, &spec_path, out).await,
        Command::Diff { exit_code } => run_diff(&conn, &spec_path, exit_code).await,
        Command::Apply { dry_run, users } => {
            apply::run_apply(&conn, &spec_path, &target, dry_run, users).await
        },
    }
}

async fn run_export(
    conn: &Connection,
    spec_path: &Path,
    out: Option<PathBuf>,
) -> Result<ExitCode, String> {
    // Why: the committed spec supplies the fields no API can read back.
    let baseline = OrgSpec::load(spec_path).ok();
    if baseline.is_none() {
        eprintln!(
            "note: no spec at {}; write-only fields will be placeholders",
            spec_path.display()
        );
    }
    let exported = export::export_org(conn, baseline.as_ref())
        .await
        .map_err(|e| e.to_string())?;
    let yaml = exported.to_yaml().map_err(|e| e.to_string())?;
    match out {
        Some(path) => {
            let write_stage = format!("could not write {}", path.display());
            std::fs::write(&path, yaml).map_err(stage_err(write_stage))?;
            eprintln!("wrote {}", path.display());
        },
        None => println!("{yaml}"),
    }
    Ok(ExitCode::SUCCESS)
}

async fn run_diff(
    conn: &Connection,
    spec_path: &Path,
    exit_code: bool,
) -> Result<ExitCode, String> {
    let desired = OrgSpec::load(spec_path).map_err(|e| e.to_string())?;
    let actual = export::export_org(conn, Some(&desired))
        .await
        .map_err(|e| e.to_string())?;
    let changes = diff::diff(&actual, &desired);
    print_changes(&changes);
    Ok(if exit_code && !changes.is_clean() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

pub fn print_changes(changes: &diff::ChangeSet) {
    let drift = changes.drift();
    if drift.is_empty() {
        println!("No drift: the org matches the spec on every readable field.");
    } else {
        println!("Drift ({}):", drift.len());
        for change in drift {
            println!("{change}");
        }
    }

    let always: Vec<_> = changes
        .changes
        .iter()
        .filter(|c| c.kind == ChangeKind::AlwaysApplied)
        .collect();
    if !always.is_empty() {
        println!("\nAlways applied (not readable from any API, so never compared):");
        for change in always {
            println!("{change}");
        }
    }
}
