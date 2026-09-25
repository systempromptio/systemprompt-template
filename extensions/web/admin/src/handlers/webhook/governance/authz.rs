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
//! `group` rule binds here, not just in the access matrix.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use std::borrow::Cow;
use systemprompt::identifiers::{Actor, SessionId};

use systemprompt_security::authz::{
    AccessControlRepository, AccessRule, AuthzDecision, AuthzRequest, ChainSources, Decision,
    DecisionTag, DenyReason, EntityRow, ParentChainIndex, ResolveBase,
};

use crate::authz::{dimensions, subject_attributes_for};
use systemprompt_security::authz::{GovernanceDecisionRecord, insert_governance_decision};

const POLICY_NAME: &str = "authz";

// Why: HTTP failures can bypass this hook; return a deny. lint-ok: http-error
fn unavailable() -> Response {
    Json(AuthzDecision::Deny {
        reason: DenyReason::PolicyViolation {
            policy: "authorization_unavailable".into(),
            detail: Cow::Borrowed("Authorization temporarily unavailable; retry later"),
        },
        policy: POLICY_NAME.into(),
    })
    .into_response()
}

async fn audit_unavailable(pool: &PgPool, req: &AuthzRequest) {
    tracing::error!(user_id = %req.user_id, trace_id = %req.trace_id, entity = %req.entity, "authorization_unavailable");
    let decision = Decision::Deny {
        reason: DenyReason::PolicyViolation {
            policy: "authorization_unavailable".into(),
            detail: Cow::Borrowed("Authorization temporarily unavailable; retry later"),
        },
    };
    audit_decision(pool, req, &[], None, &decision).await;
}

