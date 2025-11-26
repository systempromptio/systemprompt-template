# Schema Module

Schema module contains database schema definitions in SQL files, following consistent patterns for table creation, constraints, indexes, and triggers.

## Architecture Pattern

### Schema File Organization
- **One Table Per File**: Each table has its own `.sql` file
- **Consistent Naming**: Files named after the table they create
- **Complete Definitions**: Include constraints, indexes, and triggers
- **Idempotent Operations**: Use `IF NOT EXISTS` for safe re-execution

### File Structure
```
schema/
├── {{table}}.sql              # Main entity table
├── {{table}}_sessions.sql     # Session/auth table (if needed)
└── {{table}}_audit.sql        # Audit log table (if needed)
```

## Table Design Patterns

### Standard Entity Table
```sql
-- {{table}}.sql - Main {{entity}} entity table
CREATE TABLE IF NOT EXISTS {{table}} (
    -- Primary Key (UUID as TEXT for SQLite compatibility)
    uuid TEXT PRIMARY KEY,
    
    -- Core Identity Fields
    name TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    
    -- Optional Profile Fields
    full_name TEXT,
    display_name TEXT,
    avatar_url TEXT,
    
    -- Status and Flags
    status TEXT NOT NULL DEFAULT 'active',
    email_verified BOOLEAN NOT NULL DEFAULT 0,
    
    -- JSON Data Fields (stored as TEXT in SQLite)
    roles TEXT NOT NULL DEFAULT '["{{default_role}}"]',
    preferences TEXT DEFAULT '{}',
    
    -- Audit Timestamps
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Data Integrity Constraints
    CONSTRAINT check_{{table}}_status 
        CHECK (status IN ('active', 'inactive', 'suspended', 'pending', 'deleted')),
    CONSTRAINT check_{{table}}_email_format 
        CHECK (email LIKE '%@%'),
    CONSTRAINT check_{{table}}_name_length 
        CHECK (LENGTH(name) >= 2 AND LENGTH(name) <= 255),
    CONSTRAINT check_{{table}}_uuid_format
        CHECK (LENGTH(uuid) = 36)
);
```

### Performance Indexes
```sql
-- Performance Indexes for {{table}}
-- Single column indexes for common queries
CREATE INDEX IF NOT EXISTS idx_{{table}}_name ON {{table}}(name);
CREATE INDEX IF NOT EXISTS idx_{{table}}_email ON {{table}}(email);
CREATE INDEX IF NOT EXISTS idx_{{table}}_status ON {{table}}(status);
CREATE INDEX IF NOT EXISTS idx_{{table}}_created_at ON {{table}}(created_at);
CREATE INDEX IF NOT EXISTS idx_{{table}}_updated_at ON {{table}}(updated_at);

-- Composite indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_{{table}}_status_created 
    ON {{table}}(status, created_at);
CREATE INDEX IF NOT EXISTS idx_{{table}}_status_name 
    ON {{table}}(status, name);

-- JSON field indexes (SQLite 3.38+)
CREATE INDEX IF NOT EXISTS idx_{{table}}_roles_json 
    ON {{table}}(json_extract(roles, '$')) 
    WHERE json_valid(roles);
```

### Audit Triggers
```sql
-- Audit Triggers for {{table}}
-- Automatically update updated_at timestamp
CREATE TRIGGER IF NOT EXISTS trigger_{{table}}_updated_at
    AFTER UPDATE ON {{table}}
    FOR EACH ROW
    WHEN NEW.updated_at = OLD.updated_at
    BEGIN
        UPDATE {{table}} 
        SET updated_at = CURRENT_TIMESTAMP 
        WHERE uuid = NEW.uuid;
    END;

-- Audit log trigger (optional, if audit table exists)
CREATE TRIGGER IF NOT EXISTS trigger_{{table}}_audit_log
    AFTER UPDATE ON {{table}}
    FOR EACH ROW
    WHEN NEW.status != OLD.status OR NEW.email != OLD.email
    BEGIN
        INSERT INTO {{table}}_audit (
            entity_uuid, 
            action, 
            old_values, 
            new_values, 
            changed_by, 
            changed_at
        ) VALUES (
            NEW.uuid,
            'UPDATE',
            json_object(
                'status', OLD.status,
                'email', OLD.email
            ),
            json_object(
                'status', NEW.status,
                'email', NEW.email
            ),
            'system', -- TODO: Get actual user context
            CURRENT_TIMESTAMP
        );
    END;
```

