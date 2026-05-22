# Scaled Scenario — Architecture & Code Walkthrough

A technical reference for the horizontally-scaled deployment: what every
container is, what every script proves, and how a request travels from the
operator through the load balancer, across N stateless API replicas, with the
scheduler isolated on its own single node.

> **Source files referenced.** `deploy/scenarios/scaled/docker-compose.scaled.yml`,
> `deploy/scenarios/scaled/nginx.conf`,
> `deploy/scenarios/scaled/scheduler-disabled.config.yaml`,
> `.systemprompt/profiles/scaled-api/profile.yaml`,
> `.systemprompt/profiles/scaled-scheduler/profile.yaml`, and the four scripts
> in this directory. The loadtest runner lives in
> `../systemprompt-core/crates/tests/loadtest/`.

---

## 1. What the scaled scenario proves

| Property | Mechanism | Asserted by |
|----------|-----------|-------------|
| **The API tier scales horizontally** | `app` is declared with no fixed host port; nginx round-robins on docker DNS `app:8080`; `--scale app=N` registers every replica. | `01-load.sh` (p95 ≤ 500 ms, error ≤ 2 % through the LB) |
| **Load balancing is fair** | Empirical bucketing of responses by `x-served-by` (each replica stamps its replica id on every response). | `03-replica-distribution.sh` part (a) — the `lb-fairness` loadtest scenario |
| **The event/SSE bus crosses replicas** | `PostgresEventBridge` relays the in-process event bus over `LISTEN/NOTIFY`; it is started unconditionally at boot. | `03-replica-distribution.sh` part (b) — subscribe on replica B, publish on replica A, assert delivery |
| **Cron jobs do NOT double-execute** | Scheduler is deployment-time isolated: every API replica gets a scheduler-disable bind-mount; exactly one dedicated `scheduler` node runs the default scheduler config. | `04-scheduler-isolation.sh` — `infra jobs history` and per-container log markers |
| **No drift under sustained load** | Sustained run with periodic latency windows + a memory sampler across replicas. | `02-soak.sh` — `soak_analysis` p95 drift ≤ 5 %, no growing memory |

The scenario explicitly does **not** prove DB read-replica routing — see
*Known limits* below.

---

## 2. Topology

```
                      host
                  ┌────────┐
                  │operator│
                  └───┬────┘
                      │  localhost:8088 (the only published port)
                      ▼
              ┌───────────────┐
              │ lb  (nginx)   │  proxy_buffering off, read_timeout 3600s (SSE)
              │ upstream      │
              │   app:8080    │  ← docker DNS, all replicas register here
              └──────┬────────┘
                     │ round-robin
                     │
        ┌────────────┼─────────────┬──────────────┐
        ▼            ▼             ▼              ▼
    ┌───────┐   ┌───────┐     ┌───────┐     ┌─────────────┐
    │app #1 │   │app #2 │ ... │app #N │     │ scheduler   │ (deploy.replicas: 1)
    │stateless  │       │     │       │     │ runs cron   │
    │sched OFF  │       │     │       │     │ scheduler   │
    │bind-mount │       │     │       │     │ config = on │
    └────┬──────┘   ────┘     ────┘         └────┬────────┘
         │                                       │
         │  DATABASE_URL → postgres-primary      │
         └─────────────────┬─────────────────────┘
                           ▼
                  ┌──────────────────┐
                  │postgres-primary  │   wal_level=replica, max_wal_senders=10
                  │(all writes)      │   replication_slots=10, hot_standby=on
                  └────────┬─────────┘
                           │ streaming replication
                           ▼
                  ┌──────────────────┐
                  │postgres-replica  │   hot standby
                  │(NOT yet routed)  │   wired for topology realism only
                  └──────────────────┘

           (all containers above are on the `scaled` bridge network)
```

**One published port.** Only the `lb` exposes `8088`. The app replicas declare
`expose: ["8080"]` (intra-network only). This is what lets `--scale app=N`
work without host-port collisions.

---

## 3. Containers and what they do

