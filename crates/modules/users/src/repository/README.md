# Repository Module

Repository provides the database access layer following the clean module pattern with private operation implementations and strict separation of SQL queries.

## Architecture Pattern

### Clean Module Pattern
- **Main Repository**: `mod.rs` contains the `{{Entity}}Repository` struct and public methods
- **Private Operations**: Individual database operations in separate files
- **SQL Separation**: All queries in dedicated `.sql` files loaded via `include_str!`
- **No Inline SQL**: Repository methods delegate to operation modules

### File Structure
```
repository/
├── mod.rs                    # Main {{Entity}}Repository implementation + exports
├── mappers.rs               # Database row to model conversions
├── create_{{entity}}.rs     # Create operations + SQL queries
├── find_{{entity}}.rs       # Find operations (by name, ID, etc.)
├── list_{{entity}}s.rs      # List and count operations
├── update_{{entity}}.rs     # Update operations
├── delete_{{entity}}.rs     # Delete operations (soft/hard)
└── search_{{entity}}s.rs    # Search operations with pagination
```

## Implementation Guidelines

### Repository Implementation (`mod.rs`)
```rust
use sqlx::SqlitePool;
use systemprompt_core_traits::domain::repository::Repository;
use crate::models::{{Entity}};

pub struct {{Entity}}Repository {
    pool: SqlitePool,
}

impl {{Entity}}Repository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    
    /// Create operations - delegate to create_{{entity}} module
    pub async fn create_{{entity}}(&self, request: Create{{Entity}}Request) -> Result<{{Entity}}> {
        create_{{entity}}::create_{{entity}}(&self.pool, request).await
    }
    
    /// Find operations - delegate to find_{{entity}} module
    pub async fn find_by_id(&self, id: &str) -> Result<Option<{{Entity}}>> {
        find_{{entity}}::find_by_id(&self.pool, id).await
    }
    
    // ... other delegating methods
}

#[async_trait::async_trait]
impl Repository<{{Entity}}> for {{Entity}}Repository {
    async fn create(&self, entity: Create{{Entity}}Request) -> Result<{{Entity}}> {
        self.create_{{entity}}(entity).await
    }
    // ... other trait methods
}
```

### Operation Module Pattern
Each operation file should follow this structure:
```rust
//! {{Operation}} {{entity}} database operations

use anyhow::{Result, Context};
use sqlx::SqlitePool;
use tracing::{info, warn, error};

use crate::models::{{Entity}};
use super::mappers;

/// {{Operation}} {{entity}} in database
pub async fn {{operation}}_{{entity}}(
    pool: &SqlitePool,
    // ... specific parameters
) -> Result<{{ReturnType}}> {
    info!("📊 {{Operation}} {{entity}}: ...");
    
    // Load SQL query from file
    const SQL: &str = include_str!("../queries/core/{{operation}}_{{entity}}.sql");
    
    // Execute query with proper error context
    let result = sqlx::query(SQL)
        .bind(/* parameters */)
        .fetch_optional(pool)
        .await
        .context("Failed to {{operation}} {{entity}}")?;
    
    // Map database row to domain model
    match result {
        Some(row) => Ok(mappers::map_{{entity}}_row(row)?),
        None => Ok(None),
    }
}
```

## SQL Query Organization

### Query File Location
All SQL queries must be in separate `.sql` files:
```
../queries/
├── core/                           # Core CRUD operations
│   ├── create_{{entity}}.sql       # INSERT statements
│   ├── find_by_{{field}}.sql       # SELECT by specific field
│   ├── list_{{entity}}s.sql        # SELECT with pagination
│   ├── update_{{entity}}.sql       # UPDATE statements
│   ├── delete_{{entity}}_soft.sql  # Soft delete (status update)
│   └── delete_{{entity}}_hard.sql  # Hard delete (actual removal)
└── seed/                           # Seed data queries
    └── default_{{entity}}s.sql     # Initial data population
```

### Query Loading Pattern
```rust
// Load query from file - compile-time inclusion
const FIND_BY_ID_SQL: &str = include_str!("../queries/core/find_by_id.sql");

// Use in database operation
let row = sqlx::query(FIND_BY_ID_SQL)
    .bind(id)
    .fetch_optional(pool)
    .await
    .context("Failed to find {{entity}} by ID")?;
```

## Data Mapping

### Mapper Functions (`mappers.rs`)
```rust
//! Database row to domain model mapping functions

use anyhow::{Result, Context};
use sqlx::Row;
use uuid::Uuid;

use crate::models::{{Entity}};

/// Map database row to {{Entity}} model
pub fn map_{{entity}}_row(row: sqlx::sqlite::SqliteRow) -> Result<{{Entity}}> {
    Ok({{Entity}} {
        uuid: row.try_get::<String, _>("uuid")
            .context("Missing uuid field")?
            .parse::<Uuid>()
            .context("Invalid UUID format")?,
        name: row.try_get("name").context("Missing name field")?,
        // ... other field mappings with error context
        created_at: row.try_get("created_at").context("Missing created_at field")?,
        updated_at: row.try_get("updated_at").context("Missing updated_at field")?,
    })
}

/// Map optional row (for find operations)
pub fn map_optional_{{entity}}_row(row: Option<sqlx::sqlite::SqliteRow>) -> Result<Option<{{Entity}}>> {
    match row {
        Some(r) => Ok(Some(map_{{entity}}_row(r)?)),
        None => Ok(None),
    }
}

/// Map multiple rows (for list operations)
pub fn map_{{entity}}_rows(rows: Vec<sqlx::sqlite::SqliteRow>) -> Result<Vec<{{Entity}}>> {
    rows.into_iter()
        .map(map_{{entity}}_row)
        .collect::<Result<Vec<_>>>()
}
```

