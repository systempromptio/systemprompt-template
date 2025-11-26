- You are a world class rust developer following SystemPrompt Rust Standards
- **MANDATORY**: Read and follow `instructions/rust.md` for all Rust code
- Code quality criteria: file length, complexity, patterns, naming, anti-patterns
- Zero inline comments - code documents itself through naming
- Zero inline logging - use `LogService` exclusively
- Zero raw String IDs - use typed identifiers from `systemprompt_identifiers`

**ZERO TOLERANCE FOR TECH DEBT**:
- Any tech debt, redundancy, or anti-pattern discovered MUST be refactored immediately
- Never defer refactoring to a future task - fix it now as part of the current work
- If you encounter code violating `instructions/rust.md`, refactor it before proceeding
- No "TODO" comments, no "fix later" - the code you touch must meet standards when you're done

## Development Workflow & Build Policy

**CRITICAL BUILD POLICY**:
- **NEVER** build with `--release` flag unless explicitly instructed
- **ALWAYS** use debug builds for development (faster compile times)
- **ONLY** create release builds when user specifically requests deployment/production builds

**ALWAYS USE JUST COMMANDS**:
```bash
just api              # Run API server
just mcp [command]    # MCP server operations
just a2a [command]    # Agent operations
just db [command]     # Database operations
just log              # Stream logs in development
just admin-token      # Generate admin JWT
```

**Why?** Shorter, faster, handles all cargo flags, consistent across codebase, includes environment setup.

**CRITICAL**: NEVER use raw `cargo run` commands when a `just` shortcut exists.

## Docker Build Policy

**CRITICAL**: When code changes are made and need to be deployed to Docker:

1. **NEVER** run `cargo build --release` and `docker-compose build` separately
2. **ALWAYS** use the unified build script: `./infrastructure/scripts/build.sh release --docker`

**Why?** The Docker image expects binaries in `infrastructure/build-context/release/`, not `target/release/`. The build script:
- Builds release binaries
- Stages them in the correct location for Docker
- Validates all required binaries exist
- Builds the Docker image

**Workflow**:
```bash
# For code changes that need Docker deployment
./infrastructure/scripts/build.sh release --docker
docker-compose -f infrastructure/environments/docker/docker-compose.yml up -d app

# Include web build if frontend changed
./infrastructure/scripts/build.sh release --web --docker
docker-compose -f infrastructure/environments/docker/docker-compose.yml up -d app
```

**Note**: Use `up -d` (not `restart`) to recreate the container with the new image.

**Common mistake**: Running `cargo build --release` puts binaries in `target/release/` or `core/target/release/`, but Docker copies from `infrastructure/build-context/release/`. The build script handles this staging step.

## Architecture

SystemPrompt Core is a **platform-only** repository providing the foundation for building AI agent systems:
- `crates/modules/api/` - HTTP API server framework
- `crates/modules/` - Core modules (agent, ai, mcp, oauth, users, config, log, rag, database)
- `crates/shared/` - Shared models and utilities (models, traits, identifiers)

**NO service implementations** - Core is decoupled from service code (agents, MCP servers, skills).

**For Implementation Repositories** (e.g., systemprompt-blog):
- Service implementations live in separate repos
- Use core as git dependency or subtree
- Configure service paths via environment variables

**Unified Service Model** (when using implementations):
- API Server (port 8080) is main entry point
- Agents and MCP Servers are child services
- All enabled services auto-start with API
- All services auto-restart on failure

### Core Commands

| Task | Command |
|------|---------|
| Build core | `cargo build --workspace` |
| Initialize DB | `./target/debug/systemprompt db migrate` |
| Start API | `just api` |
| Run tests | `cargo test --workspace` |
| Check code | `cargo clippy --workspace` |
| Format code | `cargo fmt --all` |

**Configuration** (optional, for implementations):
- `SYSTEMPROMPT_SERVICES_PATH` - Path to service implementations
- `SYSTEMPROMPT_SKILLS_PATH` - Path to skills directory
- `SYSTEMPROMPT_CONFIG_PATH` - Path to services config file

## Module Structure

