---
title: "Marketplace Versions"
description: "Track version history for marketplace uploads. View per-user version groups, changelogs, skill snapshots, and restore previous versions when needed."
author: "systemprompt.io"
slug: "marketplace-versions"
keywords: "marketplace versions, version history, changelog, restore, upload, snapshots, skills"
kind: "guide"
public: true
tags: ["marketplace", "versions", "admin"]
published_at: "2026-02-18"
updated_at: "2026-02-18"
after_reading_this:
  - "View version history grouped by user"
  - "Understand the upload and restore version lifecycle"
  - "Read changelogs showing which skills were added, updated, or deleted"
  - "Restore a previous version to roll back changes"
related_docs:
  - title: "Marketplace"
    url: "/documentation/marketplace"
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "Skills"
    url: "/documentation/skills"
---

# Marketplace Versions

**TL;DR:** The Marketplace Versions page shows the complete version history of all marketplace uploads, grouped by user. Each version records a snapshot of the user's skills at that point in time, along with a changelog of what changed. You can filter by user, switch between a versions view and a changelog view, and restore any previous version to roll back unwanted changes.

## What You'll See

When you navigate to **Marketplace Versions** in the admin sidebar, the page displays:

- **User filter dropdown** — Select a specific user or view all users. Shows display name, email, or user ID for each user who has uploaded versions.
- **Tab buttons** — Toggle between the **Versions** tab (default) and the **Changelog** tab.
- **User groups** — Versions are organized into collapsible groups, one per user. Each group shows the user's label (display name or email), total version count, latest version number, and the date of the latest upload.

If no versions exist, an empty state message reads "No marketplace versions found."

## Versions Tab

The Versions tab shows version entries grouped by user. Each user group displays:

| Field | Description |
|-------|-------------|
| **Label** | User's display name, email, or user ID |
| **Version count** | Total number of versions for this user |
| **Latest version** | The most recent version number |
| **Latest date** | When the most recent version was created |

Expand a user group to see individual version entries. Each version shows:

- **Version number** — Sequential integer, starting at 1 and incrementing with each upload or restore.
- **Version type** — Either `upload` (new content pushed) or `restore` (rolled back to a previous version).
- **Skills count** — Number of skills included in this version's snapshot.
- **Skill names** — List of skill names contained in the version.
- **Created at** — Timestamp of when the version was created.

## Changelog Tab

The Changelog tab shows a chronological record of every change made to a user's skills across all versions. Select a user from the dropdown to load their changelog. Each changelog entry records:

| Field | Description |
|-------|-------------|
| **Action** | One of `added`, `updated`, `deleted`, or `restored` |
| **Skill ID** | The identifier of the affected skill |
| **Skill name** | The display name of the affected skill |
| **Detail** | A description of what changed (e.g. "new skill added", "skill removed", or specifics of what was updated) |
| **Version** | Which version this change belongs to |
| **Timestamp** | When the change was recorded |

## How Uploads Work

When a user uploads new skills via the marketplace API (`POST /marketplace/{user_id}`), the system:

1. **Extracts and parses** the uploaded archive to identify skills.
2. **Snapshots** the user's current skills as a JSON object for rollback purposes.
3. **Saves** the uploaded archive file to disk.
4. **Creates a new version** record with type `upload` and an incremented version number.
5. **Computes a diff** between the uploaded skills and the user's existing skills, identifying added, updated, and deleted skills.
6. **Applies the diff** — inserts new skills, updates changed skills, and removes deleted skills from the database.
7. **Records changelog entries** for each added, updated, or deleted skill.
8. **Prunes old versions** — keeps only the 3 most recent versions per user. Older version snapshot files are deleted from disk.
9. **Invalidates the git cache** so subsequent git pulls reflect the new state.

The upload response includes the new version number, counts of skills added/updated/deleted, and the full changelog.

## How Restores Work

To restore a previous version (`POST /marketplace/{user_id}/restore/{version_id}`), the system:

1. **Loads the target version** and its skills snapshot from the database.
2. **Snapshots the current state** before making changes (so the current state can itself be restored later).
3. **Creates a new version** record with type `restore` and an incremented version number. The snapshot path is recorded as `restore:v{N}` where N is the target version number.
4. **Restores skills** from the target version's snapshot, replacing the user's current skills entirely.
5. **Records a changelog entry** noting the restore action and how many skills were restored.
6. **Prunes old versions** following the same 3-version retention policy.

The restore response includes the restored version number, the new version number, and the count of skills restored.

## Version Retention

The system automatically prunes old versions to conserve storage. Only the **3 most recent versions** per user are retained. When a new version (upload or restore) is created and the user has more than 3 versions, the oldest versions are deleted and their snapshot files are removed from disk.
