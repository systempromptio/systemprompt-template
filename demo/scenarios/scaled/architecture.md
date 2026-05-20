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

The scaled `03-replica-distribution.sh` part (b) is the live proof:

1. Discover replica container IPs with `docker inspect`.
2. From inside the docker network (`docker compose exec lb`), open an SSE
   subscription against replica B's `:8080`.
3. POST a publish to replica A's `:8080`.
4. Assert the SSE stream on B received the event.

That delivery — across processes — is the empirical evidence that the bus
scales across nodes.

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

**Why deployment-time, not code-time.** `SchedulerConfig` in core has a
`distributed_lock: bool` field — but it is **dead config**: declared in the
shared model, consumed nowhere in `crates/app/scheduler/`. Job de-duplication
relies only on an in-process `tokio::Mutex`, which does not cross replicas.
The `Profile` schema has no scheduler section, so the disable cannot be
expressed in `profile.yaml` either.

The mitigation is the bind-mount of
`deploy/scenarios/scaled/scheduler-disabled.config.yaml` onto
`/app/services/scheduler/config.yaml` for every `app` replica. The dedicated
`scheduler` service is `deploy.replicas: 1` (a hard constraint) and does
**not** receive the override — it keeps the default config and is therefore
the one and only node that runs cron jobs.

This is documented in the override file's own header as **tech debt**, not a
supported topology: the proper fix is a Postgres advisory lock per job
(`pg_try_advisory_lock`) inside the scheduler runtime, gated by the existing
`distributed_lock` flag. Until that lands, the scheduler is a single point of
failure — if its node is down, no cron jobs run.

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

Part (a): runs the `lb-fairness` scenario; buckets responses by
`x-served-by`; asserts each replica's share is within ±`SPREAD_TOL` of 1/N.
Per-replica addressing: scaled `app` replicas publish no host ports, so the
script discovers container IPs with `docker inspect` and curls them from
inside the network via `docker compose exec lb`.

Part (b): SSE subscription on replica B + publish on replica A + assert
delivery — the live `PostgresEventBridge` proof described in §5.

### `04-scheduler-isolation.sh` — no duplicate cron execution

Watches a window (default 6 min — long enough to cover at least one
scheduled run) and:

1. Reads `infra jobs history` to confirm jobs *did* run.
2. Greps each container's logs for scheduler-execution markers
   (`scheduler|scheduled job|job .*(executed|running|dispatch|trigger)|cron`).
3. Asserts markers appear on `scheduler` and on **no** `app` replica.

---

## 9. Known limits / tech debt

- **DB read-replica routing is not implemented.** `postgres-replica` exists
  for topology realism and failover headroom only — all reads currently hit
  the primary. Adding a read/write split (e.g. dual `DATABASE_URL` /
  `DATABASE_WRITE_URL` already supported by `Secrets`) is open work.
- **Scheduler distributed lock is dead config.** `SchedulerConfig
  .distributed_lock` is declared but unread. Proper fix: implement a
  `pg_try_advisory_lock`-gated dispatch in `crates/app/scheduler/`. Until
  then, the single-node `scheduler` service is a SPoF.
- **`Profile` schema has no scheduler section.** The disable cannot be
  expressed in `profile.yaml`; the bind-mounted YAML override is the only
  way to disable scheduler per replica. A `scheduler` field in `Profile`
  (with `JsonSchema` + `deny_unknown_fields`) would let this live in profile
  config — alongside the other deploy-time toggles.
