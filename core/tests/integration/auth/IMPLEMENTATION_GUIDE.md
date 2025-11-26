# Auth Tests Implementation Guide

**Status**: All 5 tests are 100% stubs. Zero authentication testing implemented.

**Goal**: Comprehensive authentication and authorization testing including JWT validation, OAuth flow, permissions, and session management.

---

## Test Organization (Semantic Breakdown)

### Group 1: JWT Token Validation (3 tests)
- `test_valid_jwt_token_accepted` - Valid token grants access
- `test_expired_jwt_token_rejected` - Expired token denied
- `test_invalid_jwt_signature_rejected` - Tampered token rejected

### Group 2: OAuth Flow (2 tests)
- `test_oauth_authorization_flow` - Redirect → provider → callback → token
- `test_oauth_user_creation_on_first_login` - First login creates user

### Group 3: Permissions (3 tests)
- `test_admin_role_access` - Admin can access admin endpoints
- `test_user_role_denied_admin_endpoints` - User role blocked from admin
- `test_unauthenticated_denied_protected_endpoints` - No token → 401

### Group 4: Session Management (3 tests)
- `test_session_created_on_login` - sessions table has entry
- `test_session_cookie_set` - Set-Cookie header present
- `test_session_expiration` - Old sessions marked expired

---

## Implementation Template

```rust
use crate::common::*;
use anyhow::Result;

#[tokio::test]
async fn test_valid_jwt_token_accepted() -> Result<()> {
    // PHASE 1: Setup
    let ctx = TestContext::new().await?;

    // PHASE 2: Generate valid JWT
    let token = generate_test_jwt(&ctx, "user", "test-user-id", 3600)?;

    // PHASE 3: Make authenticated request
    let url = format!("{}/api/v1/core/agents", ctx.base_url);
    let response = ctx.http
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    // PHASE 4: Assertions
    assert_eq!(response.status(), 200, "Valid JWT should be accepted");

    let body: serde_json::Value = response.json().await?;
    assert!(body["data"].is_array(), "Response should be JSON array");

    println!("✓ Valid JWT token accepted");
    Ok(())
}

fn generate_test_jwt(ctx: &TestContext, role: &str, user_id: &str, expires_in: i64) -> Result<String> {
    // Generate a test JWT token valid for expires_in seconds
    // In real implementation: use jsonwebtoken crate with test key
    use chrono::Utc;
    use jsonwebtoken::{encode, Header, EncodingKey};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct Claims {
        pub sub: String,
        pub roles: Vec<String>,
        pub exp: i64,
        pub iat: i64,
    }

    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        roles: vec![role.to_string()],
        exp: now + expires_in,
        iat: now,
    };

    let key = EncodingKey::from_secret(b"test-secret-key");
    let token = encode(&Header::default(), &claims, &key)?;
    Ok(token)
}
```

---

## Database Validation Queries

### Test 1: JWT Validation
```sql
-- Check for test JWT usage in logs
SELECT session_id, user_id, endpoint_path, response_status
FROM endpoint_requests
WHERE endpoint_path LIKE '%agents%'
AND response_status = 200
ORDER BY requested_at DESC
LIMIT 1;

-- Expected: Successful request with 200 status
```

### Test 2: Expired Token
```sql
-- Verify failed auth attempt logged
SELECT session_id, event_type, event_category, severity, metadata
FROM analytics_events
WHERE event_type = 'auth_failed'
AND metadata->>'reason' LIKE '%expired%'
ORDER BY created_at DESC
LIMIT 1;

-- Expected: auth_failed event with 'expired' in reason
```

### Test 3: OAuth Flow
```sql
-- Verify user created during OAuth
SELECT id, email, oauth_provider, oauth_id, created_at
FROM users
WHERE oauth_provider = 'test'
ORDER BY created_at DESC
LIMIT 1;

-- Expected: New user row with OAuth details

-- Verify OAuth session created
SELECT session_id, user_id, oauth_provider, created_at
FROM oauth_sessions
WHERE oauth_provider = 'test'
ORDER BY created_at DESC
LIMIT 1;

-- Expected: Session linked to user
```

