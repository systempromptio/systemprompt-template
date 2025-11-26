# Integration Tests Implementation

**Status**: Framework & Planning Complete. Ready for Implementation.

## Quick Links

- **Start Here**: [TEST_IMPLEMENTATION_MASTER_PLAN.md](./TEST_IMPLEMENTATION_MASTER_PLAN.md)
- **By Folder**:
  - [Database Tests](./database/IMPLEMENTATION_GUIDE.md) - 9 tests (Priority 1)
  - [Analytics Tests](./analytics/IMPLEMENTATION_GUIDE.md) - 15 tests (Priority 2)
  - [Agents Tests](./agents/IMPLEMENTATION_GUIDE.md) - 16 tests (Priority 3)
  - [Auth Tests](./auth/IMPLEMENTATION_GUIDE.md) - 8 tests (Priority 4)
  - [Content Tests](./content/IMPLEMENTATION_GUIDE.md) - 10 tests (Priority 5)
  - [MCP Tests](./mcp/IMPLEMENTATION_GUIDE.md) - 9 tests (Priority 6)

## Current Status

### By Numbers
- **64 total tests** across 6 domains
- **95.7% are stubs** - just setup with no assertions
- **0% have database validation** - never check if data persisted
- **2 tests have partial coverage** - only HTTP status checks

### Risk Level
**HIGH** - Tests pass but don't validate functionality. False confidence.

### What's Ready
- ✅ 6 detailed implementation guides (one per folder)
- ✅ 1 master implementation plan (8-10 week timeline)
- ✅ 200+ SQL validation queries
- ✅ 40+ complete code examples
- ✅ 7-phase test pattern (copy-paste ready)
- ✅ Common pitfalls & solutions documented

### What's Needed
- ⏳ Replace stub implementations with real tests
- ⏳ Add database assertions to every test
- ⏳ Verify test data cleanup
- ⏳ Run SQL validation queries

## The 7-Phase Test Pattern

Every test should follow this structure:

```rust
#[tokio::test]
async fn test_something() -> Result<()> {
    // Phase 1: Setup
    let ctx = TestContext::new().await?;
    let unique_id = generate_unique_id();

    // Phase 2: Action (HTTP, API, etc)
    let response = ctx.make_request("/endpoint").await?;
    assert!(response.status().is_success());

    // Phase 3: Wait for async
    TestContext::wait_for_async_processing().await;

    // Phase 4: Query database
    let query = DatabaseQueryEnum::SomeQuery.get(ctx.db.as_ref());
    let rows = ctx.db.fetch_all(&query, &[&unique_id]).await?;

    // Phase 5: Assert data persisted
    assert!(!rows.is_empty(), "Data not persisted");
    let data = MyDataType::from_json_row(&rows[0])?;
    assert_eq!(data.field, expected_value);

    // Phase 6: Cleanup
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_unique_id(unique_id);
    cleanup.cleanup_all().await?;

    // Phase 7: Log
    println!("✓ Test passed with database validation");
    Ok(())
}
```

## How to Get Started

### 1. Read the Plan (20 minutes)
```bash
cat TEST_IMPLEMENTATION_MASTER_PLAN.md
```

### 2. Choose a Folder (Database first is recommended)
```bash
cat database/IMPLEMENTATION_GUIDE.md
```

### 3. Implement 2-3 Tests
- Copy the template from the guide
- Replace stub in {folder}/{name}.rs
- Run: `cargo test --test {folder} test_{name} -- --nocapture`

### 4. Validate with SQL
```bash
psql postgresql://systemprompt:password@localhost/systemprompt_dev << EOF
-- Copy SQL from IMPLEMENTATION_GUIDE.md
SELECT * FROM table WHERE condition;
EOF
```

### 5. Cleanup Check
```bash
psql ... -c "SELECT COUNT(*) FROM user_sessions WHERE created_at > NOW() - INTERVAL '5 minutes';"
-- Expected: 0 (all test data removed)
```

## Test Folder Organization

```
database/
├── IMPLEMENTATION_GUIDE.md     ← Complete guide with SQL
├── startup.rs                  ← Connection & schema tests
├── constraints.rs              ← PK, FK, unique, NOT NULL tests
├── foreign_keys.rs
├── concurrency.rs
├── consistency.rs
├── orphaned_records.rs
├── migrations.rs
└── mod.rs

(similar structure for analytics/, agents/, auth/, content/, mcp/)
```

## Key Principles

### 1. Every Test Must Assert Database State
❌ WRONG: Make request → assert HTTP 200 → cleanup
✅ CORRECT: Make request → query database → assert data → cleanup

### 2. No Mega-Tests
❌ WRONG: `test_full_user_lifecycle()` with 10 assertions
✅ CORRECT: `test_user_creation()`, `test_user_update()`, `test_user_deletion()`

