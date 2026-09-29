---
title: "Skills: Lifecycle & Managed Revisions"
description: "Maintainer guide to the skill lifecycle: authoring, discovery, signed distribution, runtime loading, invocation measurement, and how disk skills relate to managed revisions."
author: "systemprompt.io"
slug: "skills-lifecycle"
keywords: "skills, revisions, bundles, publication, attribution, managed resources"
kind: "guide"
public: true
tags: ["documentation", "development", "skills"]
published_at: "2026-09-13"
updated_at: "2026-09-28"
after_reading_this:
  - "Trace a skill from its services-tree source through discovery, distribution, loading, and invocation analytics"
  - "Tell a disk skill, a managed revision, a bundle digest and a publication generation apart"
  - "Find the Analysis page that answers a given question about skill use or versions"
related_playbooks:
  - title: "MCP, Tool Governance & Distribution"
    url: "/documentation/enterprise-tool-governance"
  - title: "Cost Management & FinOps"
    url: "/documentation/enterprise-cost-management"
  - title: "Audit Trail, Traceability & Observability"
    url: "/documentation/enterprise-audit-observability"
related_docs:
  - title: "Analysis: The Record of Every Conversation and Skill"
    url: "/documentation/analysis"
  - title: "Versions: Marketplace Hashes, History and Compare"
    url: "/documentation/analysis-versions"
---

# Skills: lifecycle & managed revisions

**TL;DR:** A skill is a versionable package of instructions and supporting files,
not a model or a permission grant. This instance discovers skills from `services/skills/`,
projects eligible skills into signed client bundles, and records authenticated
invocations for analysis. Analysis observes, compares and measures those skills by
version; it does not run skills against datasets, and publishing a revision is a
human review, not the output of an experiment.

## The architecture at a glance

```text
services/skills source
  -> discovered skill catalog
  -> plugin/marketplace selection
  -> signed client bundle
  -> installed client skill
  -> governed invocation and audit event
  -> usage analysis by skill and revision
  -> reviewed publication and measured adoption
```

The first five steps are the normal skill delivery path. The last two are the
Analysis section: attributed use per skill and per published revision, and the
publication record that says which revision each installation received.

## Start here

