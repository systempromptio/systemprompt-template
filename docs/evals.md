# Managed skills, evaluations, and version impact

The organizational portfolio/campaign extension and its explicit remaining gaps are
documented in [Organizational skill optimization](skill-optimization.md). The earlier
milestone descriptions below are not an all-client or production-acceptance claim.

Status: all six planned implementation milestones are represented in the `next`
source tree. Consolidated validation is pending. This document is not evidence of a
successful paid pilot, deployment, installation, rollback, or production-readiness
decision.

## Architecture

The implementation has three deliberately separate planes:

```text
authoring source -> immutable managed revision -> reviewed publication generation
                                                     |
                                                     v
controlled evaluation -> retained evidence       signed distribution
                                                     |
                                                     v
production observation <- verified attribution <- installation receipt
```

Filesystem skills are authoring inputs only. Once a resource is managed, runtime
consumers use the generation-pinned managed resolver. Missing, withdrawn, or corrupt
managed content fails closed and cannot fall back to a file with the same name.
Publication changes content selection only; it never grants model, MCP, tool, or user
permissions.

Production impact is observational. Related conversation cost can overlap across
skills and must not be summed. Request-deduplicated spend is the only cost dimension:
platform totals deduplicate request IDs, and missing attribution or accounting remains
visible. Tool-level vendor metering is out of scope until a metering source exists —
no MCP execution on this instance carries a cost, and a column with no producer would
render as a measured zero.

## Milestone 1 — managed resolution

Implementation complete; validation pending.

- Git sources retain credential references while synchronization resolves an exact
  commit, disables hooks, imports only regular `100644`/`100755` blobs, and records a
  source snapshot even when content is unchanged.
- Synchronization creates an incoming revision without changing publication. An
  upstream deletion creates a withdrawal proposal.
- Durable three-way reconciliation binds the upstream base, managed candidate, and
  incoming revision. Every conflicting path requires an explicit candidate, incoming,
  manual-digest, or delete resolution before the resolved revision is accepted.
- Revision bundles pin schema and assembler versions, preserve binary bytes and modes,
  and reject links, traversal, duplicate paths, excessive files, excessive dependency
  closure, and excessive expanded bytes.
- The forward workspace migration verifies legacy counts, bytes, and digests before
  dropping the legacy table. New evaluator projections store immutable manifests and
  exact assets and verify them on registration and read.
- `ManagedResourceResolver` returns not-managed, never-adopted, published, withdrawn,
  or integrity-failure state. Signed Bridge manifests/downloads, marketplace export,
  CLI inspection, agent skill injection, and evaluator assignment use it.

## Milestone 2 — supervised execution and shared accounting

Implementation complete; validation pending.

- Budgets are explicit owner-scoped accounts created idempotently and read by typed
  budget ID. Experiment launch requires `budget_id`; retries bind to the same account.
  The legacy per-experiment budget constructor and loop callers are removed.
- Every evaluator model request enters through an execution-bound gateway session.
  Admission reserves before dispatch, settlement is idempotent, and unknown usage
  retains the reservation. Cancelling one experiment does not freeze its shared
  account or cancel another experiment.
- The inventory-registered `evaluation_supervisor` polls every five seconds. Leases
  are fenced for 60 seconds, heartbeats occur every 20 seconds, each owner is limited
  to two live executions, and accumulated active execution time is capped at 30
  minutes.
- Assignments verify immutable dataset, rubric, configuration, and managed bundle
  digests before installation. Execution events, ordered artifacts, cancellation,
  approval waits, completion, cleanup status, and restart reconciliation are durable.
- Claude Code containers use a pinned image, a non-root UID, read-only root, one CPU,
  2 GiB memory, 128 processes, bounded `/tmp`, bounded writable workspace, and bounded
  output. One internal Docker network contains only the client and authenticated
  relay; launch stops if inspection differs.
- Execution credentials are scoped to owner, worker, execution, native session,
  permission context, expiry, live lease, and fencing token at gateway and MCP
  boundaries. The fixture MCP adapter has a closed operation enum and labels all
  fixture evidence.
- Restart reconciliation removes only owned stale Docker objects, marks expired work
  uncertain, retains unsettled reservations, and never replays an uncertain write.

## Milestone 3 — executable experiments and suggestions

Implementation complete; validation pending.

- `services/evaluations/super-admin/` contains 40 immutable authored cases and their
  fixtures: four skills with seven development and three holdout cases each. Suite
  seeding rejects any other partition shape and returns the exact case, dataset, and
  rubric revisions and digests.
- The rubric is correctness 35%, evidence 35%, coverage 20%, usefulness 10%, with a
  4/5 threshold and explicit hard gates.
- Pure preflight requires a frozen dataset and rubric, paired baseline/candidate
  variants, exact provider/model/client/image/configuration/permission/fixture/price
  settings, and a conservatively derived maximum cost. Only the candidate bundle may
  differ between paired variants.
- Evaluation pages expose explicit preflight, launch, progress, approval, cancellation,
  failure, comparison, and evidence states. JSON, Markdown, exact artifact, retained
  bundle, and revision-diff endpoints are downloadable.
- Deterministic evaluation covers arithmetic, permissions, evidence references,
  installed-bundle integrity, and write readbacks. Semantic judgments must include
  every weighted dimension, every hard gate, and valid retained references; invalid
  judgments remain unscored.
- Measurements persist hard failures, quality, latency, tokens, tool calls, accounting
  coverage, failed-attempt-inclusive spend, verified success, and cost per verified
  success.
