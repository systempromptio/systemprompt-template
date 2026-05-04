use crate::handlers::user_entities::fork_helpers::fork_single_plugin;
use crate::repositories;
use crate::types::{PluginOverview, UserSummary};
use sqlx::PgPool;
use std::path::Path;
use systemprompt::identifiers::UserId;

#[derive(Debug, Default, Clone, Copy)]
pub struct AutoforkReport {
    pub users_considered: u64,
    pub plugins_forked: u64,
    pub plugins_skipped_already_bound: u64,
    pub plugins_failed: u64,
}

pub async fn autofork_declared_plugins_for_admins(
    pool: &PgPool,
    services_path: &Path,
) -> AutoforkReport {
    let mut report = AutoforkReport::default();

    let users = match repositories::list_users(pool).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!(error = %e, "autofork: list_users failed");
            return report;
        }
    };

    for user in users {
        if !user.roles.iter().any(|r| r == "admin") {
            continue;
        }
        report.users_considered += 1;
        autofork_user(pool, services_path, &user, &mut report).await;
    }

    report
}

async fn autofork_user(
    pool: &PgPool,
    services_path: &Path,
    user: &UserSummary,
    report: &mut AutoforkReport,
) {
    let overviews = match repositories::list_plugins_for_roles(services_path, &user.roles) {
        Ok(o) => o,
        Err(e) => {
            tracing::warn!(error = %e, user_id = %user.user_id.as_str(),
                "autofork: list_plugins_for_roles failed");
            return;
        }
    };

    let display = user
        .display_name
        .as_deref()
        .unwrap_or_else(|| user.user_id.as_str());

    for overview in overviews {
        autofork_user_plugin(
            pool,
            services_path,
            &user.user_id,
            display,
            &overview,
            report,
        )
        .await;
    }
}

async fn autofork_user_plugin(
    pool: &PgPool,
    services_path: &Path,
    user_id: &UserId,
    display: &str,
    overview: &PluginOverview,
    report: &mut AutoforkReport,
) {
    match repositories::find_user_plugin(pool, user_id, &overview.id).await {
        Ok(Some(_)) => {
            report.plugins_skipped_already_bound += 1;
            return;
        }
        Ok(None) => {}
        Err(e) => {
            tracing::warn!(error = %e, user_id = %user_id.as_str(),
                plugin = %overview.id, "autofork: find_user_plugin failed");
            report.plugins_failed += 1;
            return;
        }
    }

    match fork_single_plugin(pool, user_id, display, overview, services_path, None).await {
        Ok(result) => {
            report.plugins_forked += 1;
            tracing::info!(
                user_id = %user_id.as_str(),
                plugin = %overview.id,
                skills = result.forked_skills,
                agents = result.forked_agents,
                "autofork: bound declared plugin to admin user"
            );
        }
        Err(e) => {
            report.plugins_failed += 1;
            tracing::warn!(error = %e, user_id = %user_id.as_str(),
                plugin = %overview.id, "autofork: fork_single_plugin failed");
        }
    }
}
