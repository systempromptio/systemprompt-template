//! SSR page for instance settings.

use std::sync::Arc;

use crate::error::AdminHtmlResult;
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::extract::{Extension, State};
use axum::response::Response;
use sqlx::PgPool;

use super::types::{SettingsPageData, SettingsView};

pub(crate) async fn settings_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    // Why this one propagates rather than degrading: the form below is bound to
    // these values and `collectFormData()` reads every field back out and PUTs
    // the whole object. Rendering the struct defaults would tell the user their
    // settings had been reset, and the natural response — setting them again —
    // writes those defaults over the real row. It is also the only query on the
    // page, so there is nothing else on screen to look wrong and cue doubt, and
    // nothing is lost by failing.
    let settings =
        repositories::users::user_settings::find_user_settings(&pool, &user_ctx.user_id).await?;

    let settings_view = settings.as_ref().map_or_else(
        || SettingsView {
            display_name: None,
            avatar_url: None,
            notify_daily_summary: true,
            notify_achievements: true,
            leaderboard_opt_in: true,
            timezone: "UTC".to_owned(),
        },
        |s| SettingsView {
            display_name: s.display_name.clone(),
            avatar_url: s.avatar_url.clone(),
            notify_daily_summary: s.notify_daily_summary,
            notify_achievements: s.notify_achievements,
            leaderboard_opt_in: s.leaderboard_opt_in,
            timezone: s.timezone.clone(),
        },
    );

    let data = SettingsPageData {
        page: "settings",
        title: "Account Settings",
        settings: settings_view,
        user_email: user_ctx.email.to_string(),
        user_id: user_ctx.user_id.clone(),
        username: user_ctx.username.clone(),
    };

    let value = serde_json::to_value(&data).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to serialize settings page data");
        serde_json::Value::Null
    });
    Ok(super::render_page(
        &engine, "settings", &value, &user_ctx, &mkt_ctx,
    ))
}
