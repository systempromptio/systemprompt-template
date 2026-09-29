//! `gateway_routes` reads and writes, in dispatch order.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use crate::types::GatewayRouteView;

pub const SOURCE_CODE: &str = "code";
pub const SOURCE_DASHBOARD: &str = "dashboard";

#[derive(Debug, Clone, Serialize)]
pub struct RouteRow {
    pub route: GatewayRouteView,
    pub position: i32,
    pub explicit_id: bool,
    pub source: String,
    pub updated_at: DateTime<Utc>,
}

// JSON: JSONB columns — the passthrough blocks (pricing/when/requires) and
// the header map, stored as the operator wrote them and read back as YAML
// values for the file renderer
fn yaml_of(value: Option<serde_json::Value>) -> Option<serde_yaml::Value> {
    // Why: discard-ok: a JSON value is always a YAML value; None is the only
    // honest answer if it somehow is not
    value.and_then(|v| serde_yaml::to_value(v).ok())
}

// JSON: JSONB column — a YAML block written as JSON for storage
fn json_of(value: Option<&serde_yaml::Value>) -> Option<serde_json::Value> {
    // Why: discard-ok: the block already parsed as YAML; a value JSON cannot
    // carry (a non-string key) is dropped rather than failing the write
    value.and_then(|v| serde_json::to_value(v).ok())
}

pub async fn list_gateway_routes(pool: &PgPool) -> Result<Vec<RouteRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r"SELECT id, position, name, description, model_pattern, provider, upstream_model,
                 extra_headers, pricing, when_match, requires, fallback_provider,
                 fallback_upstream_model, explicit_id, source, updated_at
            FROM gateway_routes
           ORDER BY position ASC, id ASC"
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|r| {
            // JSON: JSONB column — a flat string map; a row that does not
            // decode is a decode error, not a route with no headers
            let extra_headers =
                serde_json::from_value(r.extra_headers).map_err(|e| sqlx::Error::ColumnDecode {
                    index: "extra_headers".to_owned(),
                    source: Box::new(e),
                })?;
            Ok(RouteRow {
                route: GatewayRouteView {
                    id: r.id,
                    name: r.name,
                    description: r.description,
                    model_pattern: r.model_pattern,
                    provider: r.provider,
                    upstream_model: r.upstream_model,
                    extra_headers,
                    pricing: yaml_of(r.pricing),
                    when: yaml_of(r.when_match),
                    requires: yaml_of(r.requires),
                    fallback_provider: r.fallback_provider,
                    fallback_upstream_model: r.fallback_upstream_model,
                },
                position: r.position,
                explicit_id: r.explicit_id,
                source: r.source,
                updated_at: r.updated_at,
            })
        })
        .collect()
}

pub async fn count_gateway_routes(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM gateway_routes"#)
        .fetch_one(pool)
        .await
}

// Why: one route to write, keyed on its effective id.
#[derive(Debug, Clone)]
pub struct RouteWrite<'a> {
    pub route: &'a GatewayRouteView,
    pub position: i32,
    pub explicit_id: bool,
    pub source: &'a str,
}

pub async fn upsert_gateway_route(
    pool: &PgPool,
    write: &RouteWrite<'_>,
) -> Result<(), sqlx::Error> {
    let r = write.route;
    // JSON: JSONB column — the header map as a JSON object
    // Why: discard-ok: a string-keyed map of strings always serialises
    let headers = serde_json::to_value(&r.extra_headers).unwrap_or_default();
    sqlx::query!(
        r"INSERT INTO gateway_routes
              (id, position, model_pattern, provider, upstream_model, extra_headers, pricing,
               when_match, requires, fallback_provider, fallback_upstream_model, explicit_id,
               source, name, description, updated_at)
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, NOW())
          ON CONFLICT (id) DO UPDATE
             SET position = EXCLUDED.position,
                 name = EXCLUDED.name,
                 description = EXCLUDED.description,
                 model_pattern = EXCLUDED.model_pattern,
                 provider = EXCLUDED.provider,
                 upstream_model = EXCLUDED.upstream_model,
                 extra_headers = EXCLUDED.extra_headers,
                 pricing = EXCLUDED.pricing,
                 when_match = EXCLUDED.when_match,
                 requires = EXCLUDED.requires,
                 fallback_provider = EXCLUDED.fallback_provider,
                 fallback_upstream_model = EXCLUDED.fallback_upstream_model,
                 explicit_id = EXCLUDED.explicit_id,
                 source = EXCLUDED.source,
                 updated_at = NOW()",
        r.id,
        write.position,
        r.model_pattern,
        r.provider,
        r.upstream_model,
        headers,
        json_of(r.pricing.as_ref()),
        json_of(r.when.as_ref()),
        json_of(r.requires.as_ref()),
        r.fallback_provider,
        r.fallback_upstream_model,
        write.explicit_id,
        write.source,
        r.name,
        r.description,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_gateway_route(pool: &PgPool, id: &str) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query!("DELETE FROM gateway_routes WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected())
}

pub async fn set_gateway_route_position(
    pool: &PgPool,
    id: &str,
    position: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE gateway_routes SET position = $2, updated_at = NOW() WHERE id = $1",
        id,
        position
    )
    .execute(pool)
    .await?;
    Ok(())
}
