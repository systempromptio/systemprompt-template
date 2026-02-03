---
title: "Blog Content Creation Playbook"
description: "Create blog content for tyingshoelaces.com. Announcements, articles, and guides - each with tailored workflows."
keywords:
  - blog
  - content
  - writing
  - announcement
  - article
  - guide
category: content
---

# Blog Content Creation Playbook

Create blog content for tyingshoelaces.com. Choose your content type and follow the appropriate workflow.

## Prerequisites

**Load the [Session Playbook](../cli/session.md) first.** Verify your session and profile before starting.

```json
{ "command": "core playbooks show cli_session" }
```

**IMPORTANT: NEVER start, stop, or restart services. The API is already running.**

```json
{ "command": "admin session show" }
```

```json
{ "command": "admin agents list --enabled" }
```

---

## Quick Reference: Which Content Type?

| Type | Agent | Skill | Word Count | Category | Use For |
|------|-------|-------|------------|----------|---------|
| Announcement | blog_announcement | announcement_writing | 500-1000 | announcement | Product launches, releases, updates |
| Article (Technical) | blog_technical | technical_content_writing | 4000-6000 | article | Contrarian deep-dives, architecture analysis |
| Article (Narrative) | blog_narrative | blog_writing | 3500-5000 | article | Personal stories, lessons learned |
| Guide | blog_narrative | guide_writing | 2500-4000 | guide | Step-by-step tutorials, walkthroughs |

---

## MCP Tools

| Tool | Purpose |
|------|---------|
| `systemprompt_cli` | Execute CLI commands |
| `systemprompt_help` | Get command documentation and playbooks |

---

## CRITICAL: Context Management

**New content = New context. ALWAYS.** See the [Contexts Playbook](../cli/contexts.md) for full context management commands.

**WRONG:** Reusing `blog-general` context for multiple posts
**RIGHT:** `blog-mcp-deep-dive`, `blog-v2-release`, `blog-setup-guide` (one context per post)

---

## Execution Model: One Tool Per Message

**CRITICAL: Each step is a SEPARATE message. NEVER combine steps.**

| Step | Action | Separate Message? |
|------|--------|-------------------|
| Plan | Socratic Dialogue (articles only) | YES |
| Research | Call research_blog | YES |
| Create | Call create_blog_post | YES |
| Publish | Run publish_pipeline | YES |

---

# Section 1: Announcements

**Use for:** Product launches, feature releases, updates, news

**Fast turnaround - no Socratic dialogue required.**

## Step 1.1: Create Context

```json
{ "command": "core contexts new --name \"blog-[topic-slug]\"" }
```

Example:
```json
{ "command": "core contexts new --name \"blog-agent-mesh-2-release\"" }
```

## Step 1.2: Create Announcement (Direct)

Announcements don't require research or Socratic dialogue. Provide the details directly:

```json
{ "command": "admin agents message blog_announcement -m \"Create an announcement for:\n\nTOPIC: Agent Mesh 2.0 Release\n\nKEY FEATURES:\n- 50% faster context switching\n- New MCP tool integration\n- Improved routing algorithms\n\nSLUG: agent-mesh-2-release\nKEYWORDS: ['agent mesh', 'release', 'MCP', 'AI agents']\n\nKeep it 500-1000 words, professional and factual.\" --blocking --timeout 120" }
```

The agent will:
- Create 500-1000 words of professional content
- Use British English
- Include clear call-to-action
- Save to database with category="announcement"

## Step 1.3: Publish

```json
{ "command": "infra jobs run publish_pipeline" }
```

```json
{ "command": "core content show <slug> --source blog" }
```

---

# Section 2: Articles

**Use for:** Technical deep-dives, personal narratives, contrarian takes, lessons learned

**Requires Socratic dialogue to refine thesis and angle.**

## Step 2.1: Create Context

```json
{ "command": "core contexts new --name \"blog-[topic-slug]\"" }
```

## Step 2.2: Choose Article Type

| Type | Agent | When to Use |
|------|-------|-------------|
| Technical | blog_technical | Architecture analysis, contrarian takes, "why X is wrong" |
| Narrative | blog_narrative | Personal stories, lessons learned, "how I built X" |

## Step 2.3: Socratic Dialogue (MANDATORY)

**The blog agent is a GATEKEEPER.** It will NOT research or create until you've articulated a clear thesis.

### For Technical Articles:

```json
{ "command": "admin agents message blog_technical -m \"I want to write about:\n\nTOPIC: MCP Architecture\nTHESIS: Complex multi-agent systems are overengineered; well-designed context curation beats agent proliferation.\nCONTRARIAN ANGLE: Everyone builds more agents. I'll argue for fewer, smarter ones.\nEVIDENCE: Built a 5-agent system that outperformed a 20-agent competitor.\nREADER OUTCOME: CTOs will reconsider their agent architecture strategy.\n\nLet's refine before researching.\" --blocking" }
```

### For Narrative Articles:

```json
{ "command": "admin agents message blog_narrative -m \"I want to write about:\n\nTOPIC: Why I Quit LangChain\nSTORY: Started with LangChain, hit production walls, rebuilt from scratch.\nLESSON: Framework abstractions cost more than they save at scale.\nEVIDENCE: 3 months of debugging, 2 weeks to rebuild without it.\nREADER OUTCOME: Developers will evaluate frameworks more critically.\n\nLet's refine before researching.\" --blocking" }
```

**Wait for the agent to confirm the goal before proceeding.**

## Step 2.4: Research (Separate Message)

```json
{ "command": "admin agents message blog_technical -m \"Proceed with research.\" --blocking --timeout 120" }
```

The agent returns an `artifact_id`. Save it for the next step.

## Step 2.5: Create Content (Separate Message)

```json
{ "command": "admin agents message blog_technical -m \"Create the blog post using research. Use these specifics:\n- slug: 'mcp-architecture-simplicity'\n- keywords: ['MCP', 'architecture', 'AI agents', 'context curation']\n- category: 'article'\n- Include code examples comparing approaches\n- British English, 4000+ words minimum\" --blocking --timeout 300" }
```

## Step 2.6: Publish

```json
{ "command": "infra jobs run publish_pipeline" }
```

```json
{ "command": "core content show <slug> --source blog" }
```

---

# Section 3: Guides

**Use for:** Step-by-step tutorials, walkthroughs, setup instructions, how-to content

**Educational and practical - no personal narrative required.**

## Step 3.1: Create Context

```json
{ "command": "core contexts new --name \"blog-[topic-slug]\"" }
```

Example:
```json
{ "command": "core contexts new --name \"blog-mcp-setup-guide\"" }
```

## Step 3.2: Define Scope

Guides skip Socratic dialogue but need clear scope:

```json
{ "command": "admin agents message blog_narrative -m \"Create a guide for:\n\nTOPIC: Setting Up MCP Servers from Scratch\nCATEGORY: guide\nAUDIENCE: Developers new to SystemPrompt\nPREREQUISITES: Node.js 18+, basic CLI knowledge\nOUTCOME: Reader has a working MCP server running locally\n\nProceed with research.\" --blocking --timeout 120" }
```

## Step 3.3: Create Guide (Separate Message)

```json
{ "command": "admin agents message blog_narrative -m \"Create the guide using research. Use these specifics:\n- slug: 'mcp-setup-guide'\n- keywords: ['MCP', 'setup', 'tutorial', 'getting started']\n- category: 'guide'\n- Include all prerequisites\n- Include troubleshooting section\n- Test all code examples\n- British English, 2500-4000 words\" --blocking --timeout 300" }
```

## Step 3.4: Publish

```json
{ "command": "infra jobs run publish_pipeline" }
```

```json
{ "command": "core content show <slug> --source blog" }
```

---

## Content Requirements by Type

### Announcements
| Requirement | Value |
|-------------|-------|
| Length | 500-1000 words |
| Structure | Lead, What's New, Why This Matters, Get Started |
| Tone | Professional, factual, direct |
| Research | Optional (internal info often sufficient) |

### Articles
| Requirement | Value |
|-------------|-------|
| Length | 3500-6000 words |
| Grounding links | Minimum 5 reference links |
| Structure | Prelude, Problem/Orthodoxy, Journey/Cracks, Lesson/Truth, Conclusion |
| Tone | Personal, contrarian, evidence-based |
| Research | Required |

### Guides
| Requirement | Value |
|-------------|-------|
| Length | 2500-4000 words |
| Code examples | Required for every major step |
| Structure | Prerequisites, Steps, Troubleshooting, Summary |
| Tone | Educational, clear, practical |
| Research | Required |

---

## Voice and Style (All Types)

- British English (realise, optimise, colour)
- No colons or em-dashes in titles
- Maximum 8 words in titles
- No fabricated experiences or metrics
- Inline citations with full URLs

---

## MCP Tools Used by Agents

| Tool | MCP Server | Purpose |
|------|------------|---------|
| `research_blog` | content-manager | Research topic using Google Search |
| `create_blog_post` | content-manager | Create blog post with category |
| `memory_search` | soul-mcp | Search past content |
| `memory_store` | soul-mcp | Store completed posts |

---

## CRITICAL: Agent Capabilities

**Agents can ONLY research and create. They CANNOT edit or revise existing content.**

| Action | Method |
|--------|--------|
| Research a topic | Agent (via `research_blog` tool) |
| Create NEW content | Agent (via `create_blog_post` tool) |
| Edit existing content | **Edit file on disk** (see below) |
| Revise/expand content | **Edit file on disk** (see below) |

If content needs revision after creation, you MUST edit the markdown file directly on disk. Do NOT ask the agent to revise - it will fail or create duplicate content.

---

## CRITICAL: Content Source of Truth

**Disk files are the source of truth.** The `publish_pipeline` job ingests content from disk to database.

```
Content Architecture
┌─────────────────────────────────────────────────────────────┐
│  services/content/blog/<slug>/index.md  ◄── SOURCE OF TRUTH │
│                    │                                         │
│                    ▼ publish_pipeline ingests                │
│              Database (markdown_content)                     │
│                    │                                         │
│                    ▼ publish_pipeline prerenders             │
│              web/dist/blog/<slug>/index.html                 │
└─────────────────────────────────────────────────────────────┘
```