### Test 4: Permission Check (Admin)
```sql
-- Verify admin user can access admin endpoints
SELECT session_id, endpoint_path, response_status
FROM endpoint_requests
WHERE endpoint_path LIKE '%admin%'
AND response_status = 200
ORDER BY requested_at DESC
LIMIT 1;

-- Expected: 200 status for admin endpoints

-- Verify user role in database
SELECT id, email, roles
FROM users
WHERE id = 'test-admin-user'
AND 'admin' = ANY(roles);

-- Expected: User has 'admin' role
```

### Test 5: Permission Denied
```sql
-- Verify non-admin denied access
SELECT session_id, endpoint_path, response_status
FROM endpoint_requests
WHERE endpoint_path LIKE '%admin%'
AND response_status = 403
ORDER BY requested_at DESC
LIMIT 1;

-- Expected: 403 Forbidden

-- Verify denial event logged
SELECT event_type, event_category, severity, metadata->>'reason' as denial_reason
FROM analytics_events
WHERE event_type = 'permission_denied'
AND metadata->>'endpoint' LIKE '%admin%'
ORDER BY created_at DESC
LIMIT 1;
```

### Test 6: Session Creation
```sql
-- Verify session created for authenticated user
SELECT session_id, user_id, started_at, created_at
FROM user_sessions
WHERE user_id = 'test-user-id'
ORDER BY started_at DESC
LIMIT 1;

-- Expected: Session with user_id populated

-- Verify session has proper structure
SELECT session_id, user_id, user_type,
       CASE WHEN user_id IS NOT NULL THEN 'authenticated'
            ELSE 'anonymous' END as session_type
FROM user_sessions
WHERE session_id = 'test-session-id';
```

### Test 7: Session Cookie
```sql
-- Check session cookie in HTTP headers (captured in tests)
-- This would be validated in test code:
-- assert!(response.headers()["set-cookie"].to_str().unwrap().contains("session_id"));

-- Verify session table has cookie value
SELECT session_id, cookie_value, expires_at
FROM oauth_sessions
WHERE session_id = 'test-session-id'
AND cookie_value IS NOT NULL;
```

### Test 8: Session Expiration
```sql
-- Find expired sessions
SELECT session_id, user_id, started_at, ended_at,
       EXTRACT(EPOCH FROM (ended_at - started_at)) as duration_seconds
FROM user_sessions
WHERE ended_at < NOW()
AND ended_at IS NOT NULL
ORDER BY ended_at DESC
LIMIT 5;

-- Expected: Marked ended_at for old sessions

-- Check expiration logic
SELECT session_id,
       CASE
           WHEN EXTRACT(EPOCH FROM (NOW() - last_activity_at)) > 3600 THEN 'expired'
           ELSE 'active'
       END as status
FROM user_sessions
WHERE user_id = 'test-user-id'
ORDER BY last_activity_at DESC
LIMIT 1;
```

---

## Test Implementation Examples

### Test 1: Valid JWT
**File**: `jwt_validation.rs`

```rust
#[tokio::test]
async fn test_valid_jwt_token_accepted() -> Result<()> {
    let ctx = TestContext::new().await?;

    // Generate admin token valid for 1 hour
    let token = generate_admin_token(3600)?;

    // Make request with token
    let url = format!("{}/api/v1/core/agents", ctx.base_url);
    let response = ctx.http
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("x-fingerprint", ctx.fingerprint())
        .send()
        .await?;

    // Should succeed
    assert_eq!(response.status(), 200, "Valid JWT should be accepted");

    let body: serde_json::Value = response.json().await?;
    assert!(body["data"].is_array(), "Should return agents array");

    // Verify request logged
    TestContext::wait_for_async_processing().await;

    let session_query = DatabaseQueryEnum::FindSessionByFingerprintAny.get(ctx.db.as_ref());
    let sessions = ctx.db.fetch_all(&session_query, &[&ctx.fingerprint().to_string()]).await?;

    assert!(!sessions.is_empty(), "Authenticated session not created");

    // Cleanup
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(ctx.fingerprint().to_string());
    cleanup.cleanup_all().await?;

    println!("✓ Valid JWT token accepted and session created");
    Ok(())
}

#[tokio::test]
async fn test_expired_jwt_token_rejected() -> Result<()> {
    let ctx = TestContext::new().await?;

    // Generate expired token (0 seconds = expired immediately)
    let token = generate_admin_token(0)?;

    let url = format!("{}/api/v1/core/agents", ctx.base_url);
    let response = ctx.http
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("x-fingerprint", ctx.fingerprint())
        .send()
        .await?;

    // Should be rejected
    assert_eq!(response.status(), 401, "Expired JWT should be rejected");

    // Verify failure logged
    TestContext::wait_for_async_processing().await;

    let event_query = "SELECT event_type, severity FROM analytics_events
                      WHERE event_type LIKE '%auth%' AND severity = 'warning'
                      ORDER BY created_at DESC LIMIT 1";

    let events = ctx.db.fetch_all(event_query, &[]).await?;
    assert!(!events.is_empty(), "Auth failure not logged");

    println!("✓ Expired JWT token rejected");
    Ok(())
}

#[tokio::test]
async fn test_invalid_jwt_signature_rejected() -> Result<()> {
    let ctx = TestContext::new().await?;

    // Create tampered token
    let valid_token = generate_admin_token(3600)?;
    let parts: Vec<&str> = valid_token.split('.').collect();
    assert_eq!(parts.len(), 3, "JWT should have 3 parts");

    // Tamper with signature (last part)
    let tampered = format!("{}.{}.invalid_signature", parts[0], parts[1]);

    let url = format!("{}/api/v1/core/agents", ctx.base_url);
    let response = ctx.http
        .get(&url)
        .header("Authorization", format!("Bearer {}", tampered))
        .send()
        .await?;

    // Should be rejected
    assert_eq!(response.status(), 401, "Invalid signature should be rejected");

    println!("✓ Invalid JWT signature rejected");
    Ok(())
}
```

