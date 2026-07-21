//! `POST /govern/authz` — extension webhook implementing
//! [`systemprompt_security::authz::AuthzDecisionHook`] as an HTTP endpoint.
//!
//! Core's gateway and MCP enforcement sites POST an [`AuthzRequest`] here;
//! this handler loads the matching rules from `access_control_rules`, runs
//! the pure deny-overrides resolver, audits the decision to
//! `governance_decisions`, and returns an [`AuthzDecision`] for core to act
//! on. The audit row's `policy` is `authz` regardless of `entity_type`, so
//! `infra logs audit` can correlate gateway and MCP decisions in one stream.
//!
//! The resolver runs over core's `user` / `role` dimensions plus every subject
//! dimension this extension declares in [`crate::authz`] — today that means a
//! `department` rule binds here, not just in the access matrix.

use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use systemprompt::identifiers::{Actor, MarketplaceId};
use systemprompt_security::authz::{
    AccessControlRepository, AccessRule, AuthzDecision, AuthzRequest, Decision, DecisionTag,
    EntityKind, EntityRef, EntityRow, ResolveInput, ResolveParent, resolve,
};
use tokio::sync::RwLock;

use crate::authz::{dimensions, subject_attributes_for};
use crate::repositories::governance::{GovernanceDecisionRecord, insert_governance_decision};

const POLICY_NAME: &str = "authz";

struct CachedMarketplaceParents {
    entries: Vec<(EntityRef, Vec<AccessRule>, Option<bool>)>,
    fetched_at: Instant,
}

static MARKETPLACE_PARENT_CACHE: LazyLock<RwLock<Option<CachedMarketplaceParents>>> =
    LazyLock::new(|| RwLock::new(None));
const MARKETPLACE_PARENT_TTL: Duration = Duration::from_mins(5);

async fn marketplace_parent_entries(
    repo: &AccessControlRepository,
) -> Vec<(EntityRef, Vec<AccessRule>, Option<bool>)> {
    {
        let cache = MARKETPLACE_PARENT_CACHE.read().await;
        if let Some(ref cached) = *cache
            && cached.fetched_at.elapsed() < MARKETPLACE_PARENT_TTL
        {
            return cached.entries.clone();
        }
    }

    let entries = match repo.list_entities(EntityKind::Marketplace).await {
        Ok(rows) => {
            let mut out = Vec::with_capacity(rows.len());
            for row in rows {
                let rules = repo
                    .list_rules_for_entity(EntityKind::Marketplace, &row.id)
                    .await
                    .unwrap_or_else(|e| {
                        tracing::warn!(error = %e, marketplace_id = %row.id, "marketplace_parent_entries: rules lookup failed; treating marketplace as no-rules");
                        Vec::new()
                    });
                out.push((
                    EntityRef::Marketplace(MarketplaceId::new(&row.id)),
                    rules,
                    Some(row.default_included),
                ));
            }
            out
        },
        Err(e) => {
            tracing::warn!(error = %e, "marketplace_parent_entries: list_entities failed; resolving without cascade parent");
            Vec::new()
        },
    };

    {
        let mut cache = MARKETPLACE_PARENT_CACHE.write().await;
        *cache = Some(CachedMarketplaceParents {
            entries: entries.clone(),
            fetched_at: Instant::now(),
        });
    }
    entries
}

async fn load_rules(
    repo: &AccessControlRepository,
    req: &AuthzRequest,
) -> Result<(Vec<AccessRule>, Option<EntityRow>), Response> {
    let kind = req.entity.kind();
    let id = req.entity.id_str();
    let rules = repo
        .list_rules_for_entity(kind, id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, entity_type = %kind, entity_id = %id, "list_rules_for_entity failed");
            (StatusCode::INTERNAL_SERVER_ERROR, "list_rules failed").into_response()
        })?;
    let entity = repo.get_entity(kind, id).await.map_err(|e| {
        tracing::error!(error = %e, entity_type = %kind, entity_id = %id, "get_entity failed");
        (StatusCode::INTERNAL_SERVER_ERROR, "get_entity failed").into_response()
    })?;
    Ok((rules, entity))
}

async fn audit_decision(
    pool: &PgPool,
    req: &AuthzRequest,
    rules: &[AccessRule],
    entity: Option<&EntityRow>,
    decision: &Decision,
) {
    let (decision_tag, reason_str, justification_opt): (DecisionTag, String, Option<String>) =
        match decision {
            Decision::Allow { .. } => (DecisionTag::Allow, String::new(), None),
            Decision::Deny { reason } => (DecisionTag::Deny, reason.to_string(), None),
        };
    let id = uuid::Uuid::new_v4().to_string();
    let entity_type_str = req.entity.kind().as_str();
    let entity_id_str = req.entity.id_str();
    // variable-shape: governance audit `evaluated_rules` JSONB payload embedding
    // caller-supplied roles/attributes/context maps, not a template/response
    // body
    let evaluated = serde_json::json!({
        "entity_type": entity_type_str,
        "entity_id": entity_id_str,
        "trace_id": req.trace_id.as_str(),
        "roles": req.roles,
        "attributes": req.attributes,
        "context": req.context,
        "entity": entity,
        "justification": justification_opt,
        "rules": rules,
    });
    let actor = Actor::user(req.user_id.clone());
    let record = GovernanceDecisionRecord {
        id: &id,
        actor: &actor,
        session_id: req.trace_id.as_str(),
        tool_name: entity_id_str,
        agent_id: None,
        // Why: authz decisions are entity-keyed, not agent-keyed; entity_type
        // remains in evaluated_rules above for forensic lookup.
        agent_scope: None,
        decision: decision_tag,
        policy: POLICY_NAME,
        reason: &reason_str,
        evaluated_rules: &evaluated,
        plugin_id: None,
        act_chain: &[],
        context_id: req
            .context_id
            .as_ref()
            .map(systemprompt::identifiers::ContextId::as_str),
        task_id: req
            .task_id
            .as_ref()
            .map(systemprompt::identifiers::TaskId::as_str),
    };
    if let Err(e) = insert_governance_decision(pool, &record).await {
        tracing::error!(error = %e, "Failed to record authz decision");
    }
}

pub(crate) async fn govern_authz(
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<AuthzRequest>,
) -> Response {
    let repo = AccessControlRepository::from_pool(Arc::clone(&pool));

    let (rules, entity) = match load_rules(&repo, &req).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let mp_entries = marketplace_parent_entries(&repo).await;
    let parents: Vec<ResolveParent<'_>> = mp_entries
        .iter()
        .map(|(entity, rules, default_included)| ResolveParent {
            entity,
            rules,
            default_included: *default_included,
        })
        .collect();

    // Resolved by lookup rather than read off the request, so a department
    // change or a revocation binds on the next call instead of waiting for the
    // caller's token to refresh.
    let attributes = subject_attributes_for(&pool, &req.user_id).await;

    let decision = resolve(ResolveInput {
        entity: &req.entity,
        rules: &rules,
        user_id: &req.user_id,
        user_roles: &req.roles,
        default_included: entity.as_ref().map(|e| e.default_included),
        parents: &parents,
        attributes: &attributes,
        dimensions: dimensions(&pool),
    });

    audit_decision(&pool, &req, &rules, entity.as_ref(), &decision).await;

    let resp = match decision {
        Decision::Allow { .. } => AuthzDecision::Allow,
        Decision::Deny { reason } => AuthzDecision::Deny {
            reason,
            policy: POLICY_NAME.to_owned(),
        },
    };
    (StatusCode::OK, Json(resp)).into_response()
}
