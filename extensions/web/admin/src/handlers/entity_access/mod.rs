//! Generic entity-access HTTP handlers backing the unified `/admin/access`
//! matrix and per-entity inline panels (gateway routes, MCP servers, …).
//!
//! Wraps [`systemprompt_security::authz::AccessControlRepository`] with the
//! same endpoint shape the gateway-specific handlers use, but parameterized on
//! `entity_type`. Allowed values mirror the Postgres CHECK constraint on
//! `access_control_rules.entity_type`.

mod support;

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::RuleId;
use systemprompt_security::authz::{Access, AccessRule, UpsertRuleParams};

use crate::handlers::shared;

use support::{collect_entity_ids, parse_access, parse_rule_type, repo, validate_entity_type};

pub(crate) async fn list_entity_access_handler(
    State(pool): State<Arc<PgPool>>,
    Path((entity_type, entity_id)): Path<(String, String)>,
) -> Response {
    let kind = match validate_entity_type(&entity_type) {
        Ok(k) => k,
        Err(r) => return *r,
    };
    let r = repo(&pool);
    let rules = match r.list_rules_for_entity(kind, &entity_id).await {
        Ok(rs) => rs,
        Err(e) => {
            tracing::error!(error = %e, entity_type, entity_id, "list_rules_for_entity failed");
            return shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal error");
        },
    };
    let default_included = match r.get_entity(kind, &entity_id).await {
        Ok(Some(entity)) => entity.default_included,
        Ok(None) => false,
        Err(e) => {
            tracing::error!(error = %e, entity_type, entity_id, "get_entity failed");
            return shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal error");
        },
    };
    Json(serde_json::json!({
        "entity_type": entity_type,
        "entity_id": entity_id,
        "default_included": default_included,
        "rules": rules,
    }))
    .into_response()
}

#[derive(Debug, Deserialize)]
pub(crate) struct UpsertRuleBody {
    pub rule_type: String,
    pub rule_value: String,
    pub access: String,
    #[serde(default)]
    pub justification: Option<String>,
}

pub(crate) async fn upsert_entity_rule_handler(
    State(pool): State<Arc<PgPool>>,
    Path((entity_type, entity_id)): Path<(String, String)>,
    Json(body): Json<UpsertRuleBody>,
) -> Response {
    let kind = match validate_entity_type(&entity_type) {
        Ok(k) => k,
        Err(r) => return *r,
    };
    let Some(rule_type) = parse_rule_type(&body.rule_type) else {
        return shared::error_response(StatusCode::BAD_REQUEST, "invalid rule_type");
    };
    let Some(access) = parse_access(&body.access) else {
        return shared::error_response(StatusCode::BAD_REQUEST, "invalid access");
    };
    if body.rule_value.trim().is_empty() {
        return shared::error_response(StatusCode::BAD_REQUEST, "rule_value required");
    }
    match repo(&pool)
        .upsert_rule(UpsertRuleParams {
            entity_type: kind,
            entity_id: &entity_id,
            rule_type,
            rule_value: &body.rule_value,
            access,
            justification: body.justification.as_deref(),
        })
        .await
    {
        Ok(rule) => Json(serde_json::json!({ "rule": rule })).into_response(),
        Err(e) => {
            tracing::error!(error = %e, entity_type, entity_id, "upsert_rule failed");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
        },
    }
}

