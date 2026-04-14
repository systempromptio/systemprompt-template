# AGENTS.md — Demo Catalogue Runbook for LLM Agents

> For humans, read `demo/README.md`. This file is the agent-targeted runbook.

## Purpose

43 shell scripts across 10 categories that exercise every major surface of the platform (governance, agents, MCP, skills, infra, analytics, users, web, cloud, performance). Use this catalogue to:

- **Verify a change** — smoke-test after modifying a CLI domain, extension, or governance rule.
- **Reproduce a bug** — most incidents map to one or two specific scripts.
- **Produce evidence for a PR** — paste pass/fail output into the review.
- **Onboard a new agent session** — running the full loop teaches you what the platform can do faster than reading docs.

## Hard rule — filename order is the author's story order

Every script is prefixed `NN-` (01, 02, 03…). **That number is the author's deliberate tutorial ordering**, and each script often ends with `echo "Now run: ./demo/NN-next.sh"`. Run them **in filename order 01..NN within a category**. Do not reorder, drop, or "curate" — earlier scripts seed state that later scripts read, and the narrative only makes sense sequentially. This rule caught a previous agent that tried to be clever and it broke the demo.

## Hard preconditions

Before running any demo script, all four must be true:

| # | Precondition | Check | Fix |
|---|---|---|---|
| 1 | Workspace built | `test -x target/debug/systemprompt \|\| test -x target/release/systemprompt` | `just build` |
| 2 | Services running | `systemprompt infra services status` reports HTTP + Postgres up | `just start` |
| 3 | Token present | `test -f demo/.token` | `./demo/00-preflight.sh` |
| 4 | Seed data present (analytics/logs/traces only) | `systemprompt infra db query "SELECT COUNT(*) FROM governance_decisions"` > 0 | `./demo/01-seed-data.sh` |

Preflight and seed are idempotent — running them twice is safe.

All commands below assume **CWD = workspace root** (`/var/www/html/systemprompt-template` on the reference machine). Some scripts resolve relative paths from there.

## Run the whole catalogue (43 scripts, ~60s wall-clock)

Paste this verbatim:

```bash
mkdir -p /tmp/demo-run && : > /tmp/demo-run/results.tsv
./demo/00-preflight.sh > /tmp/demo-run/preflight.log 2>&1
./demo/01-seed-data.sh > /tmp/demo-run/seed.log 2>&1
for cat in governance agents mcp skills infrastructure analytics users web cloud performance; do
  for script in $(ls demo/$cat/[0-9]*.sh 2>/dev/null | sort); do
    name="$cat/$(basename "$script")"
    log="/tmp/demo-run/${cat}_$(basename "$script" .sh).log"
    if timeout 200 bash "$script" > "$log" 2>&1; then
      printf "PASS\t%s\n" "$name"
    else
      printf "FAIL(%d)\t%s\n" "$?" "$name"
    fi
  done
done | tee /tmp/demo-run/results.tsv
echo "---"
echo "Pass: $(grep -c ^PASS /tmp/demo-run/results.tsv)  Fail: $(grep -c ^FAIL /tmp/demo-run/results.tsv)"
```

**Per-script timeout must be ≥ 180s.** `agents/03-agent-messaging.sh` issues a blocking AI call with an internal `--timeout 60`, plus context-creation overhead. Below 180s wall-clock the wrapper kills it before the script's own handler returns.

**Expected:** 42/43 PASS on a healthy local instance. See "Known flaky" below for the one that may intermittently fail.

## Run one category or one script

```bash
# A single category in filename order:
for s in $(ls demo/governance/[0-9]*.sh | sort); do bash "$s" || break; done

# A single script:
bash demo/governance/01-happy-path.sh
```

Exit `0` = pass. Non-zero = failure. Re-run without the wrapper to see stdout live.

## Category reference

