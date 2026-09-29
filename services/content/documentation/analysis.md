---
title: "Analysis: The Record of Every Conversation and Skill"
description: "What the Analysis section of the admin console shows — the deterministic record of every gateway conversation and every skill invocation, with one judge label on top — and how to read it."
author: "systemprompt.io"
slug: "analysis"
keywords: "analysis, conversations, skills, tokens, cost, models, tool calls, governance, judge, completion, versions, marketplace hash, install rate"
kind: "guide"
public: true
tags: ["enterprise", "admin", "analytics"]
published_at: "2026-09-16"
updated_at: "2026-09-18"
after_reading_this:
  - "Name what each Analysis page answers, and open the right one for a given question"
  - "Say which planes a conversation's row is built from, and how a hook session attaches to a gateway conversation"
  - "Read the one judge score for what it is — a label on the record, not the record"
  - "Explain install rate, activation and reach on the Skills page"
related_docs:
  - title: "Measure Which Skills Are Used"
    url: "/documentation/analysis-measure-skills"
  - title: "Versions: Marketplace Hashes, History and Compare"
    url: "/documentation/analysis-versions"
  - title: "Code ↔ Instance: Sources, Planes and Sync"
    url: "/documentation/services-sync"
---

# Analysis: the record of every conversation and skill

**TL;DR:** Analysis shows what the gateway and the harness hooks recorded — who, which client and model, turns, tool calls, errors and denials, tokens and cache share, cost, latency, skills, installs — for every conversation and every skill, and puts one AI label on top of each conversation: a title, a summary, the intent, an outcome and a single 0–100 *completion* score (did the assistant deliver what was originally asked). The numbers are the point; the label is a reading aid. It starts at `/admin/analysis/conversations`.

## What the section is for

Three questions, in the order you normally ask them:

1. What is flowing through the gateway — how many conversations, by whom, on which models and clients, at what cost, with what errors — and did each one complete what the person asked?
2. Which skills are actually used, by how many of the people entitled to them, in how many conversations, at what cost, and how do those conversations go?
3. What did a new version of a marketplace do, compared with the one before it, and which devices received it?

## The pages

Every Analysis page lives at `/admin/analysis/<noun>`, the same noun and identity the Platform section uses at `/admin/<noun>` — Platform says what is configured, Analysis says what happened.

| Page | Route | What it answers |
|---|---|---|
| **Conversations** | `/admin/analysis/conversations` | Every gateway conversation in the window: KPI tiles with sparklines, three charts on one time axis, a breakdown by intent, model, client, group, project, person, skill or outcome, and the rows themselves — every figure toned, a tokens-per-turn sparkline per row, and the judge's one score. |
| **Conversation** | `/admin/analysis/conversations/{context_id}` | One conversation on every plane: the judge's label and rationale, per-turn charts (tokens, cost, latency), the turn ledger with model, status, finish reason and tokens by kind, every tool call from the ledger (intent → execution → artifact), every governance decision, every safety finding, and the skills invoked with the marketplace version served at the time. |
| **Skills** | `/admin/analysis/skills`, `/admin/analysis/skills/{plugin:skill}` | Every skill invoked in the window, grouped by marketplace: invocations with a fourteen-day sparkline, people over entitled, installs, conversations, tokens, cost per invocation, tools, errors and denials, p95, models and clients seen, and the judge's mean completion. Above the table, marketplace adoption: entitled → installed → active, by host. A skill's own page splits its invocations by model, client, group, project, person, version or outcome, and lists the conversations behind them. |
| **Reports** | `/admin/analysis/reports`, `/admin/analysis/reports/{id}` | On-demand AI reports over the record: a headline, an `ok / watch / degraded` assessment, themes with evidence links and recommendations, written the moment they are asked for from a SQL digest — never a transcript. The latest global report's headline sits in a banner under the Conversations and Skills headers; **Report on this view** writes one over the page's current filters. |
| **Versions** | `/admin/analysis/versions`, `/admin/analysis/versions/{marketplace_id}` | Every marketplace by content hash: its history with mean completion per version, what changed between versions, how each performed, and what devices received. |

Every list page shares the same furniture: KPI tiles with an icon and a trend slot, charts padded to the whole window, a filter ribbon whose pills list only the values the filtered set contains (with counts and removable chips), a breakdown whose rows link into the list and download as CSV, row checkboxes that enable a bulk bar (**Export selected**, and on Conversations **Judge selected**), and a `?` beside the title that opens the page's glossary.

## Where the numbers come from

A conversation is a gateway context. The `conversation_rollup` job keeps one row per context — `conversation_facts`, with a row per skill it invoked in `conversation_skill_facts` — re-derived within two minutes of any change on any plane:

