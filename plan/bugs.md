# Cloud Module Bugs

## Bug #1: Local profile incorrectly requires tenant

**Status:** Open
**Severity:** Blocker
**Command:** `systemprompt cloud profile create local`

### Observed Behavior
```
Create Profile: local
✓ Auto-selected tenant: sub_01kckwezf44aq2whb0a92051jf
Error: Tenant 'sub_01kckwezf44aq2whb0a92051jf' does not have a database URL configured.
```

### Expected Behavior
- Local profiles should NOT require a cloud tenant
- Local profiles only need:
  - Local PostgreSQL connection (via docker-compose)
  - API keys for AI providers
- Cloud tenant should only be required for `production` or cloud-deployed profiles

### Root Cause
`profile create` auto-selects a tenant from `tenants.json` regardless of profile type.

### Fix Required
```rust
// In profile/create.rs
if name == "local" || name == "development" {
    // Skip tenant selection entirely
    // Use local database configuration wizard
} else {
    // Require tenant for production/staging profiles
}
```

---

## Bug #2: Auto-select tenant behavior is dubious

**Status:** Open
**Severity:** Medium
**Command:** `systemprompt cloud profile create <name>`

### Observed Behavior
```
✓ Auto-selected tenant: sub_01kckwezf44aq2whb0a92051jf
```

### Problems
1. User has no visibility into which tenant was selected
2. User cannot choose between multiple tenants
3. Silent auto-selection can lead to deploying to wrong tenant

### Expected Behavior
- If only 1 tenant: Show which tenant will be used, confirm
- If multiple tenants: Prompt user to select
- If no tenants: Error with clear message to create one first

### Fix Required
```rust
// In profile/create.rs
match tenants.len() {
    0 => bail!("No tenants found. Run 'systemprompt cloud tenant create' first."),
    1 => {
        let tenant = &tenants[0];
        println!("Using tenant: {} ({})", tenant.name, tenant.id);
        // Optionally confirm
    },
    _ => {
        // Prompt user to select from list
        let selection = Select::new()
            .with_prompt("Select tenant")
            .items(&tenant_names)
            .interact()?;
    }
}
```

---

## Bug #3: Tenant types conflated

**Status:** Open
**Severity:** High

### Problem
The system conflates two different concepts:
1. **Cloud tenant** (subscription) - `sub_*` IDs, hosted on Fly.io
2. **Local tenant** - local PostgreSQL, no cloud component

### Current State
- `tenants.json` stores cloud subscriptions
- `tenant create` is supposed to create "local or cloud" but unclear separation

### Expected Behavior
Clear separation:

| Type | ID Prefix | Database | Used For |
|------|-----------|----------|----------|
| Local | `local_*` | Docker PostgreSQL | Local development |
| Cloud | `sub_*` | Cloud-managed | Production deployment |

### Questions
1. Should local "tenants" even exist in `tenants.json`?
2. Or should local just be a profile configuration without tenant concept?

---

## Workaround

The flow works, but order matters:

```bash
# Step 1: Create a LOCAL tenant (with database_url)
just tenant-create
# → Select "Local (connect to existing PostgreSQL)"
# → Enter: localhost, 5432, systemprompt, password, systemprompt

# Step 2: Now profile create will find a tenant with database_url
just profile-create local
```

The issue is that `profile create` auto-selects the CLOUD tenant (sub_*) which has no database_url. It should prefer LOCAL tenants for local profiles, or let user choose.

---

---

## Bug #4: Local tenant should CREATE database, not connect to existing

**Status:** Open
**Severity:** Blocker
**Command:** `systemprompt cloud tenant create` → Local

### Current Behavior
```
tenant create (local)
  → Prompts for: host, port, user, password, database
  → Assumes PostgreSQL already exists
  → Saves database_url
  → Connection fails if DB doesn't exist or creds don't match
```

### Expected Behavior
```
tenant create (local)
  → Prompts for: user, password, database name
  → CREATES new PostgreSQL container with those credentials
  → Starts the container
  → Saves database_url
  → Connection guaranteed to work
```

### Implementation

```rust
// In tenant.rs create_local_tenant()

async fn create_local_tenant() -> Result<StoredTenant> {
    // 1. Collect desired credentials
    let name = prompt("Tenant name", "local");
    let db_user = prompt("Database user", "systemprompt");
    let db_password = prompt_password("Database password");
    let db_name = prompt("Database name", "systemprompt");
    let port = prompt("Port", "5432");

    // 2. Generate docker-compose.yml with these credentials
    let docker_compose = generate_docker_compose(&db_user, &db_password, &db_name, &port);
    let compose_path = ".systemprompt/docker-compose.yml";
    fs::write(compose_path, docker_compose)?;

    // 3. Start PostgreSQL container
    Command::new("docker")
        .args(["compose", "-f", compose_path, "up", "-d"])
        .status()?;

    // 4. Wait for healthy
    wait_for_postgres(&db_user, &db_password, &db_name, &port).await?;

    // 5. Create tenant with working database_url
    let database_url = format!(
        "postgres://{}:{}@localhost:{}/{}",
        db_user, db_password, port, db_name
    );

    Ok(StoredTenant::new_local(id, name, database_url))
}
```

### Flow After Fix

```bash
just tenant-create
# → Select "Local"
# → Enter: user, password, database name
# → PostgreSQL container created and started automatically
# → Tenant saved with working database_url

just profile-create local
# → Works immediately, DB already running
```

---

## Bug #5: Local tenant option lacks context

**Status:** Open
**Severity:** Low (UX)
**Command:** `systemprompt cloud tenant create`

### Current
```
? Tenant type ›
❯ Local (connect to existing PostgreSQL)
  Cloud (provision via subscription)
```

### Expected
```
? Tenant type ›
❯ Local (requires running PostgreSQL - see 'just db-up')
  Cloud (provision via subscription)
```

Or show a prerequisite check:
```
? Tenant type › Local

⚠ Local tenant requires PostgreSQL running.
  Run 'just db-up' first, or use Docker:
  docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=dev postgres:16-alpine

Continue? [y/N]
```

### Fix Location
`core/crates/entry/cli/src/cloud/tenant.rs` line 42-45

```rust
let options = vec![
    "Local (requires running PostgreSQL - run 'just db-up' first)",
    "Cloud (provision via subscription)",
];
```

---

## Suggested Fix for Bug #1

In `profile/create.rs`, when profile name is "local":

```rust
// Filter to local tenants only for local profiles
let eligible_tenants: Vec<_> = if name == "local" {
    store.tenants.iter()
        .filter(|t| t.tenant_type == TenantType::Local)
        .collect()
} else {
    store.tenants.iter().collect()
};

if eligible_tenants.is_empty() && name == "local" {
    bail!("No local tenant found. Run 'systemprompt cloud tenant create' and select 'Local'.");
}
```
