# Services Module

Services module provides business logic orchestration and complex operations that span multiple repositories or require coordination between different domain concerns.

## Architecture Pattern

### Service Layer Pattern
- **Business Logic**: Orchestrates complex operations involving multiple repositories
- **Transaction Coordination**: Manages database transactions across operations
- **External Integration**: Handles communication with external services
- **Domain Rules**: Enforces business rules and domain constraints

### File Structure
```
services/
├── mod.rs                    # Service exports and module organization
├── {{entity}}_service.rs     # Core {{entity}} business operations
├── auth_service.rs           # Authentication and authorization
├── notification_service.rs   # Email/notification handling
└── integration/              # External service integrations
    ├── mod.rs               # Integration exports
    ├── email_service.rs     # Email provider integration
    └── storage_service.rs   # File storage integration
```

## Service Implementation Pattern

### Core Service Structure
```rust
//! {{Entity}} service - Business logic orchestration

use anyhow::{Result, Context};
use sqlx::SqlitePool;
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::models::{{{Entity}}, Create{{Entity}}Request, Update{{Entity}}Request};
use crate::repository::{{Entity}}Repository;
use super::auth_service::AuthService;
use super::notification_service::NotificationService;

/// {{Entity}} service for business logic orchestration
pub struct {{Entity}}Service {
    repository: {{Entity}}Repository,
    auth_service: AuthService,
    notification_service: NotificationService,
    pool: SqlitePool,
}

impl {{Entity}}Service {
    /// Create new {{entity}} service instance
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            repository: {{Entity}}Repository::new(pool.clone()),
            auth_service: AuthService::new(pool.clone()),
            notification_service: NotificationService::new(),
            pool,
        }
    }
    
    /// Create {{entity}} with full business logic
    pub async fn create_{{entity}}(&self, request: Create{{Entity}}Request) -> Result<{{Entity}}> {
        info!("🔧 Creating {{entity}} with business logic: {}", request.name);
        
        // Start transaction for complex operation
        let mut tx = self.pool.begin().await
            .context("Failed to start transaction")?;
        
        // 1. Validate business rules
        self.validate_{{entity}}_creation(&request).await?;
        
        // 2. Check for conflicts
        if let Some(_existing) = self.repository.find_by_email(&request.email).await? {
            return Err(anyhow::anyhow!("{{Entity}} with email '{}' already exists", request.email));
        }
        
        // 3. Create {{entity}} entity
        let {{entity}} = self.repository.create_{{entity}}(request.clone()).await
            .context("Failed to create {{entity}} in repository")?;
        
        // 4. Initialize {{entity}} profile/settings
        self.initialize_{{entity}}_profile(&{{entity}}).await
            .context("Failed to initialize {{entity}} profile")?;
        
        // 5. Send welcome notification (async, don't block on failure)
        if let Err(e) = self.send_welcome_notification(&{{entity}}).await {
            warn!("Failed to send welcome notification to {}: {}", {{entity}}.email, e);
            // Don't fail the entire operation for notification failure
        }
        
        // 6. Commit transaction
        tx.commit().await
            .context("Failed to commit {{entity}} creation transaction")?;
        
        info!("✅ {{Entity}} '{}' created successfully with full business logic", {{entity}}.name);
        Ok({{entity}})
    }
    
    /// Update {{entity}} with business logic and validation
    pub async fn update_{{entity}}(
        &self, 
        {{entity}}_uuid: &str, 
        request: Update{{Entity}}Request
    ) -> Result<{{Entity}}> {
        info!("🔧 Updating {{entity}} {} with business logic", {{entity}}_uuid);
        
        // 1. Fetch current {{entity}}
        let current_{{entity}} = self.repository.find_by_id({{entity}}_uuid).await?
            .ok_or_else(|| anyhow::anyhow!("{{Entity}} not found: {}", {{entity}}_uuid))?;
        
        // 2. Validate business rules for update
        self.validate_{{entity}}_update(&current_{{entity}}, &request).await?;
        
        // 3. Check for email conflicts (if email is being changed)
        if let Some(ref new_email) = request.email {
            if new_email != &current_{{entity}}.email {
                if let Some(_existing) = self.repository.find_by_email(new_email).await? {
                    return Err(anyhow::anyhow!("{{Entity}} with email '{}' already exists", new_email));
                }
                
                // Reset email verification if email changed
                // This would be handled by the repository update
            }
        }
        
        // 4. Update {{entity}}
        let updated_{{entity}} = self.repository.update_{{entity}}({{entity}}_uuid, request).await
            .context("Failed to update {{entity}} in repository")?;
        
        // 5. Handle side effects (notifications, audit logs, etc.)
        if let Err(e) = self.handle_{{entity}}_update_side_effects(&current_{{entity}}, &updated_{{entity}}).await {
            warn!("Side effects failed for {{entity}} update {}: {}", {{entity}}_uuid, e);
        }
        
        info!("✅ {{Entity}} '{}' updated successfully", updated_{{entity}}.name);
        Ok(updated_{{entity}})
    }
    
    /// Deactivate {{entity}} with business logic
    pub async fn deactivate_{{entity}}(&self, {{entity}}_uuid: &str, reason: &str) -> Result<()> {
        info!("🔧 Deactivating {{entity}} {} for reason: {}", {{entity}}_uuid, reason);
        
        // 1. Fetch {{entity}}
        let {{entity}} = self.repository.find_by_id({{entity}}_uuid).await?
            .ok_or_else(|| anyhow::anyhow!("{{Entity}} not found: {}", {{entity}}_uuid))?;
        
        // 2. Validate deactivation is allowed
        if {{entity}}.status.to_string() == "deleted" {
            return Err(anyhow::anyhow!("Cannot deactivate deleted {{entity}}"));
        }
        
        // 3. Begin transaction for complex deactivation
        let mut tx = self.pool.begin().await?;
        
        // 4. Update {{entity}} status
        self.repository.update_{{entity}}_status({{entity}}_uuid, "inactive").await?;
        
        // 5. Invalidate active sessions
        self.auth_service.invalidate_{{entity}}_sessions({{entity}}_uuid).await?;
        
        // 6. Send deactivation notification
        if let Err(e) = self.send_deactivation_notification(&{{entity}}, reason).await {
            warn!("Failed to send deactivation notification: {}", e);
        }
        
        // 7. Commit transaction
        tx.commit().await?;
        
        info!("✅ {{Entity}} '{}' deactivated successfully", {{entity}}.name);
        Ok(())
    }
    
    /// Private: Validate {{entity}} creation business rules
    async fn validate_{{entity}}_creation(&self, request: &Create{{Entity}}Request) -> Result<()> {
        // Business rule validations
        if request.name.len() < 3 {
            return Err(anyhow::anyhow!("{{Entity}} name must be at least 3 characters"));
        }
        
        if !request.email.contains('@') || !request.email.contains('.') {
            return Err(anyhow::anyhow!("Invalid email format"));
        }
        
        // Check against business rules (e.g., blacklisted domains)
        if let Some(domain) = request.email.split('@').nth(1) {
            if self.is_domain_blacklisted(domain).await? {
                return Err(anyhow::anyhow!("Email domain '{}' is not allowed", domain));
            }
        }
        
        Ok(())
    }
    
    /// Private: Initialize {{entity}} profile and settings
    async fn initialize_{{entity}}_profile(&self, {{entity}}: &{{Entity}}) -> Result<()> {
        info!("🔧 Initializing profile for {{entity}}: {}", {{entity}}.name);
        
        // Create default preferences
        let default_preferences = serde_json::json!({
            "theme": "light",
            "notifications": {
                "email": true,
                "browser": true
            },
            "timezone": "UTC"
        });
        
        // This would typically update a preferences/profile table
        // For now, we'll update the {{entity}} preferences field
        self.repository.update_{{entity}}_preferences(
            &{{entity}}.uuid.to_string(),
            &default_preferences.to_string()
        ).await?;
        
        info!("✅ Profile initialized for {{entity}}: {}", {{entity}}.name);
        Ok(())
    }
    
    /// Private: Send welcome notification
    async fn send_welcome_notification(&self, {{entity}}: &{{Entity}}) -> Result<()> {
        self.notification_service.send_welcome_email(
            &{{entity}}.email,
            {{entity}}.display_name_or_fallback()
        ).await
    }
    
    /// Private: Check if domain is blacklisted
    async fn is_domain_blacklisted(&self, domain: &str) -> Result<bool> {
        // This would check against a blacklist (database, external service, etc.)
        const BLACKLISTED_DOMAINS: &[&str] = &[
            "tempmail.org",
            "10minutemail.com",
            "example.com" // For testing
        ];
        
        Ok(BLACKLISTED_DOMAINS.contains(&domain))
    }
    
    // Additional private methods for business logic...
}
```

