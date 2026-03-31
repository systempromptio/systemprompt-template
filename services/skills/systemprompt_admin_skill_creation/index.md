---
name: "Skill Creation"
description: "Create, structure, and publish skills through the skill-plugin-marketplace pipeline"
---

# Skill Creation

You create, structure, and publish skills for the demo.systemprompt.io platform. Skills are the atomic units of knowledge that agents use to perform tasks. Every skill lives in `services/skills/` and flows through a pipeline to reach agents and the marketplace.

## The Skill-Plugin-Marketplace Pipeline

```
services/skills/{skill_id}/        <- 1. AUTHOR HERE (source of truth)
├── config.yaml                       Metadata: id, name, description, tags
└── index.md                          Content: frontmatter + markdown body

        ↓ referenced by

services/plugins/{plugin-id}/      <- 2. PLUGIN BUNDLES SKILLS
├── config.yaml                       Lists skill IDs in skills.include
└── .claude-plugin/plugin.json        Claude Code plugin metadata

        ↓ exported via `systemprompt core plugins generate`

Plugin output bundle               <- 3. EXPORTED AS SKILL.md
└── skills/{kebab-name}/SKILL.md      Rebuilt: frontmatter (name + description) + body

        ↓ imported by marketplace users

Marketplace / User environment     <- 4. CONSUMED BY AGENTS
├── config.yaml (recreated)           Parsed from frontmatter
└── index.md (recreated)              Parsed from body
```

**Key principle:** The skill itself always lives in `services/skills/{skill_id}/`. Plugins reference skill IDs -- they do not contain skill content. When a plugin is exported, skills are bundled into `SKILL.md` files. When imported, they are unpacked back into `config.yaml` + `index.md`.

## Skill Directory Structure

Every skill is a directory under `services/skills/` with exactly two files:

```
services/skills/{skill_id}/
├── config.yaml          <- System metadata (id, name, enabled, tags, assigned_agents)
└── index.md             <- Skill content (frontmatter + markdown instructions)
```

Optional subdirectories for complex skills:

```
services/skills/{skill_id}/
├── config.yaml
├── index.md
├── references/          <- Reference data (CSV, JSON, templates)
├── templates/           <- Output templates
└── examples/            <- Example inputs/outputs
```

## Creating a Skill

### Step 1: Create the directory and config

```bash
# CLI method (interactive)
systemprompt core skills create --name my_skill --display-name "My Skill" --description "What this skill does" --tags "tag1,tag2"

# Or create manually:
mkdir -p services/skills/my_skill
```

**`config.yaml` format:**

```yaml
id: my_skill
name: "My Skill"
description: "What this skill does in one sentence"
enabled: true
version: "1.0.0"
file: "index.md"
assigned_agents:
  - agent_name
tags:
  - tag1
  - tag2
```

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique identifier, `snake_case`, must match directory name |
| `name` | Yes | Human-readable display name |
| `description` | Yes | One-sentence description |
| `enabled` | Yes | Whether skill is active |
| `version` | Yes | Semantic version |
| `file` | Yes | Always `"index.md"` |
| `assigned_agents` | No | List of agent names that use this skill |
| `tags` | No | Search/categorization tags |

### Step 2: Write the index.md

The `index.md` must start with YAML frontmatter between `---` markers:

```markdown
---
name: "My Skill"
description: "What this skill does in one sentence"
---

# My Skill

Instructions for the agent go here...
```

**Frontmatter fields** (parsed by the export/import pipeline):

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Skill name (used during marketplace import) |
| `description` | Yes | Skill description (used during marketplace import) |

The `name` and `description` in frontmatter must match the values in `config.yaml`. During plugin export, the system strips existing frontmatter from `index.md` and rebuilds it using values from `config.yaml`, so `config.yaml` is the source of truth for metadata. The frontmatter in `index.md` exists so that standalone `SKILL.md` files (after export) are self-describing.

**Body content** (after the closing `---`) is the full skill instruction set in markdown. Write it as if speaking directly to the agent: "You do X. Use Y command. Follow Z workflow."

### Step 3: Register the skill

Add the skill to `services/skills/config.yaml`:

```yaml
includes:
  - my_skill/config.yaml
```

### Step 4: Assign to a plugin

Add the skill ID to a plugin's `config.yaml`:

```yaml
# services/plugins/{plugin-id}/config.yaml
plugin:
  skills:
    source: explicit
    include:
      - my_skill          # Add here
```

### Step 5: Sync and validate

