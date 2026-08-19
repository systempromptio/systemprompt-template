# Debug Infrastructure

Assess this clone's infrastructure end to end — which environments exist, how each is deployed, where each SQL
database lives — and verify that `just setup-local`, `just build`, `just start`, and the deploy path all run cleanly.
When something fails, this skill drives the fix ladder until it runs clean.

## When to Use

Use this skill when a deploy, build, or start command misbehaves, when a new operator is standing the instance up on
their own infrastructure, or when nobody is sure which environment points where. Two deployment targets are
first-class and equally supported:

- **SystemPrompt Cloud (Fly)** — `systemprompt cloud deploy` against a cloud tenant.
- **Self-hosted VM (e.g. an Oracle Cloud VM) with a remote Postgres** — your own profile, your own box, your own
  database service.

Never assume the target. Discover it in Phase 1 and debug the path this clone actually uses.

## Rules

1. **Assess before fixing.** Every finding must come from a command output run here, never from memory or this
   document alone.
2. **One fix at a time, re-verify after each** — the loop in the `systematic_debugging` skill applies.
3. **Always go through the justfile.** A bare `cargo build` bypasses the shared build coordinator and recreates the
   contention it exists to prevent.

## Phase 1 — Discover the target

```bash
just profile-check <profile>                                # one-shot validation of any profile (see Phase 1b)
ls .systemprompt/profiles/                                  # which environments exist (typically local + production)
cat .systemprompt/profiles/*/profile.yaml | grep -E "^(name|target|environment):|api_external_url|port:"
cat .systemprompt/profiles/*/secrets.json | grep -o '"database_url"[^,]*'   # DB host per env
cat .systemprompt/tenants.json                              # cloud tenants (Fly app ids, hostnames) — absent on pure self-host
ls .systemprompt/docker/                                    # local per-clone Postgres compose files
```

Build the environment table from what you found — do not invent rows:

| Environment | Profile | Deploy method | Database | External URL |
|---|---|---|---|---|
| local | `profiles/local/` | `just start` on this machine | Docker Postgres from `.systemprompt/docker/local.yaml`, port from `secrets.json` (not always 5432) | localhost |
| production (cloud) | `profiles/production/` | `just deploy` → `systemprompt cloud deploy` (Fly, driven server-side — no fly.toml exists) | Fly Postgres `systemprompt-db-prod.internal` — pooler :5433 read, :5432 write; unreachable from dev, use `systemprompt cloud db …` | per `api_external_url` |
| self-host (Oracle VM) | your own profile dir | Phase 5B checklist | remote Postgres named in that profile's `secrets.json` | your VM/domain |

Report honestly what is missing: if there is **no staging profile**, say so — staging is created by copying a profile
directory and adjusting it against `docs/profile.schema.json`, not assumed to exist.

## Phase 2 — `just setup-local` runs clean

```bash
ls .systemprompt/profiles/local/secrets.json 2>/dev/null || echo "NO LOCAL PROFILE — run just setup-local"
docker info >/dev/null 2>&1 || echo "Docker daemon not running"
just db-up && just db-logs | tail -20
```

Fix ladder:
- **Non-interactive shell** — `just setup-local` with no key argument needs a TTY; pass a provider key argument to run
  it non-interactively.
- **"Postgres did not become ready" after ~60s** — the compose file pins a host port; if you changed `pg_port`,
  re-run `just setup-local … pg_port=<port>` so it rewrites `.systemprompt/docker/local.yaml`, and verify with
  `pg_isready -h 127.0.0.1 -p <port>` using the port from `secrets.json`'s `database_url`.
- **Port already allocated** — another clone's container or a host Postgres owns the port; pick a different `pg_port`
  (each clone gets an isolated compose project, so containers never collide by name, only by port).

## Phase 3 — `just build` runs clean

```bash
just build-status        # is a run in flight? did the last run pass at this fingerprint?
just build
```

On failure, read the evidence before retrying: `.build/latest/build.json` names the log under `.build/logs/`.

Fix ladder:
- **"holds the lock (pid N); waiting"** — another agent's run; wait or attach. A dead pid is auto-cleared.
- **Suspicious instant green** — the coordinator answers from its fingerprint cache; `BUILD_FORCE=1 just build` to
  force a real run.
- **sqlx query mismatch / missing query cache** — stale `.sqlx`; run `just prepare` (needs the local DB up).
- **Unexpected core-crate errors** — check the `systemprompt-*` version pins in the root and `tests/` `Cargo.toml`
  match; a stale pin silently drops the `[patch.crates-io]` sibling and resolves the old crate.

## Phase 4 — `just start` runs clean

```bash
just server-status       # respect a server another agent already runs — do not restart it
just db-up
just start
curl -s localhost:8080/health
```

Healthy is `{"status":"healthy"}`; `"starting"` means wait and re-probe. On failure:

```bash
systemprompt infra logs view --level error --since 10m
systemprompt infra services status
systemprompt infra db status && systemprompt infra db validate
```

