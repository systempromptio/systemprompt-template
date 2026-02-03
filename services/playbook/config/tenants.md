---
title: "Tenant Management"
description: "Configure multi-tenant isolation with local and cloud tenants."
author: "SystemPrompt"
slug: "config-tenants"
keywords: "tenants, multi-tenancy, isolation, local, cloud, database"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Tenant Management

Configure multi-tenant isolation with local and cloud tenants.

> **Help**: `{ "command": "cloud tenant list" }` via `systemprompt_help`
> **Requires**: Credentials configured -> See [Credentials Playbook](../credentials/index.md)

Tenants are isolated environments that own databases and configuration.

---

## StoredTenant Struct

**Source**: `crates/infra/cloud/src/tenants.rs:32-66`

```rust
pub struct StoredTenant {
    #[validate(length(min = 1))]
    pub id: String,                            // Line 35 - Unique identifier
    #[validate(length(min = 1))]
    pub name: String,                          // Line 38 - Display name
    pub app_id: Option<String>,                // Line 41 - Fly.io app ID
    pub hostname: Option<String>,              // Line 44 - Deployment hostname
    pub region: Option<String>,                // Line 47 - Deployment region
    pub database_url: Option<String>,          // Line 50 - Local DB URL
    pub internal_database_url: Option<String>, // Line 53 - Cloud internal URL
    #[serde(default)]
    pub tenant_type: TenantType,               // Line 56 - Local | Cloud
    #[serde(default)]
    pub external_db_access: bool,              // Line 59 - Allow external connections
    pub sync_token: Option<String>,            // Line 62 - Sync authentication
    pub shared_container_db: Option<String>,   // Line 65 - Shared DB reference
}
```

### Field Details

| Field | Type | Local Tenants | Cloud Tenants |
|-------|------|---------------|---------------|
| `id` | String | `local_abc123` | UUID |
| `name` | String | User-defined | User-defined |
| `app_id` | Option | None | `sp-{tenant-id}` |
| `hostname` | Option | None | `{tenant-id}.systemprompt.io` |
| `region` | Option | None | `iad`, `lhr`, `syd` |
| `database_url` | Option | PostgreSQL URL | None (use internal) |
| `internal_database_url` | Option | None | Internal Fly.io URL |
| `tenant_type` | TenantType | `Local` | `Cloud` |
| `external_db_access` | bool | true/false | Usually false |
| `sync_token` | Option | Optional | Required for sync |
| `shared_container_db` | Option | Container name | None |

### TenantType Enum

```rust
pub enum TenantType {
    #[default]
    Local,    // Development tenant on local machine
    Cloud,    // Production tenant on SystemPrompt Cloud
}
```

---

## TenantStore Struct

**Source**: `crates/infra/cloud/src/tenants.rs:226-327`

```rust
pub struct TenantStore {
    #[validate]
    pub tenants: Vec<StoredTenant>,            // Line 229 - All registered tenants
    pub synced_at: DateTime<Utc>,              // Line 231 - Last sync timestamp
}
```

### Storage Location

The tenant store is saved to `tenants.json`:

```yaml
# profile.yaml
cloud:
  tenants_path: "../../tenants.json"  # Relative to profile dir
```

Default location: `.systemprompt/tenants.json`

---

## Tenant Store Methods

### Loading

```rust
impl TenantStore {
    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(CloudError::TenantsNotSynced);
        }

        let content = fs::read_to_string(path)?;
        let store: TenantStore = serde_json::from_str(&content)?;

        // Validate all tenants
        store.validate()?;
        Ok(store)
    }
}
```

### Saving

```rust
impl TenantStore {
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists with .gitignore
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
                fs::write(parent.join(".gitignore"), "*")?;
            }
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, &content)?;

        // Set Unix permissions to 0o600 (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }
}
```

### Finding Tenants

