//! Subcommand execution: bootstrap the platform database, then dispatch.
//!
//! Every command needs the database and nothing else — there is no external
//! service here. `grant` refuses a user who does not hold the `admin` role
//! (membership without the role gates nothing, and granting it here would
//! turn a membership tool into a role-escalation tool); `revoke` refuses to
//! remove the last platform admin without `--force`.

use std::process::ExitCode;

use systemprompt::identifiers::UserId;
use systemprompt_web_admin::repositories::organizations::crud as org_repo;
use systemprompt_web_admin::repositories::users::queries as user_repo;

use crate::cli::{Cli, Command};

const ADMIN_ROLE: &str = "admin";

fn stage_err<E: std::fmt::Display>(stage: &'static str) -> impl Fn(E) -> String {
    move |e| format!("{stage}: {e}")
}

pub async fn run(cli: Cli) -> Result<ExitCode, String> {
    let pool = connect().await?;
    match cli.command {
        Command::Grant { user } => run_grant(&pool, &user).await,
        Command::Revoke { user, force } => run_revoke(&pool, &user, force).await,
        Command::Status => run_status(&pool).await,
    }
}

async fn connect() -> Result<std::sync::Arc<sqlx::PgPool>, String> {
    use systemprompt::config::{ProfileBootstrap, SecretsBootstrap, init_config};
    use systemprompt::system::AppContext;

    ProfileBootstrap::init().map_err(stage_err("profile"))?;
    SecretsBootstrap::init().map_err(stage_err("secrets"))?;
    init_config().map_err(stage_err("config"))?;
    let ctx = AppContext::new().await.map_err(stage_err("app context"))?;
    ctx.db_pool()
        .write_pool_arc()
        .map_err(stage_err("write pool"))
}

async fn run_grant(pool: &sqlx::PgPool, user: &str) -> Result<ExitCode, String> {
    let (org_id, org_name) = platform_org(pool).await?;
    let (user_id, roles) = named_user(pool, user).await?;

    if !roles.iter().any(|r| r == ADMIN_ROLE) {
        return Err(format!(
            "{user} does not hold the `{ADMIN_ROLE}` role, so platform membership would gate \
             nothing. Grant the role first: systemprompt admin users role promote {user}"
        ));
    }

    match org_repo::find_membership_org(pool, &user_id)
        .await
        .map_err(|e| e.to_string())?
    {
        Some(existing) if existing == org_id => {
            println!("{user} is already a member of the platform organization ({org_name})");
            Ok(ExitCode::SUCCESS)
        },
        Some(existing) => Err(format!(
            "{user} belongs to organization `{existing}` and a user has exactly one \
             organization. Membership is the billing and authorization boundary, so this \
             command will not move them — detach them from `{existing}` first (Admin UI, or \
             the organization owner), then re-run the grant."
        )),
        None => {
            org_repo::insert_membership_if_absent(pool, &user_id, &org_id, ADMIN_ROLE)
                .await
                .map_err(|e| e.to_string())?;
            println!(
                "{user} joined the platform organization ({org_name}) as org admin. They must \
                 sign out and back in before the Admin UI reflects it."
            );
            Ok(ExitCode::SUCCESS)
        },
    }
}

async fn run_revoke(pool: &sqlx::PgPool, user: &str, force: bool) -> Result<ExitCode, String> {
    let (_, org_name) = platform_org(pool).await?;
    let (user_id, _) = named_user(pool, user).await?;

    let members = org_repo::count_platform_members(pool)
        .await
        .map_err(|e| e.to_string())?;
    if members <= 1 && !force {
        return Err(format!(
            "{user} is the last platform member; removing them leaves nobody able to assign \
             elevated roles from the Admin UI. Pass --force to do it anyway."
        ));
    }

    let removed = org_repo::delete_platform_membership(pool, &user_id)
        .await
        .map_err(|e| e.to_string())?;
    if removed {
        println!("{user} removed from the platform organization ({org_name})");
    } else {
        println!("{user} was not a member of the platform organization; nothing to do");
    }
    Ok(ExitCode::SUCCESS)
}

async fn run_status(pool: &sqlx::PgPool) -> Result<ExitCode, String> {
    let (org_id, org_name) = platform_org(pool).await?;
    let members = org_repo::list_members(pool, &org_id)
        .await
        .map_err(|e| e.to_string())?;

    println!("platform organization: {org_name} ({org_id})");
    if members.is_empty() {
        println!(
            "no members — no platform admin exists. Restart the server (the \
             platform_admin_bootstrap job grants the configured system admin) or run: \
             systemprompt plugins run platform grant <user>"
        );
        return Ok(ExitCode::from(1));
    }
    println!("members:");
    for m in members {
        let active = if m.is_active { "active" } else { "inactive" };
        println!("  {}  {}  {}  {}", m.user_id, m.email, m.org_role, active);
    }
    Ok(ExitCode::SUCCESS)
}

async fn platform_org(pool: &sqlx::PgPool) -> Result<(String, String), String> {
    org_repo::find_platform_organization(pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| {
            "no is_platform organization exists — the house-organization seed has not run; \
             start the server once to apply schema and seeds"
                .to_owned()
        })
}

async fn named_user(pool: &sqlx::PgPool, name: &str) -> Result<(UserId, Vec<String>), String> {
    user_repo::find_user_id_and_roles_by_name(pool, name)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("no user named `{name}` exists"))
}