### Session Table Pattern
```sql
-- {{table}}_sessions.sql - Session management table
CREATE TABLE IF NOT EXISTS {{table}}_sessions (
    -- Session Identity
    id TEXT PRIMARY KEY,
    {{entity}}_uuid TEXT NOT NULL,
    
    -- Session Data
    session_data TEXT NOT NULL DEFAULT '{}', -- JSON
    expires_at DATETIME NOT NULL,
    
    -- Tracking Information
    ip_address TEXT,
    user_agent TEXT,
    
    -- Audit Fields
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign Key Constraint
    FOREIGN KEY ({{entity}}_uuid) REFERENCES {{table}}(uuid) ON DELETE CASCADE,
    
    -- Data Integrity
    CONSTRAINT check_session_expires CHECK (expires_at > created_at),
    CONSTRAINT check_session_data_json CHECK (json_valid(session_data))
);

-- Session Indexes
CREATE INDEX IF NOT EXISTS idx_{{table}}_sessions_{{entity}}_uuid 
    ON {{table}}_sessions({{entity}}_uuid);
CREATE INDEX IF NOT EXISTS idx_{{table}}_sessions_expires_at 
    ON {{table}}_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_{{table}}_sessions_created_at 
    ON {{table}}_sessions(created_at);

-- Session Cleanup Trigger
CREATE TRIGGER IF NOT EXISTS trigger_{{table}}_sessions_cleanup
    AFTER INSERT ON {{table}}_sessions
    FOR EACH ROW
    BEGIN
        DELETE FROM {{table}}_sessions 
        WHERE expires_at < CURRENT_TIMESTAMP;
    END;
```

### Audit Table Pattern
```sql
-- {{table}}_audit.sql - Audit log table
CREATE TABLE IF NOT EXISTS {{table}}_audit (
    -- Audit Record Identity
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_uuid TEXT NOT NULL,
    
    -- Action Information
    action TEXT NOT NULL, -- INSERT, UPDATE, DELETE
    table_name TEXT NOT NULL DEFAULT '{{table}}',
    
    -- Change Data
    old_values TEXT, -- JSON
    new_values TEXT, -- JSON
    
    -- Context Information
    changed_by TEXT NOT NULL,
    changed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Optional Request Context
    request_id TEXT,
    ip_address TEXT,
    user_agent TEXT,
    
    -- Data Integrity
    CONSTRAINT check_audit_action CHECK (action IN ('INSERT', 'UPDATE', 'DELETE')),
    CONSTRAINT check_audit_old_values_json CHECK (old_values IS NULL OR json_valid(old_values)),
    CONSTRAINT check_audit_new_values_json CHECK (new_values IS NULL OR json_valid(new_values))
);

-- Audit Indexes
CREATE INDEX IF NOT EXISTS idx_{{table}}_audit_entity_uuid 
    ON {{table}}_audit(entity_uuid);
CREATE INDEX IF NOT EXISTS idx_{{table}}_audit_changed_at 
    ON {{table}}_audit(changed_at);
CREATE INDEX IF NOT EXISTS idx_{{table}}_audit_action 
    ON {{table}}_audit(action);
CREATE INDEX IF NOT EXISTS idx_{{table}}_audit_changed_by 
    ON {{table}}_audit(changed_by);
```

## Design Guidelines