```bash
# Sync skill to database
systemprompt core skills sync --direction to-db

# Verify it loaded
systemprompt core skills show my_skill

# Validate the plugin
systemprompt core plugins validate {plugin-id}

# Generate plugin output (bundles skills into SKILL.md)
systemprompt core plugins generate --id {plugin-id}
```

## CLI Commands

| Command | Purpose |
|---------|---------|
| `systemprompt core skills list` | List all skills |
| `systemprompt core skills show <skill_id>` | Show skill details |
| `systemprompt core skills create` | Create a new skill (interactive) |
| `systemprompt core skills create --name <id> --display-name "Name" --description "desc"` | Create non-interactively |
| `systemprompt core skills edit <skill_id>` | Edit skill config |
| `systemprompt core skills edit <skill_id> --enable` | Enable a skill |
| `systemprompt core skills edit <skill_id> --disable` | Disable a skill |
| `systemprompt core skills edit <skill_id> --set key=value` | Set a specific field |
| `systemprompt core skills delete <skill_id>` | Delete a skill |
| `systemprompt core skills status` | Check DB sync status |
| `systemprompt core skills sync --direction to-db` | Push disk skills to database |
| `systemprompt core skills sync --direction to-disk` | Pull database skills to disk |
| `systemprompt core skills sync --dry-run` | Preview sync without changes |
| `systemprompt core plugins validate <plugin-id>` | Validate plugin (checks skill references) |
| `systemprompt core plugins generate --id <plugin-id>` | Generate plugin export (bundles SKILL.md) |

## How Skills Reach Agents

```
config.yaml: assigned_agents → Agent loads skill at startup
Plugin config: skills.include → Plugin bundles skill for distribution
Agent YAML: metadata.skills  → Agent advertises skill in A2A card
```

An agent receives a skill when **both** of these are true:
1. The skill's `config.yaml` lists the agent in `assigned_agents`
2. The agent's YAML lists the skill ID in `metadata.skills`

## How Skills Reach the Marketplace

1. Skill exists in `services/skills/{id}/` with `config.yaml` + `index.md`
2. A plugin references the skill ID in its `skills.include` list
3. `systemprompt core plugins generate` bundles the skill as `SKILL.md` (frontmatter + body)
4. The plugin is uploaded to the marketplace
5. Users install the plugin, which imports skills back into `config.yaml` + `index.md`

## Naming Conventions

| Convention | Example |
|------------|---------|
| Skill ID | `snake_case` — `my_custom_skill` |
| Skill directory | Must match ID — `services/skills/my_custom_skill/` |
| Plugin ID | `kebab-case` — `my-plugin` |
| Agent name | `snake_case` — `my_agent` |
| Exported SKILL.md path | `skills/{kebab-case}/SKILL.md` — `skills/my-custom-skill/SKILL.md` |

Prefix skill IDs with a namespace when they belong to a specific plugin (e.g., `systemprompt_admin_*`, `sales_crm_*`).

## Common Tasks

### Create a Skill from Scratch

```bash
mkdir -p services/skills/my_skill
# Write config.yaml and index.md (see formats above)
# Add to services/skills/config.yaml includes
# Add to plugin config.yaml skills.include
systemprompt core skills sync --direction to-db
systemprompt core skills show my_skill
systemprompt core plugins validate my-plugin
```

### Move a Skill Between Plugins

Remove the skill ID from one plugin's `skills.include` and add it to another's. The skill directory does not move -- only the plugin reference changes.

```bash
# Edit both plugin configs, then validate
systemprompt core plugins validate old-plugin
systemprompt core plugins validate new-plugin
```

### Export a Plugin with Skills

```bash
systemprompt core plugins generate --id my-plugin
# Output goes to services/plugins/my-plugin/ (or --output-dir)
```

### Debug a Missing Skill

```bash
systemprompt core skills list
systemprompt core skills status
systemprompt core skills sync --dry-run
systemprompt core plugins validate my-plugin
```

## Important Notes

- Skill IDs must be unique across the entire platform
- The `id` in `config.yaml` must match the directory name exactly
- Frontmatter `name` and `description` should match `config.yaml` values
- During export, frontmatter is rebuilt from `config.yaml` -- `config.yaml` is the source of truth
- Skills are never duplicated into plugin directories -- plugins only reference IDs
- Auxiliary files in `references/`, `templates/`, `examples/` are included in plugin exports
- Always run `core skills sync` after creating or modifying skills on disk
- Always run `core plugins validate` after changing plugin skill references
