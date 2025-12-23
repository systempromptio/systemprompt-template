# Migration Bug

## Bug: Migrations report success but create no tables

**Status:** Open
**Severity:** Blocker
**Command:** `systemprompt cloud profile create local` (during migration step)

### Observed Behavior
```
✓ PostgreSQL connection verified
✔ Run database migrations? · yes
ℹ Running database migrations...
✓ Migrations completed
⚠ Admin user sync failed: Failed to check existing user: Database error: error returned from database: relation "users" does not exist
```

### Actual State
```bash
$ docker exec systemprompt-postgres-local psql -U systemprompt -d systemprompt -c "\dt"
Did not find any relations.
```

**Zero tables created despite "Migrations completed" success message.**

### Root Cause Found

In `core/crates/entry/cli/src/services/db/mod.rs` line 93-99:

```rust
let modules = Modules::load(&config.core_path)?;
let all_modules = modules.all();

if all_modules.is_empty() {
    CliService::warning("No modules found - check CORE_PATH environment variable");
    return Ok(());  // BUG: Returns success even though nothing migrated!
}
```

**Problem:** When `CORE_PATH` is not set or points to wrong location, migrations silently succeed with zero modules installed.

### Root Cause: Module loader scans wrong directories

**Loader looks in:**
```rust
let module_categories = ["modules", "mcp", "agent"];  // WRONG
```

**Actual module.yaml locations:**
```
core/crates/domain/users/module.yaml      ✅ exists
core/crates/domain/oauth/module.yaml      ✅ exists
core/crates/domain/files/module.yaml      ✅ exists
core/crates/domain/analytics/module.yaml  ✅ exists
core/crates/domain/content/module.yaml    ✅ exists
core/crates/domain/ai/module.yaml         ✅ exists
core/crates/domain/mcp/module.yaml        ✅ exists
core/crates/domain/agent/module.yaml      ✅ exists
```

### Bug Location
File: `core/crates/shared/models/src/modules.rs` line 288

```rust
fn scan_and_load(core_path: &str) -> Result<Vec<Module>> {
    let crates_dir = Path::new(core_path).join("crates");
    let module_categories = ["modules", "mcp", "agent"];  // <-- BUG: should be ["domain"]
    // ...
}
```

### Fix Required
```rust
let module_categories = ["domain"];  // or just scan crates/domain/ directly
```

### Expected Behavior
Either:
1. Migrations should create all required tables including `users`
2. Admin user sync should only run after confirming `users` table exists
3. Admin user sync should be a separate command, not part of profile creation

### Investigation Needed

```bash
# Check what tables exist after migration
docker exec systemprompt-postgres-local psql -U systemprompt -d systemprompt -c "\dt"

# Check migration files
ls -la core/crates/domain/database/migrations/

# Check if users migration exists
grep -r "CREATE TABLE.*users" core/crates/domain/database/migrations/
```

### Root Cause Location
- Admin sync code: `core/crates/entry/cli/src/cloud/sync/admin_user.rs`
- Profile create: `core/crates/entry/cli/src/cloud/profile/create.rs`
- Migrations: `core/crates/domain/database/migrations/`

### Fix Required (in core repo)

**File:** `core/crates/shared/models/src/modules.rs` line 288

**Change:**
```rust
// BEFORE (wrong)
let module_categories = ["modules", "mcp", "agent"];

// AFTER (correct)
let module_categories = ["domain"];
```

### Also Fix: Silent Success

**File:** `core/crates/entry/cli/src/services/db/mod.rs` line 96-99

```rust
// BEFORE (silent failure)
if all_modules.is_empty() {
    CliService::warning("No modules found - check CORE_PATH environment variable");
    return Ok(());
}

// AFTER (fail loudly)
if all_modules.is_empty() {
    bail!("No modules found in {}. Check CORE_PATH environment variable.", config.core_path);
}
```
