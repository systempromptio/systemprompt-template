//! `/admin/access-control` — the rule ledger.
//!
//! Every access-control rule on this instance, and whether the YAML in the
//! source repository declares it. Editing lives with the subject: a group's
//! band on its Access tab, a person's overrides on theirs. Reading here is
//! the CONSOLE tier so a project manager can audit; the two dialogs that
//! write (YAML export, new group) are gated on MANAGE.

mod data;
pub(crate) mod rules;
pub(crate) mod rules_controls;
mod view;

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::shared;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use rules_controls::RulesQuery;
use view::AccessControlPageData;

pub(crate) async fn access_control_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<RulesQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    if let Some(user) = query.user.as_deref().filter(|u| !u.is_empty()) {
        let target = format!("/admin/users/{}?tab=access", urlencoding::encode(user));
        return Ok(Redirect::permanent(&target).into_response());
    }
    let services_path = shared::get_services_path()?;

    let stats = data::load_stats(&pool).await;

    let ledger_rows = crate::repositories::access_control::rules::list_ledger_rules(&pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "access-control: rule listing failed"))
        .unwrap_or_default();
    let open_entities = crate::repositories::access_control::rules::count_open_entities(&pool)
        .await
        .unwrap_or_default();
    let declared =
        crate::repositories::access_control::yaml_declared::load_declared_rules(&services_path);
    let capped = i64::try_from(ledger_rows.len()).unwrap_or(i64::MAX)
        >= crate::repositories::access_control::rules::RULE_CAP;
    let ledger = rules::build(&ledger_rows, &declared, open_entities, &query, capped);

    let page = AccessControlPageData {
        page: "access-control",
        title: "Access control",
        can_write: user_ctx.is_admin,
        stats,
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("People & access", "/admin/users"),
            BreadcrumbView::current("Access control"),
        ],
        ledger,
    };

    Ok(super::render_typed_page(
        &engine,
        "access-control",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}
