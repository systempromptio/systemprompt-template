---
title: "Skills"
description: "Create and manage AI skills from the dashboard. Skills are reusable capabilities with content, tags, and file attachments that can be assigned to plugins and invoked by agents."
author: "systemprompt.io"
slug: "skills"
keywords: "skills, capabilities, agents, tags, custom skills, skill files, dashboard"
kind: "guide"
public: true
tags: ["skills", "dashboard", "configuration"]
published_at: "2026-02-18"
updated_at: "2026-02-18"
after_reading_this:
  - "Navigate the Skills page and manage your skill library"
  - "Create and edit custom skills with content, tags, and descriptions"
  - "Understand the difference between system and custom skills"
  - "Customize system skills by forking them into editable copies"
related_docs:
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "Create Plugin"
    url: "/documentation/create-plugin"
  - title: "Agents"
    url: "/documentation/agents"
  - title: "Hooks"
    url: "/documentation/hooks"
---

# Skills

**TL;DR:** Skills are the individual capabilities that agents can perform. The Skills page lets you browse, create, edit, enable, and delete custom skills. System skills come from plugin configurations and can be forked into editable custom copies. Each skill has an ID, name, description, content body, and optional tags.

## What You'll See

Navigate to **Skills** in the sidebar. The page shows:

- A **search bar** to filter skills by name.
- A **+ New Skill** button to create a custom skill.
- A table listing every skill available to your account.

### Skills Table

| Column | Description |
|--------|-------------|
| **Name** | The skill's display name. |
| **Skill ID** | The unique identifier shown as inline code (e.g., `email-composer`). |
| **Description** | A truncated summary of what the skill does. Hover for the full text. |
| **Status** | Toggle switch to enable or disable the skill. |
| **Actions** | Three-dot menu with Edit and Delete options. |

### Role-Based Visibility

- **Admins** see all skills across every plugin.
- **Non-admin users** see only skills that belong to plugins matching their assigned roles. If a skill is part of a plugin you do not have access to, it will not appear in your list.

## System Skills vs. Custom Skills

There are two types of skills:

| Type | Source | Editable | Command format |
|------|--------|----------|----------------|
| **System** | Defined in plugin `config.yaml` files under `services/skills/` | No (fork to customize) | `/skill-name` |
| **Custom** | Created by users through the dashboard | Yes | `/custom:<skill_id>` |

System skills are read-only because they come from plugin configuration files on disk. To modify a system skill's behavior, use the **Customize** button (available on the Plugins page) to fork it into a custom copy.

## Creating a Skill

Click **+ New Skill** to open the skill editor in create mode.

### Skill Fields

| Field | Required | Description |
|-------|----------|-------------|
| **Skill ID** | yes | Unique identifier in kebab-case (e.g., `my-skill`). Cannot be changed after creation. |
| **Name** | yes | Human-readable display name. |
| **Description** | no | Brief explanation of what the skill does. |
| **Content** | no | The skill's body content. This is the instruction text or prompt template that defines the skill's behavior. Supports Markdown. Displayed in a large textarea (15 rows). |
| **Tags** | no | Comma-separated tags for categorization and discovery (e.g., `writing, email, communication`). |

Click **Save** to create the skill, or **Cancel** to discard.

### How Content Works

The content field is the core of a skill. It contains the instructions, templates, or prompt text that an agent receives when the skill is invoked. For example, a "code review" skill might contain:

```
Review the provided code for:
- Security vulnerabilities
- Performance issues
- Code style consistency
- Missing error handling

Provide specific, actionable feedback with line references.
```

When an agent uses this skill, the content is included in the agent's context.

## Editing a Skill

Click **Edit** from the actions menu on any skill row, or navigate to `/admin/skills/edit/?id=<skill_id>`. The edit form shows the same fields as creation, except the Skill ID is read-only.

Changes are saved via the API when you click **Save**. The form submits:

- `name` -- updated display name
- `description` -- updated description
- `content` -- updated body content
- `tags` -- parsed from the comma-separated input into an array

## Enabling and Disabling Skills

Toggle the switch in the Status column to enable or disable a skill. Disabled skills remain in the system but are not available to agents. The toggle calls the API immediately -- no save button needed.

The enabled state is persisted in the database and survives restarts.

## Deleting a Skill

Select **Delete** from the actions menu on a skill row. A confirmation prompt will appear. Deletion removes the skill from the database permanently.

Only custom skills can be deleted from the Skills page. System skills are managed through their parent plugin configuration.

## Customizing System Skills

System skills cannot be edited directly, but you can fork them:

1. Go to the **Plugins** page.
2. Expand a plugin's detail row or open the Skills tab.
3. Click the **Customize** button next to a system skill.
4. The system creates a custom copy linked to the original via `base_skill_id`.
5. The copy appears with a "customized" badge and can be freely edited.

Customized skills override their base skill for the current user. The original system skill remains unchanged.

## Skill Files

Skills can have associated files that provide additional context or reference material. On the Plugins page, click the **Files** button next to any skill to browse its attachments.

File metadata includes:

| Property | Description |
|----------|-------------|
| **file_path** | Path to the file within the skill's directory. |
| **category** | File classification (e.g., `template`, `example`, `reference`). |
| **language** | Programming language or file type. |
| **size_bytes** | File size in bytes. |

File contents can be viewed and edited through the API. Changes to skill files are synced back to disk.

## Skills on the Plugins Page

The Plugins page also shows a **Skills** tab with a more detailed view of all skills across the system. This tab adds columns for:

- **Command** -- the slash command that invokes the skill.
- **Source** -- "system" (from plugin config) or "custom" (user-created).
- **Plugin** -- which plugin the skill belongs to.

The Plugins page skill view also shows aggregate stats:
- Total skills / enabled skills
- Custom skill count / system skill count

## Associating Skills with Plugins

Skills are associated with plugins through the plugin's configuration:

- When **creating a plugin**, select skills in Step 2 of the wizard.
- When **editing a plugin**, use the Skills checklist to add or remove skill associations.
- Through the API, use the update plugin skills endpoint to modify the list.

A skill can belong to multiple plugins simultaneously. Adding or removing a skill from a plugin does not delete the skill itself.
