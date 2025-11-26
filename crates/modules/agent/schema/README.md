# A2A Schema Module

Schema module contains A2A protocol-compliant database schema definitions, following the [Agent2Agent Protocol Specification](../docs/a2aspec.txt) exactly.

## A2A Protocol Compliance

### Core A2A Tables
All tables directly implement A2A specification interfaces:

- **`agent_cards`** - Core AgentCard object (A2A spec 5.5)
- **`agent_capabilities`** - AgentCapabilities (A2A spec 5.5.2)
- **`agent_extensions`** - AgentExtension (A2A spec 5.5.2.1)
- **`agent_interfaces`** - AgentInterface (A2A spec 5.5.5)
- **`agent_security_schemes`** - SecurityScheme (A2A spec 5.5.3)
- **`agent_security_requirements`** - Security requirements (A2A spec 5.5)
- **`agent_card_signatures`** - AgentCardSignature (A2A spec 5.5.6)
- **`tasks`** - Task, Message, Artifact interfaces (A2A spec 6.1-6.7)

### SystemPrompt Extension Tables
- **`agent_metadata`** - Deployment-specific fields (port, is_active, etc.)

## Architecture Pattern

### Schema File Organization
- **One Table Per File**: Each A2A entity has its own `.sql` file
- **Spec Compliance**: Field names and types match A2A specification exactly
- **JSON Arrays**: Arrays stored as JSON with `json_valid()` constraints
- **UUID Primary Keys**: Main tables use `uuid TEXT PRIMARY KEY`
- **Foreign Key References**: All related tables reference by `uuid`

### A2A Data Types Mapping
```sql
-- A2A TypeScript → SQLite Mapping
string              → TEXT
boolean             → BOOLEAN (INTEGER 0/1 in SQLite)
Array<T>           → TEXT with JSON array + json_valid() constraint
Record<string, any> → TEXT with JSON object + json_valid() constraint  
number             → INTEGER or REAL
```

## Core Table Patterns

### Main AgentCard Table
```sql
-- agent_cards.sql - A2A AgentCard interface (spec 5.5)
CREATE TABLE IF NOT EXISTS agent_cards (
    -- Primary key as UUID (A2A requirement)
    uuid TEXT PRIMARY KEY NOT NULL,
    
    -- A2A AgentCard required fields
    protocol_version TEXT NOT NULL DEFAULT '0.3.0',
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    url TEXT NOT NULL,
    version TEXT NOT NULL,
    
    -- A2A AgentCard optional array fields (JSON arrays)
    default_input_modes TEXT DEFAULT '[]',
    default_output_modes TEXT DEFAULT '[]',
    
    -- A2A AgentCard optional fields  
    preferred_transport TEXT DEFAULT 'JSONRPC',
    icon_url TEXT,
    documentation_url TEXT,
    provider_organization TEXT,
    provider_url TEXT,
    supports_authenticated_extended_card BOOLEAN DEFAULT FALSE,
    
    -- JSON validation constraints
    CONSTRAINT check_agent_cards_default_input_modes_json 
        CHECK (json_valid(default_input_modes)),
    CONSTRAINT check_agent_cards_default_output_modes_json 
        CHECK (json_valid(default_output_modes)),
    CONSTRAINT check_preferred_transport 
        CHECK (preferred_transport IN ('JSONRPC', 'GRPC', 'HTTP+JSON'))
);
```

### Related A2A Tables
```sql
-- agent_capabilities.sql - A2A AgentCapabilities (spec 5.5.2)
CREATE TABLE IF NOT EXISTS agent_capabilities (
    uuid TEXT PRIMARY KEY, -- 1:1 with agent_cards
    streaming BOOLEAN DEFAULT FALSE,
    push_notifications BOOLEAN DEFAULT FALSE,
    state_transition_history BOOLEAN DEFAULT FALSE,
    FOREIGN KEY (uuid) REFERENCES agent_cards(uuid) ON DELETE CASCADE
);

```