| What you need to do | Go to |
|---|---|
| Understand how skills are authored, distributed, loaded, and measured | [How a skill works](#how-a-skill-works) |
| See which skills are used and at what cost | [Measure which skills are used](/documentation/analysis-measure-skills) |
| Compare two marketplace versions or read a version's provenance | [Versions](/documentation/analysis-versions) |
| Review a publication or decide a withdrawal | [Versions → Distribution](/documentation/analysis-versions#distribution) |

This repository supplies configuration, admin presentation, and acceptance checks. The
pinned `systemprompt-core` crates supply skill loading, catalog and bundle assembly,
managed revisions, publication, and accounting. PostgreSQL is the persistent store
for requests, revisions, publications, receipts, and audit records.

## How a skill works

### Authoring contract

Each skill lives under `services/skills/<skill_id>/` and normally contains:

```text
services/skills/example_skill/
  config.yaml
  SKILL.md
  references/       # optional
  scripts/          # optional
  templates/        # optional
  diagnostics/      # optional
  data/              # optional
  assets/            # optional
```

`config.yaml` supplies the platform identity and catalog metadata. The fields used by
the shared disk model are `id`, `name`, `description`, `enabled`, `file`, `tags`,
`category`, and `hosts`. The public skills page also reads optional display metadata
such as `display_category`. The directory name and canonical skill
ID use `snake_case`.

`file` selects the instruction document; the shipped skills (for example
`services/skills/who_am_i/`) set it to `SKILL.md`. If it is omitted, the shared
loader looks for `index.md`. The instruction file may
have YAML front matter for client interoperability. Platform loaders remove that
front matter before injecting or hashing the instruction body.

The services loader auto-discovers these directories when it loads the active
profile. Configuration is memoized for the process lifetime, so a skill changed on
disk under `services/` is picked up when the service restarts — which for this
repository's own skills means a deploy. A skill that arrives in a **kit** does not
need one: **Import sources** recomposes the tree and refreshes the inventory inside
the running process ([Code ↔ Instance](/documentation/services-sync)). A missing
directory produces an empty catalog; an invalid or missing per-skill configuration
is skipped or rejected according to the consumer and validation stage. Run
configuration validation before treating a skill as distributable.

The skills this instance ships are base skills in its own `services/skills/` tree,
included by the `systemprompt` plugin and served through the `enterprise-demo`
marketplace. Skills a pinned kit brings are bundle skills: the kit owns their
content, and this repository still owns their entitlement.

### Discovery and inspection

Use the CLI to inspect the same configured services tree:

```bash
systemprompt core skills list
systemprompt core skills list --enabled
systemprompt core skills show who_am_i
```

`list` reports configured skill metadata. `show` returns the configuration and an
instruction preview. These commands are inspection tools: they do not install,
invoke, or publish a skill.

The public skills page independently reads enabled skill descriptors from
`services/skills/` to render the human catalog. The admin catalog at `/admin/skills`
is also read-only and identifies the source as `services/skills/<id>/config.yaml`.

### Selection and signed distribution

A plugin or marketplace selects skills explicitly, inherits the instance catalog,
or receives them through an assigned agent. Bundle assembly filters disabled skills
and host-incompatible skills, then projects each selected skill into the client
layout:

```text
skills/<kebab-case-id>/SKILL.md
skills/<kebab-case-id>/references/...
skills/<kebab-case-id>/scripts/...
```

The bundle projection changes the ID from `snake_case` to the kebab-case form clients
expect. It generates compatible `SKILL.md` front matter, preserves the instruction
body, and includes eligible text supporting files. Known binary file types, hidden
files, and `__pycache__` are excluded from this skill projection. Scripts ending in
`.sh` or `.py` are marked executable.

The bridge manifest records each skill's SHA-256 and is distributed through the
signed plugin/catalog path. A signature establishes integrity and origin; it does
not grant model, MCP, or tool access. Those entitlements remain governed separately.

### Runtime loading and invocation measurement

For server-side agents, core's `SkillService` first asks the managed-resource resolver
for a published revision of the skill; a skill that is not managed falls back to the
disk descriptor and instruction body under the active skills root, and a withheld one
fails the load. The stripped instructions are added to the agent's context as a
system message under a `# Your Skills` heading. A load emits a `skill_loaded`
AG-UI event. When the request has a task ID and the execution-step repository is
wired, it also records a skill execution step. Missing tracking context does not
prevent the instructions from loading.

Claude Code can invoke an installed skill as a slash command or through its `Skill`
tool. The `skill_invocation_events` view normalizes both signals and removes a
slash/tool duplicate observed within five seconds. Tool-based rows require a nearby
governance decision; slash commands do not generate a governed tool call and are
therefore measured from the authenticated prompt event. This is why tool-call counts
alone under-report skill usage.

Invocation analytics are observational. One conversation may invoke several skills,
and its surrounding inference cost overlaps those skills. Do not sum related
conversation cost as though each skill independently incurred it.

## Disk skills and managed revisions are different layers

The services tree remains the authoring and current runtime source for ordinary
skills. The managed-resource subsystem adds stable resource identities and immutable
source snapshots, revisions, exact-byte assets, dependency pins, candidates, and
canonical bundles. Capturing a baseline does not edit the source skill and does not
activate a managed revision.

The identifiers have different meanings:

| Identifier | What it proves |
|---|---|
| Source snapshot | Which source state was imported |
| Resource revision | Exact immutable content and provenance |
| Bundle digest | Exact assembled dependency closure and bytes |
| Publication generation | Which reviewed selection was published at a point in history |
| Installation receipt | Which published bundle a client reports installing |

Managed authoring, revision downloads, comparisons, publication records, and pinned
bundle downloads exist, and server-side agent skill loading already consults the
managed resolver. The resolver is not yet connected to every gateway, marketplace,
CLI and bridge consumer. Until that integration is
complete, do not claim that a published managed revision has replaced the configured
disk skill everywhere.

The maintainer UI for this work is under `/admin/analysis`:

- `/admin/analysis/skills` shows every skill's record — invocations, people over
  entitled, installs, the conversations behind it with their tokens, cost, tools and
  errors, and the judge's mean completion; `/admin/analysis/skills/{plugin:skill}` is
  one skill's page.
- `/admin/analysis/revisions/{id}` shows a revision's verified stored bytes and
  provenance, file by file.
- A marketplace's **Compare** view shows which skills were added, removed or changed
  between two versions; it does not prove an outcome improvement.
- `/admin/analysis/versions` keys every figure on the marketplace version hash while keeping
  missing attribution and incomplete accounting visible.
- A marketplace's Distribution view separates human review, publication,
  distribution state, and installation receipts.

Browser writes require an administrator and a matching same-origin request. Managed
repositories scope reads and mutations to the authenticated owner;
request bodies cannot select a different effective owner.
