//! Thin gateway-route convenience wrappers around the unified
//! `systemprompt_security::authz` core crate.
//!
//! Types and the resolver are re-exported directly from `systemprompt_security::authz`
//! so handler call sites compile unchanged. The repository functions below
//! preserve the historical `(pool, route_id, ...)` signatures while delegating
//! to [`systemprompt_security::authz::AccessControlRepository`] with
//! `entity_type = "gateway_route"`. Use the core repository directly for any
//! new entity types (`mcp_server`, etc.).
//!
//! Note: `default_included` lives on `access_control_entities` post-migration
//! 007, fetched via [`get_entity`] / [`upsert_entity`].

use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;
use systemprompt_security::authz::{
    AccessControlRepository, AuthzError, EntityKind, EntityRow, UpsertRuleParams,
};

pub use systemprompt_security::authz::{resolve, Access, AccessRule, Decision, RuleType};

const ENTITY_TYPE: EntityKind = EntityKind::GatewayRoute;

fn repo(pool: &PgPool) -> AccessControlRepository {
    AccessControlRepository::from_pool(Arc::new(pool.clone()))
}

fn map_err(err: &AuthzError) -> sqlx::Error {
    sqlx::Error::Decode(err.to_string().into())
}

pub async fn list_rules_for_route(
    pool: &PgPool,
    route_id: &str,
) -> Result<Vec<AccessRule>, sqlx::Error> {
    repo(pool)
        .list_rules_for_entity(ENTITY_TYPE, route_id)
        .await
        .map_err(|e| map_err(&e))
}

pub async fn list_rules_bulk(
    pool: &PgPool,
    route_ids: &[String],
) -> Result<HashMap<String, Vec<AccessRule>>, sqlx::Error> {
    repo(pool)
        .list_rules_bulk(ENTITY_TYPE, route_ids)
        .await
        .map_err(|e| map_err(&e))
}

pub async fn upsert_rule(
    pool: &PgPool,
    route_id: &str,
    rule_type: RuleType,
    rule_value: &str,
    access: Access,
) -> Result<AccessRule, sqlx::Error> {
    repo(pool)
        .upsert_rule(UpsertRuleParams {
            entity_type: ENTITY_TYPE,
            entity_id: route_id,
            rule_type,
            rule_value,
            access,
            justification: None,
        })
        .await
        .map_err(|e| map_err(&e))
}

pub async fn delete_rule(pool: &PgPool, rule_id: &str) -> Result<bool, sqlx::Error> {
    let id = systemprompt::identifiers::RuleId::new(rule_id.to_string());
    repo(pool).delete_rule(&id).await.map_err(|e| map_err(&e))
}

pub async fn get_entity(
    pool: &PgPool,
    route_id: &str,
) -> Result<Option<EntityRow>, sqlx::Error> {
    repo(pool)
        .get_entity(ENTITY_TYPE, route_id)
        .await
        .map_err(|e| map_err(&e))
}

pub async fn upsert_entity(
    pool: &PgPool,
    route_id: &str,
    default_included: bool,
    source: &str,
) -> Result<(), sqlx::Error> {
    repo(pool)
        .upsert_entity(ENTITY_TYPE, route_id, default_included, source)
        .await
        .map_err(|e| map_err(&e))
}