### Task Management (A2A Protocol Objects)
```sql
-- tasks.sql - A2A Task interface (spec 6.1)
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY NOT NULL,           -- Task.id (UUID)
    context_id TEXT NOT NULL,               -- Task.contextId 
    kind TEXT NOT NULL DEFAULT 'task',      -- Task.kind
    
    -- TaskStatus fields (spec 6.2)
    status TEXT NOT NULL,             -- TaskStatus.state
    status_message TEXT DEFAULT '{}',       -- TaskStatus.message (JSON Message object)
    status_timestamp TEXT,                  -- TaskStatus.timestamp
    
    -- Task arrays as JSON per A2A spec
    history TEXT DEFAULT '[]',              -- Task.history (Array<Message>)
    artifacts TEXT DEFAULT '[]',            -- Task.artifacts (Array<Artifact>)
    metadata TEXT DEFAULT '{}',             -- Task.metadata
    
    -- JSON validation constraints
    CONSTRAINT check_tasks_status_message_json CHECK (json_valid(status_message)),
    CONSTRAINT check_tasks_history_json CHECK (json_valid(history)),
    CONSTRAINT check_tasks_artifacts_json CHECK (json_valid(artifacts)),
    CONSTRAINT check_tasks_metadata_json CHECK (json_valid(metadata)),
    CONSTRAINT check_task_state CHECK (status IN (
        'submitted', 'working', 'input-required', 'completed', 
        'canceled', 'failed', 'rejected', 'auth-required', 'unknown'
    ))
);
```

## JSON Field Patterns

### Array Fields
```sql
-- A2A Arrays stored as JSON with validation
tags TEXT DEFAULT '[]',
examples TEXT DEFAULT '[]',
scopes TEXT DEFAULT '[]',

-- Constraints
CONSTRAINT check_tags_json CHECK (json_valid(tags)),
CONSTRAINT check_examples_json CHECK (json_valid(examples)),
CONSTRAINT check_scopes_json CHECK (json_valid(scopes))
```

### Object Fields  
```sql
-- A2A Objects stored as JSON with validation
metadata TEXT DEFAULT '{}',
params TEXT DEFAULT '{}',
status_message TEXT DEFAULT '{}',

-- Constraints
CONSTRAINT check_metadata_json CHECK (json_valid(metadata)),
CONSTRAINT check_params_json CHECK (params IS NULL OR json_valid(params)),
CONSTRAINT check_status_message_json CHECK (json_valid(status_message))
```

### JSON Indexes
```sql
-- Indexes for JSON fields (SQLite 3.38+)
CREATE INDEX IF NOT EXISTS idx_tasks_history_json
    ON tasks(json_extract(history, '$'))
    WHERE json_valid(history);
```

## A2A Specification Compliance

### Required Fields Enforcement
```sql
-- A2A AgentCard required fields (spec 5.5)
protocol_version TEXT NOT NULL DEFAULT '0.3.0',
name TEXT NOT NULL,
description TEXT NOT NULL,
url TEXT NOT NULL,
version TEXT NOT NULL,

-- A2A Task required fields (spec 6.1)  
id TEXT PRIMARY KEY NOT NULL,
context_id TEXT NOT NULL,
kind TEXT NOT NULL DEFAULT 'task',
status TEXT NOT NULL,
```

### Enum Value Constraints
```sql
-- A2A TransportProtocol enum (spec 5.5.5)
CONSTRAINT check_preferred_transport 
    CHECK (preferred_transport IN ('JSONRPC', 'GRPC', 'HTTP+JSON')),

-- A2A TaskState enum (spec 6.3)
CONSTRAINT check_task_state CHECK (status IN (
    'submitted', 'working', 'input-required', 'completed', 
    'canceled', 'failed', 'rejected', 'auth-required', 'unknown'
)),

-- A2A SecurityScheme types (spec 5.5.3)
CONSTRAINT check_scheme_type 
    CHECK (scheme_type IN ('apiKey', 'http', 'oauth2', 'openIdConnect', 'mutualTLS'))
```

### Foreign Key Relationships
```sql
-- All A2A related tables reference agent_cards by uuid
FOREIGN KEY (uuid) REFERENCES agent_cards(uuid) ON DELETE CASCADE,

-- Task relationships
FOREIGN KEY (extension_id) REFERENCES agent_extensions(id) ON DELETE CASCADE
```

## Performance Optimization

### Index Strategy
```sql
-- Primary access patterns
CREATE INDEX IF NOT EXISTS idx_agent_cards_name ON agent_cards(name);
CREATE INDEX IF NOT EXISTS idx_agent_cards_url ON agent_cards(url);
CREATE INDEX IF NOT EXISTS idx_tasks_context_id ON tasks(context_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
```