**CLI `edit` commands update the database only.** These changes are TEMPORARY - the next `publish_pipeline` run will overwrite them with disk content.

---

## Updating Content (Edit Disk Files)

**All revisions and edits MUST modify the markdown file on disk.**

### File Location

```
services/content/blog/<slug>/index.md
```

Example: `services/content/blog/open-source-ai-era/index.md`

### Revision Workflow

1. **Locate the file**:
   ```bash
   ls services/content/blog/<slug>/
   ```

2. **Edit the markdown file directly** (using your editor or Claude):
   - Update frontmatter (title, description, keywords, etc.)
   - Update body content

3. **Re-run publish pipeline**:
   ```json
   { "command": "infra jobs run publish_pipeline" }
   ```

4. **Verify changes**:
   ```json
   { "command": "core content show <slug> --source blog" }
   ```

### Frontmatter Fields

```yaml
---
title: "Your Title Here"
description: "SEO description"
author: "systemprompt.io"
slug: "url-slug"
keywords: "comma, separated, keywords"
image: "/files/images/blog/placeholder.svg"
kind: "blog"
category: "article"  # article | announcement | guide
public: true
tags: ["tag1", "tag2"]
published_at: "2026-01-31"
---
```

### Quick Metadata Updates (Temporary)

For quick, temporary changes (e.g., testing), you can use CLI edit commands. **These will be overwritten on next publish.**

```bash
# Temporary DB-only edits
systemprompt core content edit <slug> --source blog --set title="New Title"
systemprompt core content edit <slug> --source blog --public
```

To make these permanent, also update the disk file.

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Agent not responding | Check with `{ "command": "infra services status" }` |
| Content not created | Verify with `{ "command": "core content search \"[slug]\"" }` |
| Agent says "created" but content doesn't exist | **Create a NEW context** and try again |
| Research artifact not found | **Create a NEW context** |
| Wrong category in database | Edit with `core content edit <slug> --source blog --set category="guide"` |
| Content not publishing | Run `{ "command": "infra jobs run publish_pipeline" }` |
| Content too short | Explicitly state word count minimum in instructions |

---

## Architecture Notes

### Content Flow

```
1. research_blog (MCP tool) - optional for announcements
   └── Returns artifact_id
        ↓
2. create_blog_post (MCP tool)
   ├── skill_id: blog_writing | technical_content_writing | announcement_writing | guide_writing
   ├── category: announcement | article | guide
   └── Saves to database
        ↓
3. publish_pipeline (Job)
   └── Prerenders to static HTML, updates sitemap
```

### Key Architecture Rules

| Rule | Reason |
|------|--------|
| MCP tools save to content tables | Content has no FK to task_id |
| Category enables filtering | Blog listing page filters by category |
| `publish_pipeline` must run | Content is in DB but not live until published |

---

---

# Section 4: Featured Images

**Every blog post needs a featured image.** Use AI-generated images for consistent quality.

**CRITICAL:** Images require TWO operations to display:
1. Link file to content (for file management)
2. Set image field on content (for display)

See [Images Playbook](./images.md) for full documentation.

## Step 4.1: Generate Image with MCP

```bash
systemprompt plugins mcp call content-manager generate_featured_image -a '{
  "skill_id": "blog_image_generation",
  "topic": "Your Topic",
  "title": "Your Blog Title",
  "summary": "Brief description for image generation"
}' --timeout 120
```

**Save from response:**
- `Image ID` (file UUID)
- `Public URL` (e.g., `/files/images/generated/2026/02/02/abc123.png`)

## Step 4.2: Find Content ID

```bash
systemprompt core content list --source blog
```

## Step 4.3: Link Image to Content

```bash
systemprompt core content files link <file_id> --content <content_id> --role featured
```

## Step 4.4: Set Image Field (REQUIRED FOR DISPLAY)

**For database-only content (AI-generated):**

```bash
systemprompt core content edit <content_id> --set image="<public_url>"
```

Example:
```bash
systemprompt core content edit 7ed8c2cc-e4c5-41df-9ec5-334e3bbe8c6c \
  --set image="/files/images/generated/2026/02/02/fece2027-b1d7.png"
```

**For file-based content (on disk):**

Edit `services/content/blog/<slug>/index.md` frontmatter:

```yaml
---
image: "/files/images/generated/2026/02/02/your-image.png"
---
```

Then re-run publish pipeline:
```bash
systemprompt infra jobs run publish_pipeline
```

---

## Image Requirements

| Attribute | Value |
|-----------|-------|
| Resolution | 2K (2048x1152) |
| Aspect Ratio | 16:9 |
| Format | PNG |
| Provider | Gemini or OpenAI |

---

## Related Playbooks

- [Session Playbook](../cli/session.md) - Authentication and session management
- [Contexts Playbook](../cli/contexts.md) - Context management
- [Jobs Playbook](../cli/jobs.md) - Job management
- [Images Playbook](./images.md) - Full image management guide
- [Database Access Patterns](../build/database-access.md) - DB access in MCP handlers