**MANDATORY ARCHITECTURE**: All modules MUST follow this structure exactly:

```
crates/modules/{module}/
├── api/          - HTTP endpoints and route definitions
├── models/       - Data structures, validation, serialization
├── repository/   - Database operations (separate files per operation)
├── services/     - Business logic layer (MUST use repository, NEVER direct DB)
├── schema/       - Database table definitions (SQL)
└── queries/      - SQL query files (PostgreSQL)
    ├── postgres/     - PostgreSQL queries ($1, $2 placeholders)
    ├── seed/         - Seed data
    └── fixtures/     - Test fixtures
```

**Critical Rules**:
- ✅ **Queries**: PostgreSQL SQL files in postgres/ subdirectory
- ✅ **Repository**: Uses `DatabaseQueryEnum` (NEVER direct SQL constants)
- ✅ **Services**: MUST use repository methods (NEVER direct database access)
- ❌ **FORBIDDEN**: Inline SQL, direct sqlx usage outside repository layer

## Repository & Database Patterns

**CRITICAL**: All database operations use `DatabaseProvider` abstraction with `DatabaseQueryEnum`. NEVER use inline SQL in services or direct sqlx.

### Repository Pattern (MANDATORY)

**All repositories MUST use `DatabaseQueryEnum` for type-safe query references:**

```rust
use systemprompt_database::{DatabaseProvider, DatabaseQueryEnum, DbPool};

pub struct UserRepository {
    db_pool: DbPool,
}

impl UserRepository {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub async fn get_user(&self, id: &str) -> Result<Option<User>> {
        let query = DatabaseQueryEnum::GetUser.get(self.db_pool.as_ref());
        let row = self.db_pool.fetch_optional(query, &[&id]).await?;
        row.map(|r| User::from_json_row(&r)).transpose()
    }

    pub async fn create_user(&self, name: &str, email: &str) -> Result<u64> {
        let query = DatabaseQueryEnum::CreateUser.get(self.db_pool.as_ref());
        self.db_pool.execute(query, &[&name, &email]).await
    }
}
```

**Why DatabaseQueryEnum?**
- ✅ Type-safe query references (enum variants)
- ✅ IDE autocomplete shows all available queries
- ✅ Compile-time validation of query existence
- ✅ Database-agnostic query management
- ✅ Eliminates 47+ duplicate `select_query()` functions
- ✅ Easy refactoring (type system catches all usages)

**Model Deserialization** - All models must implement `from_json_row()`:
```rust
use systemprompt_database::JsonRow;

impl User {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let uuid = row.get("uuid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing uuid"))?
            .to_string();
        Ok(Self { uuid, ... })
    }
}
```

**Transactions**:
```rust
pub async fn transfer(&self, from: &str, to: &str, amount: i64) -> Result<()> {
    let mut tx = self.db.begin_transaction().await?;
    tx.execute(DEBIT, &[&amount, &from]).await?;
    tx.execute(CREDIT, &[&amount, &to]).await?;
    tx.commit().await?;
    Ok(())
}
```

**Quick Reference**:

| Operation | Pattern |
|-----------|---------|
| Fetch one (required) | `db.fetch_one(query, params)` |
| Fetch one (optional) | `db.fetch_optional(query, params)` |
| Fetch multiple | `db.fetch_all(query, params)` |
| Insert/Update/Delete | `db.execute(query, params)` |
| Get count/sum | `db.fetch_scalar_value(query, params)` |
| Transaction | `db.begin_transaction()` |

**Why?**
- ✅ Clean database abstraction layer
- ✅ Type safe parameter binding
- ✅ Testable (mock `DatabaseProvider`)
- ✅ No sqlx types leak into business logic

## DateTime Standards

**CRITICAL**: Always use `DateTime<Utc>` - NEVER format as strings for database operations.

✅ **CORRECT**:
```rust
use chrono::Utc;

let now = Utc::now();
db.execute(query, &[&user_id, &now]).await?;  // Pass DateTime<Utc> directly
```

❌ **WRONG**:
```rust
let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
db.execute(query, &[&user_id, &now_str]).await?;  // ❌ String formatting
```

