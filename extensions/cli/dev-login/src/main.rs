//! `systemprompt-dev-login` — prints a one-click admin-console login link for
//! the user named on the command line.
//!
//! Spawned by `systemprompt plugins run dev-login <user>` (or `just dev-login
//! <user>`), which hands it `SYSTEMPROMPT_PROFILE`. It refuses to issue
//! anything unless that profile is development and non-cloud, the same gate
//! that decides whether the redeem route is mounted at all. The code is
//! single-use and expires after ten minutes; only its hash is stored.

use anyhow::{Context, Result, bail, ensure};
use clap::Parser;
use systemprompt::config::{ProfileBootstrap, SecretsBootstrap, init_config};
use systemprompt::logging::CliService;
use systemprompt::system::AppContext;
use systemprompt_web_admin::repositories::dev_login::{
    DEV_LOGIN_CODE_TTL_SECONDS, find_active_user_id_by_login, insert_dev_login_code,
};
use systemprompt_web_admin::{dev_login_allowed, dev_login_url};

#[derive(Debug, Parser)]
#[command(
    name = "systemprompt-dev-login",
    about = "Print a single-use /admin login link for a user (development profiles only)"
)]
struct Cli {
    #[arg(help = "E-mail (or username) of the active account to sign in")]
    user: String,
    #[arg(
        long,
        hide = true,
        help = "Accepted for `plugins run` parity; the output is the URL either way"
    )]
    json: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let profile = ProfileBootstrap::init()
        .context("SYSTEMPROMPT_PROFILE must name a profile.yaml (run via `systemprompt plugins run dev-login`)")?;
    ensure!(
        dev_login_allowed(profile.runtime.environment, profile.target),
        "dev login is only available on a development, non-cloud profile (this one is {:?} / {:?})",
        profile.runtime.environment,
        profile.target
    );
    SecretsBootstrap::init().context("Failed to initialize secrets")?;
    init_config().context("Failed to initialize configuration")?;

    let ctx = AppContext::new()
        .await
        .context("Failed to initialize application context")?;
    let pool = ctx
        .db_pool()
        .write_pool_arc()
        .context("dev login needs a Postgres pool")?;

    let Some(user_id) = find_active_user_id_by_login(&pool, &cli.user).await? else {
        bail!("no active user matches '{}'", cli.user);
    };
    let issued = insert_dev_login_code(&pool, &user_id).await?;

    let url = dev_login_url(&profile.server.api_external_url, &issued.code);
    CliService::info(&format!(
        "Single-use link for {} (valid {} minutes, expires {}):",
        cli.user,
        DEV_LOGIN_CODE_TTL_SECONDS / 60,
        issued.expires_at.format("%H:%M:%S UTC")
    ));
    CliService::output(&url);
    Ok(())
}
