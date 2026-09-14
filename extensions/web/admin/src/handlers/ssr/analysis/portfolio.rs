//! Catalog-wide resource effectiveness using shared core metric definitions.

use axum::extract::{Query, State};
use axum::response::Response;
use axum::{Extension, Json};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::analytics::resource_metrics::{ResourceMetrics, aggregate};

use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PortfolioQuery {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub kind: Option<String>,
    pub search: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct ResourceRow {
    id: String,
    name: String,
    enabled: bool,
    source: String,
    href: String,
    metrics: ResourceMetrics,
    cost: Option<String>,
    average_tokens: Option<String>,
    quality: Option<String>,
    latency: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct PortfolioPage {
    page: &'static str,
    title: &'static str,
    kind: String,
    rows: Vec<ResourceRow>,
    start: String,
    end: String,
    search: String,
    generated_at: String,
    totals: ResourceMetrics,
    truncated: bool,
    membership_associated: bool,
}

pub(crate) async fn page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<PortfolioQuery>,
) -> AdminHtmlResult<Response> {
    let data = load(&pool, &user, query).await?;
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-portfolio",
        &data,
        &user,
        &marketplace,
    ))
}

pub(crate) async fn json(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<PortfolioQuery>,
) -> AdminResult<Json<PortfolioPage>> {
    Ok(Json(load(&pool, &user, query).await?))
}

async fn load(
    pool: &PgPool,
    user: &UserContext,
    query: PortfolioQuery,
) -> AdminResult<PortfolioPage> {
    if !user.is_admin {
        return Err(AdminError::Forbidden(
            "Administrator access required for organizational resource analysis".to_owned(),
        ));
    }
    let end = query.end.unwrap_or_else(Utc::now);
    let start = query.start.unwrap_or(end - Duration::days(30));
    let kind = query.kind.unwrap_or_else(|| "skills".to_owned());
    let search = query.search.unwrap_or_default();
    if start >= end
        || end - start > Duration::days(366)
        || !matches!(kind.as_str(), "skills" | "plugins" | "marketplaces")
        || search.len() > 200
    {
        return Err(AdminError::BadRequest(
            "Invalid resource analysis filter".to_owned(),
        ));
    }
    let root = crate::handlers::shared::get_services_path()?;
    let facts = repositories::analysis::portfolio::list_portfolio_facts(pool, start, end).await?;
    if facts.len() > 100_000 {
        return Err(AdminError::BadRequest(
            "This window exceeds the detailed analysis limit; choose a shorter window".to_owned(),
        ));
    }
    let mut rows = catalog_rows(&root, &kind, &search, &facts)?;
    rows.sort_by(|left, right| {
        right
            .metrics
            .invocations
            .cmp(&left.metrics.invocations)
            .then(left.id.cmp(&right.id))
    });
    Ok(PortfolioPage {
        page: "analysis-portfolio",
        title: "Resource effectiveness",
        membership_associated: kind == "marketplaces",
        kind,
        rows,
        start: start.to_rfc3339(),
        end: end.to_rfc3339(),
        search,
        generated_at: Utc::now().to_rfc3339(),
        totals: aggregate(facts.iter().map(|fact| &fact.fact)),
        truncated: false,
    })
}

fn catalog_rows(
    root: &std::path::Path,
    kind: &str,
    search: &str,
    facts: &[repositories::analysis::portfolio::PortfolioFact],
) -> AdminResult<Vec<ResourceRow>> {
    let skills = repositories::marketplace::plugins::list_skill_catalog(root)?;
    let plugins = repositories::marketplace::plugins::list_plugin_catalog(root)?;
    let markets = repositories::marketplace::manifests::list_marketplace_configs(root)?;
    let mut rows = Vec::new();
    let matches = |id: &str, name: &str| {
        search.is_empty()
            || id.to_lowercase().contains(&search.to_lowercase())
            || name.to_lowercase().contains(&search.to_lowercase())
    };
    match kind {
        "skills" => {
            for skill in skills {
                if matches(skill.id.as_str(), &skill.name) {
                    let metrics = aggregate(
                        facts
                            .iter()
                            .filter(|fact| fact.skill == skill.id.as_str())
                            .map(|fact| &fact.fact),
                    );
                    rows.push(row(
                        (skill.id.as_str(), &skill.name),
                        skill.enabled,
                        skill.source_path,
                        kind,
                        metrics,
                    ));
                }
            }
        },
        "plugins" => {
            for plugin in plugins {
                if matches(plugin.id.as_str(), &plugin.name) {
                    let metrics = aggregate(
                        facts
                            .iter()
                            .filter(|fact| fact.plugin == plugin.id.as_str())
                            .map(|fact| &fact.fact),
                    );
                    rows.push(row(
                        (plugin.id.as_str(), &plugin.name),
                        plugin.enabled,
                        plugin.source_path,
                        kind,
                        metrics,
                    ));
                }
            }
        },
        _ => {
            for market in markets {
                if matches(market.id.as_str(), &market.name) {
                    let metrics = aggregate(
                        facts
                            .iter()
                            .filter(|fact| market.plugins.contains(&fact.plugin))
                            .map(|fact| &fact.fact),
                    );
                    rows.push(row(
                        (market.id.as_str(), &market.name),
                        market.enabled,
                        market.source_path,
                        kind,
                        metrics,
                    ));
                }
            }
        },
    }
    Ok(rows)
}

fn row(
    identity: (&str, &str),
    enabled: bool,
    source: String,
    kind: &str,
    metrics: ResourceMetrics,
) -> ResourceRow {
    let (id, name) = identity;
    ResourceRow {
        href: format!("/admin/catalog/{kind}/{}", urlencoding::encode(id)),
        id: id.to_owned(),
        name: name.to_owned(),
        enabled,
        source,
        cost: metrics
            .related_cost_microdollars
            .map(|value| format!("${:.6}", value as f64 / 1_000_000.0)),
        average_tokens: metrics
            .average_tokens_per_measured_request
            .map(|value| format!("{value:.0}")),
        quality: metrics
            .average_quality_score
            .map(|value| format!("{value:.2}")),
        latency: metrics
            .average_latency_ms
            .map(|value| format!("{value:.0}")),
        metrics,
    }
}