| Service | Image / build | Role | Scaling |
|---------|---------------|------|---------|
| `postgres-primary` | `postgres:18-alpine` with `wal_level=replica`, `max_wal_senders=10`, `max_replication_slots=10`, `hot_standby=on` | Accepts every write; streams WAL to the replica via the `replicator` role created by `primary-init.sh`. | 1 |
| `postgres-replica` | `postgres:18-alpine` | Hot standby initialised from the primary with `pg_basebackup -R`. Wired for failover headroom and topology realism; **the engine does not currently read from it.** | 1 |
| `app` | `Dockerfile.scaled-prebuilt`, scaled-api profile copied into each replica's own writable layer; `scheduler-disabled.config.yaml` bind-mounted onto `services/scheduler/config.yaml`. | Stateless API tier. Every replica handles full request lifecycle (governance, routing, audit). Never runs cron. | `--scale app=N` (default 3) |
| `scheduler` | `Dockerfile.scaled-prebuilt`, scaled-scheduler profile, **no** scheduler-disable override. | The one and only node that runs cron jobs. Otherwise functionally identical to an `app` replica. | `deploy.replicas: 1` (hard constraint) |
| `lb` | `nginx:alpine` with `nginx.conf` | TLS-terminating-equivalent edge. Round-robin upstream `app:8080`. Streaming-tuned (`proxy_buffering off`, `proxy_read_timeout 3600s`). | 1 |

**Profile copy, not bind-mount.** Both `app` and `scheduler` *copy* their
profile into the container's own writable layer in the entrypoint, then exec
the real entrypoint:

```yaml
entrypoint:
  - "sh"
  - "-c"
  - "mkdir -p /app/services/profiles/docker
      && cp /seed/profile.yaml /app/services/profiles/docker/profile.yaml
      && exec /app/entrypoint.sh"
```

Why: the engine normalises the profile during init (it rewrites in place),
which fails on a read-only mount, and a single writable host file shared by N
replicas would race. The seed stays pristine; each replica gets an
independent writable copy.

---

## 4. Request flow through the LB

```
operator ──HTTP──► localhost:8088
                       │
                   ┌───▼────┐
                   │ nginx  │ resolver 127.0.0.11 valid=10s
                   │        │ upstream app_replicas:
                   │        │   server app:8080 resolve max_fails=3
                   │        │
                   │ /lb-health → 200 "lb ok"   (LB-self health)
                   │ /api/v1/health → upstream  (real replica health)
                   │ /         → upstream      (round-robin)
                   └───┬────┘
                       │
                pick next replica (docker DNS rotates)
                       │
                       ▼
                ┌───────────────┐
                │ app  (#k)     │  X-Forwarded-For/Proto/Real-IP set
                │  served_by    │  stamps `x-served-by: <replica-id>`
                │  middleware   │  on every response
                └──────┬────────┘
                       │
                       ▼
               full request lifecycle:
               authenticate → authz → governance pipeline →
               (optional) gateway → audit-spine writes →
               response back through nginx → operator
```

`x-served-by` is the empirical witness used by `03-replica-distribution.sh`
part (a). The fairness assertion is purely measurement, not a model of
nginx's documented behaviour.

**SSE / streaming.** `proxy_buffering off`, `proxy_request_buffering off`,
`proxy_read_timeout 3600s`, and `Connection: $connection_upgrade` keep the
upstream connection open for streamed model output. Without these settings,
nginx defaults would buffer the response and cut idle streams at 60 s.

---

## 5. Cross-replica event fan-out

The engine has an in-process event bus. With multiple replicas, a publisher
on replica A and a subscriber on replica B live in different processes and
would not naturally see each other's events. The engine bridges them with
**`PostgresEventBridge`**, a `LISTEN/NOTIFY` relay started unconditionally
at server boot — every replica `LISTEN`s on the same channel and `NOTIFY`s
into it when local code publishes an event.

```
replica A                 postgres                  replica B
   │                          │                         │
   │ publish event ──────────►│ NOTIFY <channel>        │
   │                          │ ─────────────────────►  │ LISTEN (open)
   │                          │                         │ deliver to local
   │                          │                         │ in-process bus
   │                          │                         │ → SSE stream
   │                          │                         │ → subscriber receives
```

The scaled `03-replica-distribution.sh` part (b) is the live proof, driven
entirely through real, authenticated core surfaces (core 0.11):

1. Discover replica container IPs with `docker inspect`; run all curls from
   inside an `app` replica (it ships `curl`; `nginx:alpine` only has `wget`).
2. Create a context owned by the demo user on replica A
   (`POST /api/v1/core/contexts/`).
3. Open an SSE subscription on replica **B** for that user's A2A events
   (`GET /api/v1/stream/a2a`).
4. Route a real `A2AEvent::TaskStatusUpdate` on replica **A**
   (`POST /api/v1/core/contexts/{id}/events` → `forward_event` →
   `EventRouter::route_a2a`), tagged with a unique token.
5. Assert replica B's SSE stream received that token.

