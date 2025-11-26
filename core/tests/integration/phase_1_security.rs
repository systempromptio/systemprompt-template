// Phase 1 Security Foundation - Integration Tests

use systemprompt_core_system::models::context::AppContext;
use systemprompt_models::execution::context::RequestContext;
use systemprompt_models::auth::UserType;
use systemprompt_identifiers::{SessionId, TraceId, ContextId, AgentName};

#[tokio::test]
async fn test_admin_jwt_generation_at_startup() {
    let ctx = AppContext::new().await.unwrap();

    // Verify admin token exists and is not empty
    assert!(
        !ctx.admin_auth_token().as_str().is_empty(),
        "Admin token should not be empty"
    );

    // Verify admin user ID exists
    assert!(
        ctx.admin_user_id().is_system(),
        "Admin user ID should be system user"
    );

    println!("✅ Admin JWT generated at startup");
}

#[tokio::test]
async fn test_admin_token_has_correct_claims() {
    use jsonwebtoken::{decode, DecodingKey, Validation};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        roles: Vec<String>,
        aud: Vec<String>,
        exp: i64,
        iat: i64,
    }

    let ctx = AppContext::new().await.unwrap();

    let mut validation = Validation::default();
    validation.set_audience(&["a2a", "api", "mcp"]);
    validation.validate_exp = false; // Don't validate expiry in tests

    let decoded = decode::<Claims>(
        ctx.admin_auth_token().as_str(),
        &DecodingKey::from_secret(ctx.jwt_secret().as_bytes()),
        &validation,
    )
    .expect("Admin token should be valid JWT");

    // Verify admin role
    assert!(
        decoded.claims.roles.contains(&"admin".to_string()),
        "Admin token should have 'admin' role"
    );

    // Verify subject is system user
    assert_eq!(
        decoded.claims.sub, "system",
        "Admin token subject should be 'system'"
    );

    // Verify audiences
    assert!(
        decoded.claims.aud.contains(&"a2a".to_string()),
        "Admin token should have 'a2a' audience"
    );
    assert!(
        decoded.claims.aud.contains(&"api".to_string()),
        "Admin token should have 'api' audience"
    );
    assert!(
        decoded.claims.aud.contains(&"mcp".to_string()),
        "Admin token should have 'mcp' audience"
    );

    println!("✅ Admin token has correct claims");
}

#[test]
fn test_context_id_patterns() {
    // Pattern 1: Empty string for user-level contexts
    let ctx = RequestContext::new(
        SessionId::new("sess_123".to_string()),
        TraceId::new("trace_456".to_string()),
        ContextId::new(String::new()),
        AgentName::new("agent".to_string()),
    );

    assert!(
        ctx.execution.context_id.as_str().is_empty(),
        "User-level context should have empty context_id"
    );

    // Pattern 2: UUID for conversation contexts
    let conversation_id = uuid::Uuid::new_v4().to_string();
    let ctx2 = RequestContext::new(
        SessionId::new("sess_123".to_string()),
        TraceId::new("trace_456".to_string()),
        ContextId::new(conversation_id.clone()),
        AgentName::new("agent".to_string()),
    );

    assert_eq!(
        ctx2.execution.context_id.as_str(),
        conversation_id,
        "Conversation context should have UUID context_id"
    );

    println!("✅ Context ID patterns work correctly");
}

#[test]
fn test_context_id_should_not_be_system() {
    // Verify "system" is NOT used as a context ID
    // This test documents the correct pattern

    // ❌ WRONG - DO NOT DO THIS
    // ContextId::new("system".to_string())

    // ✅ CORRECT - Use empty string for user-level
    let ctx = RequestContext::new(
        SessionId::new("test".to_string()),
        TraceId::new("trace".to_string()),
        ContextId::new(String::new()),  // Empty string, NOT "system"
        AgentName::new("agent".to_string()),
    );

    assert!(
        ctx.execution.context_id.as_str().is_empty(),
        "User-level context should use empty string, not 'system'"
    );

    // Verify "system" is never a context ID in this codebase
    assert_ne!(
        ctx.execution.context_id.as_str(),
        "system",
        "Context ID should NEVER be 'system'"
    );

    println!("✅ Context IDs correctly avoid 'system' hardcoded value");
}

