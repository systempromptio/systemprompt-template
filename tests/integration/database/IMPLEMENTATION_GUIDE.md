# Database Tests Implementation Guide

**Status**: All 7 tests are stubs. This guide provides the full implementation plan.

**Goal**: Comprehensive database validation including migrations, constraints, concurrency, and data integrity.

---

## Test Organization (Semantic Breakdown)

Instead of megasize tests, break into focused units:

### Group 1: Connection & Initialization (2 tests)
- `test_postgres_connection_pool_established` - Verify connection pool
- `test_database_schema_initialized` - Verify all tables exist

### Group 2: Migrations (3 tests)
- `test_migrations_run_sequentially` - Verify migration order
- `test_migrations_idempotent` - Can run migrations multiple times safely
- `test_migration_rollback_not_supported` - Document limitation

### Group 3: Constraints (4 tests)
- `test_primary_keys_enforced` - Can't insert duplicate IDs
- `test_foreign_keys_enforced` - Can't insert orphaned records
- `test_unique_constraints_enforced` - Can't insert duplicate unique fields
- `test_not_null_constraints_enforced` - Can't insert NULL in required fields

### Group 4: Data Types (3 tests)
- `test_timestamp_fields_store_correctly` - UTC timestamps work
- `test_json_fields_store_and_retrieve` - JSON serialization
- `test_numeric_fields_store_correctly` - Integers, floats, decimals

### Group 5: Concurrency (3 tests)
- `test_concurrent_inserts_succeed` - Multiple writers don't conflict
- `test_concurrent_reads_dont_block_writes` - MVCC works
- `test_transaction_isolation_level` - Verify transaction isolation

### Group 6: Integrity Checks (2 tests)
- `test_no_orphaned_records_exist` - Every foreign key points to valid row
- `test_all_required_columns_have_values` - No NULL in required fields

---

## Implementation Template

```rust
use crate::common::*;
use anyhow::Result;

#[tokio::test]
async fn test_postgres_connection_pool_established() -> Result<()> {
    // PHASE 1: Setup
    let ctx = TestContext::new().await?;

    // PHASE 2: Test specific behavior
    // Execute a simple query to verify connection
    let query = "SELECT 1 as test_value";
    let rows = ctx.db.fetch_all(&query, &[]).await?;

    // PHASE 3: Assertions
    assert!(!rows.is_empty(), "Connection pool not responding");
    let test_val = rows[0].get("test_value")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    assert_eq!(test_val, 1, "SQL query returned unexpected value");

    // PHASE 4: Log result
    println!("✓ Connection pool established and responding");
    Ok(())
}
```

**Key Pattern**:
1. Create TestContext (handles environment, database connection)
2. Execute test-specific behavior
3. Make assertions on results
4. Log success
5. Cleanup is automatic (TestCleanup in context)

---

## Database Validation Queries

Run these SQL queries AFTER each test to verify database state:

### After Connection Test
```sql
-- Verify connection parameters
SELECT current_user, current_database(), now();

-- Expected output:
-- current_user: systemprompt
-- current_database: systemprompt_dev
-- now: 2025-11-11 ...
```

### After Schema Initialization Test
```sql
-- List all tables in public schema
SELECT table_name
FROM information_schema.tables
WHERE table_schema = 'public'
ORDER BY table_name;

-- Expected tables:
-- - analytics_events
-- - user_sessions
-- - agent_tasks
-- - task_messages
-- - ai_requests
-- - endpoint_requests
-- - markdown_content
-- - services
-- - and more...

-- Count total tables
SELECT COUNT(*) as table_count
FROM information_schema.tables
WHERE table_schema = 'public';

-- Expected: >= 15 tables
```

### After Migration Test
```sql
-- Check all migrations completed
SELECT migration_version, installed_on
FROM schema_migrations
ORDER BY migration_version DESC;

-- Expected: All migrations listed with dates

-- Verify schema_migrations table itself exists
SELECT EXISTS (
    SELECT 1
    FROM information_schema.tables
    WHERE table_schema = 'public'
    AND table_name = 'schema_migrations'
) as migration_table_exists;
```

### After Primary Key Test
```sql
-- Verify primary keys exist
SELECT table_name, constraint_name, constraint_type
FROM information_schema.table_constraints
WHERE table_schema = 'public'
AND constraint_type = 'PRIMARY KEY'
ORDER BY table_name;

-- Expected: 15+ PRIMARY KEY constraints

-- Verify user_sessions has primary key
SELECT constraint_name
FROM information_schema.table_constraints
WHERE table_schema = 'public'
AND table_name = 'user_sessions'
AND constraint_type = 'PRIMARY KEY';

-- Expected: session_id_pk (or similar)
```

