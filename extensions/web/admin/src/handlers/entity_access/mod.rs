//! Generic entity-access HTTP handlers backing the unified `/admin/access`
//! matrix and per-entity inline panels (gateway routes, MCP servers, …).
//!
//! Wraps [`systemprompt_security::authz::AccessControlRepository`] with the
//! same endpoint shape the gateway-specific handlers use, but parameterized on
//! `entity_type`. Allowed values mirror the Postgres CHECK constraint on
//! `access_control_rules.entity_type`.

mod support;
mod types;

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use systemprompt::identifiers::RuleId;
use systemprompt_security::authz::{Access, AccessRule, UpsertRuleParams};

use crate::error::{AdminError, AdminResult};

use support::{collect_entity_ids, parse_access, parse_rule_type, repo, validate_entity_type};
use types::{
    AllAccessQuery, ApplyTemplateBody, ApplyTemplateResponse, DefaultIncludedBody,
    EntityAccessEntry, EntityAccessResponse, EntityDefaultResponse, ListAllEntityAccessResponse,
    UpsertRuleBody, UpsertRuleResponse,
};

pub(crate) async fn list_entity_access_handler(
    State(pool): State<Arc<PgPool>>,
    Path((entity_type, entity_id)): Path<(String, String)>,
) -> AdminResult<Response> {
    let kind = validate_entity_type(&entity_type)?;
    let r = repo(&pool);
    let rules = r
        .list_rules_for_entity(kind, &entity_id)
        .await
        .map_err(AdminError::internal)?;
    let default_included = r
        .get_entity(kind, &entity_id)
        .await
        .map_err(AdminError::internal)?
        .is_some_and(|entity| entity.default_included);
    Ok(Json(EntityAccessResponse {
        entity_type,
        entity_id,
        default_included,
        rules,
    })
    .into_response())
}

pub(crate) async fn upsert_entity_rule_handler(
    State(pool): State<Arc<PgPool>>,
    Path((entity_type, entity_id)): Path<(String, String)>,
    Json(body): Json<UpsertRuleBody>,
) -> AdminResult<Response> {
    let kind = validate_entity_type(&entity_type)?;
    let rule_type = parse_rule_type(&body.rule_type)
        .ok_or_else(|| AdminError::BadRequest("invalid rule_type".to_owned()))?;
    let access = parse_access(&body.access)
        .ok_or_else(|| AdminError::BadRequest("invalid access".to_owned()))?;
    if body.rule_value.trim().is_empty() {
        return Err(AdminError::BadRequest("rule_value required".to_owned()));
    }
    let rule = repo(&pool)
        .upsert_rule(UpsertRuleParams {
            entity_type: kind,
            entity_id: &entity_id,
            rule_type,
            rule_value: &body.rule_value,
            access,
            justification: body.justification.as_deref(),
        })
        .await
        .map_err(AdminError::internal)?;
    Ok(Json(UpsertRuleResponse { rule }).into_response())
}

pub(crate) async fn delete_entity_rule_handler(
    State(pool): State<Arc<PgPool>>,
    Path((entity_type, _entity_id, rule_id)): Path<(String, String, String)>,
) -> AdminResult<Response> {
    validate_entity_type(&entity_type)?;
    if !repo(&pool)
        .delete_rule(&RuleId::new(rule_id))
        .await
        .map_err(AdminError::internal)?
    {
        return Err(AdminError::NotFound("rule not found".to_owned()));
    }
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

pub(crate) async fn set_entity_default_handler(
    State(pool): State<Arc<PgPool>>,
    Path((entity_type, entity_id)): Path<(String, String)>,
    Json(body): Json<DefaultIncludedBody>,
) -> AdminResult<Response> {
    let kind = validate_entity_type(&entity_type)?;
    repo(&pool)
        .upsert_entity(kind, &entity_id, body.default_included, "admin:dashboard")
        .await
        .map_err(AdminError::internal)?;
    Ok(Json(EntityDefaultResponse {
        entity_type,
        entity_id,
        default_included: body.default_included,
    })
    .into_response())
}

/// Bulk-list every entity of the given type with its rules and default.
///
/// Entity ids come from the on-disk profile (`gateway_route`) or
/// `services/mcp/*.yaml` (`mcp_server`), not from the database.
pub(crate) async fn list_all_entity_access_handler(
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AllAccessQuery>,
) -> AdminResult<Response> {
    let kind = validate_entity_type(&query.entity_type)?;
    let entity_ids = collect_entity_ids(&query.entity_type)?;
    let r = repo(&pool);
    let bulk = r
        .list_rules_bulk(kind, &entity_ids)
        .await
        .map_err(AdminError::internal)?;
    let mut entries: Vec<EntityAccessEntry> = Vec::with_capacity(entity_ids.len());
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
        entries.push(EntityAccessEntry {
            entity_id: eid.clone(),
            default_included,
            rules,
        });
    }
    Ok(Json(ListAllEntityAccessResponse {
        entity_type: query.entity_type,
        entities: entries,
    })
    .into_response())
}

/// Apply a department/role template across every entity of a given type.
///
/// Per-entity failures are counted rather than aborting the run, and the
/// gateway-ACL export fires once at the end instead of per rule.
pub(crate) async fn apply_template_handler(
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<ApplyTemplateBody>,
) -> AdminResult<Response> {
    let kind = validate_entity_type(&body.entity_type)?;
    let rule_type = parse_rule_type(&body.subject_type)
        .ok_or_else(|| AdminError::BadRequest("invalid subject_type".to_owned()))?;
    if body.subject_value.trim().is_empty() {
        return Err(AdminError::BadRequest("subject_value required".to_owned()));
    }
    if !["allow", "deny", "clear"].contains(&body.action.as_str()) {
        return Err(AdminError::BadRequest(
            "action must be allow|deny|clear".to_owned(),
        ));
    }

    let entity_ids = collect_entity_ids(&body.entity_type)?;
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
                    rule_type: rule_type.clone(),
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

    Ok(Json(ApplyTemplateResponse {
        applied,
        failed,
        entity_count: entity_ids.len(),
    })
    .into_response())
}
