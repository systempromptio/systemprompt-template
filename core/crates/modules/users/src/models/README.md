# Models Module

Models define the data structures for the module organized by domain context, following clean architecture principles with clear separation between different usage contexts.

## Architecture Pattern

### Domain-Based Organization
Models are organized by usage context rather than entity type:
- **Entity Models**: Core domain entities and their state representations
- **Command Models**: Request/response structures for CLI operations
- **Web Models**: DTOs and API models for HTTP endpoints

### File Structure
```
models/
├── mod.rs                    # Re-exports and module organization
├── entity/                   # Core domain entities
│   ├── mod.rs               # Entity re-exports
│   ├── {{entity}}.rs        # Main entity and status enums
│   └── session.rs           # Session/auth models (if needed)
├── command/                  # Command request/response models
│   ├── mod.rs               # Command re-exports
│   └── {{entity}}.rs        # Create/Update request structures
└── web/                      # Web API models
    └── mod.rs               # Web-specific DTOs and API models
```

## Entity Models (`entity/`)

### Core Entity Pattern
```rust
//! Core {{entity}} domain entity

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Core {{Entity}} domain entity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct {{Entity}} {
    pub uuid: Uuid,
    pub name: String,
    pub email: String,
    pub full_name: Option<String>,
    pub display_name: Option<String>,
    pub status: {{Entity}}Status,
    pub email_verified: bool,
    pub roles: String,                    // JSON string - parsed by helper methods
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl {{Entity}} {
    /// Parse roles from JSON string to Vec<String>
    pub fn parsed_roles(&self) -> Vec<String> {
        serde_json::from_str(&self.roles)
            .unwrap_or_else(|_| vec!["{{default_role}}".to_string()])
    }
    
    /// Check if {{entity}} has specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.parsed_roles().contains(&role.to_string())
    }
    
    /// Get display name or fallback to username
    pub fn display_name_or_fallback(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.name)
    }
}
```

### Status Enums
```rust
/// {{Entity}} status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "lowercase")]
pub enum {{Entity}}Status {
    Active,
    Inactive,
    Suspended,
    Pending,
    Deleted,
}

impl {{Entity}}Status {
    /// Check if status represents an active state
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
    
    /// Check if status allows login/operations
    pub fn can_authenticate(&self) -> bool {
        matches!(self, Self::Active | Self::Inactive)
    }
}

impl std::fmt::Display for {{Entity}}Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Inactive => write!(f, "inactive"),
            Self::Suspended => write!(f, "suspended"),
            Self::Pending => write!(f, "pending"),
            Self::Deleted => write!(f, "deleted"),
        }
    }
}

impl std::str::FromStr for {{Entity}}Status {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            "suspended" => Ok(Self::Suspended),
            "pending" => Ok(Self::Pending),
            "deleted" => Ok(Self::Deleted),
            _ => Err(format!("Invalid {{entity}} status: {}", s)),
        }
    }
}
```

## Command Models (`command/`)

### Request Structures
```rust
//! Command request/response models for {{entity}} operations

use serde::{Deserialize, Serialize};

/// Request to create a new {{entity}}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Create{{Entity}}Request {
    pub name: String,
    pub email: String,
    pub full_name: Option<String>,
    pub display_name: Option<String>,
    pub roles: Option<Vec<String>>,
    pub status: Option<String>,
}

impl Create{{Entity}}Request {
    /// Validate the create request
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.name.trim().is_empty() {
            errors.push("Name cannot be empty".to_string());
        }
        
        if self.email.trim().is_empty() {
            errors.push("Email cannot be empty".to_string());
        } else if !self.email.contains('@') {
            errors.push("Invalid email format".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Request to update an existing {{entity}}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Update{{Entity}}Request {
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub display_name: Option<String>,
    pub status: Option<String>,
    pub roles: Option<Vec<String>>,
}

impl Update{{Entity}}Request {
    /// Check if request has any fields to update
    pub fn has_updates(&self) -> bool {
        self.email.is_some() ||
        self.full_name.is_some() ||
        self.display_name.is_some() ||
        self.status.is_some() ||
        self.roles.is_some()
    }
}
```

## Web Models (`web/`)

### API DTOs
```rust
//! Web API models and DTOs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// API request to create {{entity}} via REST endpoint
#[derive(Debug, Deserialize)]
pub struct Create{{Entity}}ApiRequest {
    pub name: String,
    pub email: String,
    pub full_name: Option<String>,
    pub display_name: Option<String>,
    pub roles: Option<Vec<String>>,
}

/// API request to update {{entity}} via REST endpoint  
#[derive(Debug, Deserialize)]
pub struct Update{{Entity}}ApiRequest {
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub display_name: Option<String>,
    pub status: Option<String>,
    pub roles: Option<Vec<String>>,
}

/// Query parameters for listing {{entities}}
#[derive(Debug, Deserialize)]
pub struct List{{Entity}}sQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub search: Option<String>,
    pub status: Option<String>,
    pub role: Option<String>,
}

impl Default for List{{Entity}}sQuery {
    fn default() -> Self {
        Self {
            limit: Some(20),
            offset: Some(0),
            search: None,
            status: None,
            role: None,
        }
    }
}
```

