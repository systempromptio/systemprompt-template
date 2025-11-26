# Queries Module

Queries module contains all SQL query files organized by purpose, following the strict separation principle where no SQL is written inline in Rust code.

## Architecture Pattern

### SQL Separation Principle
- **No Inline SQL**: All SQL queries must be in separate `.sql` files
- **Compile-Time Loading**: Queries loaded via `include_str!` macro
- **Purpose-Based Organization**: Queries organized by operation type and purpose
- **Consistent Naming**: Clear, descriptive file names matching operations

### File Structure
```
queries/
├── core/                         # Core CRUD operations
│   ├── create_{{entity}}.sql     # INSERT statements
│   ├── find_by_{{field}}.sql     # SELECT by specific field
│   ├── list_{{entity}}s.sql      # SELECT with pagination
│   ├── update_{{entity}}.sql     # UPDATE statements
│   ├── delete_{{entity}}_soft.sql # Soft delete (status update)
│   └── delete_{{entity}}_hard.sql # Hard delete (actual removal)
└── seed/                         # Seed data queries
    └── default_{{entity}}s.sql   # Initial data population
```

## Core Operations (`core/`)

### Create Operations
```sql
-- create_{{entity}}.sql - Insert new {{entity}} record
INSERT INTO {{table}} (
    uuid,
    name, 
    email,
    full_name,
    display_name,
    status,
    email_verified,
    roles,
    avatar_url,
    created_at,
    updated_at
) VALUES (
    ?, -- uuid
    ?, -- name
    ?, -- email  
    ?, -- full_name
    ?, -- display_name
    ?, -- status
    ?, -- email_verified
    ?, -- roles (JSON string)
    ?, -- avatar_url
    CURRENT_TIMESTAMP, -- created_at
    CURRENT_TIMESTAMP  -- updated_at
);
```

### Find Operations
```sql
-- find_by_id.sql - Find {{entity}} by UUID
SELECT 
    uuid,
    name,
    email,
    full_name,
    display_name,
    status,
    email_verified,
    roles,
    avatar_url,
    created_at,
    updated_at
FROM {{table}}
WHERE uuid = ?
  AND status != 'deleted';

-- find_by_name.sql - Find {{entity}} by name
SELECT 
    uuid,
    name,
    email,
    full_name,
    display_name,
    status,
    email_verified,
    roles,
    avatar_url,
    created_at,
    updated_at
FROM {{table}}
WHERE name = ?
  AND status != 'deleted';

-- find_by_email.sql - Find {{entity}} by email
SELECT 
    uuid,
    name,
    email,
    full_name,
    display_name,
    status,
    email_verified,
    roles,
    avatar_url,
    created_at,
    updated_at
FROM {{table}}
WHERE email = ?
  AND status != 'deleted';
```

### List Operations
```sql
-- list_{{entity}}s.sql - List {{entities}} with pagination
SELECT 
    uuid,
    name,
    email,
    full_name,
    display_name,
    status,
    email_verified,
    roles,
    avatar_url,
    created_at,
    updated_at
FROM {{table}}
WHERE status != 'deleted'
ORDER BY created_at DESC
LIMIT ? OFFSET ?;

-- count_{{entity}}s.sql - Count total {{entities}}
SELECT COUNT(*) as total_count
FROM {{table}}
WHERE status != 'deleted';

-- list_{{entity}}s_by_status.sql - List {{entities}} filtered by status
SELECT 
    uuid,
    name,
    email,
    full_name,
    display_name,
    status,
    email_verified,
    roles,
    avatar_url,
    created_at,
    updated_at
FROM {{table}}
WHERE status = ?
ORDER BY created_at DESC
LIMIT ? OFFSET ?;

-- list_{{entity}}s_by_role.sql - List {{entities}} with specific role
SELECT 
    uuid,
    name,
    email,
    full_name,
    display_name,
    status,
    email_verified,
    roles,
    avatar_url,
    created_at,
    updated_at
FROM {{table}}
WHERE status != 'deleted'
  AND roles LIKE '%' || ? || '%'  -- JSON contains role
ORDER BY created_at DESC
LIMIT ? OFFSET ?;
```

### Search Operations
```sql
-- search_{{entity}}s.sql - Search {{entities}} by term
SELECT 
    uuid,
    name,
    email,
    full_name,
    display_name,
    status,
    email_verified,
    roles,
    avatar_url,
    created_at,
    updated_at
FROM {{table}}
WHERE status != 'deleted'
  AND (
    name LIKE '%' || ? || '%' 
    OR email LIKE '%' || ? || '%'
    OR full_name LIKE '%' || ? || '%'
    OR display_name LIKE '%' || ? || '%'
  )
ORDER BY 
  CASE WHEN name LIKE ? || '%' THEN 1 ELSE 2 END,
  created_at DESC
LIMIT ? OFFSET ?;

-- search_{{entity}}s_count.sql - Count search results
SELECT COUNT(*) as total_count
FROM {{table}}
WHERE status != 'deleted'
  AND (
    name LIKE '%' || ? || '%' 
    OR email LIKE '%' || ? || '%'
    OR full_name LIKE '%' || ? || '%'
    OR display_name LIKE '%' || ? || '%'
  );
```