### Audit Triggers
```sql
-- Auto-update timestamp trigger (following users module pattern)
CREATE TRIGGER IF NOT EXISTS tasks_updated_at
    AFTER UPDATE ON tasks
    FOR EACH ROW
    WHEN NEW.updated_at = OLD.updated_at
    BEGIN
        UPDATE tasks SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;
```

## SystemPrompt Extensions

### Agent Metadata (Non-A2A)
```sql
-- agent_metadata.sql - SystemPrompt deployment fields
CREATE TABLE IF NOT EXISTS agent_metadata (
    uuid TEXT PRIMARY KEY, -- 1:1 with agent_cards (A2A v0.3.0 compliant)

    -- SystemPrompt deployment fields
    port INTEGER NOT NULL CHECK(port > 0 AND port <= 65535) UNIQUE,
    is_enabled BOOLEAN DEFAULT TRUE,
    is_primary BOOLEAN DEFAULT FALSE,
    system_prompt TEXT,
    mcp_servers TEXT DEFAULT '[]' CHECK (json_valid(mcp_servers)),

    -- Audit fields
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (uuid) REFERENCES agent_cards(uuid) ON DELETE CASCADE
);
```

## Design Guidelines

### A2A Protocol Fidelity
1. **Exact Specification Match**: Field names, types, and constraints match A2A spec
2. **No Missing Fields**: All A2A required and optional fields represented
3. **JSON for Complex Types**: Arrays and objects stored as validated JSON
4. **Enum Constraints**: All A2A enums enforced with CHECK constraints
5. **Foreign Key Integrity**: Proper relationships between A2A entities

### Data Integrity
1. **JSON Validation**: All JSON fields have `json_valid()` constraints
2. **Non-NULL Defaults**: JSON fields default to valid empty values (`[]`, `{}`)
3. **Referential Integrity**: Foreign keys with CASCADE deletes
4. **Audit Trails**: Timestamps and triggers for data changes

### Performance
1. **Strategic Indexes**: Based on A2A query patterns and access methods
2. **JSON Indexes**: For searchable JSON array/object fields  
3. **Composite Indexes**: For common multi-field queries
4. **Partial Indexes**: With WHERE clauses for filtered data sets

## Testing A2A Schema

### Schema Validation
```rust
#[cfg(test)]
mod a2a_schema_tests {
    use sqlx::SqlitePool;
    
    #[tokio::test]
    async fn test_a2a_agent_card_creation() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        
        // Apply A2A schema
        let schema_sql = include_str!("../schema/agent_cards.sql");
        sqlx::query(schema_sql).execute(&pool).await.unwrap();
        
        // Test A2A required fields
        let result = sqlx::query(
            "INSERT INTO agent_cards (uuid, name, description, url, version) 
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind("550e8400-e29b-41d4-a716-446655440000")
        .bind("Test Agent")
        .bind("A test A2A agent")
        .bind("https://test.example.com/a2a")
        .bind("1.0.0")
        .execute(&pool).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test] 
    async fn test_a2a_json_validation() {
        let pool = create_a2a_test_schema().await;
        
        // Test valid JSON array on tasks
        let result = sqlx::query(
            "INSERT INTO tasks (id, uuid, context_id, status, history)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind("task-test-1")
        .bind("550e8400-e29b-41d4-a716-446655440000")
        .bind("context-1")
        .bind("pending")
        .bind(r#"[{"step": "start", "timestamp": "2023-01-01T00:00:00Z"}]"#) // Valid JSON array
        .execute(&pool).await;
        assert!(result.is_ok());

        // Test invalid JSON should fail
        let result = sqlx::query(
            "INSERT INTO tasks (id, uuid, context_id, status, history)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind("task-test-2")
        .bind("550e8400-e29b-41d4-a716-446655440001")
        .bind("context-2")
        .bind("pending")
        .bind("invalid json") // Invalid JSON
        .execute(&pool).await;
        assert!(result.is_err());
    }
}
```

## References

- [Agent2Agent Protocol Specification](../docs/a2aspec.txt)
- [A2A TypeScript Types](../types/src/types.ts)
- [Users Schema Module](../../users/src/schema/README.md) (Pattern Reference)
- [SQLite JSON Extensions](https://sqlite.org/json1.html)
- [Module Architecture Guide](../MODULE.md)