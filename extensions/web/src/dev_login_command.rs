//! `dev-login issue <user>`: print a single-use, ten-minute sign-in link for an
//! active account. Refused unless the web member contributed the `dev` SSO
//! provider, which it does only on a development, non-cloud profile whose
//! services tree requests it.

use std::sync::Arc;

use clap::Subcommand;
use systemprompt::database::DbPool;
use systemprompt::extension::cli::{
    CliCommandGroup, CommandDescriptor, CommandError, CommandHost, CommandOutput, DataImpact,
    DescribeCommand,
};
use systemprompt_web_admin::repositories::dev_login::{
    DEV_LOGIN_CODE_TTL_SECONDS, find_active_user_id_by_login, insert_dev_login_code,
};

use crate::WebSettings;
use crate::sign_in::SignInSetup;

#[derive(Debug, Subcommand)]
pub(crate) enum DevLoginCommands {
    #[command(about = "Print a single-use sign-in link for an active account")]
    Issue {
        #[arg(help = "E-mail (or username) of the active account to sign in")]
        user: String,
    },
}

impl DescribeCommand for DevLoginCommands {
    fn descriptor(&self) -> CommandDescriptor {
        CommandDescriptor::FULL.with_data_impact(DataImpact::Destructive)
    }
}

#[derive(Debug, thiserror::Error)]
enum DevLoginError {
    #[error("the development login is not enabled on this profile")]
    Disabled,
    #[error("no active user matches '{0}'")]
    NoUser(String),
}

impl CliCommandGroup for DevLoginCommands {
    const NAME: &'static str = "dev-login";
    const ABOUT: &'static str = "Development sign-in links (development, non-cloud profiles only)";

    async fn run(self, host: &dyn CommandHost) -> Result<CommandOutput, CommandError> {
        let Self::Issue { user } = self;
        let resolved = host.resolved().await.map_err(CommandError::new)?;
        let setup = resolved
            .get::<Arc<SignInSetup>>()
            .map_err(CommandError::new)?;
        if !setup.options().dev {
            return Err(CommandError::new(DevLoginError::Disabled));
        }
        let settings = resolved
            .get::<Arc<WebSettings>>()
            .map_err(CommandError::new)?;
        let pool = resolved
            .get::<DbPool>()
            .map_err(CommandError::new)?
            .write_pool();
        let user_id = find_active_user_id_by_login(&pool, &user)
            .await
            .map_err(CommandError::new)?
            .ok_or_else(|| CommandError::new(DevLoginError::NoUser(user.clone())))?;
        let issued = insert_dev_login_code(&pool, &user_id)
            .await
            .map_err(CommandError::new)?;
        let url = systemprompt_web_admin::dev_login_url(&settings.api_external_url, &issued.code);
        Ok(CommandOutput::text(format!(
            "Single-use link for {user} (valid {} minutes, expires {}):\n{url}",
            DEV_LOGIN_CODE_TTL_SECONDS / 60,
            issued.expires_at.format("%H:%M:%S UTC")
        )))
    }
}