### Field Naming Conventions
```sql
-- Identity Fields
uuid TEXT PRIMARY KEY              -- Primary key as UUID string
name TEXT NOT NULL UNIQUE          -- Human-readable identifier
email TEXT NOT NULL UNIQUE         -- Contact information

-- Profile Fields  
full_name TEXT                     -- Complete name
display_name TEXT                  -- UI display name
avatar_url TEXT                    -- Profile image URL

-- Status Fields
status TEXT NOT NULL DEFAULT 'active'  -- Entity status enum
email_verified BOOLEAN NOT NULL DEFAULT 0  -- Verification flags

-- JSON Fields (stored as TEXT)
roles TEXT DEFAULT '[]'            -- Array of role strings
preferences TEXT DEFAULT '{}'      -- Key-value configuration
metadata TEXT DEFAULT '{}'         -- Additional structured data

-- Audit Fields (required for all tables)
created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
```

### Data Type Standards
```sql
-- SQLite Data Types
TEXT        -- Strings, UUIDs, JSON, enums
INTEGER     -- Numbers, counts, IDs
REAL        -- Decimal numbers
BOOLEAN     -- Use INTEGER (0/1) for SQLite compatibility
DATETIME    -- Use TEXT with ISO 8601 format or INTEGER unix timestamp
JSON        -- Store as TEXT with json_valid() constraints
```

### Constraint Patterns
```sql
-- Primary Key Constraints
uuid TEXT PRIMARY KEY                    -- UUID as primary key
id INTEGER PRIMARY KEY AUTOINCREMENT    -- Auto-incrementing integer

-- Uniqueness Constraints
UNIQUE(name)                            -- Single column unique
UNIQUE(name, email)                     -- Composite unique

-- Check Constraints
CHECK (status IN ('active', 'inactive', 'suspended'))  -- Enum values
CHECK (email LIKE '%@%')                                -- Basic format
CHECK (LENGTH(name) >= 2 AND LENGTH(name) <= 255)      -- Length limits
CHECK (json_valid(metadata))                            -- JSON validation

-- Foreign Key Constraints
FOREIGN KEY ({{entity}}_uuid) REFERENCES {{table}}(uuid) ON DELETE CASCADE
FOREIGN KEY ({{entity}}_uuid) REFERENCES {{table}}(uuid) ON DELETE SET NULL
```

## Schema Evolution

### Migration Strategy
```sql
-- Migration pattern for schema changes
-- Add new columns with DEFAULT values
ALTER TABLE {{table}} ADD COLUMN new_field TEXT DEFAULT 'default_value';

-- Create new indexes
CREATE INDEX IF NOT EXISTS idx_{{table}}_new_field ON {{table}}(new_field);

-- Update existing data (if needed)
UPDATE {{table}} SET new_field = 'calculated_value' WHERE condition;

-- Add constraints after data migration
-- Note: SQLite doesn't support ADD CONSTRAINT, so recreate table if needed
```

### Version Management
```sql
-- Schema version tracking table
CREATE TABLE IF NOT EXISTS schema_versions (
    module_name TEXT PRIMARY KEY,
    version INTEGER NOT NULL,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    checksum TEXT NOT NULL
);

-- Track {{module}} schema version
INSERT OR REPLACE INTO schema_versions (module_name, version, checksum)
VALUES ('{{module}}', 1, 'schema_checksum_here');
```

## Performance Optimization

### Index Strategy
```sql
-- Query Pattern Analysis:
-- 1. Single WHERE conditions → Single column indexes
-- 2. Multi-column WHERE → Composite indexes (most selective first)
-- 3. ORDER BY queries → Include ORDER BY column in index
-- 4. JOIN operations → Foreign key indexes

-- Example: Query "WHERE status = ? ORDER BY created_at DESC"
CREATE INDEX idx_{{table}}_status_created_desc ON {{table}}(status, created_at DESC);

-- Example: Query "WHERE name LIKE ? AND status = ?"
CREATE INDEX idx_{{table}}_status_name ON {{table}}(status, name);
```