| Category | Count | Proves | Cost | Notes |
|---|---|---|---|---|
| `governance` | 8 | Hook allow/deny, scope enforcement, secret detection, audit trail, rate limits, hooks API | Free | Exercises the `/hooks/govern` endpoint directly + the audit tables |
| `agents` | 5 | Discovery, config validation, live messaging, tracing, A2A registry | ~$0.02 | `03-agent-messaging.sh` makes a real AI call — requires a working provider and ≥180s wrapper timeout |
| `mcp` | 3 | Server inventory, governance-gated access tracking, tool execution analytics | Free | |
| `skills` | 5 | Skill lifecycle → content → files → plugins → contexts | Free | Sequential: later scripts read state written by earlier ones |
| `infrastructure` | 5 | Services, DB, jobs, logs, config | Free | Read-only |
| `analytics` | 8 | Overview → agents → cost → requests → sessions → content → conversations → tools | Free | Requires seed data (step 4 above) |
| `users` | 4 | CRUD, roles, sessions, IP bans | Free | `04-ip-ban.sh` adds and removes a test IP — fully reversible |
| `web` | 2 | Content model inventory, sitemap generation + validate | Free | Read-only |
| `cloud` | 1 | Read-only: `whoami`, `status`, `profile list` | Free | Does not mutate remote state |
| `performance` | 2 | Request tracing end-to-end, 2000-request load test | Free | Load test takes ~60–90s wall-clock |

Script counts must match: `ls demo/<cat>/[0-9]*.sh | wc -l`. If they don't, someone added or removed a script — re-read `demo/<cat>/README.md` to understand the new sequence.

## Exit-code semantics

| Code | Meaning | Agent action |
|---|---|---|
| `0` | Script ran its full arc | PASS — record and move on |
| `1` | A command inside the script failed under `set -e` | Inspect `/tmp/demo-run/<cat>_<name>.log`, find the last command before the failure |
| `124` | `timeout` wrapper killed the script (wall-clock exceeded) | Re-run with a larger timeout. The script is not broken |
| `127` | `CLI binary not found` or similar | Run `just build` |

## Known flaky

- **`agents/03-agent-messaging.sh`** — issues a blocking AI inference call with internal `--timeout 60`. If the configured AI provider is slow, rate-limiting, or temporarily unavailable, the call errors within 60s and `set -e` exits 1. This is a real external dependency, not a script bug. **On failure: re-run the script once** before reporting a regression. If it fails twice, check the provider: `systemprompt admin config show | grep -i provider` and `systemprompt infra logs view --level error --since 5m`.

That is the only known flaky script in the catalogue.

## Common failure modes

| Symptom in log | Cause | Fix |
|---|---|---|
| `CLI binary not found` | Workspace not built | `just build` |
| `No token file. Run ./demo/00-preflight.sh first.` | Preflight not run | `./demo/00-preflight.sh` |
| `503 Service Unavailable` / connection refused on `localhost:8080` | Services not running | `just start` |
| `governance_decisions: 0 rows` in analytics output | Seed data missing | `./demo/01-seed-data.sh` |
| `context deadline exceeded` inside `agents/03-agent-messaging.sh` | AI provider slow/down | Retry once; then check provider health |
| Exit 124 on any script | Wrapper timeout too short | Raise wrapper timeout to ≥ 200s |

## Reference run (known-good)

Recorded 2026-04-14 on local dev against a clean `just build && just start && ./demo/00-preflight.sh && ./demo/01-seed-data.sh`:

- **42/43 PASS**, 59s wall-clock.
- Sole failure: `agents/03-agent-messaging.sh` (exit 1, AI provider did not respond within the internal 60s blocking timeout). Retried successfully in the previous session — known flaky per above.
- Load test (`performance/02-load-test.sh`) issued 2000 requests, 900 deny audit rows, zero dropped.

Agents running the catalogue should diff their own result against this row-for-row: 43 PASS lines, same category order, within ±30s wall-clock on comparable hardware.

## Hard rules for agents

1. **Filename order is sacred.** 01..NN within a category is the author's narrative. Never reorder, never drop a script "because it looked like a duplicate". If you think two scripts are redundant, read both headers — the answer is almost always that they test different code paths.
2. **Do not commit `demo/.token`.** It is gitignored.
3. **Never silently skip a failure.** If a script fails, capture its log and surface it. The user prefers a red result to a lie.
4. **Destructive scripts are self-announcing.** Every mutating script echoes what it will do in its header and cleans up after itself (e.g. `users/04-ip-ban.sh`). Read the header before running a script outside the known catalogue.
5. **Do not "fix" a demo script without reading it first.** Most "failures" are environmental (token, services, seed, AI provider). Scripts are correct unless proven otherwise by two independent clean-environment runs.
