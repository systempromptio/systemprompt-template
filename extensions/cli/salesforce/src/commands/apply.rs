//! The `apply` subcommand: make the target org match the spec, and print what
//! changed.
//!
//! Ordering around the metadata deploy is the substance of this module — see
//! the why-notes in [`run_apply`].

use std::path::Path;
use std::process::ExitCode;

use systemprompt_web_admin::repositories::users::salesforce_identity;
use systemprompt_web_admin::salesforce_org::apply::{self, ApplyReport};
use systemprompt_web_admin::salesforce_org::{Connection, OrgSpec, TargetOrg, diff, export};

use super::print_changes;

pub async fn run_apply(
    conn: &Connection,
    spec_path: &Path,
    target: &TargetOrg,
    dry_run: bool,
    extra_users: Vec<String>,
) -> Result<ExitCode, String> {
    let my_domain = &target.my_domain;
    let certificate = target.certificate_pem.as_deref();

    // Why: checked before anything runs, including in --dry-run. A deploy
    // without the certificate clears the app's digital signature and breaks the
    // grant this command authenticates with, so refusing early is the whole
    // point — failing at the deploy would leave the writes before it applied.
    apply::check_certificate_present(certificate).map_err(|e| e.to_string())?;

    let desired = OrgSpec::load(spec_path).map_err(|e| e.to_string())?;
    let actual = export::export_org(conn, Some(&desired))
        .await
        .map_err(|e| e.to_string())?;
    print_changes(&diff::diff(&actual, &desired));

    println!(
        "\n{} {my_domain}",
        if dry_run { "Validating" } else { "Applying" }
    );

    let mut report = ApplyReport::default();
    let (assignees, db_note) = collect_assignees(extra_users).await;
    if let Some(note) = db_note {
        report.manual_followups.push(note);
    }

    // Why: order is load-bearing, and it runs on BOTH sides of the deploy.
    //
    // Before: the deploy flips the app to AdminApprovedPreAuthorized, and from
    // that moment only holders of the permission set can authenticate. Nobody
    // may be mid-air when that lands.
    //
    // After: the deploy *destroys* the SetupEntityAccess grants — observed on a
    // live org, where a grant that existed before an apply was gone after it,
    // leaving every user with `invalid_app_access: user is not admin approved`.
    // Re-asserting afterwards repairs that. Both calls are idempotent, so the
    // second is a no-op whenever the deploy leaves the grants alone.
    if dry_run {
        println!("  permission sets, grants and assignments: skipped (dry run)");
        println!("  hosted MCP servers: skipped (dry run)");
    } else {
        apply::apply_permission_sets(conn, &desired, &mut report)
            .await
            .map_err(|e| e.to_string())?;
        apply::apply_assignments(conn, &desired, &assignees, &mut report)
            .await
            .map_err(|e| e.to_string())?;
        report_permission_sets(&report);
        apply::apply_hosted_mcp_servers(conn, &desired, &mut report)
            .await
            .map_err(|e| e.to_string())?;
        report_servers(&report);
    }

    let deploy = apply::apply_metadata(conn, &desired, certificate, dry_run)
        .await
        .map_err(|e| e.to_string())?;
    let failed = !deploy.success;
    if failed {
        println!("  metadata deploy {}: FAILED", deploy.id);
        for line in deploy.failure_lines() {
            println!("    {line}");
        }
        if let Some(message) = &deploy.error_message {
            println!("    {message}");
        }
    } else {
        println!("  metadata deploy {}: {}", deploy.id, deploy.status);
    }
    report.deploy = Some(deploy);

    // Why: see the ordering note above — the deploy drops SetupEntityAccess
    // grants, so they are re-created here. Skipped when the deploy failed,
    // because a rolled-back deploy has not changed the policy either.
    if !dry_run && !failed {
        repair_after_deploy(conn, &desired, &assignees, &mut report).await?;
    }

    if !report.manual_followups.is_empty() {
        println!("\nNeeds a human:");
        for note in &report.manual_followups {
            println!("  - {note}");
        }
    }
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

async fn repair_after_deploy(
    conn: &Connection,
    desired: &OrgSpec,
    assignees: &[String],
    report: &mut ApplyReport,
) -> Result<(), String> {
    let mut repair = ApplyReport::default();
    apply::apply_permission_sets(conn, desired, &mut repair)
        .await
        .map_err(|e| e.to_string())?;
    apply::apply_assignments(conn, desired, assignees, &mut repair)
        .await
        .map_err(|e| e.to_string())?;
    if repair.app_grants_created.is_empty() && repair.assignments_created.is_empty() {
        println!("  post-deploy check: grants and assignments survived the deploy");
    } else {
        for grant in &repair.app_grants_created {
            println!("  re-created app grant the deploy dropped: {grant}");
        }
        for assignment in &repair.assignments_created {
            println!("  re-created assignment the deploy dropped: {assignment}");
        }
    }
    report.manual_followups.extend(repair.manual_followups);
    Ok(())
}

// Why: an unreachable database degrades to a note rather than failing the
// apply. The metadata half is independently useful, and refusing to configure
// an org because a Postgres container is down would be the wrong trade.
async fn collect_assignees(extra: Vec<String>) -> (Vec<String>, Option<String>) {
    match load_db_usernames().await {
        Ok(names) => (merge_assignees(names, extra), None),
        Err(e) => (
            merge_assignees(Vec::new(), extra),
            Some(db_unreachable_note(&e)),
        ),
    }
}

#[doc(hidden)]
pub fn merge_assignees(mut from_db: Vec<String>, extra: Vec<String>) -> Vec<String> {
    from_db.extend(extra);
    from_db.sort();
    from_db.dedup();
    from_db
}

#[doc(hidden)]
pub fn db_unreachable_note(error: &str) -> String {
    format!(
        "could not read Salesforce usernames from the platform database ({error}). \
         Only the --user values were assigned; re-run this apply once the \
         database is reachable to assign everyone else."
    )
}

async fn load_db_usernames() -> Result<Vec<String>, String> {
    use systemprompt::config::{ProfileBootstrap, SecretsBootstrap, init_config};
    use systemprompt::system::AppContext;

    ProfileBootstrap::init().map_err(super::stage_err("profile"))?;
    SecretsBootstrap::init().map_err(super::stage_err("secrets"))?;
    init_config().map_err(super::stage_err("config"))?;
    let ctx = AppContext::new()
        .await
        .map_err(super::stage_err("app context"))?;
    let pool = ctx
        .db_pool()
        .write_pool_arc()
        .map_err(super::stage_err("write pool"))?;
    salesforce_identity::list_salesforce_usernames(&pool)
        .await
        .map_err(|e| e.to_string())
}

fn report_servers(report: &ApplyReport) {
    for name in &report.servers_activated {
        println!("  activated hosted MCP server {name}");
    }
    if report.servers_activated.is_empty() {
        println!("  hosted MCP servers: already active");
    }
}

fn report_permission_sets(report: &ApplyReport) {
    for name in &report.permission_sets_created {
        println!("  created permission set {name}");
    }
    for grant in &report.app_grants_created {
        println!("  granted app access {grant}");
    }
    for assignment in &report.assignments_created {
        println!("  assigned {assignment}");
    }
    if report.permission_sets_created.is_empty()
        && report.app_grants_created.is_empty()
        && report.assignments_created.is_empty()
    {
        println!("  permission sets, grants and assignments: already correct");
    }
}