**Reading DateTimes**:
```rust
use systemprompt_database::parse_database_datetime;

let created_at = row.get("created_at")
    .and_then(|v| parse_database_datetime(v))
    .ok_or_else(|| anyhow!("Invalid created_at"))?;
```

**Schema Types**:
- PostgreSQL: `TIMESTAMP DEFAULT CURRENT_TIMESTAMP`

**Why?** PostgreSQL requires proper TIMESTAMP types. `DbValue::Timestamp` handles database-specific formatting automatically.

## Type Duplication Anti-Patterns

**CRITICAL**: One domain concept = one type in `crates/shared/models/`. Multiple types with identical fields defeats Rust's type safety.

**Before adding a new type, search**:
```bash
grep -r "pub struct YourType" crates --include="*.rs"
grep -r "field1.*field2.*field3" crates --include="*.rs"
```

**Prevention Rules**:
1. **Single Source of Truth** - Domain types belong in `crates/shared/models/`
2. **Import, Don't Copy** - Use existing types
3. **Search First** - Always search before creating
4. **Naming Matters** - Different names for same concept is still duplication

❌ **ANTI-PATTERN - Magic Numbers**:
```rust
if port == 5002 { /* use auth */ }  // ❌ Port determines behavior
```

✅ **CORRECT - Configuration**:
```rust
let config = get_service_config(service_id)?;
if config.oauth.required { /* use auth */ }  // ✅ Config determines behavior
```

**Checklist**:
- [ ] Search for similar types: `grep -r "pub struct.*<YourType>"`
- [ ] Check `crates/shared/models/` has what you need
- [ ] If duplicate exists, import it
- [ ] If truly new, add to `crates/shared/models/`

## Logging Services

Two distinct services:
- `LogService` - Database storage for audit/debugging with analytics context
- `CliService` - Display to users with styling

### LogService Pattern

**CRITICAL**: Always use `logger` variable name and `.ok()` to ignore logging errors:

```rust
// ✅ CORRECT
logger.info("module", "message").await.ok();
logger.error("module", &format!("Error: {}", e)).await.ok();

// ❌ WRONG
let _ = logger.info("module", "message").await;  // Don't use let _
let log = LogService::new(...);  // Don't use 'log' variable
```

### Context-Required Pattern

**CRITICAL**: LogService requires context at construction.

**Pattern 1: REST Handlers**:
```rust
pub async fn handler(
    Extension(req_ctx): Extension<RequestContext>,
    State(ctx): State<AppContext>,
) -> impl IntoResponse {
    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());
    logger.info("module", "message").await.ok();
}
```

**Pattern 2: CLI Tools**:
```rust
let logger = LogService::system(database.clone());
logger.info("setup", "Setting up...").await.ok();
```

**Pattern 3: Background Tasks**:
```rust
let log_context = LogContext::new()
    .with_session_id(&request.session_id.unwrap_or("system".to_string()))
    .with_trace_id(&request.trace_id.unwrap_or_else(|| Uuid::new_v4().to_string()))
    .with_user_id(&request.user_id.unwrap_or("anonymous".to_string()));

let logger = LogService::new(pool.clone(), log_context);
```

**LogService API**:
```rust
// Simple logging (2 parameters)
logger.info("module", "message").await.ok();
logger.error("module", "error").await.ok();

// Rich logging (4 parameters)
logger.log(LogLevel::Info, "module", "message", Some(json!({"key": "value"}))).await.ok();
```

**Development Workflow**: Always run `just log` in background terminal during development.

### JSON-RPC Error Builder

```rust
// ✅ CORRECT - Builder with embedded logging
let error = JsonRpcError::invalid_request()
    .with_data(json!("Details"))
    .log_error(format!("Error: {}", e))
    .build(&request_id, &logger)
    .await;

// Helper functions
unauthorized_response(reason, request_id, logger).await;
forbidden_response(reason, request_id, logger).await;
```

## JSON vs SQL Storage

**CRITICAL**: Storing queryable data as JSON TEXT defeats SQL type safety.

### When to Use JSON ✅