`route_a2a` writes an `event_outbox` row and emits a Postgres `NOTIFY`; the
`PostgresEventBridge` on replica B consumes it and re-injects into the local
user-scoped `A2A_BROADCASTER`. That delivery — across two processes — is the
empirical evidence that the bus scales across nodes. The A2A broadcaster is
user-scoped, so the subscriber and the router must authenticate as the same
user; the orchestrator mints a token signed by the scaled stack's own secret
(see `run.sh`) because the scaled-api profile's `jwt_secret` differs from the
local profile's.

---

## 6. Scheduler isolation

```
   ┌──────────────────────────────────────────────────────────┐
   │ ALL replicas built from the same image                   │
   │                                                          │
   │  app #1   app #2   app #N           scheduler            │
   │    │       │        │                  │                 │
   │    └───────┴────────┘                  │                 │
   │           │ each gets a bind-mount:    │ no override:    │
   │           │ scheduler-disabled.config  │ default config  │
   │           ▼   yaml → enabled: false    ▼                 │
   │     scheduler runtime sees           scheduler runtime   │
   │     `enabled: false` → no jobs       loads all jobs and  │
   │     fire here                        runs the cron loop  │
   └──────────────────────────────────────────────────────────┘
```

**Two independent layers now guard against double-execution.**

1. **Code-time (core 0.11): a real distributed lock.** `SchedulerConfig
   .distributed_lock` defaults to `true` and is now wired through
   `crates/app/scheduler/src/services/scheduling/dispatch.rs`: before running a
   job, a replica calls `try_acquire_job_lock` (a Postgres advisory lock) and
   skips the tick if a peer already holds it (`event =
   "scheduler.job.skipped_by_lock"`). So even if N replicas all had the
   scheduler enabled, each scheduled tick would execute on exactly one of them.
   This supersedes the earlier "dead config" state — the flag is live.

2. **Deployment-time (this stack): scheduler disabled on the API tier.** Every
   `app` replica bind-mounts
   `deploy/scenarios/scaled/scheduler-disabled.config.yaml` onto
   `/app/services/scheduler/config.yaml` (`enabled: false`), so the cron loop
   never starts there at all. The dedicated `scheduler` service is
   `deploy.replicas: 1` and keeps the default (enabled) config. This belt-and-
   suspenders layer keeps the API tier doing nothing but serve requests, and is
   what `04-scheduler-isolation.sh` observes.

The `Profile` schema still has no scheduler section, so the per-replica disable
is expressed via the bind-mounted YAML override rather than `profile.yaml` — see
*Known limits*.

`04-scheduler-isolation.sh` watches a window covering at least one scheduled
run and asserts:
- `infra jobs history` shows jobs executed during the window;
- `docker compose logs <container>` shows scheduler-execution markers only on
  the `scheduler` container, on **no** `app` replica.

---

## 7. Postgres replication

```
postgres-primary                      postgres-replica
  │                                       │
  │ first init:                            │
  │  /docker-entrypoint-initdb.d/         │
  │    10-replication.sh  creates the     │
  │    `replicator` role and opens        │
  │    pg_hba on the bridge network only  │
  │                                       │
  │                                       │ first boot on empty PGDATA:
  │                                       │  pg_basebackup -R
  │                                       │  → standby.signal written
  │                                       │  → primary_conninfo set
  │                                       │
  │  WAL stream  ──────────────────────►  │ replay; hot_standby=on
  │                                       │
  │                                       │ readable as a hot standby
```

Replication slots (`max_replication_slots=10`) and `wal_level=replica` make
the primary suitable as a streaming-replication source. The
`host`/`postgres-primary`-only `pg_hba` rule means the replicator role can
only authenticate from inside the docker network — never from the host.

The replica is wired for **topology realism and failover headroom**. The
engine has no DB read/write split today, so reads still hit the primary. This
is intentional and called out in the findings table as an open item.

The replica clones the primary with `pg_basebackup`. Its entrypoint runs as
**root** so it can `chown` the freshly-mounted (root-owned) named volume to the
`postgres` user before cloning, then `docker-entrypoint.sh` drops privileges
itself. (Running the container as `user: postgres` instead left the volume
root-owned and looped on `pg_wal: Permission denied`.)

---

## 7a. Authentication — the shared RSA signing key

Every authenticated path (the gateway, `/api/v1/core/contexts`,
`/api/v1/stream/*`, the governance hook) is RS256-validated. The stack mints
and validates JWTs with an **RSA signing key** at
`signing_key_path: /app/services/profiles/docker/signing_key.pem`.

