---
title: "Activity Tracking"
description: "How the Enterprise Demo platform captures and classifies every AI interaction through 13 activity categories, 11 action types, and 13 entity types."
author: "systemprompt.io"
slug: "activity-tracking"
keywords: "activity tracking, event categories, activity categories, audit trail, event hooks, webhook"
kind: "guide"
public: true
tags: ["analytics", "observability", "activity"]
published_at: "2026-03-25"
updated_at: "2026-03-25"
after_reading_this:
  - "Understand the 13 activity categories and what triggers each one"
  - "Know the 11 action types and 13 entity types used for event classification"
  - "Understand how webhook events from Claude Code are mapped to activity records"
related_docs:
  - title: "Dashboard"
    url: "/documentation/dashboard"
  - title: "Metrics Reference"
    url: "/documentation/metrics-reference"
  - title: "Audit Trails & Events"
    url: "/documentation/events"
  - title: "CLI Analytics"
    url: "/documentation/cli-analytics"
---

# Activity Tracking

Every AI interaction is captured, classified, and stored as a structured activity record. The platform ingests webhook events from Claude Code sessions and maps them to a rich activity model with categories, actions, and entity types.

## Activity Categories

The platform defines 13 activity categories, each representing a distinct type of AI interaction:

| Category | Description | Triggered By |
|----------|-------------|--------------|
| **Login** | User authentication events | OAuth login, magic link, passkey |
| **Session** | AI session lifecycle | `claude_code_SessionStart`, `claude_code_SessionEnd` |
| **Prompt** | User prompt submissions | `claude_code_UserPromptSubmit` |
| **SkillUsage** | Skill and tool invocations | `claude_code_PostToolUse` with skill context |
| **MarketplaceEdit** | Skill, plugin, agent, hook, or marketplace CRUD operations | Admin create/update/delete actions |
| **MarketplaceConnect** | Marketplace upload and restore operations | Marketplace sync, import, restore |
| **UserManagement** | User administration events | Role changes, department assignments |
| **ToolUsage** | Internal tool usage | `claude_code_PostToolUse` (Bash, Read, Write, Edit, etc.) |
| **Error** | Tool failures and errors | `claude_code_PostToolUseFailure` |
| **AgentResponse** | Agent completion events | `claude_code_Stop` |
| **Notification** | Permission and notification prompts | `claude_code_Notification` |
| **TaskCompletion** | Task completion events | `claude_code_TaskCompleted` |
| **Compaction** | Context compaction events | `claude_code_PreCompact` |

These categories map directly to the `ActivityCategory` enum in the Rust codebase at `extensions/web/src/admin/activity/enums.rs`.

## Action Types

Each activity record includes an action type that describes what happened:

| Action | Description |
|--------|-------------|
| **LoggedIn** | User authenticated successfully |
| **Started** | Session or process began |
| **Ended** | Session or process completed |
| **Submitted** | User submitted a prompt or form |
| **Used** | Tool or skill was invoked |
| **Created** | New entity was created |
| **Updated** | Existing entity was modified |
| **Deleted** | Entity was removed |
| **Imported** | Entity was imported from external source |
| **Uploaded** | Content was uploaded to marketplace |
| **Restored** | Entity was restored from backup or version |

## Entity Types

Activity records track which type of entity was involved:

| Entity | Description |
|--------|-------------|
| **Session** | AI conversation session |
| **Skill** | Platform-level skill |
| **Plugin** | Plugin bundle |
| **Hook** | Event hook |
| **McpServer** | MCP server |
| **Marketplace** | Marketplace collection |
| **User** | User account |
| **Prompt** | User prompt |
| **Agent** | AI agent |
| **UserSkill** | User-forked skill |
| **UserAgent** | User-forked agent |
| **UserHook** | User-forked hook |
| **Tool** | Internal tool (Bash, Read, Write, etc.) |

## Webhook Event Mapping

Claude Code sends webhook events to the platform in real time. The activity recording system maps each `claude_code_*` event to structured activity records:

| Webhook Event | Category | Action | Details Captured |
|---------------|----------|--------|-----------------|
| `claude_code_SessionStart` | Session | Started | Model, project path, source |
| `claude_code_SessionEnd` | Session | Ended | Session duration |
| `claude_code_UserPromptSubmit` | Prompt | Submitted | Prompt content |
| `claude_code_PostToolUse` | ToolUsage | Used | Tool name, parameters, output |
| `claude_code_PostToolUseFailure` | Error | Used | Tool name, error message |
| `claude_code_SubagentStart` | Session | Started | Subagent type |
| `claude_code_Stop` | AgentResponse | Ended | Response metadata |
| `claude_code_StatusLine` | *(aggregation only)* | — | Cost, tokens, model |
| `claude_code_Notification` | Notification | Submitted | Notification content |
| `claude_code_TaskCompleted` | TaskCompletion | Ended | Task metadata |
| `claude_code_PreCompact` | Compaction | Started | Compaction metadata |

## Rich Event Details

The activity recording system extracts contextual details from each event:

- **Tool details:** Bash commands executed, files read/written, search queries
- **Session info:** Model used, project path, session source
- **Error messages:** Failure reasons for tool errors
- **Subagent types:** Type and context of spawned subagents

All details are stored as JSONB metadata, enabling flexible querying and filtering.

## Storage

Activity records are stored in two tables:

- **`user_activity`** — Rich timeline events with category, action, entity type, entity name, description, and metadata
- **`plugin_usage_events`** — Raw webhook events with event type, tool name, plugin ID, session ID, and metadata

Daily aggregations in `plugin_usage_daily` and session summaries in `plugin_session_summaries` provide fast querying for dashboard metrics.
