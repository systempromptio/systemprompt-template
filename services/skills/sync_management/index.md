---
title: "Sync Management Skill"
slug: "sync-management"
description: "Guide for synchronizing files and database between local and cloud"
author: "systemprompt"
published_at: "2024-01-01"
type: "skill"
category: "skills"
keywords: "sync, files, database, push, pull, cloud, infrastructure"
---

# Sync Management

You manage synchronization between local and cloud environments. Use the infrastructure MCP tools to keep configurations and data in sync.

## Available Tools

### sync_files
Syncs configuration files:
- Agent YAML configurations
- Skill configurations and content
- Web theme configurations
- Content markdown files

**Directions:**
- `push` - Upload local files to cloud (overwrites cloud)
- `pull` - Download cloud files to local (overwrites local)

### sync_database
Syncs database records:
- Agent definitions
- Skill definitions
- Context configurations

**Directions:**
- `push` - Upload local database to cloud
- `pull` - Download cloud database to local

### sync_status
Check current sync state:
- Last sync timestamps
- Pending changes
- Deployment status

### sync_all
Complete sync operation:
- Syncs all files
- Syncs all database records
- Optionally deploys (on push)

## Sync Workflows

### Push Local Changes to Cloud

1. **Check Status**
   ```
   sync_status
   ```

2. **Sync Files**
   ```
   sync_files with direction: "push"
   ```

3. **Sync Database**
   ```
   sync_database with direction: "push"
   ```

Or use `sync_all` with direction "push" for everything.

### Pull Cloud State to Local

1. **Check Status**
   ```
   sync_status
   ```

2. **Sync Files**
   ```
   sync_files with direction: "pull"
   ```

3. **Sync Database**
   ```
   sync_database with direction: "pull"
   ```

Or use `sync_all` with direction "pull" for everything.

## What Gets Synced

### Files (sync_files)

| Category | Location | Examples |
|----------|----------|----------|
| Agents | `services/agents/` | agent.yml configs |
| Skills | `services/skills/` | config.yml, index.md |
| Content | `services/content/` | blog posts, pages |
| Web | `services/web/` | theme config |

### Database (sync_database)

| Table | Contents |
|-------|----------|
| agents | Agent definitions |
| skills | Skill definitions |
| contexts | Context configurations |

## Best Practices

- **Always check status first** before syncing
- **Backup before pull** if you have local changes
- **Push after local edits** to update cloud
- **Pull before local edits** to get latest state
- **Use sync_all sparingly** - it does everything

## Conflict Resolution

Sync operations are **overwrite** operations:
- `push` overwrites cloud with local
- `pull` overwrites local with cloud

There is no merge - be careful with sync direction.

## Example Prompts

- "Sync my local config changes to cloud"
- "Pull the latest configuration from production"
- "What's the current sync status?"
- "Push all my changes including database"
