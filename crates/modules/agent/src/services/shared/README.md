# Shared Service Infrastructure

Common utilities, traits, and types shared across all agent services. Provides foundation for consistent error handling, configuration management, authentication, and service interfaces.

## Module Structure

```
shared/
├── mod.rs          # Module exports
├── error.rs        # Service error types
├── config.rs       # Configuration types
├── traits.rs       # Service lifecycle traits
├── auth.rs         # Authentication & authorization
├── retry.rs        # Retry logic with exponential backoff
├── timeout.rs      # Timeout utilities
└── utility.rs      # Service utilities (endpoint parsing, ID generation, validation)
```

## Core Modules

### `error.rs` - Service Error Types

**Purpose**: Unified error handling across all agent services

**Exports**:
- `ServiceError` - Comprehensive error enum covering all service failure modes
- `Result<T>` - Type alias for `Result<T, ServiceError>`

**Error Categories**:
```rust
pub enum ServiceError {
    Database(String),           // Database operation failures
    Repository(String),         // Repository layer errors
    Network(String),            // Network/HTTP errors
    Authentication(String),     // Auth failures
    Authorization(String),      // Permission denied
    Validation(String, String), // Input validation (field, reason)
    NotFound(String),           // Resource not found
    ServiceUnavailable(String), // Service down/unreachable
    Timeout(u64),              // Operation timeout (milliseconds)
    Configuration(String, String), // Config errors (key, reason)
    Conflict(String),          // Resource conflict
    Internal(String),          // Internal errors
    Logging(String),           // Logging failures
    Capacity(String),          // Resource limits exceeded
}
```

**Usage**:
```rust
use crate::services::shared::{ServiceError, Result};

fn fetch_agent(id: &str) -> Result<Agent> {
    repository.get(id)?
        .ok_or_else(|| ServiceError::NotFound(format!("Agent {}", id)))
}
```

### `config.rs` - Configuration Types

**Purpose**: Service configuration structures with validation

**Exports**:
- `ServiceConfiguration` - General service config (timeouts, retries, connections)
- `RuntimeConfiguration` - Agent runtime configuration
- `RuntimeConfigurationBuilder` - Builder pattern for runtime config
- `ConnectionConfiguration` - Connection pooling settings
- `ConfigValidation` - Trait for config validation
- `AgentServiceConfig` - Agent-specific configuration

**Example**:
```rust
use crate::services::shared::{RuntimeConfiguration, RuntimeConfigurationBuilder};

let config = RuntimeConfigurationBuilder::new(
    "agent-uuid".to_string(),
    "my-agent".to_string()
)
.port(9000)
.require_auth()
.build();
```

### `traits.rs` - Service Lifecycle Traits

**Purpose**: Standard interfaces for service management

**Exports**:
- `ServiceLifecycle` - Core lifecycle management (initialize, start, stop)
- `Service` - Extended service interface (capabilities, version)

**Interface**:
```rust
#[async_trait]
pub trait ServiceLifecycle: Send + Sync {
    type Config: ConfigValidation;

    async fn initialize(config: Self::Config) -> Result<Self> where Self: Sized;
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    fn name(&self) -> &str;
}

#[async_trait]
pub trait Service: ServiceLifecycle {
    fn capabilities(&self) -> Vec<String>;
    fn version(&self) -> &str;
}
```

### `auth.rs` - Authentication & Authorization

**Purpose**: JWT validation and user authentication

**Exports**:
- `JwtValidator` - JWT token validator
- `Claims` - JWT claims structure
- `AuthenticatedUser` - Validated user info
- `extract_bearer_token()` - Parse Authorization header
- `AuthConfig` - Authentication configuration
- `OAuthConfig` - OAuth2 configuration

**Usage**:
```rust
use crate::services::shared::auth::{JwtValidator, extract_bearer_token};

let validator = JwtValidator::new(jwt_secret);
let token = extract_bearer_token(auth_header)?;
let claims = validator.validate_token(token)?;
let user = AuthenticatedUser::from(claims);
```

### `retry.rs` - Retry Logic

**Purpose**: Exponential backoff retry for transient failures

**Exports**:
- `retry_operation()` - Retry with configurable backoff
- `retry_operation_with_backoff()` - Simplified retry with defaults
- `RetryConfiguration` - Retry parameters