pub(crate) async fn delete_entity_rule_handler(
    State(pool): State<Arc<PgPool>>,
    Path((entity_type, _entity_id, rule_id)): Path<(String, String, String)>,
) -> Response {
    if let Err(r) = validate_entity_type(&entity_type) {
        return *r;
    }
    match repo(&pool).delete_rule(&RuleId::new(rule_id.clone())).await {
        Ok(true) => (StatusCode::NO_CONTENT, ()).into_response(),
        Ok(false) => shared::error_response(StatusCode::NOT_FOUND, "rule not found"),
        Err(e) => {
            tracing::error!(error = %e, rule_id, "delete_rule failed");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
        },
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct DefaultIncludedBody {
    pub default_included: bool,
}

pub(crate) async fn set_entity_default_handler(
    State(pool): State<Arc<PgPool>>,
    Path((entity_type, entity_id)): Path<(String, String)>,
    Json(body): Json<DefaultIncludedBody>,
) -> Response {
    let kind = match validate_entity_type(&entity_type) {
        Ok(k) => k,
        Err(r) => return *r,
    };
    match repo(&pool)
        .upsert_entity(kind, &entity_id, body.default_included, "admin:dashboard")
        .await
    {
        Ok(()) => Json(serde_json::json!({
            "entity_type": entity_type,
            "entity_id": entity_id,
            "default_included": body.default_included,
        }))
        .into_response(),
        Err(e) => {
            tracing::error!(error = %e, entity_type, entity_id, "upsert_entity failed");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
        },
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct AllAccessQuery {
    pub entity_type: String,
}

/// Bulk-list every entity of the given type with its rules + default. Used by
/// the `/admin/access` matrix view. Routes/IDs come from the on-disk profile
/// (`gateway_route`) or `services/mcp/*.yaml` (`mcp_server`).
pub(crate) async fn list_all_entity_access_handler(
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AllAccessQuery>,
) -> Response {
    let kind = match validate_entity_type(&query.entity_type) {
        Ok(k) => k,
        Err(r) => return *r,
    };
    let entity_ids = match collect_entity_ids(&query.entity_type) {
        Ok(ids) => ids,
        Err(resp) => return *resp,
    };
    let r = repo(&pool);
    let bulk = match r.list_rules_bulk(kind, &entity_ids).await {
        Ok(m) => m,
        Err(e) => {
            tracing::error!(error = %e, entity_type = %query.entity_type, "list_rules_bulk failed");
            return shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal error");
        },
    };
    let mut entries: Vec<serde_json::Value> = Vec::with_capacity(entity_ids.len());
    for eid in &entity_ids {
        let default_included = r
            .get_entity(kind, eid)
            .await
            .inspect_err(
                |e| tracing::warn!(error = %e, eid = %eid, "entity_access: get_entity failed"),
            )
            .ok()
            .flatten()
            .is_some_and(|e| e.default_included);
        let rules: Vec<AccessRule> = bulk.get(eid).cloned().unwrap_or_default();
        entries.push(serde_json::json!({
            "entity_id": eid,
            "default_included": default_included,
            "rules": rules,
        }));
    }
    Json(serde_json::json!({
        "entity_type": query.entity_type,
        "entities": entries,
    }))
    .into_response()
}

#[derive(Debug, Deserialize)]
pub(crate) struct ApplyTemplateBody {
    pub entity_type: String,
    pub subject_type: String,
    pub subject_value: String,
    /// One of: "allow", "deny", "clear".
    pub action: String,
}

/// Apply a department/role template across every entity of a given type.
/// Wraps repeated [`upsert_rule`]/[`delete_rule`] calls and triggers the
/// gateway-ACL export once at the end.
pub(crate) async fn apply_template_handler(
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<ApplyTemplateBody>,
) -> Response {
    let kind = match validate_entity_type(&body.entity_type) {
        Ok(k) => k,
        Err(r) => return *r,
    };
    let Some(rule_type) = parse_rule_type(&body.subject_type) else {
        return shared::error_response(StatusCode::BAD_REQUEST, "invalid subject_type");
    };
    if body.subject_value.trim().is_empty() {
        return shared::error_response(StatusCode::BAD_REQUEST, "subject_value required");
    }
    if !["allow", "deny", "clear"].contains(&body.action.as_str()) {
        return shared::error_response(StatusCode::BAD_REQUEST, "action must be allow|deny|clear");
    }

    let entity_ids = match collect_entity_ids(&body.entity_type) {
        Ok(ids) => ids,
        Err(resp) => return *resp,
    };
    let r = repo(&pool);
    let mut applied = 0usize;
    let mut failed = 0usize;

    for eid in &entity_ids {
        if body.action == "clear" {
            let existing = r.list_rules_for_entity(kind, eid).await.unwrap_or_default();
            for rule in existing {
                if rule.rule_type == rule_type && rule.rule_value == body.subject_value {
                    if r.delete_rule(&rule.id).await.is_ok() {
                        applied += 1;
                    } else {
                        failed += 1;
                    }
                }
            }
        } else {
            let access = if body.action == "deny" {
                Access::Deny
            } else {
                Access::Allow
            };
            match r
                .upsert_rule(UpsertRuleParams {
                    entity_type: kind,
                    entity_id: eid,
                    rule_type,
                    rule_value: &body.subject_value,
                    access,
                    justification: None,
                })
                .await
            {
                Ok(_) => applied += 1,
                Err(e) => {
                    tracing::warn!(error = %e, entity_id = %eid, "apply_template upsert failed");
                    failed += 1;
                },
            }
        }
    }

    Json(serde_json::json!({
        "applied": applied,
        "failed": failed,
        "entity_count": entity_ids.len(),
    }))
    .into_response()
}
