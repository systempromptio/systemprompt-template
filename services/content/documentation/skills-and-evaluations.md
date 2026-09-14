---
title: "Skills, Evaluations & Controlled Experiments"
description: "Maintainer guide to the Astound skill lifecycle, evaluation architecture, recorded-traffic judge loop, and current controlled-experiment workflow."
author: "Astound Digital"
slug: "skills-and-evaluations"
keywords: "skills, evaluations, experiments, rubrics, benchmark, revisions, workers, evidence, budgets"
kind: "guide"
public: true
tags: ["documentation", "development", "skills", "evaluations"]
published_at: "2026-09-13"
updated_at: "2026-09-13"
after_reading_this:
  - "Trace a skill from its services-tree source through discovery, distribution, loading, and invocation analytics"
  - "Distinguish recorded-traffic evaluations from controlled skill experiments"
  - "Run and inspect the evaluation workflow that is available today without overstating experiment readiness"
  - "Identify the frozen inputs, identity boundaries, evidence, and accounting needed for a valid experiment"
related_playbooks:
  - title: "MCP, Tool Governance & Distribution"
    url: "/documentation/enterprise-tool-governance"
  - title: "Cost Management & FinOps"
    url: "/documentation/enterprise-cost-management"
  - title: "Audit Trail, Traceability & Observability"
    url: "/documentation/enterprise-audit-observability"
related_code:
  - title: "Evaluation delivery contract and progress ledger"
    url: "https://github.com/systempromptio/systemprompt-astound/blob/next/docs/evals.md"
  - title: "Super Admin benchmark configuration"
    url: "https://github.com/systempromptio/systemprompt-astound/tree/next/services/evaluations/super-admin"
---

# Skills, Evaluations & Controlled Experiments

**TL;DR:** A skill is a versionable package of instructions and supporting files,
not a model or a permission grant. Astound discovers skills from `services/skills/`,
projects eligible skills into signed client bundles, and records authenticated
invocations for analysis. There are two separate evaluation paths: the runnable
`systemprompt admin evals` loop judges recorded gateway traffic, while controlled
skill experiments compare frozen revisions in an isolated execution matrix. The
controlled-experiment data model and administrative surfaces exist, and a node-local
supervisor implementation is being integrated, but no paid end-to-end pilot has
established the complete workflow. Queuing an experiment is not by itself proof that
one ran successfully.

## The architecture at a glance

The full improvement loop is:

```text
services/skills source
  -> discovered skill catalog
  -> plugin/marketplace selection
  -> signed client bundle
  -> installed client skill
  -> governed invocation and audit event
  -> usage analysis or evaluation case
  -> immutable baseline and candidate revisions
  -> controlled experiment matrix
  -> evidence, deterministic checks, judgment and human review
  -> reviewed publication and measured adoption
```

The first five steps are the normal skill delivery path. Recorded-traffic evaluation
can operate on ordinary completed AI requests without creating an experiment. The
remaining steps form the managed experiment and publication path; parts of that path
remain under integration or intentionally idle without evaluator configuration.

## Start here