**Example**:
```rust
use crate::services::shared::retry::{retry_operation_with_backoff, RetryConfiguration};

let result = retry_operation_with_backoff(
    || async { fetch_remote_data().await },
    3,  // max attempts
    Duration::from_millis(100)  // initial delay
).await?;
```

### `timeout.rs` - Timeout Utilities

**Purpose**: Operation timeout enforcement

**Exports**:
- `execute_with_timeout()` - Execute operation with timeout
- `execute_with_custom_timeout()` - Execute with specific timeout type
- `TimeoutConfiguration` - Timeout settings
- `TimeoutType` - Timeout categories (Connect, Read, Write, Default)

**Example**:
```rust
use crate::services::shared::timeout::{execute_with_timeout, TimeoutConfiguration};

let result = execute_with_timeout(
    Duration::from_secs(30),
    fetch_data()
).await?;
```

### `utility.rs` - Service Utilities

**Purpose**: Common service helper functions

**Exports**:
- `parse_service_endpoint()` - Parse URL into components
- `ServiceEndpoint` - Structured endpoint (scheme, host, port, path)
- `generate_unique_service_id()` - Generate UUID-based service ID
- `validate_service_name()` - Validate service name format

**Example**:
```rust
use crate::services::shared::utility::{parse_service_endpoint, generate_unique_service_id};

let endpoint = parse_service_endpoint("https://api.example.com:8080/v1")?;
println!("Host: {}, Port: {}", endpoint.host, endpoint.port);

let service_id = generate_unique_service_id("my-service");
```

## Module Re-exports

The `mod.rs` provides convenient top-level access to commonly used types:

```rust
pub use error::{ServiceError, Result};
pub type ServiceResult<T> = Result<T>;

pub use config::{
    ServiceConfiguration,
    RuntimeConfiguration,
    RuntimeConfigurationBuilder,
    ConnectionConfiguration,
    ConfigValidation,
    AgentServiceConfig
};

pub use traits::{ServiceLifecycle, Service};

pub use auth::{
    JwtValidator,
    Claims,
    AuthenticatedUser,
    extract_bearer_token,
    AuthConfig,
    OAuthConfig
};
```

Access other modules directly:
```rust
use crate::services::shared::retry::retry_operation;
use crate::services::shared::timeout::execute_with_timeout;
use crate::services::shared::utility::parse_service_endpoint;
```

## Design Principles

1. **No Inline Comments**: Code is self-documenting through clear naming
2. **Single Responsibility**: Each module has one focused purpose
3. **Repository Pattern**: Database queries belong in repository layer (not utilities)
4. **Error Clarity**: Explicit, actionable error types with context
5. **Type Safety**: Strong types prevent runtime errors
6. **Async First**: All I/O operations are async

## Dependencies

### Internal
- `crate::repository::RepositoryError` - Repository error conversions

### External
- `thiserror` - Error derivation
- `async-trait` - Async trait support
- `serde` - Serialization
- `sqlx` - Database (error conversions only)
- `reqwest` - HTTP client (error conversions only)
- `jsonwebtoken` - JWT validation
- `uuid` - Unique ID generation
- `url` - URL parsing
- `tokio` - Async runtime

## Testing Guidelines

### Unit Tests
- Test error conversions (sqlx::Error → ServiceError)
- Test configuration validation
- Test retry logic with mocked failures
- Test timeout behavior
- Test JWT validation
- Test utility functions (parsing, validation)

### Integration Tests
- Test trait implementations with real services
- Test authentication flow
- Test retry with actual network calls
- Test timeout with real operations

## Migration Notes

**Changes from original design**:
1. ✅ Deleted `discovery.rs` (dead code with broken imports)
2. ✅ Deleted `plan.md` (outdated planning doc)
3. ✅ Merged `trait.rs` + `lifecycle.rs` → `traits.rs` (reduced fragmentation)
4. ✅ Moved `resolve_agent_uuid` to `AgentRepository::resolve_uuid()` (repository pattern)
5. ✅ Moved `AuthConfig` + `OAuthConfig` from config.rs to auth.rs (better cohesion)
6. ✅ Kept `utility.rs` focused on service utilities (removed DB queries)

**Benefits**:
- Eliminated dead code
- Enforced repository pattern
- Reduced file fragmentation (7 meaningful files vs 11 scattered files)
- Clear separation of concerns
- Accurate documentation
