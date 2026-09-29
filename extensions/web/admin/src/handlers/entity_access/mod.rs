//! Generic entity-access HTTP handlers backing the unified `/admin/access`
//! matrix and per-entity inline panels (gateway routes, MCP servers, …).
//!
//! Wraps [`systemprompt_security::authz::AccessControlRepository`] with the
//! same endpoint shape the gateway-specific handlers use, but parameterized on
//! `entity_type`. Allowed values mirror the Postgres CHECK constraint on
//! `access_control_rules.entity_type`.

mod support;
mod template;
mod types;

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use systemprompt::identifiers::RuleId;
use systemprompt_security::authz::{
    Access, AccessRule, DASHBOARD_SOURCE, EntityRef, RuleType, UpsertRuleParams,
};

use crate::activity::RuleChange;
use crate::error::{AdminError, AdminResult};
use crate::repositories::config::gateway::registered_routes_from_services;
use crate::types::UserContext;

use support::{
    collect_entity_ids, parse_access, parse_subject, record_change, repo, rule_subject,
    validate_entity_type,
};
use template::{Tally, TemplateSubject, clear_subject_rules, upsert_subject_rule};
use types::{
    AllAccessQuery, ApplyTemplateBody, ApplyTemplateResponse, DefaultIncludedBody,
    EntityAccessEntry, EntityAccessResponse, EntityDefaultResponse, ListAllEntityAccessResponse,
    RemoveRuleQuery, UpsertRuleBody, UpsertRuleResponse,
};

pub(crate) async fn list_entity_access_handler(
    State(pool): State<Arc<PgPool>>,
    Path((entity_type, entity_id)): Path<(String, String)>,
) -> AdminResult<Response> {
    let kind = validate_entity_type(&entity_type)?;
    let entity = EntityRef::from_kind_and_id(kind, &entity_id)
        .map_err(|error| AdminError::Unprocessable(error.to_string()))?;
    let r = repo(&pool);
    let rules = r
        .list_rules_for_entity(entity.kind(), entity.id_str())
        .await
        .map_err(AdminError::internal)?;
    let default_included = r
        .get_entity(entity.kind(), entity.id_str())
        .await
        .map_err(AdminError::internal)?
        .is_some_and(|entity| entity.default_included);
    Ok(Json(EntityAccessResponse {
        entity_type: entity.kind().as_str().to_owned(),
        entity_id: entity.id_str().to_owned(),
        default_included,
        rules,
    })
    .into_response())
}

pub(crate) async fn upsert_entity_rule_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path((entity_type, entity_id)): Path<(String, String)>,
    Json(body): Json<UpsertRuleBody>,
) -> AdminResult<Response> {
    let kind = validate_entity_type(&entity_type)?;
    // Why: the emptiness check precedes subject parsing — a blank value would
    // fail the parse too, and "invalid rule_type" for a missing value points
    // the caller at the wrong field.
    if body.rule_value.trim().is_empty() {
        return Err(AdminError::BadRequest("rule_value required".to_owned()));
    }
    let (rule_type, rule_value) = parse_subject(&pool, &body.rule_type, &body.rule_value).await?;
    let access = parse_access(&body.access)
        .ok_or_else(|| AdminError::BadRequest("invalid access".to_owned()))?;
    // Why: a shared band needs a stated reason; a per-person override may go
    // without one. Same rule as the replace-all path in `access_control.rs`.
    let justification = body
        .justification
        .as_deref()
        .map(str::trim)
        .filter(|j| !j.is_empty());
    if rule_type != RuleType::USER && justification.is_none() {
        return Err(AdminError::BadRequest(format!(
            "a reason (justification) is required for the {rule_type} rule on '{rule_value}'"
        )));
    }
    let rule = repo(&pool)
        .upsert_rule(UpsertRuleParams {
            entity_type: kind,
            entity_id: &entity_id,
            rule_type: rule_type.clone(),
            rule_value: &rule_value,
            access,
            justification,
            source: DASHBOARD_SOURCE,
        })
        .await
        .map_err(AdminError::internal)?;
    if body
        .valid_until
        .is_some_and(|until| until <= chrono::Utc::now())
    {
        return Err(AdminError::BadRequest(
            "valid_until must be in the future".to_owned(),
        ));
    }
    crate::repositories::access_control::validity::set_rule_validity(
        pool.as_ref(),
        rule.id.as_str(),
        body.valid_until,
    )
    .await?;
    record_change(
        &pool,
        &user_ctx.user_id,
        RuleChange {
            entity_type: kind.as_str(),
            entity_id: &entity_id,
            subject: &format!("{rule_type}:{rule_value}"),
            access: Some(&body.access),
            reason: justification,
        },
    )
    .await;
    Ok(Json(UpsertRuleResponse {
        rule,
        valid_until: body.valid_until,
    })
    .into_response())
}