### Update Operations
```sql
-- update_{{entity}}.sql - Update {{entity}} fields
UPDATE {{table}} 
SET 
    email = COALESCE(?, email),
    full_name = COALESCE(?, full_name),
    display_name = COALESCE(?, display_name), 
    status = COALESCE(?, status),
    roles = COALESCE(?, roles),
    avatar_url = COALESCE(?, avatar_url),
    updated_at = CURRENT_TIMESTAMP
WHERE uuid = ?;

-- update_{{entity}}_status.sql - Update {{entity}} status only
UPDATE {{table}}
SET 
    status = ?,
    updated_at = CURRENT_TIMESTAMP
WHERE uuid = ?;

-- update_{{entity}}_email_verification.sql - Update email verification
UPDATE {{table}}
SET 
    email_verified = ?,
    updated_at = CURRENT_TIMESTAMP
WHERE uuid = ?;

-- update_{{entity}}_roles.sql - Update {{entity}} roles
UPDATE {{table}}
SET 
    roles = ?,
    updated_at = CURRENT_TIMESTAMP
WHERE uuid = ?;
```

### Delete Operations
```sql
-- delete_{{entity}}_soft.sql - Soft delete (set status to deleted)
UPDATE {{table}}
SET 
    status = 'deleted',
    updated_at = CURRENT_TIMESTAMP
WHERE uuid = ?;

-- delete_{{entity}}_hard.sql - Hard delete (permanent removal)
DELETE FROM {{table}}
WHERE uuid = ?;

-- delete_{{entity}}s_by_status.sql - Bulk delete by status
DELETE FROM {{table}}
WHERE status = ?;
```

## Seed Data (`seed/`)

### Default Data
```sql
-- default_{{entity}}s.sql - Default {{entities}} for development
INSERT OR IGNORE INTO {{table}} (
    uuid,
    name,
    email,
    full_name,
    display_name,
    status,
    email_verified,
    roles,
    created_at,
    updated_at
) VALUES 
-- System Administrator
(
    '550e8400-e29b-41d4-a716-446655440000',
    'admin',
    'admin@systemprompt.dev',
    'System Administrator',
    'Admin',
    'active',
    1,
    '["admin", "{{default_role}}"]',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
),
-- Demo {{Entity}}
(
    '550e8400-e29b-41d4-a716-446655440001',
    'demo_{{entity}}',
    'demo@example.com',
    'Demo {{Entity}}',
    'Demo',
    'active',
    1,
    '["{{default_role}}"]',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
),
-- Test {{Entity}} (inactive)
(
    '550e8400-e29b-41d4-a716-446655440002',
    'test_{{entity}}',
    'test@example.com',
    'Test {{Entity}}',
    'Test',
    'inactive',
    0,
    '["{{default_role}}"]',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);
```

## Query Loading Pattern

### Rust Integration
```rust
//! Query loading examples from repository operations

use anyhow::{Result, Context};
use sqlx::SqlitePool;

/// Find {{entity}} by ID - loads query from file
pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<{{Entity}}>> {
    // Load SQL query from file at compile time
    const FIND_BY_ID_SQL: &str = include_str!("../queries/core/find_by_id.sql");
    
    let row = sqlx::query(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(pool)
        .await
        .context("Failed to find {{entity}} by ID")?;
    
    match row {
        Some(r) => Ok(Some(super::mappers::map_{{entity}}_row(r)?)),
        None => Ok(None),
    }
}

/// List {{entities}} with pagination - loads query from file
pub async fn list_{{entities}}(pool: &SqlitePool, limit: i32, offset: i32) -> Result<Vec<{{Entity}}>> {
    const LIST_SQL: &str = include_str!("../queries/core/list_{{entities}}.sql");
    
    let rows = sqlx::query(LIST_SQL)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .context("Failed to list {{entities}}")?;
    
    super::mappers::map_{{entity}}_rows(rows)
}

/// Search {{entities}} - loads query from file with multiple bindings
pub async fn search_{{entities}}(
    pool: &SqlitePool,
    search_term: &str,
    limit: i32,
    offset: i32
) -> Result<Vec<{{Entity}}>> {
    const SEARCH_SQL: &str = include_str!("../queries/core/search_{{entities}}.sql");
    
    let rows = sqlx::query(SEARCH_SQL)
        .bind(search_term)  // name LIKE
        .bind(search_term)  // email LIKE
        .bind(search_term)  // full_name LIKE
        .bind(search_term)  // display_name LIKE
        .bind(search_term)  // name prefix priority
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .context("Failed to search {{entities}}")?;
    
    super::mappers::map_{{entity}}_rows(rows)
}
```

## Query Conventions