### After Foreign Key Test
```sql
-- List all foreign keys
SELECT
    tc.constraint_name,
    tc.table_name,
    kcu.column_name,
    ccu.table_name as referenced_table,
    ccu.column_name as referenced_column
FROM information_schema.table_constraints AS tc
JOIN information_schema.key_column_usage AS kcu
    ON tc.constraint_name = kcu.constraint_name
    AND tc.table_schema = kcu.table_schema
JOIN information_schema.constraint_column_usage AS ccu
    ON ccu.constraint_name = tc.constraint_name
    AND ccu.table_schema = tc.table_schema
WHERE tc.table_schema = 'public'
AND tc.constraint_type = 'FOREIGN KEY'
ORDER BY tc.table_name;

-- Expected: Multiple foreign keys referencing:
-- - user_sessions -> users
-- - task_messages -> agent_tasks
-- - ai_requests -> agent_tasks
-- - endpoint_requests -> user_sessions
-- - etc.
```

### After Unique Constraint Test
```sql
-- List all unique constraints
SELECT constraint_name, table_name, column_name
FROM information_schema.constraint_column_usage
WHERE table_schema = 'public'
AND constraint_name LIKE '%_unique'
ORDER BY table_name;

-- Examples:
-- user_sessions: fingerprint_hash_unique
-- users: email_unique
-- agents: name_unique
```

### After NOT NULL Test
```sql
-- List all NOT NULL columns
SELECT table_name, column_name, is_nullable
FROM information_schema.columns
WHERE table_schema = 'public'
AND is_nullable = 'NO'
ORDER BY table_name, column_name;

-- For user_sessions, expected NOT NULL:
-- - session_id
-- - started_at
-- - user_type
-- - is_bot
```

### After Timestamp Test
```sql
-- Verify timestamp column types
SELECT table_name, column_name, data_type, column_default
FROM information_schema.columns
WHERE table_schema = 'public'
AND data_type IN ('timestamp with time zone', 'timestamp without time zone')
ORDER BY table_name;

-- Expected columns (with timezone):
-- - user_sessions.started_at
-- - user_sessions.last_activity_at
-- - user_sessions.ended_at
-- - analytics_events.created_at
-- - agent_tasks.created_at
-- - etc.

-- Verify defaults use CURRENT_TIMESTAMP
SELECT table_name, column_name, column_default
FROM information_schema.columns
WHERE table_schema = 'public'
AND column_default LIKE '%CURRENT%'
ORDER BY table_name;
```

### After JSON Storage Test
```sql
-- Verify JSON columns exist and store correctly
SELECT table_name, column_name, data_type
FROM information_schema.columns
WHERE table_schema = 'public'
AND data_type = 'json'
ORDER BY table_name;

-- Expected JSON columns:
-- - analytics_events.metadata
-- - ai_requests.metadata
-- - endpoint_requests.metadata
-- - etc.

-- Verify JSON can be queried
SELECT COUNT(*) as json_columns_present
FROM information_schema.columns
WHERE table_schema = 'public'
AND data_type = 'json';

-- Expected: >= 5
```

### After Concurrent Insert Test
```sql
-- Verify no conflicts from concurrent inserts
SELECT table_name, COUNT(*) as row_count
FROM information_schema.tables
WHERE table_schema = 'public'
GROUP BY table_name
ORDER BY row_count DESC;

-- Example: If test inserted 100 concurrent rows into user_sessions:
-- SELECT COUNT(*) FROM user_sessions WHERE created_at > NOW() - INTERVAL '5 minutes';
-- Expected: 100 (all inserts successful)

-- Verify no duplicate primary keys
SELECT session_id, COUNT(*) as count
FROM user_sessions
GROUP BY session_id
HAVING COUNT(*) > 1;

-- Expected: Empty result (no duplicates)
```

### After Concurrency Test (Reads Don't Block Writes)
```sql
-- Verify transaction isolation works
-- Run while concurrent test is executing:

-- Terminal 1 (read in transaction):
BEGIN TRANSACTION;
SELECT COUNT(*) FROM user_sessions;
COMMIT;

-- Terminal 2 (write concurrently - should not be blocked):
INSERT INTO user_sessions (session_id, started_at) VALUES ('test-' || uuid_generate_v4(), NOW());
-- Should complete quickly, not wait for Terminal 1

-- Verify with locks (should be minimal during read-heavy workload):
SELECT * FROM pg_stat_activity WHERE state = 'idle in transaction';
-- Expected: Empty or very few
```