Fix ladder:
- **No binary** — `just build` first (`just start` refuses outright only when no binary exists).
- **Port 8080 in use** — `systemprompt infra services start --profile local --kill-port-process`, or change
  `server.port` in the profile.
- **Binary STALE / `replaced`** in `just server-status` — rebuild, then restart.
- **Migration checksum drift** — `just repair-migrations` (there is deliberately no destructive reset recipe).
- **Pages load without assets** — `just publish`, or `systemprompt infra jobs run publish_pipeline` (also runs at
  every server start — check `systemprompt infra jobs history publish_pipeline`).

## Phase 5A — Deploy: SystemPrompt Cloud (Fly)

```bash
systemprompt cloud auth whoami
just deploy-check                        # cloud doctor — preflight only, no build or push
systemprompt cloud deploy --profile production --dry-run
just deploy                              # runs build-all first, then cloud deploy
```

Verify after deploying:

```bash
just status                              # cloud status --profile production
systemprompt cloud db status --profile production
systemprompt cloud domain status --profile production
curl -s https://<api_external_url>/api/v1/health
```

The production database is only reachable through `systemprompt cloud db …` (`external_db_access: false`) — a local
`psql` failing against it is expected, not a fault.

## Phase 5B — Deploy: self-hosted VM (Oracle) with a remote Postgres

This is the path for operators running their own box. Work the checklist top to bottom; every later step assumes the
earlier ones are green.

**1. Profile.** Create `.systemprompt/profiles/<name>/{profile.yaml,secrets.json}` (copy the shape of
`profiles/production/`, validate fields against `docs/profile.schema.json`). Set `api_external_url`, CORS origins,
`trusted_proxies`, and `server.port` for your VM. Unknown YAML keys fail loudly at boot — a profile load error names
the offending key.

**2. Remote Postgres reachable from the VM.** Put the remote DB in that profile's `secrets.json` `database_url`,
then prove it from the VM itself:

```bash
pg_isready -h <db-host> -p <db-port>
psql "<database_url>" -c "select 1"
```

On failure, work outward in this order — this is the classic Oracle Cloud blocker:
1. Is the Postgres service itself up and listening on that port?
2. **OCI Security List / Network Security Group** — ingress rule for the Postgres port from the VM's subnet.
3. **VM firewall** — Oracle Linux images default-deny; check `firewalld`/`iptables` on both machines.
4. Credentials and `sslmode` in the URL (managed Postgres services often require `sslmode=require`).

**3. Ship the artifacts.** `.systemprompt/profiles/production/docker/Dockerfile` is the authoritative manifest of
what a deployment needs: the release `systemprompt` binary, the MCP server binaries, `web/dist/`, `storage/`,
`services/`, and the profile directory. Either build that image with your profile substituted, use the root
`docker-compose.yml` with `DATABASE_URL` pointed at the remote Postgres instead of the bundled `postgres` service, or
copy those paths to the VM directly after `just build-all`.

**4. Migrate, then serve.** Always migrate with the same binary you are about to run:

```bash
systemprompt infra db migrate --profile <name>
SYSTEMPROMPT_PROFILE=/path/to/profiles/<name>/profile.yaml systemprompt infra services serve --foreground
curl -s localhost:<port>/api/v1/health
```

Wrap the serve command in a systemd unit (or the compose file) for restarts; open ingress for the HTTP port.

**Common faults:**

| Symptom | Cause | Fix |
|---|---|---|
| Boot hangs then DB errors | remote Postgres unreachable | step 2 ladder (service → OCI security list → VM firewall → sslmode) |
| "relation does not exist" | migrations never ran against the remote DB | `infra db migrate --profile <name>` with the current binary |
| Pages render without CSS/JS | assets not shipped or pipeline not run | ship `web/dist` + `storage/`, then `infra jobs run publish_pipeline` |
| Profile fails to load | unknown YAML key | fix the named key against `docs/profile.schema.json` |
| Healthy locally, dead externally | HTTP port not opened in OCI security list / VM firewall | open ingress for `server.port` |
| Every tool call suddenly denied | `services/governance/config.yaml` deleted — a missing file **enables** all four stages | restore the file; never disable governance by deleting it |

## Phase 6 — Assessment report

End every run with a report in this shape:

1. **Environment table** (from Phase 1): environment → profile → deploy method → database location → status.
2. **Command health**: setup-local / build / start / deploy, each ✅ clean or ❌ with the failing output quoted.
3. **Findings**, ranked by severity. Each finding states expected vs actual, quotes the evidencing command output,
   and gives a copy-paste fix.
4. **Recommendations** — gaps against the reference wiring (e.g. no staging profile, secrets in the wrong place,
   deploying an unmigrated binary), each with the concrete next command.

## References

- `systematic_debugging` — the diagnose → fix → verify loop this skill's fix ladders plug into.
- `systemprompt_cli` — the full CLI surface for deeper inspection (`infra`, `cloud`, `admin`, `analytics`).
- `docs/RELEASING.md` — release/deploy procedure; this repo runs no hosted CI, `just preflight` is the gate.
- `docs/profile.schema.json` — the profile schema every environment must satisfy.