- **Gateway** (`ai_requests`): turns and side calls, provider and model per request, input, output, cache-read, cache-write and reasoning tokens, cost, latency percentiles, status and finish reason, client kind, attestation and wire protocol, group and project stamped at insert.
- **Tool ledger**: the model's tool intents joined to their executions and results, so a row can say "asked for 5, executed 4, 1 failed, 2 artifacts". A tool call and an artifact are different things even when they are counted alike: `artifact_kind` (schema `46_tool_artifacts.sql`) marks a call as an artifact only when a person can view or retrieve what it produced — a file the assistant edited, wrote or read (`file`), an MCP Apps UI resource (`ui`), a typed card such as a table, chart or report (`card`), or a retained structured body with a preview (`body`). A shell command, a search, a listing, a Skill or Task call, or an untyped result is a plain tool call. The `tool_activity` view applies the rule to every ledger row, and the Tools page, the Artifacts page, the conversation record and the Skills figures all read it.
- **Governance**: every allow, warn and deny the chain took on the conversation's tool calls; the gateway safety scanners' findings and blocks.
- **Hooks**: prompts, events and skill invocations from the harness, attached through the Claude Code session id the gateway records on each request.

The Skills page counts invocations from the hooks and reads everything else from the same per-conversation rows, so a figure on a skill is the same figure on the conversations that produced it. *People / entitled* is distinct invokers over the people the access-control rules reach; *installs* are distinct consumers holding a verified installation receipt for the skill, on any host; *install rate* is installed over entitled and *activation* is active over installed.

## The judge

One job, `conversation_judge`, runs every five minutes. It queues every conversation with at least one turn that has been quiet for thirty minutes or whose harness session has ended, reads the transcript (credentials redacted) and asks the configured model — in one structured call whose schema the provider enforces — for a title, a summary, the intent (development, business analysis, operations, admin & config, writing & comms, research & learning, other), an outcome (achieved, partial, abandoned, unclear) and the completion score. A conversation that keeps growing is judged again. The job's own gateway calls are audited under its identity and never counted as conversations; a daily cost cap stops a run rather than letting a backlog burn budget.

The judge is switched on by `hooks.judge: true` on the plugin that carries the governance hooks, and runs automatically only when the profile sets `judge.automatic: true`. With that switch off, a manual request still queues the conversation and the next tick reads it: **Judge now** on a conversation's page, the **Judge** button on any unjudged row of the Conversations or Skills lists, **Judge N unjudged** in the Conversations header (every unjudged conversation the current filters select, up to 200), or **Judge selected** in the bulk bar after ticking rows. A judged row shows its verdict — intent, outcome, summary and rationale — in a card on hover or keyboard focus of the title.

## Reports

A report is a one-off, on-demand reading of the record by the judge model. Requesting one (from `/admin/analysis/reports`, or **Report on this view** / **Generate** in the banner on Conversations and Skills) computes a digest in SQL — totals, the top skills, models and people, the worst and best judged conversations, cost outliers, denial clusters, the intent and client mix — stores it as the request's `inputs`, and starts the one structured call to the judge's model (the `conversation_judge` scheduler entry's provider and model) on a background task. Nothing from a transcript is included and nothing is scheduled. The report page shows a loading widget and refreshes itself when the verdict lands, usually within 10–40 seconds; a failed call is recorded with its reason and **Retry** starts a fresh one. Every version stays in the history with the request id, tokens and cost of its call, and a report can be regenerated over the same scope and window at any time.

## Versions

A marketplace version is the sha256 of everything it delivers — its own config, every plugin it includes and every skill those plugins ship. It is recorded at boot and on every inventory sync, and a conversation is tied to the version being served when it first invoked one of the marketplace's skills, never to when the judge ran. Versions has three views per marketplace — **History**, **Compare** and **Distribution** — described in [Versions: Marketplace Hashes, History and Compare](/documentation/analysis-versions).

## Access

Every Analysis page needs console access; this instance has no marketplace-participant tier. The manual review form and the withdrawal decisions on a marketplace's Distribution view require manage rights; without them the pages render read-only. See [Access Control](/documentation/access-control).

## Troubleshooting

### A conversation shows no skills, tools or governance

**Symptom:** The row has turns, tokens and cost, but the skills, tools and governance columns are empty.
**Cause:** The gateway recorded no Claude Code session id on its requests, so nothing from the hook plane can attach. Plain API clients and some third-party harnesses have no hooks.
**Solution:** Nothing to fix for those clients; the gateway columns are complete. For Claude Code, check the plugin carrying the governance hooks is installed.

### Nothing is judged

**Symptom:** Every row says "not judged yet".
**Cause:** Either no enabled plugin sets `hooks.judge: true`, the profile's `judge.automatic` is off, or no AI provider is configured for the process.
**Solution:** Check the boot log for "conversation judge configured"; queue one conversation with **Judge now** and watch `systemprompt infra jobs run conversation_judge`.

### A skill has invocations but no cost

**Symptom:** The skill row counts invocations and people, but conversations, tokens and cost are zero.
**Cause:** The sessions that invoked it produced no gateway conversation with the same session id — the inference went elsewhere, or the client did not send its session id.
**Solution:** Route the client through this gateway and make sure it sends its session id; the conversation then lands within a minute of its first turn.

## Related pages

- [Measure Which Skills Are Used](/documentation/analysis-measure-skills)
- [Versions: Marketplace Hashes, History and Compare](/documentation/analysis-versions)
- [Code ↔ Instance: Sources, Planes and Sync](/documentation/services-sync)