## Authentication Service

### Auth Service Pattern
```rust
//! Authentication service - Auth and session management

use anyhow::{Result, Context};
use sqlx::SqlitePool;
use tracing::{info, warn, error};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

use crate::models::{{Entity}};
use crate::repository::{{Entity}}Repository;

pub struct AuthService {
    repository: {{Entity}}Repository,
    pool: SqlitePool,
}

impl AuthService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            repository: {{Entity}}Repository::new(pool.clone()),
            pool,
        }
    }
    
    /// Authenticate {{entity}} and create session
    pub async fn authenticate_{{entity}}(
        &self,
        identifier: &str, // email or username
        password: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>
    ) -> Result<AuthResult> {
        info!("🔐 Authenticating {{entity}}: {}", identifier);
        
        // 1. Find {{entity}} by email or username
        let {{entity}} = if identifier.contains('@') {
            self.repository.find_by_email(identifier).await?
        } else {
            self.repository.find_by_name(identifier).await?
        };
        
        let {{entity}} = {{entity}}.ok_or_else(|| anyhow::anyhow!("Invalid credentials"))?;
        
        // 2. Check {{entity}} status
        if !{{entity}}.can_authenticate() {
            return Err(anyhow::anyhow!("Account is not active"));
        }
        
        // 3. Verify password (this would use proper password hashing)
        if !self.verify_password(password, &{{entity}}.password_hash).await? {
            warn!("Failed authentication attempt for {{entity}}: {}", {{entity}}.name);
            return Err(anyhow::anyhow!("Invalid credentials"));
        }
        
        // 4. Create session
        let session = self.create_session(
            &{{entity}},
            ip_address,
            user_agent
        ).await?;
        
        // 5. Update last login
        if let Err(e) = self.update_last_login(&{{entity}}.uuid.to_string()).await {
            warn!("Failed to update last login for {}: {}", {{entity}}.name, e);
        }
        
        info!("✅ {{Entity}} '{}' authenticated successfully", {{entity}}.name);
        Ok(AuthResult {
            {{entity}},
            session,
            expires_at: session.expires_at,
        })
    }
    
    /// Validate session and get {{entity}}
    pub async fn validate_session(&self, session_id: &str) -> Result<Option<{{Entity}}>> {
        // 1. Find session
        let session = self.find_session(session_id).await?;
        let session = match session {
            Some(s) => s,
            None => return Ok(None),
        };
        
        // 2. Check expiration
        if session.expires_at < Utc::now() {
            self.invalidate_session(session_id).await?;
            return Ok(None);
        }
        
        // 3. Get {{entity}}
        let {{entity}} = self.repository.find_by_id(&session.{{entity}}_uuid).await?;
        
        // 4. Extend session if needed
        if let Some(ref {{entity}}) = {{entity}} {
            if self.should_extend_session(&session).await? {
                let _ = self.extend_session(session_id).await;
            }
        }
        
        Ok({{entity}})
    }
    
    /// Invalidate all sessions for {{entity}}
    pub async fn invalidate_{{entity}}_sessions(&self, {{entity}}_uuid: &str) -> Result<()> {
        info!("🔐 Invalidating all sessions for {{entity}}: {}", {{entity}}_uuid);
        
        const INVALIDATE_SQL: &str = r#"
            DELETE FROM {{table}}_sessions 
            WHERE {{entity}}_uuid = ?
        "#;
        
        sqlx::query(INVALIDATE_SQL)
            .bind({{entity}}_uuid)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    // Private methods for session management...
    async fn create_session(
        &self,
        {{entity}}: &{{Entity}},
        ip_address: Option<&str>,
        user_agent: Option<&str>
    ) -> Result<Session> {
        let session_id = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::hours(24);
        
        const CREATE_SESSION_SQL: &str = r#"
            INSERT INTO {{table}}_sessions (
                id, {{entity}}_uuid, expires_at, ip_address, user_agent, session_data
            ) VALUES (?, ?, ?, ?, ?, ?)
        "#;
        
        let session_data = serde_json::json!({
            "created_at": Utc::now(),
            "ip_address": ip_address,
            "user_agent": user_agent
        }).to_string();
        
        sqlx::query(CREATE_SESSION_SQL)
            .bind(&session_id)
            .bind({{entity}}.uuid.to_string())
            .bind(&expires_at)
            .bind(ip_address)
            .bind(user_agent)
            .bind(&session_data)
            .execute(&self.pool)
            .await?;
        
        Ok(Session {
            id: session_id,
            {{entity}}_uuid: {{entity}}.uuid.to_string(),
            expires_at,
            session_data,
        })
    }
}

#[derive(Debug)]
pub struct AuthResult {
    pub {{entity}}: {{Entity}},
    pub session: Session,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct Session {
    pub id: String,
    pub {{entity}}_uuid: String,
    pub expires_at: DateTime<Utc>,
    pub session_data: String,
}
```