### Storage Optimization
```sql
-- Use appropriate storage classes
name TEXT                    -- Variable length strings
status TEXT                  -- Short enum strings (consider INTEGER for high-volume)
created_at INTEGER          -- Unix timestamp (more compact than TEXT datetime)
preferences TEXT            -- JSON data compressed as TEXT

-- Normalize repeated data
-- Instead of storing full role names in JSON:
-- roles TEXT DEFAULT '["administrator", "user"]'
-- Consider: role_ids TEXT DEFAULT '[1,2]' with separate roles table
```

## Security Considerations

### Data Protection
```sql
-- Sensitive data handling
password_hash TEXT NOT NULL           -- Never store plain passwords
salt TEXT NOT NULL                    -- Use unique salt per password
api_key_hash TEXT                     -- Hash API keys
email_normalized TEXT GENERATED ALWAYS AS (lower(trim(email))) VIRTUAL  -- Normalized email

-- Soft delete for data retention
status TEXT CHECK (status IN ('active', 'inactive', 'deleted'))
deleted_at DATETIME                   -- Track deletion time
deleted_by TEXT                       -- Track who deleted
```

### Access Control
```sql
-- Row-level security simulation (SQLite doesn't have RLS)
-- Use views with security context
CREATE VIEW {{table}}_secure AS
SELECT * FROM {{table}}
WHERE status != 'deleted'
  AND (
    -- Add security logic here based on application context
    uuid = current_user_uuid() OR 
    has_permission(current_user_uuid(), '{{table}}_read')
  );
```

## Testing Schema

### Schema Validation
```rust
#[cfg(test)]
mod schema_tests {
    use sqlx::SqlitePool;
    
    #[tokio::test]
    async fn test_schema_creation() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        
        // Apply schema
        let schema_sql = include_str!("../schema/{{table}}.sql");
        sqlx::query(schema_sql).execute(&pool).await.unwrap();
        
        // Verify table exists
        let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='{{table}}'")
            .fetch_one(&pool)
            .await
            .unwrap();
        
        let table_name: String = result.get("name");
        assert_eq!(table_name, "{{table}}");
    }
    
    #[tokio::test]
    async fn test_constraints() {
        let pool = create_test_schema().await;
        
        // Test unique constraint
        let result = sqlx::query("INSERT INTO {{table}} (uuid, name, email) VALUES ('uuid1', 'test', 'test@example.com')")
            .execute(&pool).await;
        assert!(result.is_ok());
        
        let result = sqlx::query("INSERT INTO {{table}} (uuid, name, email) VALUES ('uuid2', 'test', 'different@example.com')")
            .execute(&pool).await;
        assert!(result.is_err()); // Should fail due to unique name constraint
    }
}
```

## Best Practices

### Schema Design
1. **UUID Primary Keys**: Use TEXT UUIDs for distributed systems
2. **Consistent Naming**: Follow snake_case for tables and columns
3. **Audit Fields**: Always include created_at and updated_at
4. **Soft Deletes**: Use status field instead of hard deletes
5. **JSON Validation**: Add json_valid() constraints for JSON columns

### Performance
1. **Strategic Indexes**: Create indexes based on query patterns
2. **Composite Indexes**: Order columns by selectivity (most selective first)
3. **Partial Indexes**: Use WHERE clauses in indexes when applicable
4. **Index Maintenance**: Monitor and drop unused indexes

### Security
1. **Input Validation**: Use CHECK constraints for data integrity
2. **Foreign Keys**: Enforce referential integrity
3. **Sensitive Data**: Hash passwords and API keys
4. **Access Patterns**: Design schema to support access control

## References

- [SQLite Schema Documentation](https://sqlite.org/lang_createtable.html)
- [SQLite Constraints](https://sqlite.org/lang_createtable.html#constraints)
- [SQLite Indexes](https://sqlite.org/lang_createindex.html)
- [Database Module](../database/README.md)
- [Query Patterns](../queries/README.md)
- [Module Architecture Guide](../../MODULE.md)