### After Orphaned Records Test
```sql
-- This test specifically validates referential integrity
-- It should query for records that violate foreign key constraints

-- Example: Task messages without parent tasks
SELECT COUNT(*) as orphaned_messages
FROM task_messages tm
WHERE NOT EXISTS (
    SELECT 1 FROM agent_tasks at WHERE at.task_id = tm.task_id
);

-- Expected: 0 (no orphaned records)

-- Example: Analytics events without sessions
SELECT COUNT(*) as orphaned_events
FROM analytics_events ae
WHERE NOT EXISTS (
    SELECT 1 FROM user_sessions s WHERE s.session_id = ae.session_id
);

-- Expected: 0

-- Example: Endpoint requests without sessions
SELECT COUNT(*) as orphaned_requests
FROM endpoint_requests er
WHERE NOT EXISTS (
    SELECT 1 FROM user_sessions s WHERE s.session_id = er.session_id
);

-- Expected: 0

-- Comprehensive orphaned record check:
WITH orphaned_checks AS (
    SELECT 'task_messages' as table_name, COUNT(*) as orphaned_count
    FROM task_messages tm
    WHERE NOT EXISTS (SELECT 1 FROM agent_tasks at WHERE at.task_id = tm.task_id)

    UNION ALL

    SELECT 'analytics_events', COUNT(*)
    FROM analytics_events ae
    WHERE NOT EXISTS (SELECT 1 FROM user_sessions s WHERE s.session_id = ae.session_id)

    UNION ALL

    SELECT 'endpoint_requests', COUNT(*)
    FROM endpoint_requests er
    WHERE NOT EXISTS (SELECT 1 FROM user_sessions s WHERE s.session_id = er.session_id)

    UNION ALL

    SELECT 'ai_requests', COUNT(*)
    FROM ai_requests air
    WHERE NOT EXISTS (SELECT 1 FROM agent_tasks at WHERE at.task_id = air.task_id)
)
SELECT table_name, orphaned_count
FROM orphaned_checks
WHERE orphaned_count > 0;

-- Expected: Empty result (0 orphaned records)
```

---

## Test Implementation Examples

### Test 1: Connection Pool Test
**File**: `startup.rs`

```rust
#[tokio::test]
async fn test_postgres_connection_pool_established() -> Result<()> {
    let ctx = TestContext::new().await?;

    // Execute simple query
    let rows = ctx.db.fetch_all(
        "SELECT 1 as test_value",
        &[]
    ).await?;

    assert!(!rows.is_empty(), "No rows returned from test query");
    assert!(rows[0].get("test_value").is_some(), "Missing test_value column");

    println!("✓ PostgreSQL connection pool established");
    Ok(())
}

#[tokio::test]
async fn test_database_connection_pool() -> Result<()> {
    let ctx = TestContext::new().await?;

    // Try multiple queries to verify pool handles concurrent requests
    for i in 0..5 {
        let rows = ctx.db.fetch_all(
            "SELECT $1::int as iteration",
            &[&(i as i32)]
        ).await?;

        let iteration = rows[0].get("iteration")
            .and_then(|v| v.as_i64())
            .map(|n| n as i32);

        assert_eq!(iteration, Some(i), "Iteration mismatch");
    }

    println!("✓ Connection pool handles concurrent requests");
    Ok(())
}
```

**Validation SQL** (run after test):
```sql
-- Verify database is responsive
SELECT current_database(), current_user, pg_postmaster_start_time();

-- Verify connection pool statistics
SELECT sum(numbackends) as total_connections
FROM pg_stat_database
WHERE datname = current_database();
```

---

### Test 2: Schema Initialization Test
**File**: `startup.rs`

```rust
#[tokio::test]
async fn test_database_schema_initialized() -> Result<()> {
    let ctx = TestContext::new().await?;

    // Define required tables
    let required_tables = vec![
        "user_sessions",
        "analytics_events",
        "agent_tasks",
        "task_messages",
        "ai_requests",
        "endpoint_requests",
        "markdown_content",
        "services",
    ];

    // Query information_schema for each table
    for table in required_tables {
        let query = "SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = $1";
        let rows = ctx.db.fetch_all(query, &[&table]).await?;

        assert!(!rows.is_empty(), "Table '{}' does not exist", table);
    }

    println!("✓ All required tables exist");
    Ok(())
}
```