## Notification Service

### Notification Pattern
```rust
//! Notification service - Email and messaging

use anyhow::{Result, Context};
use tracing::{info, warn, error};
use serde_json::json;

pub struct NotificationService {
    // Email provider would be configured here
    email_client: Option<EmailClient>,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            email_client: EmailClient::from_config(),
        }
    }
    
    /// Send welcome email to new {{entity}}
    pub async fn send_welcome_email(&self, email: &str, name: &str) -> Result<()> {
        info!("📧 Sending welcome email to: {}", email);
        
        let template_data = json!({
            "name": name,
            "login_url": "https://systemprompt.dev/login",
            "support_email": "support@systemprompt.dev"
        });
        
        self.send_template_email(
            email,
            "Welcome to SystemPrompt",
            "welcome_{{entity}}",
            template_data
        ).await
    }
    
    /// Send {{entity}} deactivation notification
    pub async fn send_deactivation_notification(&self, {{entity}}: &{{Entity}}, reason: &str) -> Result<()> {
        info!("📧 Sending deactivation notification to: {}", {{entity}}.email);
        
        let template_data = json!({
            "name": {{entity}}.display_name_or_fallback(),
            "reason": reason,
            "support_email": "support@systemprompt.dev"
        });
        
        self.send_template_email(
            &{{entity}}.email,
            "Account Deactivated",
            "{{entity}}_deactivated",
            template_data
        ).await
    }
    
    /// Send password reset email
    pub async fn send_password_reset_email(&self, email: &str, reset_token: &str) -> Result<()> {
        info!("📧 Sending password reset email to: {}", email);
        
        let reset_url = format!("https://systemprompt.dev/reset-password?token={}", reset_token);
        
        let template_data = json!({
            "reset_url": reset_url,
            "expires_in": "24 hours",
            "support_email": "support@systemprompt.dev"
        });
        
        self.send_template_email(
            email,
            "Password Reset Request",
            "password_reset",
            template_data
        ).await
    }
    
    /// Private: Send templated email
    async fn send_template_email(
        &self,
        to_email: &str,
        subject: &str,
        template_name: &str,
        template_data: serde_json::Value
    ) -> Result<()> {
        match &self.email_client {
            Some(client) => {
                client.send_template_email(to_email, subject, template_name, template_data).await
            }
            None => {
                warn!("Email client not configured, skipping email to: {}", to_email);
                Ok(())
            }
        }
    }
}

struct EmailClient {
    // Email provider configuration
}

impl EmailClient {
    fn from_config() -> Option<Self> {
        // Initialize email client from configuration
        // Return None if not configured
        None
    }
    
    async fn send_template_email(
        &self,
        _to: &str,
        _subject: &str, 
        _template: &str,
        _data: serde_json::Value
    ) -> Result<()> {
        // Implement actual email sending
        Ok(())
    }
}
```