- The committed seed lives at
  `.systemprompt/profiles/scaled-api/signing_key.pem` (generate with
  `systemprompt admin keys generate --output <path>`), copied into every `app`
  and `scheduler` replica by the entrypoint. **All replicas share one key**, so
  a token minted on any replica validates on every other — required for the LB
  to round-robin authenticated requests and for the cross-replica SSE proof.
- Without the key file the stack cannot mint or validate **any** token and
  every authenticated request 401s. (This was the original demo failure: the
  scaled profile shipped no key and used a relative `signing_key_path` that
  resolved to a nonexistent `/app/signing_key.pem`.)
- `SYSTEMPROMPT_PROFILE` is pinned in each service's compose `environment:` so
  the host `.env` (loaded via `env_file:`) cannot leak a host-absolute profile
  path into the container and break in-container CLI calls.
- `run.sh` mints an admin token inside an `app` replica via
  `admin session login --token-only` and exports it to the proof scripts.

---

## 8. The four scripts

### `01-load.sh` — LB-fronted throughput + latency

Builds and invokes the core loadtest binary against `http://localhost:8088`
(the LB). Asserts the scaled SLO: `p95 ≤ 500 ms` and `error_rate ≤ 2 %`.
Output: `results/load.json`. No AI inference scenarios; safe to run for free.

### `02-soak.sh` — sustained load, drift detection

Runs the loadtest harness's native `soak` profile (default ~1 hour) against
the LB, with two independent drift measurements:

- **Latency drift** is read from the runner's `time_series[]` (one window per
  `--sample-interval-secs`): mean p95 of the last vs first N windows. Pass if
  the relative drift ≤ 5 %.
- **Memory drift** is a background sampler that sums RSS across `app`
  replicas every `MEM_INTERVAL` seconds; a soak-analysis block is appended to
  the runner's JSON.

A soak surfaces what a single burst hides: connection-pool exhaustion, GC
pressure, leak-led p95 creep.

### `03-replica-distribution.sh` — LB spread + cross-replica fan-out

Part (a): runs the `lb-fairness` scenario through the LB; buckets responses by
`x-served-by`; asserts each replica's share is within ±`SPREAD_TOL` of 1/N.
Scaled `app` replicas publish no host ports, so for part (b) the script
discovers container IPs with `docker inspect` and reaches them from inside the
network via `docker exec <app-replica>` (which has `curl`).

Part (b): create a context, subscribe to the A2A SSE stream on replica B,
route a real A2A event on replica A, assert delivery — the live
`PostgresEventBridge` proof described in §5. All curls run from inside an `app`
replica (it has `curl`; `nginx:alpine` only ships `wget`).

### `04-scheduler-isolation.sh` — no duplicate cron execution

Proves which node runs the cron engine, independent of cron timing:

1. Scans each container's **full** log (via `docker logs <cid>` — *not*
   `docker compose logs`, which takes a service name and silently returns
   nothing for a container id) for cron-engine markers
   (`Scheduler started`, `tokio_cron_scheduler`, job-dispatch wording),
   excluding the disabled-path lines.
2. Asserts the engine started on the `scheduler` node (>0 markers) and on **no**
   `app` replica (0 markers each) — the deployment-time isolation.
3. `infra jobs history` is read as a soft confirmation that jobs exist; it is
   not the pass condition (per-job dispatch is debug-level and cadence-bound).

---

## 9. Known limits / tech debt

- **DB read-replica routing is not implemented.** `postgres-replica` exists
  for topology realism and failover headroom only — all reads currently hit
  the primary. Adding a read/write split (e.g. dual `DATABASE_URL` /
  `DATABASE_WRITE_URL` already supported by `Secrets`) is open work.
- **Scheduler distributed lock is now live (was previously dead config).**
  `SchedulerConfig.distributed_lock` defaults to `true` and gates a Postgres
  advisory-lock claim in `crates/app/scheduler/src/services/scheduling/dispatch.rs`,
  so cron ticks de-duplicate across replicas. The deployment-time
  scheduler-disable mount is therefore now a second, belt-and-suspenders layer
  rather than the sole guard. Remaining open item: with `deploy.replicas: 1` the
  dedicated `scheduler` node is still a single point of *availability* (if it is
  down, no cron runs) even though it is no longer a correctness risk — running
  the scheduler on >1 node with the lock enabled would remove that SPoF.
- **`Profile` schema has no scheduler section.** The disable cannot be
  expressed in `profile.yaml`; the bind-mounted YAML override is the only
  way to disable scheduler per replica. A `scheduler` field in `Profile`
  (with `JsonSchema` + `deny_unknown_fields`) would let this live in profile
  config — alongside the other deploy-time toggles.
