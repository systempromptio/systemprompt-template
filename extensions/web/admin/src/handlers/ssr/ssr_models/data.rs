//! Data loading for the Model Selection page: gateway routes joined with
//! per-user access rules, and the selected user's `ai_requests` usage.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt_security::authz::{Access, AccessControlRepository, EntityKind, RuleType};

use crate::repositories;
use crate::repositories::analytics::requests::{
    RequestFilter, RequestPage, RequestSortSpec, list_requests_paged,
};
use crate::util::time_range::{TimeRangeQuery, parse_time_range};

use super::view::{ModelRowView, UsageRowView, UsageTotalsView};
use crate::handlers::ssr::format::format_cost;

const USAGE_ROWS: i64 = 25;

pub(super) async fn load_model_rows(
    pool: &PgPool,
    selected_user: Option<&str>,
) -> Result<Vec<ModelRowView>, crate::error::AdminHtmlError> {
    let cfg = repositories::config::gateway::get_gateway_config()
        .map_err(|e| crate::error::AdminHtmlError::internal(e.to_string()))?;

    let repo = AccessControlRepository::from_pool(Arc::new(pool.clone()));
    let mut rows = Vec::with_capacity(cfg.routes.len());
    for route in cfg.routes {
        let deny_rule = match selected_user {
            None => None,
            Some(uid) => repo
                .list_rules_for_entity(EntityKind::GatewayRoute, &route.id)
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!(route = %route.id, error = %e, "Failed to list route rules");
                    vec![]
                })
                .into_iter()
                .find(|r| {
                    r.rule_type == RuleType::USER && r.rule_value == uid && r.access == Access::Deny
                }),
        };
        let denied = deny_rule.is_some();
        rows.push(ModelRowView {
            upstream_model: route
                .upstream_model
                .clone()
                .unwrap_or_else(|| route.model_pattern.clone()),
            model_pattern: route.model_pattern,
            provider: route.provider,
            deny_rule_id: deny_rule.map(|r| r.id.to_string()).unwrap_or_default(),
            denied,
            status_label: if denied { "Disabled" } else { "Enabled" },
            route_id: route.id,
        });
    }
    Ok(rows)
}

pub(super) async fn load_usage(
    pool: &PgPool,
    selected_user: Option<&str>,
) -> (Vec<UsageRowView>, UsageTotalsView) {
    let empty_totals = UsageTotalsView {
        requests: 0,
        input_tokens: 0,
        output_tokens: 0,
        cost: "$0.00".to_owned(),
        denied_requests: 0,
    };
    let Some(uid) = selected_user else {
        return (vec![], empty_totals);
    };

    let filter = RequestFilter {
        user_id: Some(systemprompt::identifiers::UserId::new(uid.to_owned())),
        ..RequestFilter::default()
    };
    let range = parse_time_range(&TimeRangeQuery::default());
    let page = RequestPage {
        sort: RequestSortSpec::default(),
        limit: USAGE_ROWS,
        offset: 0,
    };

    let (rows, total) = list_requests_paged(pool, &filter, range, page)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(user_id = %uid, error = %e, "Failed to load user usage");
            (vec![], 0)
        });

    let mut input_tokens = 0i64;
    let mut output_tokens = 0i64;
    let mut cost_micro = 0i64;
    let mut denied_requests = 0i64;
    let usage: Vec<UsageRowView> = rows
        .into_iter()
        .map(|r| {
            input_tokens += i64::from(r.input_tokens.unwrap_or(0));
            output_tokens += i64::from(r.output_tokens.unwrap_or(0));
            cost_micro += r.cost_microdollars;
            if r.status != "completed" {
                denied_requests += 1;
            }
            UsageRowView {
                request_id: r.request_id.to_string(),
                created_at: r.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                model: r.model,
                provider: r.provider,
                is_completed: r.status == "completed",
                status: r.status,
                tokens: format!(
                    "{}/{}",
                    r.input_tokens.unwrap_or(0),
                    r.output_tokens.unwrap_or(0)
                ),
                cost: format_cost(r.cost_microdollars),
                latency_ms: i64::from(r.latency_ms.unwrap_or(0)),
                deny_count: r.deny_count,
            }
        })
        .collect();

    let totals = UsageTotalsView {
        requests: total,
        input_tokens,
        output_tokens,
        cost: format_cost(cost_micro),
        denied_requests,
    };
    (usage, totals)
}
