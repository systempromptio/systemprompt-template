# OAuth Module

## Purpose

Provides OAuth 2.0 authorization server functionality including client credential management, authorization flows, refresh token handling, WebAuthn support, and service-to-service authentication.

## Architecture

### Repositories

The module uses the repository pattern for all database operations:

- **ClientRepository** - OAuth client CRUD operations (insert, get, update, delete, list)
- **OAuthRepository** - Authorization code and token management, plus authenticated user retrieval
- **AnalyticsRepository** - Client usage analytics and error tracking
- **ServiceIdentityRepository** - Service-to-service authentication and identity management
- **WebAuthnRepository** - WebAuthn credential management for passwordless authentication

### Key Features

- **OAuth 2.0 Authorization Code Flow** - Standard authorization code grant for web applications
- **Refresh Token Management** - Long-lived tokens for obtaining new access tokens
- **Client Credentials** - Secure client secret management with hashing
- **WebAuthn Support** - FIDO2/WebAuthn passwordless authentication
- **Role-Based Access Control (RBAC)** - Fine-grained permissions with roles and scopes
- **Service Identity** - Machine-to-machine authentication for service integrations

### Database Tables

The OAuth module manages the following tables:

- `oauth_clients` - Registered OAuth clients with configuration
- `oauth_client_redirect_uris` - Allowed redirect URIs per client
- `oauth_client_grant_types` - Grant types supported by client
- `oauth_client_response_types` - Response types supported by client
- `oauth_client_scopes` - Scopes allowed per client
- `oauth_client_contacts` - Contact information for client admins
- `oauth_auth_codes` - Temporary authorization codes (short-lived)
- `oauth_refresh_tokens` - Refresh tokens for obtaining new access tokens
- `service_identities` - Service-to-service authentication identities
- `webauthn_credentials` - FIDO2 credentials for passwordless auth
- `webauthn_challenges` - Temporary challenges for WebAuthn verification

### Query Architecture

All database queries use the `DatabaseQueryEnum` pattern for type safety:

- Queries are defined in `queries/` directory as SQL files
- Each query has SQLite and PostgreSQL variants
- The enum maps to files via `models/queries/oauth.rs`
- All database operations go through repositories (never direct SQL)

#### Cleanup Operations

The module provides client maintenance operations:

- `delete_unused_clients()` - Remove clients that have never been used
- `list_unused_clients()` - List clients marked for potential removal
- `delete_stale_clients()` - Remove clients unused for extended period
- `list_stale_clients()` - List clients approaching deletion threshold
- `delete_inactive_clients()` - Remove marked inactive clients
- `list_inactive_clients()` - List clients marked inactive
- `deactivate_old_test_clients()` - Deactivate test clients by age

#### Analytics Operations

Track client usage metrics:

- `get_client_analytics()` - Aggregated usage metrics for all clients
- `get_client_analytics_by_id()` - Usage metrics for specific client
- `get_client_errors()` - Error statistics across all clients
- `get_client_errors_by_id()` - Error metrics for specific client

## Module Structure

```
crates/modules/oauth/
├── src/
│   ├── api/                 - HTTP endpoints and routes
│   ├── models/              - Data structures and validation
│   ├── repository/          - Database operations layer
│   │   ├── oauth.rs         - Main OAuth operations
│   │   ├── clients/         - Client management
│   │   ├── analytics.rs     - Usage analytics
│   │   ├── service_identity.rs - Service authentication
│   │   └── webauthn.rs      - WebAuthn credentials
│   ├── services/            - Business logic (uses repositories)
│   └── queries/             - SQL query files
│       ├── clients/         - Client CRUD queries
│       ├── oauth/           - Token & auth code queries
│       ├── analytics/       - Analytics queries
│       ├── service/         - Service identity queries
│       └── webauthn/        - WebAuthn queries
└── schema/                  - Database table definitions
    ├── oauth_*.sql          - SQLite table schemas
    └── postgres/            - PostgreSQL variants
```

## Database Query Enum Integration

The `GetAuthenticatedUser` variant (moved from `GetOAuthUser`):

- **Location** - `DatabaseQueryEnum::GetAuthenticatedUser` (Users module)
- **Purpose** - Fetch user by ID with roles and email
- **SQL** - `crates/modules/users/src/queries/core/get_authenticated_user.sql`
- **Used By** - OAuth repository to get full user details

## Usage Examples

### Creating an OAuth Client

```rust
let repo = OAuthRepository::new(db_pool);
let client = repo.create_client(
    "client_id_123",
    "hashed_secret",
    "My Application",
    &["https://app.example.com/callback"],
    Some(&["authorization_code"]),
    Some(&["code", "id_token"]),
    &["openid", "profile"],
    Some("private_key_jwt"),
    Some("https://app.example.com"),
    None,  // no logo URI
    Some(&["admin@example.com"]),
).await?;
```

### Getting Authenticated User

```rust
let repo = OAuthRepository::new(db_pool);
let auth_user = repo.get_authenticated_user("user_id_123").await?;
println!("User: {}", auth_user.name);
```

### Listing Analytics

```rust
let repo = AnalyticsRepository::new(db_pool);
let all_analytics = repo.get_client_analytics().await?;
let client_specific = repo.get_client_analytics_by_id("client_123").await?;
```

## Testing

The module includes tests for:

- Client CRUD operations
- Authorization code flow
- Token management
- WebAuthn credential handling
- Service identity authentication

Run tests with:
```bash
cargo test -p systemprompt-core-oauth
```

## Security Considerations

- **Client Secrets** - Always hashed using bcrypt, never stored plaintext
- **Authorization Codes** - Short-lived (15 minutes), single-use only
- **Refresh Tokens** - Long-lived but can be revoked
- **RBAC** - All operations check user roles and scopes
- **WebAuthn** - FIDO2 compliant, resistant to phishing
- **Cleanup** - Stale credentials are automatically cleaned up

## Performance Notes

- Client operations are indexed by `client_id` for fast lookups
- Token queries use compound indexes (client_id + created_at)
- Analytics queries may benefit from materialized views for high-volume deployments
- Connection pooling recommended for production use

## Future Improvements

- [ ] Implement OpenID Connect (OIDC) support
- [ ] Add consent screen customization
- [ ] Support dynamic client registration
- [ ] Implement device authorization flow
- [ ] Add token introspection endpoint
- [ ] Performance optimization for analytics queries