## Integration Services

### External Service Pattern
```rust
//! External service integrations

pub mod email_service;
pub mod storage_service;
pub mod analytics_service;

use anyhow::Result;

/// Configuration for external services
#[derive(Debug, Clone)]
pub struct IntegrationConfig {
    pub email: Option<EmailConfig>,
    pub storage: Option<StorageConfig>,
    pub analytics: Option<AnalyticsConfig>,
}

#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub provider: String,
    pub api_key: String,
    pub from_address: String,
}

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub provider: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Debug, Clone)]
pub struct AnalyticsConfig {
    pub provider: String,
    pub tracking_id: String,
}

/// Initialize all integration services
pub fn initialize_integrations(config: IntegrationConfig) -> Result<IntegratedServices> {
    Ok(IntegratedServices {
        email: config.email.map(email_service::EmailProvider::new),
        storage: config.storage.map(storage_service::StorageProvider::new),
        analytics: config.analytics.map(analytics_service::AnalyticsProvider::new),
    })
}

pub struct IntegratedServices {
    pub email: Option<email_service::EmailProvider>,
    pub storage: Option<storage_service::StorageProvider>,
    pub analytics: Option<analytics_service::AnalyticsProvider>,
}
```

## Service Testing

### Service Test Patterns
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    
    async fn create_test_service() -> {{Entity}}Service {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        setup_test_schema(&pool).await.unwrap();
        {{Entity}}Service::new(pool)
    }
    
    #[tokio::test]
    async fn test_create_{{entity}}_with_business_logic() {
        let service = create_test_service().await;
        
        let request = Create{{Entity}}Request {
            name: "test_{{entity}}".to_string(),
            email: "test@example.com".to_string(),
            full_name: Some("Test {{Entity}}".to_string()),
            // ... other fields
        };
        
        let result = service.create_{{entity}}(request).await;
        assert!(result.is_ok());
        
        let {{entity}} = result.unwrap();
        assert_eq!({{entity}}.name, "test_{{entity}}");
        assert_eq!({{entity}}.status.to_string(), "active");
    }
    
    #[tokio::test]
    async fn test_business_rule_validation() {
        let service = create_test_service().await;
        
        let invalid_request = Create{{Entity}}Request {
            name: "ab".to_string(), // Too short
            email: "invalid-email".to_string(),
            // ... other fields
        };
        
        let result = service.create_{{entity}}(invalid_request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least 3 characters"));
    }
}
```

## Best Practices

### Service Design
1. **Single Responsibility**: Each service handles one domain area
2. **Transaction Management**: Use transactions for multi-step operations
3. **Error Handling**: Provide meaningful error messages with context
4. **Business Rules**: Centralize business logic validation
5. **External Integration**: Handle external service failures gracefully

### Code Quality
1. **Dependency Injection**: Pass dependencies through constructor
2. **Async Operations**: Use async/await for I/O operations
3. **Logging**: Structured logging for all service operations
4. **Testing**: Comprehensive unit and integration tests
5. **Documentation**: Clear documentation for business logic

### Security
1. **Input Validation**: Validate all inputs at service boundaries
2. **Authorization**: Check permissions before operations
3. **Audit Logging**: Log all significant business operations
4. **Secret Management**: Use secure configuration for API keys
5. **Rate Limiting**: Implement rate limiting for external calls

## References

- [Business Logic Patterns](https://martinfowler.com/eaaCatalog/)
- [Transaction Patterns](https://docs.rs/sqlx/latest/sqlx/struct.Transaction.html)
- [Repository Patterns](../repository/README.md)
- [Model Patterns](../models/README.md)
- [Module Architecture Guide](../../MODULE.md)