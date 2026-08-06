//! Row builders and non-session token minters shared by the seeded suites.
//!
//! [`crate::handler_variants`] drives every page against a database holding
//! nothing but the two principals, which pins the *empty* branch of each
//! template. The suites in this half need the opposite: a session that exists,
//! a context with messages, a trace with decisions. A page that renders its
//! empty state correctly and its populated state not at all passes the first
//! and fails here, which is the split worth having.
//!
//! Ids are UUID-suffixed. Migration `025_demo_organizations` seeds three demo
//! customers with ten users and roughly a thousand `ai_requests`, so a fixture
//! that reused a plausible id would be asserting against seeded rows without
//! knowing it.

use std::collections::BTreeMap;

use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, Header, encode};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt::models::auth::{
    JwtAudience, JwtClaims, Permission, RateLimitTier, TokenType, UserType,
};

use crate::globals;

/// A fresh, collision-proof id fragment.
pub fn unique(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4().simple())
}

/// Mint a token for an arbitrary audience / scope / `plugin_id` triple.
///
/// The hook endpoints validate against `aud=hook` and `scope=hook:*`, which
/// neither [`crate::principal`]'s admin token nor any core minter produces —
/// `JwtService::generate_admin_token` hard-codes the standard audiences. The
/// claims are therefore assembled here so a case can also mint the *wrong*
/// token on purpose and prove the validator rejects it.
pub struct TokenSpec<'a> {
    pub subject: &'a str,
    pub audiences: Vec<JwtAudience>,
    pub scopes: Vec<Permission>,
    pub plugin_id: Option<&'a str>,
}

impl<'a> TokenSpec<'a> {
    /// A well-formed hook token: `aud=hook`, both hook scopes, a plugin id.
    pub fn hook(subject: &'a str) -> Self {
        Self {
            subject,
            audiences: vec![JwtAudience::Hook],
            scopes: vec![Permission::HookTrack, Permission::HookGovern],
            plugin_id: Some("contract-plugin"),
        }
    }

    /// A token the secrets endpoints accept: `aud=plugin`, no scopes.
    ///
    /// `validate_plugin_jwt` checks the audience and nothing else, and the
    /// resource audience is one no minter in core produces — the admin session
    /// token carries the standard set, which is why it is rejected there.
    pub fn plugin(subject: &'a str) -> Self {
        Self {
            subject,
            audiences: vec![JwtAudience::Resource("plugin".to_owned())],
            scopes: Vec::new(),
            plugin_id: Some("contract-plugin"),
        }
    }
}

pub fn mint(spec: &TokenSpec<'_>) -> String {
    let now = Utc::now();
    let claims = JwtClaims {
        sub: spec.subject.to_owned(),
        iat: now.timestamp(),
        exp: (now + Duration::hours(1)).timestamp(),
        nbf: Some(now.timestamp()),
        iss: globals::jwt_issuer(),
        aud: spec.audiences.clone(),
        jti: uuid::Uuid::new_v4().to_string(),
        scope: spec.scopes.clone(),
        username: "hook@contract.test".to_owned(),
        email: "hook@contract.test".to_owned(),
        user_type: UserType::Service,
        roles: vec!["user".to_owned()],
        attributes: BTreeMap::new(),
        client_id: None,
        token_type: TokenType::Bearer,
        auth_time: now.timestamp(),
        session_id: None,
        rate_limit_tier: Some(RateLimitTier::Service),
        plugin_id: spec.plugin_id.map(ToOwned::to_owned),
        act: None,
    };

    let kid =
        systemprompt_security::keys::authority::active_kid().expect("an installed signing key");
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_owned());
    let key = systemprompt_security::keys::authority::encoding_key().expect("an encoding key");
    encode(&header, &claims, key).expect("sign the token")
}

/// Insert a user with the `user` role. `users.name` is unique, so the email
/// doubles as the name exactly as the production provisioning paths do.
pub async fn insert_user(pool: &PgPool, id: &str, email: &str) -> UserId {
    sqlx::query(
        "INSERT INTO users (id, name, email, display_name, status, email_verified, roles)
         VALUES ($1, $2, $2, $3, 'active', true, ARRAY['user'])",
    )
    .bind(id)
    .bind(email)
    .bind(email)
    .execute(pool)
    .await
    .expect("insert user");
    UserId::new(id.to_owned())
}

/// Give a user a `user_profile_ext` row at a chosen share-token version.
///
/// Users are provisioned without one, and `find_share_token_version` reports
/// that absence as `Ok(None)` — which the public manifest endpoint answers the
/// same way it answers a forged token. A share token is therefore only
/// verifiable once this row exists.
pub async fn insert_profile_ext(pool: &PgPool, user_id: &UserId, version: i32) {
    sqlx::query(
        "INSERT INTO user_profile_ext (user_id, share_token_version) VALUES ($1, $2)
         ON CONFLICT (user_id) DO UPDATE SET share_token_version = EXCLUDED.share_token_version",
    )
    .bind(user_id.as_str())
    .bind(version)
    .execute(pool)
    .await
    .expect("insert user_profile_ext");
}

