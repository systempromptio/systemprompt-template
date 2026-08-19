# AGENTS.md — Demo Harness Runbook for LLM Agents

> For humans, read `demo/README.md`. This file is the agent-targeted runbook.

## Purpose

The generic template's 43-script demo catalogue was removed when this fork
narrowed to the Salesforce + Claude Code use case. What remains is the working
harness: preflight (token mint), seeding, and the sweep runner.

## Hard preconditions

Before running any demo script, all of these must be true:

| # | Precondition | Check | Fix |
|---|---|---|---|
| 1 | Workspace built | `test -x target/debug/systemprompt || test -x target/release/systemprompt` | `just build` |
| 2 | Services running | `systemprompt infra services status` reports HTTP + Postgres up | `just start` |
| 3 | Token present | `test -f demo/.token` | `./demo/00-preflight.sh` |

Preflight and seed are idempotent — running them twice is safe.

All commands assume **CWD = workspace root**. Scripts resolve relative paths
from there.

## Usage

```bash
./demo/00-preflight.sh   # verify services, mint admin token → demo/.token
./demo/01-seed-data.sh   # seed analytics/log/trace data + fixtures/ uploads
./demo/sweep.sh          # run any demo scripts present, pass/fail summary
```

## Rules

1. **Never commit tokens.** `demo/.token` and `demo/.token.user` are gitignored;
   keep it that way.
2. **Governance is disabled on this instance** — every stage in
   `services/governance/config.yaml` is `enabled: false`. Scripts that assert a
   deny will not see one; the audit trail still records
   `decision=allow, policy=governance_disabled`.
3. **Read a script's header before running it** — mutating scripts announce
   what they change and clean up after themselves.