Use JSON **ONLY** when:
1. **Truly extensible data** with unpredictable schema
2. **No queries needed** on contents
3. **Debugging/auxiliary data** (e.g., `logs.metadata`)
4. **External protocol extensions** (e.g., A2A `metadata`)
5. **Low cardinality user attributes** (e.g., `users.roles`)

### When to Use SQL Columns

Use SQL columns when:
1. **Known, stable structure**
2. **Need to query/filter/join**
3. **Relationships between entities** (foreign keys)
4. **Arrays that should be junction tables**
5. **Need referential integrity**

### Decision Tree

```
Do you need to query this data?
├─ YES → Use SQL columns/tables
│   ├─ Simple field? → Add column
│   ├─ One-to-many? → Related table + FK
│   └─ Many-to-many? → Junction table
│
└─ NO → Consider JSON (verify conditions)
    ├─ Unpredictable schema? → JSON ✅
    ├─ Debugging/auxiliary? → JSON ✅
    ├─ Protocol extension? → JSON ✅
    ├─ Low cardinality attribute? → JSON ✅
    └─ Otherwise → SQL columns ❌
```

**Benefits of SQL**: Type safety, query performance, referential integrity, clear schema, maintainability.

## DatabaseQueryEnum Pattern (MANDATORY)

**CRITICAL**: All SQL queries MUST be accessed through `DatabaseQueryEnum`. This pattern eliminates duplication and provides type safety.

### Configuration

```bash
# PostgreSQL (required)
export DATABASE_TYPE=postgres
export DATABASE_URL=postgresql://user:pass@localhost:5432/dbname
```

### Architecture

```
DatabaseQueryEnum variant (in database crate)
        ↓
Module query mapping (crates/modules/database/src/models/queries/{module}.rs)
        ↓
SQL files (crates/modules/{module}/src/queries/*.sql)
        ↓
Repository uses enum (DatabaseQueryEnum::Variant.get(db))
        ↓
Service uses repository (never touches database directly)
```

### Query File Structure (MANDATORY)

```
crates/modules/{module}/src/queries/
├── postgres/
│   └── operation_name.sql      # PostgreSQL ($1, $2 placeholders)
├── seed/
└── fixtures/
```

### Adding New Queries (3 Steps)

**Step 1**: Create PostgreSQL SQL file in your module
```bash
# PostgreSQL version
crates/modules/users/src/queries/postgres/get_user_by_email.sql
```

**Step 2**: Add enum variant to `crates/modules/database/src/models/types.rs`
```rust
pub enum DatabaseQueryEnum {
    // ... existing variants ...
    GetUserByEmail,  // Add your variant
}
```

**Step 3**: Add mapping in `crates/modules/database/src/models/queries/{module}.rs`
```rust
pub fn get_query(variant: DatabaseQueryEnum, is_postgres: bool) -> Option<&'static str> {
    match (variant, is_postgres) {
        // ... existing mappings ...
        (DatabaseQueryEnum::GetUserByEmail, true) =>
            Some(include_str!("../../../../users/src/queries/postgres/get_user_by_email.sql")),
        _ => None,
    }
}
```

**Step 4**: Use in repository
```rust
pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
    let query = DatabaseQueryEnum::GetUserByEmail.get(self.db_pool.as_ref());
    let row = self.db_pool.fetch_optional(query, &[&email]).await?;
    row.map(|r| User::from_json_row(&r)).transpose()
}
```

### PostgreSQL SQL Reference

| Syntax | Use Case |
|--------|----------|
| `$1, $2` | Parameter placeholders |
| `CURRENT_TIMESTAMP` | Current timestamp |
| `CURRENT_DATE` | Current date |
| `dt + INTERVAL '1 day'` | Date arithmetic |
| `INSERT ... ON CONFLICT DO UPDATE` | Upsert |
| `INSERT ... ON CONFLICT DO NOTHING` | Conditional insert |
| `SERIAL PRIMARY KEY` | Auto-increment |
| `VARCHAR(255)` | Indexed text |
| `ILIKE` | Case-insensitive search |

### Benefits

