//! `/admin/models` — the Pi demo model-selection screen.
//!
//! One page that closes the demo loop: which models the gateway exposes,
//! whether the selected user may call each one (a `user`-band deny in
//! `access_control_rules` overrides the role allow on the next request),
//! and what that user has actually done — requests, tokens, cost, denials.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

mod data;
mod view;

use view::{ModelsPageData, build_kpis, build_user_options};

const BASE_URL: &str = "/admin/models";

#[derive(Debug, Deserialize)]
pub(crate) struct ModelsQuery {
    user_id: Option<systemprompt::identifiers::UserId>,
    q: Option<String>,
}

// Why: the catalogue is a fixed list read from YAML, so the search is a
// substring match over the columns a reader would scan for by eye — no
// repository query is involved.
fn matches_search(row: &view::ModelRowView, needle: &str) -> bool {
    [
        row.model_pattern.as_str(),
        row.provider.as_str(),
        row.route_id.as_str(),
        row.upstream_model.as_str(),
    ]
    .iter()
    .any(|field| field.to_lowercase().contains(needle))
}

pub(crate) async fn models_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<ModelsQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let all_users =
        repositories::users::queries::list_users(&pool, &repositories::scope::SubjectScope::All)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Failed to list users for model selection");
                vec![]
            });

    let selected_id: Option<String> = params
        .user_id
        .map(|u| u.to_string())
        .filter(|s| !s.is_empty());
    let users = build_user_options(&all_users, selected_id.as_deref());

    let selected_user_label = users
        .iter()
        .find(|u| u.selected)
        .map(|u| u.label.clone())
        .unwrap_or_default();
    let selected_user_id = selected_id.clone().unwrap_or_default();
    let has_selection = selected_id.is_some();

    let search = params
        .q
        .map(|q| q.trim().to_owned())
        .filter(|q| !q.is_empty())
        .unwrap_or_default();
    let needle = search.to_lowercase();
    let models: Vec<view::ModelRowView> = data::load_model_rows(&pool, selected_id.as_deref())
        .await?
        .into_iter()
        .filter(|row| needle.is_empty() || matches_search(row, &needle))
        .collect();
    let (usage, usage_totals) = data::load_usage(&pool, selected_id.as_deref()).await;

    let requests_link = if has_selection {
        format!(
            "/admin/requests?user_id={}",
            urlencoding::encode(&selected_user_id)
        )
    } else {
        "/admin/requests".to_owned()
    };

    let data = ModelsPageData {
        page: "models",
        title: "Models",
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Platform", "/admin/models"),
            BreadcrumbView::current("Models"),
        ],
        base_url: BASE_URL,
        kpis: build_kpis(&models, &usage_totals, has_selection),
        users,
        has_selection,
        selected_user_id,
        selected_user_label,
        model_count: models.len(),
        has_search: !search.is_empty(),
        search,
        models,
        usage_count: usage.len(),
        usage,
        has_usage: usage_totals.requests > 0,
        usage_totals,
        requests_link,
    };

    Ok(super::render_typed_page(
        &engine, "models", &data, &user_ctx, &mkt_ctx,
    ))
}