## Error Handling

### Error Context Pattern
Always provide meaningful error context:
```rust
// Good: Specific error context
.context("Failed to create {{entity}} with name '{}'", name)?

// Good: Operation-specific context  
.context("Database connection failed during {{operation}}")?

// Bad: Generic error
.context("Database error")?
```

### Result Types
- **Single Entity**: `Result<{{Entity}}>` or `Result<Option<{{Entity}}>>`
- **Multiple Entities**: `Result<Vec<{{Entity}}>>`
- **Count Operations**: `Result<i64>`
- **Existence Checks**: `Result<bool>`
- **Mutations**: `Result<{{Entity}}>` (return updated entity)

## Transaction Management

### Transaction Pattern
```rust
/// Complex operation requiring transaction
pub async fn complex_{{operation}}(
    pool: &SqlitePool,
    // parameters
) -> Result<{{Entity}}> {
    let mut tx = pool.begin().await
        .context("Failed to start transaction")?;
    
    // First operation
    let result1 = sqlx::query(SQL1)
        .bind(param1)
        .execute(&mut *tx)
        .await
        .context("Failed first operation")?;
    
    // Second operation
    let result2 = sqlx::query(SQL2)
        .bind(param2)
        .execute(&mut *tx)
        .await
        .context("Failed second operation")?;
    
    tx.commit().await
        .context("Failed to commit transaction")?;
    
    Ok(final_result)
}
```

## Naming Conventions

### Repository Struct
- **Format**: `{{Entity}}Repository` (PascalCase)
- **Examples**: `UserRepository`, `ServerRepository`, `ToolRepository`

### Method Names
- **Create**: `create_{{entity}}()`, `bulk_create_{{entities}}()`
- **Read**: `find_by_{{field}}()`, `list_{{entities}}()`, `count_{{entities}}()`
- **Update**: `update_{{entity}}()`, `update_{{field}}()`
- **Delete**: `delete_{{entity}}()`, `soft_delete_{{entity}}()`

### File Names
- **Operations**: `{{action}}_{{entity}}.rs` (snake_case)
- **Examples**: `create_user.rs`, `find_user.rs`, `list_users.rs`

## Performance Guidelines

### Query Optimization
1. **Indexed Fields**: Ensure frequently queried fields have database indexes
2. **Pagination**: Always implement LIMIT/OFFSET for list operations
3. **Select Specific**: Avoid `SELECT *`, specify needed columns
4. **Connection Pooling**: Reuse connections via SqlitePool

### Caching Strategy
```rust
// For frequently accessed, rarely changed data
pub async fn find_by_id_cached(&self, id: &str) -> Result<Option<{{Entity}}>> {
    // Check cache first
    if let Some(cached) = self.cache.get(id).await? {
        return Ok(Some(cached));
    }
    
    // Fallback to database
    let entity = self.find_by_id(id).await?;
    
    // Update cache
    if let Some(ref e) = entity {
        self.cache.set(id, e.clone()).await?;
    }
    
    Ok(entity)
}
```

## Testing Requirements

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    
    async fn create_test_pool() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        // Run migrations/schema setup
        setup_test_schema(&pool).await.unwrap();
        pool
    }
    
    #[tokio::test]
    async fn test_create_{{entity}}() {
        let pool = create_test_pool().await;
        let repo = {{Entity}}Repository::new(pool);
        
        let request = Create{{Entity}}Request {
            // ... test data
        };
        
        let result = repo.create_{{entity}}(request).await;
        assert!(result.is_ok());
        
        let {{entity}} = result.unwrap();
        assert_eq!({{entity}}.name, "test_name");
    }
}
```

### Integration Tests
- Test repository with real database connections
- Test transaction rollback scenarios
- Test concurrent access patterns
- Test error handling for invalid data

## Best Practices

### Implementation
1. **Single Responsibility**: One operation type per file
2. **Clean Delegation**: Repository delegates to operation modules
3. **SQL Separation**: No inline SQL, all queries in files
4. **Error Context**: Meaningful error messages with context
5. **Type Safety**: Strong typing for IDs and domain models

### Code Quality
1. **No Business Logic**: Repository only handles data access
2. **Consistent Mapping**: Centralized row-to-model conversion
3. **Transaction Safety**: Proper transaction management
4. **Resource Management**: Connection pooling and cleanup
5. **Documentation**: Clear method and module documentation

### Security
1. **Parameterized Queries**: Never concatenate SQL strings
2. **Input Validation**: Validate parameters before database calls
3. **No Secrets Logging**: Avoid logging sensitive data
4. **SQL Injection Prevention**: Use sqlx bound parameters

## Module Integration

Repository integrates with other modules through:
- **Models Layer**: Domain entities and request structures
- **Commands Layer**: CLI operations call repository methods
- **Web Layer**: HTTP handlers use repository for data access
- **Database Layer**: Schema and migration management

## References

- [SystemPrompt Core Traits](../../../traits/src/domain/repository/)
- [Module Architecture Guide](../../MODULE.md)
- [Database Schema](../database/README.md)
- [SQL Queries](../queries/README.md)