```rust
impl TenantStore {
    pub fn find_tenant(&self, id: &str) -> Option<&StoredTenant> {
        self.tenants.iter().find(|t| t.id == id)
    }

    pub fn is_empty(&self) -> bool {
        self.tenants.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tenants.len()
    }
}
```

### Staleness Check

```rust
impl TenantStore {
    pub fn is_stale(&self, max_age: Duration) -> bool {
        Utc::now() - self.synced_at > max_age
    }
}
```

---

## Creating Tenants

### Local Tenant

```rust
impl StoredTenant {
    pub fn new_local(id: String, name: String, database_url: String) -> Self {
        Self {
            id,
            name,
            database_url: Some(database_url),
            tenant_type: TenantType::Local,
            ..Default::default()
        }
    }

    pub fn new_local_shared(
        id: String,
        name: String,
        database_url: String,
        shared_container_db: String
    ) -> Self {
        Self {
            id,
            name,
            database_url: Some(database_url),
            shared_container_db: Some(shared_container_db),
            tenant_type: TenantType::Local,
            ..Default::default()
        }
    }
}
```

### Cloud Tenant

```rust
pub struct NewCloudTenantParams {
    pub id: String,
    pub name: String,
    pub app_id: String,
    pub hostname: String,
    pub region: String,
    pub internal_database_url: Option<String>,
    pub sync_token: Option<String>,
}

impl StoredTenant {
    pub fn new_cloud(params: NewCloudTenantParams) -> Self {
        Self {
            id: params.id,
            name: params.name,
            app_id: Some(params.app_id),
            hostname: Some(params.hostname),
            region: Some(params.region),
            internal_database_url: params.internal_database_url,
            sync_token: params.sync_token,
            tenant_type: TenantType::Cloud,
            ..Default::default()
        }
    }
}
```

---

## State Query Methods

**Source**: `crates/infra/cloud/src/tenants.rs:159-224`

```rust
impl StoredTenant {
    // Check if using shared PostgreSQL container
    pub fn uses_shared_container(&self) -> bool {
        self.shared_container_db.is_some()
    }

    // Check if has local database URL
    pub fn has_database_url(&self) -> bool {
        self.database_url.is_some()
    }

    // Get local database URL
    pub fn get_local_database_url(&self) -> Option<&String> {
        self.database_url.as_ref()
    }

    // Type checks
    pub fn is_cloud(&self) -> bool {
        matches!(self.tenant_type, TenantType::Cloud)
    }

    pub fn is_local(&self) -> bool {
        matches!(self.tenant_type, TenantType::Local)
    }

    // Cloud credential checks
    pub fn is_sync_token_missing(&self) -> bool {
        self.is_cloud() && self.sync_token.is_none()
    }

    pub fn is_database_url_masked(&self) -> bool {
        self.database_url
            .as_ref()
            .map(|url| url.contains(":***@"))
            .unwrap_or(false)
    }

    pub fn has_missing_credentials(&self) -> bool {
        self.is_cloud() && (
            self.is_sync_token_missing() ||
            !self.is_database_url_masked()
        )
    }
}
```

---

## tenants.json Format

```json
{
  "tenants": [
    {
      "id": "local_19bff27604c",
      "name": "my-project",
      "tenant_type": "local",
      "database_url": "postgres://systemprompt:localdev@localhost:5432/systemprompt",
      "external_db_access": false,
      "shared_container_db": "systemprompt-shared-db"
    },
    {
      "id": "999bc654-9a64-49bc-98be-db976fc84e76",
      "name": "my-project-prod",
      "tenant_type": "cloud",
      "app_id": "sp-999bc6549a64",
      "hostname": "999bc6549a64.systemprompt.io",
      "region": "iad",
      "internal_database_url": "postgres://user:pass@internal-db:5432/tenant",
      "sync_token": "sp_sync_abc123..."
    }
  ],
  "synced_at": "2026-02-01T10:00:00Z"
}
```

