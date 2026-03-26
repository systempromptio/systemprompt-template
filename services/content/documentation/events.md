---
title: "Events"
description: "Browse and search the audit log of all AI activity events. Filter by type, inspect metadata, and track tool usage, sessions, prompts, and errors across your organization."
author: "systemprompt.io"
slug: "events"
keywords: "events, audit log, activity, tool use, sessions, monitoring, compliance"
kind: "guide"
public: true
tags: ["events", "audit", "admin"]
published_at: "2026-02-18"
updated_at: "2026-02-18"
after_reading_this:
  - "Browse the full event log with pagination"
  - "Search events by user, tool, or session"
  - "Understand the different event types"
  - "Inspect raw event metadata for debugging"
related_docs:
  - title: "Dashboard"
    url: "/documentation/dashboard"
  - title: "Users"
    url: "/documentation/users"
  - title: "Hooks"
    url: "/documentation/hooks"
  - title: "Jobs"
    url: "/documentation/jobs"
  - title: "Presentation"
    url: "/documentation/presentation"
---

# Events

**TL;DR:** The Events page is a paginated, searchable audit log of every AI activity event in your organization. Each event records who did what, when, with which tool, in which session. You can expand any row to inspect full metadata including the raw JSON payload. This page is admin-only.

> **See this in the presentation:** [Slide 11: Full AI Trace](/documentation/presentation#slide-11)

## Access Control

The Events page (`/admin/events/`) is **admin-only**. Non-admin users receive a 403 Forbidden response.

## What You'll See

### Event Table

The main view is a data table with the following columns:

| Column | Description |
|--------|-------------|
| **Timestamp** | When the event occurred, formatted as a date |
| **User** | Display name of the user who triggered the event |
| **Event Type** | Color-coded badge indicating what happened |
| **Tool** | The tool or skill name involved, shown as inline code |
| **Plugin** | The plugin ID associated with the event |
| **Session** | Truncated session ID shown as inline code |

### Event Types

Events are classified by type, each displayed with a distinct color badge:

| Type | Badge | Description |
|------|-------|-------------|
| `claude_code_PostToolUse` | Blue "Tool Use" | A tool was successfully invoked |
| `claude_code_PostToolUseFailure` | Red "Tool Failure" | A tool invocation failed |
| `claude_code_SessionStart` | Green "Session Start" | A new AI session began |
| `claude_code_SessionEnd` | Yellow "Session End" | An AI session ended |
| `claude_code_Stop` | Blue "Turn Complete" | An AI turn completed |
| `claude_code_SubagentStart` | Blue "Subagent Start" | A subagent was spawned |
| `claude_code_SubagentStop` | Yellow "Subagent Stop" | A subagent finished |
| `claude_code_UserPromptSubmit` | Purple "User Prompt" | A user submitted a prompt |

Any other event types appear with a gray badge showing the raw type string.

### Searching

The search bar at the top accepts free-text queries. Type your search term and press **Enter** to filter events. The search matches against users, tools, and sessions. The search term is preserved in the URL as a query parameter, so you can share filtered views.

You can also filter by event type using the `event_type` query parameter in the URL (e.g., `/admin/events/?event_type=claude_code_PostToolUse`).

### Pagination

Events are displayed in pages of 50 (the default limit). Navigation controls in the toolbar show:

- Current range (e.g., "1-50 of 1,234")
- **Newer** button — Navigate to more recent events
- **Older** button — Navigate to older events

Pagination preserves your search and event type filters.

### Expanding Event Details

Click any event row to expand an inline detail panel. The expanded view shows:

| Field | Description |
|-------|-------------|
| **Event ID** | Unique identifier for the event |
| **Full Type** | Complete event type string |
| **Session ID** | Full session identifier |
| **User ID** | The user's unique ID |
| **Timestamp** | Full formatted timestamp |

A collapsible **Raw Metadata** section shows the complete JSON metadata payload for the event. This is useful for debugging hook behavior, inspecting tool parameters, or understanding what data was captured.

Click the row again to collapse the detail panel. The chevron icon rotates to indicate expanded state.

## Use Cases

### Audit Trail

Events provide a complete audit trail of all AI interactions. Every tool use, session, and prompt is recorded with the user, timestamp, and full metadata. This supports compliance requirements and organizational oversight.

### Debugging Tool Failures

Filter for `claude_code_PostToolUseFailure` events to find failed tool invocations. Expand the event to inspect the raw metadata, which includes error details and the tool parameters that caused the failure.

### Session Analysis

Search by session ID to see all events within a single AI session. This shows the complete sequence of prompts, tool uses, and subagent activity for that session.

### User Activity Investigation

Search by username to see all events for a specific user. This is useful for understanding usage patterns or investigating specific incidents.

## Troubleshooting

**No events showing** — Verify that hooks are configured to track events. Check `systemprompt infra logs view --level error` for any database issues.

**Search returns no results** — The search matches against user names, tool names, and session IDs. Try a broader search term. Note that search requires pressing Enter to execute.

**Missing event types** — Some event types only appear when specific features are in use. For example, `SubagentStart` events only appear when subagents are configured and spawned.

**Raw metadata is empty** — Some events may have minimal metadata. Tool use events typically have the richest metadata including tool parameters and results.