async fn parent_index(repo: &AccessControlRepository) -> Result<ParentChainIndex, Response> {
    let services = systemprompt::loader::ServicesBootstrap::get().map_err(|error| {
        tracing::error!(%error, "authorization_unavailable: services");
        unavailable()
    })?;
    ParentChainIndex::load(repo, Arc::new(ChainSources::from_services(services)))
        .await
        .map_err(|error| {
            tracing::error!(%error, "authorization_unavailable: parent policies");
            unavailable()
        })
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
        .map_err(|error| {
            tracing::error!(%error, %id, "authorization_unavailable: entity rules");
            unavailable()
        })?;
    let entity = repo.get_entity(kind, id).await.map_err(|error| {
        tracing::error!(%error, %id, "authorization_unavailable: entity");
        unavailable()
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
            Decision::Warn { reason } => (DecisionTag::Warn, reason.to_string(), None),
            Decision::Deny { reason } => (DecisionTag::Deny, reason.to_string(), None),
            // Why: `build_response` refuses a hold, so the audit records the
            // deny that the caller actually received, carrying the hold's own
            // reason so the mounted-where-it-cannot-be-honoured policy is
            // identifiable from the audit row alone.
            Decision::Pending { reason } => {
                tracing::error!(
                    %reason,
                    "a governance hold reached the PreToolUse webhook, which answers \
                     synchronously and cannot park a call; refusing it"
                );
                (DecisionTag::Deny, reason.to_string(), None)
            },
        };
    let id = uuid::Uuid::new_v4().to_string();
    let entity_type_str = req.entity.kind().as_str();
    let entity_id_str = req.entity.id_str();
    // JSON: variable-shape: governance audit `evaluated_rules` JSONB payload
    // embedding caller-supplied roles/attributes/context maps, not a
    // template/response body
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
    // Why: enforcement sites without an explicit context still need one the
    // session's other rows join to; deriving keeps them in a single context.
    let context_id = req.context_id.clone().unwrap_or_else(|| {
        req.session_id.as_ref().map_or_else(
            systemprompt::identifiers::ContextId::legacy,
            systemprompt::identifiers::ContextId::derived_from_session,
        )
    });
    let record = GovernanceDecisionRecord {
        id: &id,
        actor: &actor,
        // Why: the attested session, so a gateway decision keys to the same
        // session row as the prompt gate and the `ai_requests` row it belongs
        // to. Enforcement sites without a session (server-attach RBAC, MCP)
        // send none and store the empty string; `trace_id` below is what keeps
        // those rows correlatable, so this column never carries a trace id.
        session_id: req.session_id.as_ref().map_or("", SessionId::as_str),
        tool_name: entity_id_str,
        agent_id: None,
        // Why: authz decisions are entity-keyed, not agent-keyed; entity_type
        // remains in evaluated_rules above for forensic lookup.
        agent_scope: None,
        tool_use_id: None,
        decision: decision_tag,
        policy: POLICY_NAME,
        reason: &reason_str,
        evaluated_rules: &evaluated,
        client_id: req
            .client_id
            .as_ref()
            .map(systemprompt::identifiers::ClientId::as_str),
        plugin_id: None,
        act_chain: &req.act_chain,
        trace_id: Some(req.trace_id.as_str()),
        context_id: context_id.as_str(),
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
    Json(mut req): Json<AuthzRequest>,
) -> Response {
    // Why: lint-ok: http-error — a hook answers 200 with a decision; an error
    // status reads as "hook unavailable" and lets the call through
    let repo = AccessControlRepository::from_pool(Arc::clone(&pool));

    let (rules, entity) = match load_rules(&repo, &req).await {
        Ok(v) => v,
        Err(resp) => {
            audit_unavailable(&pool, &req).await;
            return resp;
        },
    };

    let chains = match parent_index(&repo).await {
        Ok(chains) => chains,
        Err(response) => {
            audit_unavailable(&pool, &req).await;
            return response;
        },
    };

    // Why: resolved by lookup rather than read off the request, so a group
    // change or a revocation binds on the next call instead of waiting for the
    // caller's token to refresh.
    let attributes = match subject_attributes_for(&pool, &req.user_id).await {
        Ok(attributes) => attributes,
        Err(error) => {
            tracing::error!(%error, user_id = %req.user_id, trace_id = %req.trace_id, "authorization_unavailable: attributes");
            audit_unavailable(&pool, &req).await;
            return unavailable();
        },
    };
    let identity = match crate::repositories::users::queries::find_identity_envelope(
        &pool,
        &req.user_id,
    )
    .await
    {
        Ok(Some(identity)) if identity.status == "active" => identity,
        _ => {
            audit_unavailable(&pool, &req).await;
            return unavailable();
        },
    };

    req.roles = identity.roles;
    let decision = chains.resolve(
        req.entity.kind(),
        req.entity.id_str(),
        ResolveBase {
            rules: &rules,
            user_id: &req.user_id,
            user_roles: &req.roles,
            default_included: entity.as_ref().map(|e| e.default_included),
            attributes: &attributes,
            dimensions: dimensions(&pool),
        },
    );

    audit_decision(&pool, &req, &rules, entity.as_ref(), &decision).await;

    let resp = match decision {
        // Why: warn permits, so it joins the allow arm. This plane has no warn
        // verdict of its own; the finding survives on the audit row written
        // just above.
        Decision::Allow { .. } | Decision::Warn { .. } => AuthzDecision::Allow,
        Decision::Deny { reason } => AuthzDecision::Deny {
            reason,
            policy: POLICY_NAME.to_owned(),
        },
        // Why: this endpoint answers synchronously and cannot park a call, so
        // a hold degrades to a deny rather than an allow — same reasoning as
        // core's `RuleBasedHook`.
        Decision::Pending { .. } => AuthzDecision::Deny {
            reason: DenyReason::PolicyViolation {
                policy: "require_approval".to_owned(),
                detail: Cow::Borrowed(
                    "approval required, but this enforcement point cannot hold a request",
                ),
            },
            policy: POLICY_NAME.to_owned(),
        },
    };
    (StatusCode::OK, Json(resp)).into_response()
}
