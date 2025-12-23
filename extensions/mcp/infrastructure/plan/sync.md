# Sync System Architecture

## Overview

Two types of sync operations:

| Type | Direction | Purpose |
|------|-----------|---------|
| **Cloud Sync** | Local ↔ Production API | Transfer files/database via HTTP |
| **Local Sync** | Disk ↔ Database | Import/export YAML/markdown to PostgreSQL |

---

## 1. Data (Database) Sync

Cloud sync of database records between environments.

```
┌─────────────────┐                      ┌─────────────────┐
│  Local Postgres │                      │  Prod Postgres  │
│                 │                      │                 │
│  - agents       │  ←── Pull ───────    │  - agents       │
│  - skills       │  ─── Push ───────→   │  - skills       │
│  - contexts     │                      │  - contexts     │
└─────────────────┘                      └─────────────────┘
                         ↕
              POST/GET /api/v1/cloud/tenants/{id}/database
```

### Push (Local → Cloud)

1. Query local PostgreSQL
2. Build `DatabaseExport` JSON:
   ```rust
   DatabaseExport {
       agents: Vec<AgentExport>,
       skills: Vec<SkillExport>,
       contexts: Vec<ContextExport>,
       timestamp: DateTime<Utc>,
   }
   ```
3. POST to `/api/v1/cloud/tenants/{tenant_id}/database/import`
4. Cloud upserts all records by UUID

### Pull (Cloud → Local)

1. GET from `/api/v1/cloud/tenants/{tenant_id}/database/export`
2. Receive `DatabaseExport` JSON
3. Upsert to local PostgreSQL:
   ```sql
   INSERT INTO agents ... ON CONFLICT (id) DO UPDATE SET ...
   INSERT INTO agent_skills ... ON CONFLICT (id) DO UPDATE SET ...
   INSERT INTO contexts ... ON CONFLICT (id) DO UPDATE SET ...
   ```

---

## 2. Files Sync

Cloud sync of configuration files.

```
┌─────────────────────────┐              ┌─────────────────┐
│  Local services/        │              │  Cloud Storage  │
│                         │              │                 │
│  ├── agents/            │  ←── Pull    │  tarball.tar.gz │
│  ├── skills/            │  ─── Push →  │  + manifest     │
│  ├── config/            │              │                 │
│  ├── mcp/               │              │                 │
│  └── profiles/          │              │                 │
└─────────────────────────┘              └─────────────────┘
```

### Push (Local → Cloud)

1. Collect files from `services/`
2. Create manifest with SHA256 checksums
3. Create gzip tarball
4. POST to `/api/v1/cloud/tenants/{tenant_id}/files`

### Pull (Cloud → Local)

1. GET from `/api/v1/cloud/tenants/{tenant_id}/files`
2. Validate manifest checksums
3. Extract tarball to `services/`

---

## 3. Skills Local Sync

Sync between YAML files on disk and local database.

```
┌─────────────────────────┐              ┌─────────────────┐
│  services/skills/       │              │  Local Postgres │
│                         │              │                 │
│  skill_a/               │  ←── Pull    │  agent_skills   │
│    config.yml           │  ─── Push →  │  table          │
│    SKILL.md             │              │                 │
│  skill_b/               │              │                 │
│    config.yml           │              │                 │
└─────────────────────────┘              └─────────────────┘
```

### Push (Disk → Database)

1. Read YAML files from `services/skills/*/config.yml`
2. Read instruction files (`SKILL.md` or `index.md`)
3. Parse skill definitions
4. Calculate hash: `SHA256(name + description + instructions)`
5. Upsert to `agent_skills` table via `SkillIngestionService`

### Pull (Database → Disk)

1. Query `agent_skills` table
2. Calculate hash for comparison
3. For modified/removed skills:
   - Export to YAML config
   - Export instructions to markdown

### Diff Detection

