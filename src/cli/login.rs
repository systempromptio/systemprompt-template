use crate::cli::config::get_global_config;
use anyhow::Result;
use clap::Args;
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;
use systemprompt_core_logging::CliService;
use systemprompt_core_oauth::services::generation::tokens::{
    generate_access_token_jti, generate_jwt, JwtConfig,
};
use systemprompt_core_system::AuthenticatedUser;
use systemprompt_core_system::{AppContext, Config};
use systemprompt_core_users::UserRepository;
use systemprompt_identifiers::SessionId;
use systemprompt_models::auth::{JwtAudience, Permission};
use uuid::Uuid;

#[derive(Args)]
pub struct LoginArgs {
    /// User role (default: admin)
    #[arg(default_value = "admin")]
    role: String,

    /// Token expiry in hours (default: 24)
    #[arg(long)]
    expiry: Option<i64>,
}

pub async fn execute(args: LoginArgs) -> Result<()> {
    dotenv::dotenv().ok();

    let config = Config::global();
    let jwt_secret = &config.jwt_secret;

    let app_context = Arc::new(AppContext::new().await?);
    let user_repo = UserRepository::new(app_context.db_pool().clone());

    let user = user_repo.find_by_role(&args.role).await?.ok_or_else(|| {
        anyhow::anyhow!(
            "No user found with role '{}'. Create a user with this role first.",
            args.role
        )
    })?;

    let user_uuid = Uuid::parse_str(&user.uuid)?;
    let permissions: Vec<Permission> = user
        .roles
        .iter()
        .filter_map(|s| Permission::from_str(s).ok())
        .collect();

    let authenticated_user =
        AuthenticatedUser::new(user_uuid, user.name, Some(user.email), permissions.clone());

    let jwt_config = JwtConfig {
        permissions: permissions.clone(),
        audience: JwtAudience::standard(),
        expires_in_hours: args.expiry.or(Some(24)),
    };

    let session_id = SessionId::new(format!("sess_{}", uuid::Uuid::new_v4().simple()));
    let token = generate_jwt(
        &authenticated_user,
        jwt_config.clone(),
        generate_access_token_jti(),
        &session_id,
        jwt_secret,
    )?;

    let config = get_global_config();
    if config.is_json_output() {
        let scopes_strings: Vec<String> = permissions.iter().map(|s| s.to_string()).collect();
        let audience_strings: Vec<String> =
            jwt_config.audience.iter().map(|a| a.to_string()).collect();

        CliService::json(&json!({
            "token": token,
            "user_id": authenticated_user.id.to_string(),
            "role": args.role,
            "scopes": scopes_strings,
            "audience": audience_strings,
            "expires_in_hours": jwt_config.expires_in_hours
        }));
    } else {
        println!("{}", token);
    }

    Ok(())
}