---

### Test 2: OAuth Flow
**File**: `oauth_flow.rs`

```rust
#[tokio::test]
async fn test_oauth_authorization_flow() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    // Step 1: Initiate OAuth
    let oauth_url = format!("{}/api/v1/auth/oauth/authorize", ctx.base_url);
    let response = ctx.http
        .get(&oauth_url)
        .query(&[("provider", "test"), ("redirect_uri", "http://localhost:3000/callback")])
        .header("x-fingerprint", &fingerprint)
        .send()
        .await?;

    assert_eq!(response.status(), 302, "Should redirect to provider");
    let location = response.headers()
        .get("location")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| anyhow::anyhow!("No redirect location"))?;

    assert!(location.contains("oauth"), "Should redirect to OAuth provider");

    // Step 2: Simulate provider callback (in real test, would hit callback endpoint)
    let callback_url = format!("{}/api/v1/auth/oauth/callback", ctx.base_url);
    let callback_response = ctx.http
        .get(&callback_url)
        .query(&[
            ("code", "test-auth-code"),
            ("state", "test-state"),
            ("provider", "test")
        ])
        .header("x-fingerprint", &fingerprint)
        .send()
        .await?;

    assert!(callback_response.status().is_success(), "Callback should succeed");

    // Step 3: Verify user created in database
    TestContext::wait_for_async_processing().await;

    let user_query = "SELECT id, email, oauth_provider
                     FROM users
                     WHERE oauth_provider = 'test'
                     ORDER BY created_at DESC
                     LIMIT 1";

    let user_rows = ctx.db.fetch_all(user_query, &[]).await?;
    assert!(!user_rows.is_empty(), "User not created from OAuth");

    // Cleanup
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ OAuth authorization flow completed");
    Ok(())
}
```

---

### Test 3: Permission Enforcement
**File**: `permissions.rs`

```rust
#[tokio::test]
async fn test_admin_role_access() -> Result<()> {
    let ctx = TestContext::new().await?;

    // Generate admin token
    let admin_token = generate_admin_token(3600)?;

    // Access admin endpoint
    let url = format!("{}/api/v1/core/admin/users", ctx.base_url);
    let response = ctx.http
        .get(&url)
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await?;

    assert_eq!(response.status(), 200, "Admin should access admin endpoints");

    println!("✓ Admin role can access admin endpoints");
    Ok(())
}

#[tokio::test]
async fn test_user_role_denied_admin_endpoints() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    // Create user with non-admin role
    let user_token = generate_user_token(3600, "user")?;

    // Try to access admin endpoint
    let url = format!("{}/api/v1/core/admin/users", ctx.base_url);
    let response = ctx.http
        .get(&url)
        .header("Authorization", format!("Bearer {}", user_token))
        .header("x-fingerprint", &fingerprint)
        .send()
        .await?;

    assert_eq!(response.status(), 403, "User role should be denied admin access");

    // Verify denial logged
    TestContext::wait_for_async_processing().await;

    let event_query = "SELECT event_type, severity
                      FROM analytics_events
                      WHERE event_type = 'permission_denied'
                      AND severity = 'warning'
                      ORDER BY created_at DESC
                      LIMIT 1";

    let events = ctx.db.fetch_all(event_query, &[]).await?;
    assert!(!events.is_empty(), "Permission denial not logged");

    // Cleanup
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ User role denied admin endpoints");
    Ok(())
}

#[tokio::test]
async fn test_unauthenticated_denied_protected_endpoints() -> Result<()> {
    let ctx = TestContext::new().await?;

    // Request without token
    let url = format!("{}/api/v1/core/agents", ctx.base_url);
    let response = ctx.http
        .get(&url)
        .send()
        .await?;

    assert_eq!(response.status(), 401, "Unauthenticated should be denied");

    println!("✓ Unauthenticated denied protected endpoints");
    Ok(())
}
```