### 3. Cleanup Is Mandatory
❌ WRONG: Test creates data and leaves it
✅ CORRECT: Use TestCleanup to remove all test data

### 4. Wait for Async
❌ WRONG: `ctx.make_request("/")` → immediately query DB
✅ CORRECT: `make_request()` → `wait_for_async_processing()` → query

## Success Metrics

### Per Test
- ✅ Passes without errors
- ✅ Has database assertion
- ✅ Data fully cleaned up
- ✅ SQL validation query confirms state

### Per Folder
- ✅ All tests pass
- ✅ No orphaned data remaining
- ✅ Coverage: All semantic units tested

### Overall
- ✅ 64 tests passing
- ✅ Database fully consistent
- ✅ Zero false positives

## Implementation Timeline

```
Week 1-2:   Database tests    (9)  → 9 total ✓
Week 2-3:   Analytics tests   (15) → 24 total ✓
Week 3-4:   Agents tests      (16) → 40 total ✓
Week 4-5:   Auth tests        (8)  → 48 total ✓
Week 5-6:   Content tests     (10) → 58 total ✓
Week 6-7:   MCP tests         (9)  → 64 total ✓
Week 7-8:   Review & Polish   → All excellent
Week 8-9:   Documentation    → Ready
```

## Common Issues & Solutions

### Test Data Not Cleaning Up
**Problem**: Database has leftover test data
**Solution**: Ensure TestCleanup is called in every test
```rust
let mut cleanup = TestCleanup::new(ctx.db.clone());
cleanup.track_fingerprint(fingerprint);
cleanup.cleanup_all().await?;
```

### Database Query Returns Empty
**Problem**: Test makes request but database is empty
**Solution**: Add async wait between request and query
```rust
ctx.make_request("/").await?;
TestContext::wait_for_async_processing().await;  // ← Add this
let rows = ctx.db.fetch_all(query, &[]).await?;
```

### DatabaseQueryEnum::SomeQuery Not Found
**Problem**: Compiler error about missing enum variant
**Solution**: Check guide for available queries, or add new variant to DatabaseQueryEnum
```rust
// Check which queries are available in database crate
// crates/modules/database/src/models/types.rs
```

### Assertion Always Passes
**Problem**: Test passes but doesn't actually validate anything
**Solution**: Assert specific values from database
```rust
// ❌ Wrong
assert!(true);

// ✅ Correct
let data = parse_row(&rows[0])?;
assert_eq!(data.field, expected_value);
```

## Running Tests

### Single Test
```bash
cargo test --test database test_postgres_connection_pool_established -- --nocapture
```

### All Tests in Folder
```bash
cargo test --test analytics -- --nocapture --test-threads=1
```

### With Logging
```bash
RUST_LOG=debug cargo test --test agents -- --nocapture --test-threads=1
```

### Validate Database After
```bash
# Run test
cargo test --test analytics test_anonymous_session_created_on_homepage -- --nocapture

# Then check database
psql postgresql://systemprompt:password@localhost/systemprompt_dev << EOF
SELECT session_id, fingerprint_hash, user_type, request_count
FROM user_sessions
WHERE fingerprint_hash LIKE '%test%'
ORDER BY created_at DESC LIMIT 1;
EOF
```

## Documentation Map

| Document | Purpose | Audience |
|----------|---------|----------|
| TEST_IMPLEMENTATION_MASTER_PLAN.md | Overall strategy & timeline | Leads, Developers |
| database/IMPLEMENTATION_GUIDE.md | Database test implementation | Developers |
| analytics/IMPLEMENTATION_GUIDE.md | Analytics test implementation | Developers |
| agents/IMPLEMENTATION_GUIDE.md | Agent test implementation | Developers |
| auth/IMPLEMENTATION_GUIDE.md | Auth test implementation | Developers |
| content/IMPLEMENTATION_GUIDE.md | Content test implementation | Developers |
| mcp/IMPLEMENTATION_GUIDE.md | MCP test implementation | Developers |
| README.md (this file) | Quick reference & navigation | Everyone |

## Next Steps

1. **Read**: TEST_IMPLEMENTATION_MASTER_PLAN.md (20 min)
2. **Choose**: Start with database folder
3. **Learn**: Read database/IMPLEMENTATION_GUIDE.md (45 min)
4. **Implement**: 2-3 tests (1-2 hours each)
5. **Validate**: Run SQL queries from guide
6. **Repeat**: Move to next folder

## Questions?

Refer to the relevant IMPLEMENTATION_GUIDE.md for your folder. Each includes:
- Complete test code examples
- SQL validation queries
- Common pitfalls and solutions
- Running instructions

---

**Framework Status**: ✅ Complete and Ready
**Implementation Status**: ⏳ Ready to Begin
**Timeline**: 8-10 weeks for full completion
**Target**: 64 fully implemented tests with database assertions