**Type Safety**:
- ✅ Compiler catches missing queries at build time
- ✅ IDE autocomplete shows all available queries
- ✅ Safe refactoring (rename detection)
- ✅ No string typos or invalid query names

**Code Quality**:
- ✅ Eliminates 47+ duplicate `select_query()` functions
- ✅ 30-40% reduction in repository boilerplate
- ✅ Single source of truth for all queries
- ✅ Self-documenting (enum variants)

**Maintenance**:
- ✅ Easy to discover what queries exist
- ✅ Easy to add new queries (3 simple steps)
- ✅ Easy to remove queries (compiler shows all usages)
- ✅ Clean, maintainable query management

**Performance**:
- **PostgreSQL**: True concurrent writes, MVCC, scalable, production-ready
- **Zero runtime cost**: Query selection happens at compile time

## Database Tool

**CRITICAL**: Always use `just db` shortcuts.

### Database Initialization

**IMPORTANT**: Before running the API for the first time, you MUST initialize the database schema:

```bash
./target/debug/systemprompt db migrate       # Run all schema migrations
```

This creates all required tables (`services`, `markdown_content`, `logs`, etc.). Without this, the API will fail to start with errors like:
- `relation "services" does not exist`
- `Cannot start API without MCP servers`

**First-time setup workflow**:
```bash
cargo build                                  # Build the systemprompt binary
./target/debug/systemprompt db migrate       # Initialize database schema
just api                                     # Start the API server
```

### Database Operations

```bash
just db tables                               # List tables
just db describe <table>                     # Show schema
just db query "SELECT * FROM agents"         # Query (table format)
just db query "SELECT * FROM agents" json    # Query (JSON format)
just db execute "DELETE FROM old_data"       # Execute statement
just db info                                 # Database info
just db migrate                              # Run schema migrations
```

Requires PostgreSQL (configured via DATABASE_URL environment variable).

## Authentication & Admin Tokens

SystemPrompt uses JWT with RBAC. Agent CRUD requires admin role.

**Generate Token**:
```bash
just admin-token  # 24-hour admin token
```

**Token Details**:
- Expiry: 24 hours
- Role: admin
- Audience: web, a2a, api, mcp
- Scopes: user, admin

**RBAC Model**:
- **Roles** (custom claim): `admin`, `user`
- **Scopes** (OAuth 2.0): `user`, `admin`
- Authorization uses `roles` field in JWT

**Usage**:
```bash
ADMIN_TOKEN=$(just admin-token)

curl -H "Authorization: Bearer $ADMIN_TOKEN" \
     -H "Content-Type: application/json" \
     http://localhost:8080/api/v1/core/agents
```

## Agent Systems

**IMPORTANT**: Agents are config-driven (no database for definitions).

**Source of Truth**: `crates/services/agents/agents.yml`
**Runtime State**: `services` table (PID, port, status)

### Agent Registry

```rust
let registry = AgentRegistry::new().await?;
let agent = registry.get_agent_by_uuid(uuid).await?;
let all_agents = registry.list_agents().await?;
```

### Agent Orchestrator

```bash
just a2a list                     # Status table
just a2a start --agent-id {ID}    # Start agent
just a2a stop --agent-id {ID}     # Stop agent
just a2a restart --agent-id {ID}  # Restart agent
just a2a cleanup                  # Recover orphaned processes
```

### Agent Service Proxy

**Purpose**: Proxy A2A protocol requests to running agents
**Base URL**: `/api/v1/agents/{agent_id}/*`
**Requirement**: Agent must be running to receive requests

---

**Further Reading**:
- `/plan/tech-debt/README.md` - DatabaseQueryEnum migration guide (mandatory pattern)
- `/plan/db/COMPLETE_REFACTOR_PLAN.md` - PostgreSQL migration details
- `/plan/db/04-QUERY-PATTERNS.md` - Database query patterns
- `/plan/tech-debt/DATETIME_HANDLING_MESS.md` - DateTime migration guide
- `/plan/db/JSON-STORAGE-ANTIPATTERN-REMEDIATION.md` - JSON storage remediation