---

### Test 4: Session Management
**File**: `session_management.rs`

```rust
#[tokio::test]
async fn test_session_created_on_login() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    // Login with token
    let token = generate_admin_token(3600)?;

    let url = format!("{}/api/v1/core/agents", ctx.base_url);
    let response = ctx.http
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("x-fingerprint", &fingerprint)
        .send()
        .await?;

    assert!(response.status().is_success());

    // Verify session created
    TestContext::wait_for_async_processing().await;

    let session_query = DatabaseQueryEnum::FindSessionByFingerprintAny.get(ctx.db.as_ref());
    let sessions = ctx.db.fetch_all(&session_query, &[&fingerprint]).await?;

    assert!(!sessions.is_empty(), "Session not created");

    let session = SessionData::from_json_row(&sessions[0])?;
    assert!(session.user_id.is_some(), "Session should have user_id");

    // Cleanup
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ Session created on login");
    Ok(())
}

#[tokio::test]
async fn test_session_cookie_set() -> Result<()> {
    let ctx = TestContext::new().await?;

    let token = generate_admin_token(3600)?;

    let url = format!("{}/api/v1/core/agents", ctx.base_url);
    let response = ctx.http
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert!(response.status().is_success());

    // Check for Set-Cookie header
    let has_cookie = response.headers()
        .iter()
        .any(|(name, value)| {
            name.as_str() == "set-cookie"
                && value.to_str().unwrap_or("").contains("session")
        });

    assert!(has_cookie, "Session cookie not set");

    println!("✓ Session cookie set on authenticated request");
    Ok(())
}

fn generate_admin_token(expires_in: i64) -> Result<String> {
    // Implementation using jsonwebtoken crate
    // Returns JWT with admin role
    Ok("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...".to_string())
}

fn generate_user_token(expires_in: i64, role: &str) -> Result<String> {
    // Implementation using jsonwebtoken crate
    // Returns JWT with specified role
    Ok("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...".to_string())
}
```

---

## Running the Tests

```bash
# Run all auth tests
cargo test --test auth --all -- --nocapture

# Run specific test
cargo test --test auth test_valid_jwt_token_accepted -- --nocapture
```

## Post-Test Validation

```bash
# Verify user created during OAuth
psql ... -c "SELECT id, oauth_provider, created_at FROM users ORDER BY created_at DESC LIMIT 5;"

# Check auth events
psql ... -c "SELECT event_type, severity, metadata FROM analytics_events
            WHERE event_type LIKE '%auth%' ORDER BY created_at DESC LIMIT 10;"

# Verify sessions
psql ... -c "SELECT session_id, user_id, user_type FROM user_sessions
            WHERE user_id IS NOT NULL ORDER BY started_at DESC LIMIT 5;"
```

---

## Summary

| Test | Coverage | Database Queries |
|------|----------|------------------|
| Valid JWT | Token accepted | endpoint_requests status=200 |
| Expired JWT | Token rejected | analytics_events type=auth_failed |
| OAuth Flow | User created | users table oauth_provider |
| Admin Access | Role check | endpoint_requests status=200 |
| User Denied | Permission denied | endpoint_requests status=403 |
| Session Create | Session table | user_sessions with user_id |
| Session Cookie | HTTP header | set-cookie header present |
| Expiration | Old sessions | user_sessions ended_at < NOW() |

**Target**: All 8 tests fully implemented with JWT generation, role-based access control, and database verification.