## Model Conversions

### Between Model Types
```rust
// Convert command model to entity
impl From<Create{{Entity}}Request> for {{Entity}} {
    fn from(request: Create{{Entity}}Request) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: request.name,
            email: request.email,
            full_name: request.full_name,
            display_name: request.display_name,
            status: {{Entity}}Status::Active,
            email_verified: false,
            roles: serde_json::to_string(&request.roles.unwrap_or_default())
                .unwrap_or_else(|_| "[]".to_string()),
            avatar_url: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

// Convert web model to command model
impl From<Create{{Entity}}ApiRequest> for Create{{Entity}}Request {
    fn from(api_request: Create{{Entity}}ApiRequest) -> Self {
        Self {
            name: api_request.name,
            email: api_request.email,
            full_name: api_request.full_name,
            display_name: api_request.display_name,
            roles: api_request.roles,
            status: Some("active".to_string()),
        }
    }
}
```

## Validation Patterns

### Input Validation
```rust
/// Validation trait for model validation
pub trait Validate {
    type Error;
    
    fn validate(&self) -> Result<(), Self::Error>;
}

impl Validate for Create{{Entity}}Request {
    type Error = Vec<String>;
    
    fn validate(&self) -> Result<(), Self::Error> {
        let mut errors = Vec::new();
        
        // Required field validation
        if self.name.trim().is_empty() {
            errors.push("Name is required".to_string());
        }
        
        // Format validation
        if !self.email.contains('@') {
            errors.push("Invalid email format".to_string());
        }
        
        // Business rule validation
        if self.name.len() > 255 {
            errors.push("Name too long (max 255 characters)".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

## Serialization Patterns

### JSON Handling
```rust
use serde_json;

impl {{Entity}} {
    /// Serialize {{entity}} to JSON string
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    
    /// Deserialize {{entity}} from JSON string
    pub fn from_json_string(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
    
    /// Convert to JSON value for flexible usage
    pub fn to_json_value(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}
```

## Database Integration

### SQLx Integration
```rust
use sqlx::{FromRow, Type};

// Enable automatic row mapping
#[derive(FromRow)]
pub struct {{Entity}} {
    // ... fields match database columns
}

// Custom enum handling for database
#[derive(Type)]
#[sqlx(type_name = "TEXT")]
pub enum {{Entity}}Status {
    // ... enum variants
}
```

## Testing Patterns

### Test Data Builders
```rust
#[cfg(test)]
pub mod test_builders {
    use super::*;
    
    pub struct {{Entity}}Builder {
        name: String,
        email: String,
        status: {{Entity}}Status,
        // ... other fields
    }
    
    impl {{Entity}}Builder {
        pub fn new() -> Self {
            Self {
                name: "test_{{entity}}".to_string(),
                email: "test@example.com".to_string(),
                status: {{Entity}}Status::Active,
                // ... default values
            }
        }
        
        pub fn with_name(mut self, name: &str) -> Self {
            self.name = name.to_string();
            self
        }
        
        pub fn with_status(mut self, status: {{Entity}}Status) -> Self {
            self.status = status;
            self
        }
        
        pub fn build(self) -> {{Entity}} {
            {{Entity}} {
                uuid: Uuid::new_v4(),
                name: self.name,
                email: self.email,
                status: self.status,
                // ... other field assignments
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }
        }
    }
    
    impl Default for {{Entity}}Builder {
        fn default() -> Self {
            Self::new()
        }
    }
}
```

## Best Practices

### Model Design
1. **Single Responsibility**: Each model serves one specific purpose
2. **Domain-Driven**: Models reflect business domain concepts
3. **Immutability**: Prefer immutable fields where possible
4. **Type Safety**: Use enums for constrained values
5. **Validation**: Include validation methods for data integrity

### Code Quality
1. **Documentation**: Clear documentation for all public types
2. **Consistent Naming**: Follow Rust naming conventions
3. **Error Handling**: Proper error types for validation
4. **Serialization**: Consistent serde annotations
5. **Testing**: Comprehensive test coverage with builders

### Security
1. **Input Sanitization**: Validate all user inputs
2. **No Secrets**: Never include passwords or tokens in models
3. **Safe Defaults**: Use secure defaults for optional fields
4. **Audit Fields**: Include created_at/updated_at for tracking

## Module Integration

Models integrate with other modules through:
- **Repository Layer**: Database persistence via sqlx traits
- **Commands Layer**: CLI operations via command models
- **Web Layer**: HTTP endpoints via web DTOs
- **Validation**: Consistent validation patterns across layers

## References

- [Serde Documentation](https://serde.rs/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Module Architecture Guide](../../MODULE.md)
- [Repository Patterns](../repository/README.md)
- [Command Patterns](../commands/README.md)