/// `ai_requests.session_id` carries a foreign key to `user_sessions`, so a
/// request with a session needs the session row first.
pub async fn insert_session(pool: &PgPool, session_id: &str, user_id: &UserId) {
    sqlx::query("INSERT INTO user_sessions (session_id, user_id) VALUES ($1, $2)")
        .bind(session_id)
        .bind(user_id.as_str())
        .execute(pool)
        .await
        .expect("insert user session");
}

pub async fn insert_context(
    pool: &PgPool,
    context_id: &str,
    user_id: &UserId,
    session_id: Option<&str>,
    name: &str,
) {
    sqlx::query(
        "INSERT INTO user_contexts (context_id, user_id, session_id, name) VALUES ($1, $2, $3, $4)",
    )
    .bind(context_id)
    .bind(user_id.as_str())
    .bind(session_id)
    .bind(name)
    .execute(pool)
    .await
    .expect("insert user context");
}

/// `ai_requests.context_id` is NOT NULL; core's sentinel stands in for a row
/// that belongs to no known context.
pub const LEGACY_CONTEXT_ID: &str = "00000000-0000-0000-0000-4c4547414359";

pub struct RequestSpec<'a> {
    pub id: String,
    pub user_id: &'a UserId,
    pub session_id: Option<&'a str>,
    pub trace_id: Option<&'a str>,
    pub context_id: Option<&'a str>,
    pub status: &'a str,
}

pub async fn insert_request(pool: &PgPool, spec: &RequestSpec<'_>) {
    sqlx::query(
        "INSERT INTO ai_requests (
             id, request_id, user_id, session_id, trace_id, context_id,
             provider, model, input_tokens, output_tokens, tokens_used,
             cost_microdollars, latency_ms, status, actor_kind, actor_id,
             created_at, updated_at)
         VALUES ($1, $1, $2, $3, $4, COALESCE($5, $7), 'anthropic', 'claude-contract-model',
                 100, 20, 120, 5000, 250, $6, 'user', $2, NOW(), NOW())",
    )
    .bind(&spec.id)
    .bind(spec.user_id.as_str())
    .bind(spec.session_id)
    .bind(spec.trace_id)
    .bind(spec.context_id)
    .bind(spec.status)
    .bind(LEGACY_CONTEXT_ID)
    .execute(pool)
    .await
    .expect("insert ai_request");
}

/// A `governance_decisions` row. `trace_id` is what the trace explorer groups
/// on, so a decision without one is invisible to the pages under test.
pub struct DecisionSpec<'a> {
    pub id: String,
    pub user_id: &'a UserId,
    pub session_id: &'a str,
    pub decision: &'a str,
    pub policy: &'a str,
    pub tool_name: &'a str,
}

pub async fn insert_decision(pool: &PgPool, spec: &DecisionSpec<'_>) {
    sqlx::query(
        "INSERT INTO governance_decisions (
             id, user_id, session_id, context_id, tool_name, decision, policy, reason,
             actor_kind, actor_id, created_at)
         VALUES ($1, $2, $3, $7, $4, $5, $6, 'contract fixture', 'user', $2, NOW())",
    )
    .bind(&spec.id)
    .bind(spec.user_id.as_str())
    .bind(spec.session_id)
    .bind(spec.tool_name)
    .bind(spec.decision)
    .bind(spec.policy)
    .bind(LEGACY_CONTEXT_ID)
    .execute(pool)
    .await
    .expect("insert governance decision");
}

pub async fn insert_summary(pool: &PgPool, session_id: &str, user_id: &UserId, title: &str) {
    sqlx::query(
        "INSERT INTO plugin_session_summaries
             (id, session_id, user_id, started_at, tool_uses, prompts, errors,
              model, status, ai_title, total_events)
         VALUES ($1, $2, $3, NOW() - INTERVAL '1 hour', 3, 2, 0,
                 'claude-contract-model', 'active', $4, 5)",
    )
    .bind(unique("summary"))
    .bind(session_id)
    .bind(user_id.as_str())
    .bind(title)
    .execute(pool)
    .await
    .expect("insert session summary");
}

pub async fn insert_event(pool: &PgPool, user_id: &UserId, session_id: &str, tool: &str) {
    sqlx::query(
        "INSERT INTO plugin_usage_events
             (id, user_id, session_id, event_type, tool_name, created_at)
         VALUES ($1, $2, $3, 'claude_code_PostToolUse', $4, NOW())",
    )
    .bind(unique("event"))
    .bind(user_id.as_str())
    .bind(session_id)
    .bind(tool)
    .execute(pool)
    .await
    .expect("insert plugin usage event");
}

/// Insert an access-control grant, creating the catalog row the FK requires.
pub async fn insert_acl_rule(
    pool: &PgPool,
    entity_type: &str,
    entity_id: &str,
    rule_type: &str,
    rule_value: &str,
    access: &str,
) {
    sqlx::query(
        "INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
         VALUES ($1, $2, false, 'contract-fixture')
         ON CONFLICT (entity_type, entity_id) DO NOTHING",
    )
    .bind(entity_type)
    .bind(entity_id)
    .execute(pool)
    .await
    .expect("insert access control entity");

    sqlx::query(
        "INSERT INTO access_control_rules
             (id, entity_type, entity_id, rule_type, rule_value, access)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(unique("rule"))
    .bind(entity_type)
    .bind(entity_id)
    .bind(rule_type)
    .bind(rule_value)
    .bind(access)
    .execute(pool)
    .await
    .expect("insert access control rule");
}