**Validation SQL**:
```sql
-- List all tables created
SELECT table_name, created_date
FROM information_schema.tables
WHERE table_schema = 'public'
ORDER BY table_name;

-- Verify specific table structure
\d user_sessions
```

---

### Test 3: Primary Key Enforcement Test
**File**: `constraints.rs`

```rust
#[tokio::test]
async fn test_primary_keys_enforced() -> Result<()> {
    let ctx = TestContext::new().await?;

    // Generate test session ID
    let session_id = "test-pk-" + &uuid::Uuid::new_v4().to_string();

    // Insert first record - should succeed
    let insert_query = "INSERT INTO user_sessions (session_id, started_at) VALUES ($1, NOW())";
    ctx.db.execute(insert_query, &[&session_id]).await?;

    // Try to insert duplicate - should fail
    let result = ctx.db.execute(insert_query, &[&session_id]).await;

    assert!(result.is_err(), "Duplicate primary key was not rejected");
    assert!(
        result.unwrap_err().to_string().contains("duplicate")
        || result.unwrap_err().to_string().contains("unique"),
        "Wrong error type for duplicate key"
    );

    println!("✓ Primary key constraints enforced");
    Ok(())
}
```

**Validation SQL**:
```sql
-- Verify no duplicate session_ids
SELECT session_id, COUNT(*) as count
FROM user_sessions
WHERE session_id LIKE 'test-pk-%'
GROUP BY session_id
HAVING COUNT(*) > 1;

-- Expected: Empty result (constraint enforced)
```

---

### Test 4: Foreign Key Constraint Test
**File**: `foreign_keys.rs`

```rust
#[tokio::test]
async fn test_foreign_keys_enforced() -> Result<()> {
    let ctx = TestContext::new().await?;

    // Try to insert analytics_event with non-existent session_id
    let fake_session_id = "non-existent-" + &uuid::Uuid::new_v4().to_string();

    let insert_query = "INSERT INTO analytics_events
        (session_id, event_type, event_category, severity)
        VALUES ($1, $2, $3, $4)";

    let result = ctx.db.execute(
        insert_query,
        &[&fake_session_id, &"page_view", &"user_action", &"info"]
    ).await;

    assert!(result.is_err(), "Foreign key constraint was not enforced");
    assert!(
        result.unwrap_err().to_string().contains("foreign key"),
        "Wrong error type for foreign key violation"
    );

    println!("✓ Foreign key constraints enforced");
    Ok(())
}
```

**Validation SQL**:
```sql
-- Verify foreign key constraints exist
SELECT constraint_name, table_name, column_name
FROM information_schema.constraint_column_usage
WHERE constraint_name LIKE '%fk_%'
ORDER BY table_name;

-- Try to insert orphaned record (should fail in test)
-- If this query returns rows, constraint is NOT enforced:
SELECT COUNT(*) as orphaned
FROM analytics_events ae
WHERE NOT EXISTS (
    SELECT 1 FROM user_sessions s WHERE s.session_id = ae.session_id
);
-- Expected: 0
```

---

### Test 5: Unique Constraint Test
**File**: `constraints.rs`

```rust
#[tokio::test]
async fn test_unique_constraints_enforced() -> Result<()> {
    let ctx = TestContext::new().await?;

    let fingerprint = "test-unique-" + &uuid::Uuid::new_v4().to_string();

    // First insert with fingerprint - should succeed
    let query = "INSERT INTO user_sessions
        (session_id, fingerprint_hash, started_at)
        VALUES ($1, $2, NOW())";

    ctx.db.execute(
        query,
        &[&("test-" + &uuid::Uuid::new_v4().to_string()), &fingerprint]
    ).await?;

    // Second insert with same fingerprint - should fail
    let result = ctx.db.execute(
        query,
        &[&("test-" + &uuid::Uuid::new_v4().to_string()), &fingerprint]
    ).await;

    assert!(result.is_err(), "Unique constraint was not enforced");

    println!("✓ Unique constraints enforced");
    Ok(())
}
```

**Validation SQL**:
```sql
-- Check for duplicate fingerprints
SELECT fingerprint_hash, COUNT(*) as count
FROM user_sessions
WHERE fingerprint_hash IS NOT NULL
GROUP BY fingerprint_hash
HAVING COUNT(*) > 1;

-- Expected: Empty result
```

---

### Test 6: NOT NULL Constraint Test
**File**: `constraints.rs`