- Suggestions are generated only from development failures, use separately metered
  suggestion traffic, and retain proposed changes, hypothesis, supporting failures,
  reservation, request, and originating evidence. Holdout consumption blocks another
  independent-improvement claim until fresh holdout revisions exist.

## Milestone 4 — publication, installation, and rollback

Implementation complete; validation pending.

- Reviews bind administrator identity, candidate revision, exact bundle digest,
  comparison evidence, limitations, action, operation key, and expected generation.
  Initial adoption and evaluated improvement are distinct actions.
- Publication history explicitly exposes approved, distributed, and
  installation-verified states. The outbox is claimed and completed idempotently.
- Distribution references the retained publication generation and bundle digest.
  Installation receipts are accepted only after every revision/path digest, byte
  length, and executable flag matches the retained closure.
- Receipts and publication decisions are immutable. Rollback is a new reviewed
  generation referencing previously retained content, followed by its own distribution
  and installation receipt.
- `/admin/analysis/publications` provides review, publish, distribution, receipt,
  withdrawal, and rollback controls without changing access policy.

## Milestone 5 — continuous version impact

Implementation complete; validation pending.

- Invocation attribution retains authenticated installation ID, publication
  generation, resource revision, traffic class, and status. A revision is `verified`
  only when the claimed generation/revision and authenticated native session match an
  immutable installation receipt. Unsupported or historical evidence stays revision
  unknown.
- `/admin/analysis/impact` filters by revision, skill, traffic class, and time. It
  shows invocation/request samples, failures, latency, tokens, assessed outcomes,
  quality, attribution coverage, accounting coverage, and related conversation cost.
- Production, fixture, live-evaluation, suggestion, and judge traffic are separate.
  The invocation-to-conversation-to-request/tool drilldown retains denominators and
  attribution status.
- Reviewed production-failure capture verifies owner and production traffic, accepts
  bounded sanitized evidence, and creates a development-only immutable case revision.
- All impact queries read durable audit and receipt tables and do not depend on an
  evaluator worker being online. The page labels uncertainty and the non-causal nature
  of before/after associations.

## Milestone 6 — cleanup and acceptance tooling

Implementation complete; validation pending.

- Disabled direct evaluation/replay services, the per-experiment budget constructor,
  and frozen-workspace read/write paths have been removed after caller migration.
- Fault coverage includes ownership and replay conflicts, corruption, stale leases,
  expired approvals, publication races, concurrent budget contention, provider
  disconnects, uncertain restart writes, missing evidence output, and failed or
  unacknowledged cleanup.
- `just evals-clean-lifecycle ...` runs baseline publication/install, candidate
  publication/install, and rollback-as-new-generation/install in the constrained clean
  client, verifying bytes, lengths, modes, receipts, and digests.
- `just evals-paid-pilot ...` selects exactly two authored cases per baseline skill,
  runs baseline and candidate once each with bounded judging, includes retry,
  suggestion, tool, and auxiliary maxima, and uses one $5 account. If the conservative
  maximum exceeds $5, it writes `budget-blocked` and does not reduce the matrix.
- `just evals-atlassian-acceptance ...` performs an authenticated Atlassian operation
  only when the tool advertises `readOnlyHint`, then verifies the pilot retained the
  dedicated platform test-record read/write/readback/restore evidence.
- `just evals-screens ...` captures desktop and mobile Evaluations, comparison,
  publication, installation, rollback, and version-impact evidence with checksums.

## Operator sequence

Do not publish from a score or suggestion automatically.

1. Capture or synchronize immutable baseline and candidate revisions.
2. Seed the authored suite and retain its returned case/dataset/rubric mapping.
3. Create one shared budget account.
4. Submit the complete frozen specification to preflight; review the matrix digest,
   maximum, permissions, and availability.
5. Explicitly launch against the same `budget_id` and idempotency key.
6. Review all executions, evidence, hard failures, unscored judgments, reservations,
   and accounting coverage.
7. For an improvement, establish eligible development and fresh holdout evidence,
   commit the evaluated content through the normal source workflow, and retain
   server-side Git verification plus the campaign's evaluation attestation. Changed
   content requires reevaluation; a score alone cannot authorize publication.
8. Record a human publication review with the expected generation.
9. Claim and deliver the exact outbox event, install on a clean client, and submit the
   exact receipt.
10. If needed, review rollback to retained content as another generation, distribute,
   install, and receipt it.
11. Monitor later verified cohorts separately from unknown attribution.

## Validation still required

The implementation status above must not be changed to accepted until the consolidated
campaign passes: core CI and exact Systemprompt pin; static, Clippy, unit, integration,
contract, coverage, and full preflight gates; migration and fault exercises; permitted
test-environment deployment only; the complete affordable paid pilot; authenticated
Atlassian/test-record acceptance; clean candidate and rollback installation receipts;
and reviewed desktop/mobile evidence.

No deployment to `sp-dev.systemprompt.digital` is authorized by this work.

## Source map

- Managed domain: `../systemprompt-core/crates/domain/marketplace/src/managed/`
- Evaluation domain: `../systemprompt-core/crates/domain/evaluation/src/`
- Supervisor: `../systemprompt-core/crates/app/scheduler/src/services/evaluator/`
- Admin APIs/UI: `extensions/web/admin/src/handlers/`
- Benchmark: `services/evaluations/super-admin/`
- Version impact migration: `extensions/web/schema/migrations/056_skill_version_impact.sql`
- Acceptance runners: `scripts/evaluator/`
