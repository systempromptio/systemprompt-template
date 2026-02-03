---
title: "Documentation Authoring Guide"
description: "Standards for creating and editing documentation pages. Structure, linking, validation, grounding."
author: "SystemPrompt"
slug: "guide-documentation"
keywords: "documentation, authoring, linking, validation, grounding"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Documentation Authoring

Standards for creating and editing documentation pages in SystemPrompt.

---

## Core Principles

1. **Prose-First** - Clear, digestible text in logical sections; code and tables are exceptions
2. **Grounded** - Every claim links to source (code, spec, or authoritative reference)
3. **Linked** - External links to code examples, not inline dumps
4. **Navigable** - Clear hierarchy, consistent linking, discoverable paths
5. **Current** - Updated when code changes, dated for freshness
6. **Single Source** - One canonical location per topic, link don't duplicate
7. **Aligned** - Content fulfills the promises made in `after_reading_this`
8. **Accessible** - Technical depth paired with plain-language summaries
9. **Collapsible** - Technical details collapsed by default when they interrupt flow

---

## File Structure

```
services/content/documentation/
├── index.md                    # Docs root - navigation hub
├── installation.md             # Installation guide
├── licensing.md                # Licensing info
├── playbooks.md                # Playbooks guide
├── config/
│   ├── index.md                # Section index
│   ├── profiles.md
│   ├── docker.md
│   └── secrets.md
├── services/
│   ├── index.md                # Section index
│   ├── agents.md
│   └── ...
└── extensions/
    ├── index.md                # Section index
    └── ...
```

---

## Required Frontmatter

```yaml
---
title: "Page Title"
description: "SEO description (150-160 characters). What the reader will learn."
slug: "section/page-name"
kind: "guide"
public: true
published_at: "2025-01-27"
after_reading_this:
  - "First learning objective (action verb)"
  - "Second learning objective"
related_playbooks:
  - title: "Playbook Name"
    url: "/playbooks/playbook-id"
related_code:
  - title: "File Name"
    url: "https://github.com/org/repo/blob/main/path/file.rs#L1-L50"
---
```

### Required Fields

| Field | Required | Notes |
|-------|----------|-------|
| `title` | Yes | Matches H1, 60 chars max |
| `description` | Yes | 150-160 chars, actionable |
| `slug` | Yes | Full path: `section/page-name` |
| `kind` | Yes | `guide`, `reference`, `docs-index`, `page` |
| `public` | Yes | `true` to publish |
| `published_at` | Yes | ISO date |
| `after_reading_this` | Yes | 3-5 learning objectives |

---

## Content Types

| Kind | Use For |
|------|---------|
| `docs-index` | Section index pages |
| `guide` | How-to guides, tutorials |
| `reference` | API, CLI, config reference |
| `page` | Static pages (FAQ, glossary) |

---

## Learning Objectives

The `after_reading_this` field defines what readers will learn. These are promises you must fulfill in the content.

**Guidelines:**
- 3-5 objectives per page
- Start with action verbs: "Install", "Configure", "Build", "Understand"
- Be specific and measurable
- Match content order

**Good examples:**
```yaml
after_reading_this:
  - "Install the SystemPrompt CLI from crates.io"
  - "Configure database connection settings"
  - "Run your first database migration"
```

---

## Heading Hierarchy

| Level | Use For | Example |
|-------|---------|---------|
| `#` H1 | Page title only | `# Extension System` |
| `##` H2 | Major sections | `## Extension Traits` |
| `###` H3 | Subsections within H2 | `### Basic Extension` |
| `####` H4 | Rare, details within H3 | `#### Constructor Options` |

---

## Linking Strategy

**Always use absolute paths from docs root:**

```markdown
<!-- Correct -->
[Installation](/documentation/installation)
[Services](/documentation/services)

<!-- Wrong -->
[Installation](../getting-started/installation.md)
```

---

## Grounding Requirements

**Every CLI command or code reference MUST include a grounding link:**

| Reference Type | Grounding Link Required |
|----------------|------------------------|
| CLI command | Link to `--help` output or playbook |
| Rust crate | Link to crates.io page |
| Code snippet | Link to GitHub source file |
| Configuration | Link to schema or config file |

---

## Validation Checklist

Before committing documentation:

- [ ] `title` under 60 characters
- [ ] `description` is 150-160 characters
- [ ] `slug` matches file path
- [ ] `after_reading_this` has 3-5 items with action verbs
- [ ] Each learning objective has matching section
- [ ] H1 matches frontmatter title
- [ ] Single H1 (page title only)
- [ ] H2 sections for major topics
- [ ] No skipped heading levels
- [ ] All internal links use absolute paths
- [ ] All CLI commands have grounding links
- [ ] All code references link to GitHub source
- [ ] `related_playbooks` links to at least 1 playbook
- [ ] Page listed in `docs_sidebar` in navigation.yaml

---

## Publishing Workflow

```json
{ "command": "infra jobs run publish_pipeline" }
{ "command": "core content show <slug> --source documentation" }
```

---

## Quick Reference

| Task | Action |
|------|--------|
| Create page | Add `.md` in `services/content/documentation/` |
| Create section | Add directory with `index.md` |
| Link page | Add to `docs_sidebar` in navigation.yaml |
| Internal link | `[Text](/documentation/section/page)` |
| Anchor link | `[Section](#heading-in-kebab-case)` |
| Code reference | Link to GitHub blob URL |
| Publish | `infra jobs run publish_pipeline` |