// Why: the reason is optional here — a subject page removing its own
// override has nothing to explain — and recorded whenever it is given.
pub(crate) async fn delete_entity_rule_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path((entity_type, entity_id, rule_id)): Path<(String, String, String)>,
    Query(query): Query<RemoveRuleQuery>,
) -> AdminResult<Response> {
    let kind = validate_entity_type(&entity_type)?;
    let subject = rule_subject(&pool, kind, &entity_id, &rule_id).await;
    if !repo(&pool)
        .delete_rule(&RuleId::new(rule_id))
        .await
        .map_err(AdminError::internal)?
    {
        return Err(AdminError::NotFound("rule not found".to_owned()));
    }
    let reason = query
        .reason
        .as_deref()
        .map(str::trim)
        .filter(|r| !r.is_empty());
    record_change(
        &pool,
        &user_ctx.user_id,
        RuleChange {
            entity_type: kind.as_str(),
            entity_id: &entity_id,
            subject: &subject,
            access: None,
            reason,
        },
    )
    .await;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

pub(crate) async fn set_entity_default_handler(
    State(pool): State<Arc<PgPool>>,
    Path((entity_type, entity_id)): Path<(String, String)>,
    Json(body): Json<DefaultIncludedBody>,
) -> AdminResult<Response> {
    let kind = validate_entity_type(&entity_type)?;
    registered_routes_from_services()?.require(kind, &entity_id)?;
    let entity = EntityRef::from_kind_and_id(kind, &entity_id)
        .map_err(|error| AdminError::Unprocessable(error.to_string()))?;
    repo(&pool)
        .upsert_entity(
            entity.kind(),
            entity.id_str(),
            body.default_included,
            "admin:dashboard",
        )
        .await
        .map_err(AdminError::internal)?;
    Ok(Json(EntityDefaultResponse {
        entity_type: entity.kind().as_str().to_owned(),
        entity_id: entity.id_str().to_owned(),
        default_included: body.default_included,
    })
    .into_response())
}

// Why: entity ids come from the profile's dispatchable routes (`gateway_route`)
// or `services/mcp/*.yaml` (`mcp_server`), not from the database.
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
        let entity = EntityRef::from_kind_and_id(kind, eid).map_err(AdminError::internal)?;
        let default_included = r
            .get_entity(entity.kind(), entity.id_str())
            .await
            .inspect_err(
                |e| tracing::warn!(error = %e, eid = %eid, "entity_access: get_entity failed"),
            )
            .ok()
            .flatten()
            .is_some_and(|e| e.default_included);
        let rules: Vec<AccessRule> = bulk.get(eid).cloned().unwrap_or_default();
        entries.push(EntityAccessEntry {
            entity_id: entity.id_str().to_owned(),
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

pub(crate) async fn apply_template_handler(
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<ApplyTemplateBody>,
) -> AdminResult<Response> {
    let kind = validate_entity_type(&body.entity_type)?;
    if body.subject_value.trim().is_empty() {
        return Err(AdminError::BadRequest("subject_value required".to_owned()));
    }
    let (rule_type, rule_value) =
        parse_subject(&pool, &body.subject_type, &body.subject_value).await?;
    if !["allow", "deny", "clear"].contains(&body.action.as_str()) {
        return Err(AdminError::BadRequest(
            "action must be allow|deny|clear".to_owned(),
        ));
    }
    let justification = body
        .justification
        .as_deref()
        .map(str::trim)
        .filter(|j| !j.is_empty());
    if body.action != "clear" && rule_type != RuleType::USER && justification.is_none() {
        return Err(AdminError::BadRequest(
            "a reason (justification) is required when granting or denying a shared band"
                .to_owned(),
        ));
    }

    let entity_ids = collect_entity_ids(&body.entity_type)?;
    let r = repo(&pool);
    let subject = TemplateSubject {
        kind,
        rule_type: &rule_type,
        rule_value: &rule_value,
        justification,
    };
    let mut tally = Tally::default();

    for eid in &entity_ids {
        if body.action == "clear" {
            tally += clear_subject_rules(&r, &subject, eid).await;
        } else {
            let access = if body.action == "deny" {
                Access::Deny
            } else {
                Access::Allow
            };
            tally += upsert_subject_rule(&r, &subject, eid, access).await;
        }
    }

    Ok(Json(ApplyTemplateResponse {
        applied: tally.applied,
        failed: tally.failed,
        entity_count: entity_ids.len(),
    })
    .into_response())
}