```rust
#[tokio::test]
async fn test_not_null_constraints_enforced() -> Result<()> {
    let ctx = TestContext::new().await?;

    // Try to insert session without started_at (required field)
    let query = "INSERT INTO user_sessions (session_id, started_at) VALUES ($1, NULL)";
    let result = ctx.db.execute(
        query,
        &[&("test-" + &uuid::Uuid::new_v4().to_string())]
    ).await;

    assert!(result.is_err(), "NOT NULL constraint was not enforced");
    assert!(
        result.unwrap_err().to_string().contains("not-null")
        || result.unwrap_err().to_string().contains("NULL"),
        "Wrong error type for NULL violation"
    );

    println!("✓ NOT NULL constraints enforced");
    Ok(())
}
```

**Validation SQL**:
```sql
-- Verify NOT NULL columns exist
SELECT table_name, column_name
FROM information_schema.columns
WHERE table_schema = 'public'
AND is_nullable = 'NO'
ORDER BY table_name, column_name;

-- Verify user_sessions required fields
SELECT column_name, is_nullable, column_default
FROM information_schema.columns
WHERE table_name = 'user_sessions'
ORDER BY ordinal_position;
```

---

### Test 7: Timestamp Storage Test
**File**: `consistency.rs`

```rust
#[tokio::test]
async fn test_timestamp_fields_store_correctly() -> Result<()> {
    let ctx = TestContext::new().await?;

    use chrono::Utc;

    let session_id = "test-ts-" + &uuid::Uuid::new_v4().to_string();
    let before = Utc::now();

    // Insert with timestamp
    ctx.db.execute(
        "INSERT INTO user_sessions (session_id, started_at) VALUES ($1, NOW())",
        &[&session_id]
    ).await?;

    let after = Utc::now();

    // Retrieve and verify timestamp
    let rows = ctx.db.fetch_all(
        "SELECT started_at FROM user_sessions WHERE session_id = $1",
        &[&session_id]
    ).await?;

    assert!(!rows.is_empty(), "Session not found");

    let stored_ts = rows[0].get("started_at")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid timestamp value"))?;

    let stored_time = DateTime::parse_from_rfc3339(stored_ts)?
        .with_timezone(&Utc);

    // Verify timestamp is between before and after
    assert!(stored_time >= before, "Stored timestamp is before test start");
    assert!(stored_time <= after, "Stored timestamp is after test end");

    println!("✓ Timestamps stored and retrieved correctly");
    Ok(())
}
```

**Validation SQL**:
```sql
-- Check timestamp column types
SELECT table_name, column_name, data_type, column_default
FROM information_schema.columns
WHERE table_schema = 'public'
AND data_type LIKE 'timestamp%'
ORDER BY table_name;

-- Verify recent timestamps have correct precision
SELECT session_id, started_at, EXTRACT(MICROSECOND FROM started_at) as microseconds
FROM user_sessions
WHERE session_id LIKE 'test-ts-%'
ORDER BY started_at DESC
LIMIT 5;

-- Verify DEFAULT CURRENT_TIMESTAMP works
SELECT session_id,
       EXTRACT(EPOCH FROM (NOW() - started_at)) as seconds_since_creation
FROM user_sessions
WHERE session_id LIKE 'test-ts-%'
ORDER BY started_at DESC
LIMIT 1;
-- Expected: < 1 second for recently created records
```

---

### Test 8: Concurrent Inserts Test
**File**: `concurrency.rs`

```rust
#[tokio::test]
async fn test_concurrent_inserts_succeed() -> Result<()> {
    let ctx = std::sync::Arc::new(TestContext::new().await?);

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let ctx = Arc::clone(&ctx);
            tokio::spawn(async move {
                let session_id = format!("concurrent-test-{}-{}", i, uuid::Uuid::new_v4());
                ctx.db.execute(
                    "INSERT INTO user_sessions (session_id, started_at) VALUES ($1, NOW())",
                    &[&session_id]
                ).await
            })
        })
        .collect();

    // Wait for all concurrent inserts
    let mut success_count = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => success_count += 1,
            _ => {}
        }
    }

    assert_eq!(success_count, 10, "Not all concurrent inserts succeeded");

    println!("✓ All 10 concurrent inserts succeeded");
    Ok(())
}
```

**Validation SQL**:
```sql
-- Verify all concurrent inserts were persisted
SELECT COUNT(*) as concurrent_sessions
FROM user_sessions
WHERE session_id LIKE 'concurrent-test-%';

-- Expected: 10

-- Verify no duplicates
SELECT session_id, COUNT(*) as count
FROM user_sessions
WHERE session_id LIKE 'concurrent-test-%'
GROUP BY session_id
HAVING COUNT(*) > 1;

-- Expected: Empty result (no duplicates)
```