---

## Linking Tenants to Profiles

Profiles reference tenants via `cloud.tenant_id`:

```yaml
# .systemprompt/profiles/local/profile.yaml
cloud:
  tenant_id: local_19bff27604c
  tenants_path: "../../tenants.json"
```

### Resolution Flow

1. Profile specifies `cloud.tenant_id`
2. Load TenantStore from `cloud.tenants_path`
3. Find tenant by ID: `TenantStore::find_tenant(id)`
4. Use tenant's database URL or internal URL

---

## Local vs Cloud Tenant Differences

| Aspect | Local Tenant | Cloud Tenant |
|--------|--------------|--------------|
| Database | Docker container | Managed PostgreSQL |
| URL | `database_url` field | `internal_database_url` field |
| Hostname | `localhost:port` | `{id}.systemprompt.io` |
| Sync token | Optional | Required for sync |
| App ID | None | Fly.io app ID |
| Region | None | `iad`, `lhr`, `syd` |
| External access | Configurable | Usually disabled |

---

## Sync Token

Cloud tenants use sync tokens for authenticated synchronization:

```json
{
  "id": "999bc654-...",
  "sync_token": "sp_sync_abc123..."
}
```

### Sync Token Usage

- Required for `systemprompt cloud sync push/pull`
- Validates write access to tenant
- Rotatable via `systemprompt cloud tenant rotate-sync-token`

### Missing Sync Token Check

```rust
if tenant.is_sync_token_missing() {
    warn!("Sync token missing for cloud tenant {}", tenant.id);
}
```

---

## File Security

### Permissions

Tenant files are saved with restricted permissions:

```rust
#[cfg(unix)]
fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
```

- Owner: read + write
- Group: none
- Other: none

### Gitignore

Parent directory automatically gets `.gitignore`:

```rust
if !parent.exists() {
    fs::create_dir_all(parent)?;
    fs::write(parent.join(".gitignore"), "*")?;
}
```

---

## CLI Commands

### List Tenants

```bash
systemprompt cloud tenant list
```

### Show Tenant Details

```bash
systemprompt cloud tenant show
systemprompt cloud tenant show <tenant-id>
```

### Create Tenant

```bash
# Local tenant
systemprompt cloud tenant create --type local --name my-project

# Cloud tenant
systemprompt cloud tenant create --region iad --name my-project
```

### Select Tenant

```bash
systemprompt cloud tenant select <tenant-id>
```

### Rotate Credentials

```bash
systemprompt cloud tenant rotate-credentials <tenant-id> -y
systemprompt cloud tenant rotate-sync-token <tenant-id> -y
```

---

## Troubleshooting

**"Tenants not synced"**
- Run `systemprompt cloud tenant list` to sync
- Verify credentials are valid

**"Tenant not found"**
- Check tenant ID in profile's `cloud.tenant_id`
- Run `systemprompt cloud tenant list` to refresh

**"Database connection failed"**
- For local: ensure Docker container is running
- For cloud: verify `internal_database_url` is set

**"Sync token missing"**
- Cloud tenants require sync token for sync operations
- Run `systemprompt cloud tenant rotate-sync-token`

**"Permission denied"**
- Check file permissions on tenants.json
- Should be `0600` (owner read/write only)

---

## Quick Reference

| Task | Command |
|------|---------|
| List tenants | `systemprompt cloud tenant list` |
| Show tenant | `systemprompt cloud tenant show <id>` |
| Create local | `systemprompt cloud tenant create --type local` |
| Create cloud | `systemprompt cloud tenant create --region iad` |
| Select tenant | `systemprompt cloud tenant select <id>` |
| Rotate creds | `systemprompt cloud tenant rotate-credentials <id>` |
| Rotate sync | `systemprompt cloud tenant rotate-sync-token <id>` |
| Delete tenant | `systemprompt cloud tenant delete <id> -y` |