### Naming Conventions
- **Format**: `{{action}}_{{entity}}_{{qualifier}}.sql`
- **Examples**:
  - `create_user.sql`
  - `find_by_email.sql` 
  - `list_users_by_status.sql`
  - `update_user_roles.sql`
  - `delete_user_soft.sql`

### Parameter Binding
- **Use `?` placeholders**: Always use parameterized queries
- **Order matters**: Parameters bound in order they appear in SQL
- **Document bindings**: Comment parameter order in complex queries
- **No string concatenation**: Never build SQL strings dynamically

### Query Structure
```sql
-- query_name.sql - Brief description of what this query does
-- Parameters:
--   1. param1 (TYPE) - Description of first parameter
--   2. param2 (TYPE) - Description of second parameter

SELECT 
    column1,
    column2,
    column3
FROM table_name t
WHERE t.condition1 = ?      -- param1
  AND t.condition2 = ?      -- param2
  AND t.status != 'deleted' -- Always exclude soft-deleted records
ORDER BY t.created_at DESC
LIMIT ? OFFSET ?;           -- param3, param4
```

## Performance Guidelines

### Query Optimization
1. **Use Indexes**: Ensure WHERE clause columns are indexed
2. **Limit Results**: Always use LIMIT for list operations
3. **Avoid SELECT ***: Select only needed columns
4. **Filter Early**: Apply WHERE conditions before JOINs when possible
5. **Use EXPLAIN**: Analyze query execution plans

### Index Strategy
```sql
-- Performance indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_{{table}}_name ON {{table}}(name);
CREATE INDEX IF NOT EXISTS idx_{{table}}_email ON {{table}}(email);
CREATE INDEX IF NOT EXISTS idx_{{table}}_status ON {{table}}(status);
CREATE INDEX IF NOT EXISTS idx_{{table}}_created_at ON {{table}}(created_at);
CREATE INDEX IF NOT EXISTS idx_{{table}}_status_created ON {{table}}(status, created_at);
```

### Query Patterns
```sql
-- Good: Efficient pagination
SELECT columns 
FROM table 
WHERE conditions 
ORDER BY indexed_column 
LIMIT ? OFFSET ?;

-- Good: Composite index usage
WHERE status = ? AND created_at > ?

-- Avoid: Unindexed pattern matching
WHERE column LIKE '%' || ? || '%'

-- Better: Prefix matching when possible
WHERE column LIKE ? || '%'
```

## Testing SQL Queries

### Query Testing
```rust
#[cfg(test)]
mod query_tests {
    use super::*;
    use sqlx::SqlitePool;
    
    async fn create_test_pool() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        // Initialize schema
        setup_test_schema(&pool).await.unwrap();
        pool
    }
    
    #[tokio::test]
    async fn test_find_by_id_query() {
        let pool = create_test_pool().await;
        
        // Test data setup
        let test_id = "test-uuid-123";
        setup_test_{{entity}}(&pool, test_id).await.unwrap();
        
        // Test query
        let result = find_by_id(&pool, test_id).await.unwrap();
        assert!(result.is_some());
        
        let {{entity}} = result.unwrap();
        assert_eq!({{entity}}.uuid.to_string(), test_id);
    }
    
    #[tokio::test] 
    async fn test_search_query_performance() {
        let pool = create_test_pool().await;
        
        // Setup large dataset
        setup_large_test_dataset(&pool).await.unwrap();
        
        let start = std::time::Instant::now();
        let results = search_{{entities}}(&pool, "test", 10, 0).await.unwrap();
        let duration = start.elapsed();
        
        // Performance assertion
        assert!(duration.as_millis() < 100); // Query should complete in <100ms
        assert!(!results.is_empty());
    }
}
```

## Best Practices

### SQL Quality
1. **Readable Formatting**: Consistent indentation and line breaks
2. **Clear Comments**: Document complex queries and parameter usage
3. **Standard SQL**: Use SQLite-compatible SQL syntax
4. **Error Handling**: Handle constraint violations gracefully
5. **Transaction Safety**: Design queries to work within transactions

### File Organization
1. **Logical Grouping**: Group related queries in subdirectories
2. **Consistent Naming**: Follow established naming conventions
3. **Version Control**: Track SQL changes with meaningful commit messages
4. **Documentation**: Include README files for complex query sets
5. **Testing**: Test queries with realistic data volumes

### Security
1. **Parameterized Queries**: Always use parameter binding
2. **Input Validation**: Validate parameters before query execution
3. **Constraint Checks**: Use database constraints for data integrity
4. **Access Control**: Apply appropriate database permissions
5. **Audit Logging**: Include audit fields (created_at, updated_at)

## References

- [SQLite SQL Syntax](https://sqlite.org/lang.html)
- [SQLx Query Documentation](https://docs.rs/sqlx/)
- [Repository Patterns](../repository/README.md)
- [Database Schema](../database/README.md)
- [Module Architecture Guide](../../MODULE.md)