---

### Test 9: Orphaned Records Test
**File**: `orphaned_records.rs`

```rust
#[tokio::test]
async fn test_orphaned_record_detection() -> Result<()> {
    let ctx = TestContext::new().await?;

    // Count various orphaned record scenarios
    let orphaned_checks = vec![
        (
            "task_messages without tasks",
            "SELECT COUNT(*) as count FROM task_messages tm
             WHERE NOT EXISTS (SELECT 1 FROM agent_tasks at WHERE at.task_id = tm.task_id)"
        ),
        (
            "analytics_events without sessions",
            "SELECT COUNT(*) as count FROM analytics_events ae
             WHERE NOT EXISTS (SELECT 1 FROM user_sessions s WHERE s.session_id = ae.session_id)"
        ),
        (
            "endpoint_requests without sessions",
            "SELECT COUNT(*) as count FROM endpoint_requests er
             WHERE NOT EXISTS (SELECT 1 FROM user_sessions s WHERE s.session_id = er.session_id)"
        ),
        (
            "ai_requests without tasks",
            "SELECT COUNT(*) as count FROM ai_requests air
             WHERE NOT EXISTS (SELECT 1 FROM agent_tasks at WHERE at.task_id = air.task_id)"
        ),
    ];

    for (description, query) in orphaned_checks {
        let rows = ctx.db.fetch_all(query, &[]).await?;
        let count = rows[0].get("count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        assert_eq!(count, 0, "Found orphaned records: {}", description);
    }

    println!("✓ No orphaned records detected");
    Ok(())
}
```

**Validation SQL**:
```sql
-- Complete orphaned records check (run this after test)
WITH orphan_summary AS (
    SELECT 'task_messages → agent_tasks' as relationship,
           COUNT(*) as orphaned
    FROM task_messages tm
    WHERE NOT EXISTS (SELECT 1 FROM agent_tasks WHERE task_id = tm.task_id)

    UNION ALL

    SELECT 'analytics_events → sessions', COUNT(*)
    FROM analytics_events ae
    WHERE NOT EXISTS (SELECT 1 FROM user_sessions WHERE session_id = ae.session_id)

    UNION ALL

    SELECT 'endpoint_requests → sessions', COUNT(*)
    FROM endpoint_requests er
    WHERE NOT EXISTS (SELECT 1 FROM user_sessions WHERE session_id = er.session_id)

    UNION ALL

    SELECT 'ai_requests → tasks', COUNT(*)
    FROM ai_requests
    WHERE NOT EXISTS (SELECT 1 FROM agent_tasks WHERE task_id = ai_requests.task_id)
)
SELECT relationship, orphaned
FROM orphan_summary
WHERE orphaned > 0;

-- Expected: Empty result (0 orphaned records across all relationships)
```

---

## Running the Tests

```bash
# Run all database tests
cargo test --test database --all -- --nocapture

# Run specific test
cargo test --test database test_primary_keys_enforced -- --nocapture

# Run with verbose output
RUST_LOG=debug cargo test --test database -- --nocapture --test-threads=1
```

## Post-Test Validation Workflow

After running each test suite:

```bash
# 1. Connect to database
psql postgresql://systemprompt:systemprompt_dev_password@localhost:5432/systemprompt_dev

# 2. Run relevant validation SQL from above

# 3. Check for orphaned records
# (See "After Orphaned Records Test" section above)

# 4. Verify no test data remains
SELECT COUNT(*) FROM user_sessions WHERE session_id LIKE 'test-%';
-- Expected: 0 (all test data cleaned up)
```

---

## Summary

| Test | Coverage | Database Validation |
|------|----------|-------------------|
| Connection Pool | ✓ | SELECT 1 |
| Schema Init | ✓ | information_schema.tables |
| Migrations | ✓ | schema_migrations table |
| Primary Keys | ✓ | Attempt duplicate insert |
| Foreign Keys | ✓ | Attempt orphaned insert |
| Unique Constraints | ✓ | Attempt duplicate unique value |
| NOT NULL | ✓ | Attempt NULL insert |
| Timestamps | ✓ | RFC3339 parsing + time range |
| Concurrency | ✓ | COUNT(*) of inserted rows |
| Orphaned Records | ✓ | LEFT JOIN with NOT EXISTS |

**Target**: All 9 tests fully implemented with database validation queries running in parallel.