| What you need to do | Go to |
|---|---|
| Judge recent recorded gateway traffic | [Run evaluations now](#run-evaluations-now) |
| Capture a skill baseline and inspect experiment foundations | [Prepare and inspect controlled experiments](#prepare-and-inspect-controlled-experiments) |
| Understand how skills are authored, distributed, loaded, and measured | [How a skill works](#how-a-skill-works) |
| Check what is actually complete | [What is ready now](#what-is-ready-now) |

Astound supplies configuration, benchmark content, admin presentation, and acceptance
checks. The pinned `systemprompt-core` crates supply skill loading, catalog and bundle
assembly, evaluation services, experiment repositories, worker transport, evidence,
and accounting. PostgreSQL is the persistent store for requests, runs, revisions,
experiments, evidence, and audit records.

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
`category`, and `hosts`. This fork also uses display metadata such as
`display_category` for its public skills page. The directory name and canonical skill
ID use `snake_case`.

`file` selects the instruction document; the shipped skills set it to `SKILL.md`.
If it is omitted, the shared loader looks for `index.md`. The instruction file may
have YAML front matter for client interoperability. Platform loaders remove that
front matter before injecting or hashing the instruction body.

The services loader auto-discovers these directories when it loads the active
profile. Configuration is memoized for the process lifetime, so restart the service
after changing skill configuration. A missing directory produces an empty catalog;
an invalid or missing per-skill configuration is skipped or rejected according to
the consumer and validation stage. Run configuration validation before treating a
skill as distributable.

### Discovery and inspection

Use the CLI to inspect the same configured services tree:

```bash
systemprompt core skills list
systemprompt core skills list --enabled
systemprompt core skills show admin_daily_brief
```

`list` reports configured skill metadata. `show` returns the configuration and an
instruction preview. These commands are inspection tools: they do not install,
invoke, evaluate, or publish a skill.

The public `/skills/` page independently reads enabled skill descriptors to render
the human catalog. The admin catalog at `/admin/catalog/skills` is also read-only and
identifies the source as `services/skills/<id>/config.yaml`.

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

For server-side agents, `SkillService` resolves the active skills root, reads the
descriptor and instruction body, and returns the stripped instructions. `SkillInjector`
adds them to the agent prompt under `Writing Guidance`. A load emits a `skill_loaded`
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
| Experiment ID | Which frozen comparison matrix was requested |

Managed authoring, revision downloads, comparisons, publication records, and pinned
bundle downloads exist. The shared resolver is not yet connected to every gateway,
marketplace, CLI, agent, bridge, and evaluator consumer. Until that integration is
complete, do not claim that a published managed revision has replaced the configured
disk skill everywhere.

The maintainer UI for this work is under `/admin/analysis`:

- `/admin/analysis/skills` shows observed skill use.
- `/admin/analysis/versions` captures and inspects the four-skill Super Admin baseline.
- A revision page can create a text candidate without modifying the baseline.
- The comparison page shows file and provenance changes; it does not prove an outcome
  improvement.
- `/admin/analysis/evaluations` lists existing experiment records and their execution
  and accounting states.
- `/admin/analysis/impact` groups revision-attributed invocation cohorts while keeping
  missing attribution and incomplete accounting visible.
- `/admin/analysis/publications` separates human review, publication generation,
  distribution state, and installation receipts. This lifecycle surface is under
  integration and does not make an unevaluated candidate safe to publish.

Browser writes require an administrator and a matching same-origin request. Managed
and experiment repositories scope reads and mutations to the authenticated owner;
request bodies cannot select a different effective owner.

## Two evaluation systems

### 1. Recorded-traffic judge and replay loop

`systemprompt admin evals` is available now. It samples completed, non-synthetic
gateway requests, excluding job-originated judge and replay traffic so the evaluator
does not recursively grade itself. Request mode samples individual requests;
`--conversations` selects the latest completed request per conversation context.

For each sample, the loop:

1. Reconstructs the stored prompt transcript and response.
2. Sends them to the selected judge provider and model with a structured rubric.
3. Persists the 1–5 score, dimension scores, verdict, rationale, judge request ID,
   and measured judge cost.
4. If the verdict is partial or failed and contains a repair hint, replays the
   original canonical prompt with that hint and judges the new response.
5. Marks the original result repaired only when the replay passes.

The built-in rubric scores correctness, helpfulness, and completeness equally and
passes at 4/5. A named database rubric can replace it. LLM judgments are evidence for
review, not objective truth; inspect failures and rationales rather than reporting
only an average.

### 2. Controlled skill experiments

The controlled experiment model compares exact variants against versioned cases and
a rubric. An experiment freezes:

- 1–100 immutable case revisions and an optional dataset revision;
- one rubric revision with 1–20 weighted dimensions and explicit hard gates;
- 1–16 client/model/provider variants, including client version and SHA-256 digests
  for the skill bundle, configuration, and worker image;
- 1–10 repetitions, fixture or live mode, and a quality, cost, or latency objective;
- provider prices, tool configuration, permissions, dataset and rubric digests,
  plus fixture clock and timezone when frozen settings are supplied;
- an explicit shared budget account and per-experiment maximum cost.

The matrix expands to `cases × variants × repetitions` execution records. Preflight
validates the complete matrix and compares its maximum cost with the shared budget;
it must not silently remove cases to fit. A worker uses an environment-scoped bearer
credential to claim owner-scoped work. Assignments verify the active fenced lease and
the case, rubric, skill, and configuration snapshots. Heartbeats renew the lease;
stale fencing tokens cannot mutate the execution.

Worker transport provides assignment retrieval, short-lived execution access,
ordered events, approval requests, deterministic measurements, evidence submission,
cleanup reporting, completion, and restart reconciliation. Successful completion
requires evidence from the current fence, a zero exit code, and confirmed cleanup.
Execution completion remains separate from a passing score and from complete billing.

Evidence-backed scoring requires every rubric dimension, a 1–5 score per dimension,
references that resolve to submitted evidence, the exact hard-gate set, and a
rationale. Missing or invalid judgments remain unscored. A failed hard gate cannot be
hidden by a good weighted average.

Fixture-mode execution has a separate, non-public MCP registration with a closed
operation set for deterministic Atlassian reads, platform usage reads, and dedicated
platform test-record reads, writes, and restoration. Every result is labeled as
fixture evidence. Writes require the digest of the value that was read and report
whether the new value was read back successfully; fixture access is not authority to
write to Atlassian or ordinary platform records.

## The Super Admin benchmark

`services/evaluations/super-admin/` contains authored benchmark input for four skills:

- `admin_daily_brief`
- `admin_critical_projects`
- `admin_ai_usage`
- `systemprompt_cli`

The suite has 40 cases: seven development and three holdout cases per skill. Cases
cover pagination, freshness, arithmetic, permissions, missing and contradictory
evidence, prompt injection, authorized and unauthorized writes, audit discovery, and
timeout reconciliation. Fixtures are deterministic evidence sources, not answers,
and their records must never be presented as live facts.

The semantic rubric weights correctness 35%, evidence 35%, coverage 20%, and
usefulness 10%, with a default pass threshold of 4/5. Unauthorized writes, fabricated
evidence, unverified execution identity, unverified installed revisions, and evidence
integrity failures are hard failures.

The named pilot selects eight cases—two per skill—and is designed around one shared
$5 budget for baseline, candidate, judges, retries, and auxiliary AI calls. That
configuration is a target acceptance matrix, not evidence that the pilot has run.
No paid pilot or live experiment has been launched from the checked-in suite.

## Run evaluations now

### Prerequisites

Use a local or otherwise disposable instance with PostgreSQL initialized, a configured
AI provider, and completed gateway requests whose prompts and responses were retained.
The CLI constructs its AI service from the selected profile. These commands can make
paid provider calls, including repair replays, so start with a small sample and an
explicit budget.

Confirm the commands and the material you intend to assess:

```bash
systemprompt admin evals --help
systemprompt core skills list --enabled
systemprompt core skills show admin_daily_brief
```

Generate a few representative requests through the gateway before running an
evaluation. Sampling ignores incomplete requests, synthetic rows, job actors, and
records without a completed transcript.

### Run a small judge pass

The following example samples at most five requests from the preceding hour and stops
starting new samples after recorded judge spend reaches 100,000 microdollars ($0.10):

```bash
systemprompt admin evals run \
  --sample-size 5 \
  --window-hours 1 \
  --budget-microdollars 100000
```

The recorded-traffic limit is a loop stop, not a pre-dispatch reservation.
`budget_microdollars` tracks judge requests only: spend is checked before the next
sampled request, and repair-generation cost is not added to that counter. One judge
call can therefore cross the threshold, and repair replay adds paid inference outside
the reported judge total. The controlled experiment accounts use atomic reservations
instead. Use a suitably inexpensive judge model and a conservative limit.

Narrow a run when you need a meaningful cohort:

```bash
systemprompt admin evals run \
  --sample-size 10 \
  --window-hours 24 \
  --provider anthropic \
  --model <served-model-id> \
  --judge-provider anthropic \
  --judge-model <judge-model-id> \
  --conversations \
  --budget-microdollars 500000
```

Provider and model filters select the traffic under test. Judge provider and model
select what performs the scoring. Omit them to use the profile defaults. You can also
isolate one conversation with `--context-id <context-id>` or select a named rubric
with `--rubric <name>`.

### Inspect, preserve, and replay evidence

The run command returns its run ID and counts for scored, failed, replayed, repaired,
and judge cost. Inspect persisted evidence rather than relying on the summary:

```bash
systemprompt admin evals list --limit 20
systemprompt admin evals show <run-id>
```

Promote a useful completed request into the golden case set when a maintainer has
reviewed its input and can state the expected behavior:

```bash
systemprompt admin evals promote <ai-request-id> \
  --name "Daily brief with incomplete status data" \
  --expectation "Reports unknown coverage and does not default the project to green" \
  --tags daily-brief,coverage
```

Promotion snapshots the canonical prompt and links it to the source request. It does
not automatically add the case to the Super Admin JSON suite or run it in a controlled
experiment.

To run a new replay evaluation over a prior run's unrepaired failures:

```bash
systemprompt admin evals replay <run-id> \
  --budget-microdollars 100000
```

Replay calls the original request's provider and model with the stored repair hint,
then invokes the selected judge. Treat external side effects with care: this generic
recorded-traffic replay path is not the controlled experiment runner and does not
provide its isolated fixture transport, approval binding, or uncertain-write
reconciliation.

## Prepare and inspect controlled experiments

The local evaluator recipes are useful for infrastructure and authoring checks:

```bash
just evals-up
just evals-probe
```

`evals-up` starts an isolated evaluation PostgreSQL service. `evals-probe` checks that
the pinned native client can start under the container restrictions. Client probes
have no network, provider credentials, host mounts, or Docker socket. These checks do
not run inference, MCP operations, approvals, scoring, or publication.

On the main local application, sign in as an administrator and open:

1. `/admin/analysis/versions` and capture the current four-skill baseline.
2. Inspect each immutable revision and its exact files.
3. Create a candidate only when you have a rationale; compare it with its baseline.
4. Open `/admin/analysis/evaluations` to inspect experiment records in your owner
   scope, including matrix size, state, and settled/reserved/capped spend.

Stop the isolated services when finished; its database volume is retained:

```bash
just evals-down
```

There is no validated end-to-end run command to document yet. The current admin APIs
can create revisions and shared budgets, preflight and queue experiment records,
enroll workers, and inspect evidence. A node-local scheduler supervisor is present in
the current source and is designed to stay idle until its worker, pinned client and
relay images, and control network are configured. Its presence does not establish the
complete native-client workflow or a quality result from the authored benchmark. Do
not manually queue a paid experiment and infer from the queued record alone that
background execution began or completed correctly.

## What is ready now

| Capability | Current state | What the result means |
|---|---|---|
| Skill discovery, CLI inspection, catalog, and signed bundle projection | Runnable | The configured skill can be found and packaged for eligible clients |
| Skill invocation analysis | Runnable with authenticated client events | Observed use, not causal quality or cost attribution |
| Recorded-traffic judge, automatic repair replay, list/show, explicit replay, and case promotion | Runnable and paid | LLM assessment of retained gateway traffic |
| Four-skill baseline capture, immutable candidates, file comparison, and verified revision bundles | Runnable authoring slice | Exact content history, not an evaluated improvement or universal runtime activation |
| Experiment specs, shared accounts, preflight, queue records, leases, evidence, deterministic scoring primitives, inspection pages, and node-local supervisor source | Integration in progress | Component behavior and inspectable state, not accepted end-to-end experiment evidence |
| Resolver-backed activation across every skill consumer | Incomplete | A managed publication must not yet be described as universally active |
| Accepted supervised native-client workflow, complete provider-wide accounting, outcome workflow, clean installation, and rollback proof | Incomplete | No end-to-end acceptance claim is available |
| Eight-case $5 pilot and live experiment | Not run | The checked-in suite is authored input only |

Use `just evals-test` and `just evals-integration` for component validation and
`just evals-probe` for client availability. None is a substitute for the missing
end-to-end, fault, and live acceptance commands described in the
[evaluation architecture and readiness guide](https://github.com/systempromptio/systemprompt-astound/blob/next/docs/evals.md).

## Evidence required for a future experiment claim

A credible controlled result must retain the exact source, revision, bundle, client
image, model, provider-price, tool-configuration, permission, dataset, and rubric
identities. It must also retain experiment and execution IDs, lease-bound events,
provider requests and costs, deterministic measurements, judgments with resolvable
evidence, approvals and readbacks for writes, cleanup state, and unresolved
reservations.

Report every attempted execution, hard failure, unscored outcome, and incomplete
accounting item. Cost per successful task is total attempted spend divided by verified
successful tasks; if nothing succeeds, it is undefined rather than zero. An eight-case
pilot can prove pipeline behavior and expose examples for review. It cannot establish
statistical superiority for the full workload.