```rust
struct SkillDiffItem {
    skill_id: String,
    file_path: String,
    status: DiffStatus,      // Added | Removed | Modified
    disk_hash: Option<String>,
    db_hash: Option<String>,
}
```

---

## 4. Content Local Sync

Sync between markdown files on disk and local database.

```
┌─────────────────────────┐              ┌─────────────────┐
│  services/content/      │              │  Local Postgres │
│                         │              │                 │
│  blog/                  │  ←── Pull    │  content        │
│    post-1.md            │  ─── Push →  │  table          │
│    post-2.md            │              │                 │
│  legal/                 │              │  (source_id     │
│    privacy.md           │              │   + slug)       │
└─────────────────────────┘              └─────────────────┘
```

### Push (Disk → Database)

1. Read markdown files with frontmatter
2. Parse title, slug, metadata from frontmatter
3. Extract body content
4. Calculate hash: `SHA256(title + body)`
5. Upsert to `content` table by `source_id + slug`

### Pull (Database → Disk)

1. Query `content` table by `source_id`
2. For modified/removed content:
   - Generate frontmatter YAML
   - Write markdown file

### Diff Detection

```rust
struct ContentDiffItem {
    slug: String,
    source_id: String,
    status: DiffStatus,
    disk_hash: Option<String>,
    db_hash: Option<String>,
}
```

---

## 5. Deploy

Only on push, after files and database sync.

```
1. cargo build --release
2. npm run build (web assets)
3. docker build -t {image}:{tag}
4. docker push {registry}/{image}:{tag}
5. POST /api/v1/cloud/tenants/{tenant_id}/deploy
6. Cloud pulls image and restarts containers
```

---

## Complete Workflows

### Local → Production (Push)

```
sync_files push      →  Upload services/*.yml
sync_database push   →  Export agents/skills/contexts
deploy_crate         →  Build → Push → Deploy
```

### Production → Local (Pull)

```
sync_files pull      →  Download services/*.yml
sync_database pull   →  Import agents/skills/contexts
(no deploy)
```

### Disk → Database (Local Push)

```
sync_skills push     →  Import YAML to agent_skills table
sync_content push    →  Import markdown to content table
```

### Database → Disk (Local Pull)

```
sync_skills pull     →  Export agent_skills to YAML
sync_content pull    →  Export content to markdown
```

---

## API Endpoints

| Operation | Method | Endpoint |
|-----------|--------|----------|
| Files Push | POST | `/api/v1/cloud/tenants/{id}/files` |
| Files Pull | GET | `/api/v1/cloud/tenants/{id}/files` |
| DB Push | POST | `/api/v1/cloud/tenants/{id}/database/import` |
| DB Pull | GET | `/api/v1/cloud/tenants/{id}/database/export` |
| Deploy | POST | `/api/v1/cloud/tenants/{id}/deploy` |

All endpoints require: `Authorization: Bearer {api_token}`

---

## Core Implementation Files

| Component | Location |
|-----------|----------|
| SyncService | `systemprompt-core/crates/app/sync/src/lib.rs` |
| FileSyncService | `systemprompt-core/crates/app/sync/src/files.rs` |
| DatabaseSyncService | `systemprompt-core/crates/app/sync/src/database.rs` |
| SkillsLocalSync | `systemprompt-core/crates/app/sync/src/local/skills_sync.rs` |
| ContentLocalSync | `systemprompt-core/crates/app/sync/src/local/content_sync.rs` |
| SyncApiClient | `systemprompt-core/crates/app/sync/src/api_client.rs` |

---

## MCP Tool Interface

```json
{
  "target": "files | database | content | skills | all",
  "direction": "push | pull",
  "dry_run": true,
  "filter": "optional filter"
}
```

| Target | Push | Pull |
|--------|------|------|
| files | Upload to cloud | Download from cloud |
| database | Export DB to cloud | Import cloud to DB |
| content | Disk → local DB | Local DB → disk |
| skills | Disk → local DB | Local DB → disk |
| all | files + database + deploy | files + database |