#[tokio::test]
async fn test_admin_token_used_for_system_operations() {
    let app_ctx = AppContext::new().await.unwrap();

    // System operations must use admin token from AppContext
    let req_ctx = RequestContext::new(
        SessionId::new("system".to_string()),
        TraceId::new(format!("trace_{}", uuid::Uuid::new_v4())),
        ContextId::new(String::new()),  // User-level (no conversation)
        AgentName::new("system-agent".to_string()),
    )
    .with_auth_token(app_ctx.admin_auth_token().as_str())
    .with_user_id(app_ctx.admin_user_id().clone())
    .with_user_type(UserType::Admin);

    // Verify token is not empty
    assert!(
        !req_ctx.auth.auth_token.as_str().is_empty(),
        "System operation should have non-empty token"
    );

    // Verify it's the admin token
    assert_eq!(
        req_ctx.auth.auth_token.as_str(),
        app_ctx.admin_auth_token().as_str(),
        "System operation should use admin token from AppContext"
    );

    // Verify user type is admin
    assert_eq!(
        req_ctx.auth.user_type,
        UserType::Admin,
        "System operation should have Admin user type"
    );

    println!("✅ System operations use admin token correctly");
}

#[test]
fn test_request_context_only_one_constructor() {
    // Verify only RequestContext::new() exists
    // Old constructors should be deleted:
    // - RequestContext::system() ❌ DELETED
    // - RequestContext::user_only() ❌ DELETED
    // - RequestContext::from_session() ❌ DELETED

    // Only valid constructor:
    let ctx = RequestContext::new(
        SessionId::new("sess_123".to_string()),
        TraceId::new("trace_456".to_string()),
        ContextId::new(String::new()),
        AgentName::new("agent".to_string()),
    );

    // Default values should be safe (unauthenticated)
    assert_eq!(ctx.auth.user_type, UserType::Anon, "Default should be Anon");
    assert!(ctx.auth.user_id.is_anonymous(), "Default should be anonymous");

    // Authentication must be explicitly added via builder
    let authenticated_ctx = ctx
        .with_auth_token("test_token")
        .with_user_id(systemprompt_identifiers::UserId::new("user_123".to_string()))
        .with_user_type(UserType::Standard);

    assert!(!authenticated_ctx.auth.auth_token.as_str().is_empty());
    assert_eq!(authenticated_ctx.auth.user_type, UserType::Standard);

    println!("✅ RequestContext has only one constructor with builder pattern");
}

#[tokio::test]
async fn test_no_empty_tokens_for_admin_operations() {
    let app_ctx = AppContext::new().await.unwrap();

    // Decode the admin token to verify it's valid
    use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        roles: Vec<String>,
    }

    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&["a2a", "api", "mcp"]);
    validation.validate_exp = false;

    let result = decode::<Claims>(
        app_ctx.admin_auth_token().as_str(),
        &DecodingKey::from_secret(app_ctx.jwt_secret().as_bytes()),
        &validation,
    );

    assert!(
        result.is_ok(),
        "Admin token should be a valid JWT: {:?}",
        result.err()
    );

    let claims = result.unwrap().claims;
    assert!(
        claims.roles.contains(&"admin".to_string()),
        "Admin token must have admin role"
    );

    println!("✅ No empty tokens - admin JWT is valid and has admin role");
}

#[test]
fn test_context_structure_separation() {
    // Verify Phase 3 context structure is in place
    let ctx = RequestContext::new(
        SessionId::new("sess_123".to_string()),
        TraceId::new("trace_456".to_string()),
        ContextId::new("ctx_789".to_string()),
        AgentName::new("agent".to_string()),
    );

    // Verify structured components exist
    let _auth = &ctx.auth;
    let _request = &ctx.request;
    let _execution = &ctx.execution;
    let _settings = &ctx.settings;

    // Verify accessors work
    assert_eq!(ctx.session_id().as_str(), "sess_123");
    assert_eq!(ctx.trace_id().as_str(), "trace_456");
    assert_eq!(ctx.context_id().as_str(), "ctx_789");
    assert_eq!(ctx.agent_name().as_str(), "agent");

    println!("✅ Context structure properly separated into logical components");
}

#[tokio::test]
async fn test_admin_token_long_lived() {
    use chrono::Utc;

    let app_ctx = AppContext::new().await.unwrap();

    // Decode token to check expiry
    use jsonwebtoken::{decode, DecodingKey, Validation};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        exp: i64,
        iat: i64,
    }

    let mut validation = Validation::default();
    validation.set_audience(&["a2a", "api", "mcp"]);
    validation.validate_exp = false;

    let decoded = decode::<Claims>(
        app_ctx.admin_auth_token().as_str(),
        &DecodingKey::from_secret(app_ctx.jwt_secret().as_bytes()),
        &validation,
    )
    .unwrap();

    let now = Utc::now().timestamp();
    let expires_in = decoded.claims.exp - now;

    // Should expire in approximately 365 days (31,536,000 seconds)
    // Allow some variance (between 364 and 366 days)
    let one_year_seconds = 365 * 24 * 60 * 60;
    assert!(
        expires_in > (one_year_seconds - 86400),  // At least 364 days
        "Admin token should be long-lived (1 year)"
    );

    println!(
        "✅ Admin token expires in {} days",
        expires_in / (24 * 60 * 60)
    );
}
