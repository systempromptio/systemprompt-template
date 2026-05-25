# Air-Gap Scenario — Architecture & Code Walkthrough

A technical reference for the air-gap deployment scenario: what every container
is, what every script proves, and how a `/v1/messages` call travels from the
operator's host to the mock inference endpoint and back — fully sealed.

> **Source files referenced.** `deploy/scenarios/airgap/docker-compose.airgap.yml`,
> `.systemprompt/profiles/airgap/profile.yaml`,
> `services/gateway/policies.yaml`, `services/ai/config.yaml`,
> `extensions/web/jobs/src/publish.rs`, and the four scripts in this directory.
> The mock inference server lives in the sibling
> `../systemprompt-core/crates/tests/mock-inference/`.

---

## 1. What "air-gapped" means here

The closure guarantee has two layers, measured independently:

| Layer | Mechanism | Asserted by |
|-------|-----------|-------------|
| **Network** | The `app`, `postgres`, `mock-inference`, `monitor` containers attach to a Docker network with `internal: true` — no gateway, no NAT, no host-routable path. | `01-egress-assert.sh` part 1 — `ss -tunp` / `conntrack -L` from the `monitor` sidecar |
| **Config** | The gateway routes (profile) point provider traffic at `http://mock-inference:8080`; the AI service providers (`services/ai/config.yaml`) have `endpoint: ${ANTHROPIC_ENDPOINT}` which the air-gap `secrets.json` resolves to the same internal mock. The gateway policy (`ai_gateway_policies`) allow-lists a finite model set; un-listed models are denied with **403** before any upstream call. | `01-egress-assert.sh` part 2 (denial precedes upstream) and `03-governance.sh` (allowed → 200 via mock, denied → 403) |

The point is that closure no longer rests only on the network boundary — both
config and topology are belt-and-braces.

---

## 2. Topology

```
                                              host
                                           ┌────────┐
                              localhost:8090│ operator│
                                           └────┬───┘
                                                │  TCP
                  ┌─────────────────────────────┼──────────────────────────┐
                  │ airgap-ingress (bridge)     │                          │
                  │   ┌───────────────────┐     │                          │
                  │   │ ingress (socat)   │ ◄───┘                          │
                  │   │ 8090 → app:8080   │                                │
                  │   └─────────┬─────────┘                                │
                  └─────────────┼────────────────────────────────────────┬─┘
                                │ joins both nets — the ONE demarcation │
   ┌────────────────────────────┼───────────────────────────────────────┼──┐
   │ airgap-internal (internal: true — NO gateway, NO NAT)               │ │
   │                            │                                        │ │
   │            ┌───────────────▼────────────────┐                       │ │
   │            │ app  (systemprompt-template)   │                       │ │
   │            │  - profile dir mounted :ro     │ ──► mock-inference    │ │
   │            │  - reads .../profiles/airgap   │      :8080            │ │
   │            └────────┬───────────────────────┘                       │ │
   │                     │ DATABASE_URL                                  │ │
   │             ┌───────▼──────────┐         ┌──────────────────────┐   │ │
   │             │ postgres         │         │ monitor (netshoot)   │   │ │
   │             │ (audit + state)  │         │ → ss/conntrack proof │   │ │
   │             └──────────────────┘         └──────────────────────┘   │ │
   │                                                                     │ │
   └─────────────────────────────────────────────────────────────────────┘ │
                                                                           │
                                       no other path off airgap-internal ──┘
```

**Why two networks.** Docker silently drops `ports:` on a container attached
only to an `internal: true` network — there is no host-routable interface to
publish from. The `ingress` socat container is the *only* container on both
networks; it forwards `host:8090 → app:8080`. The `app` itself never joins a
bridge network, so the system under test cannot egress even if its config were
misconfigured.

---

## 3. Containers and what they do

| Service | Image | Purpose | Networks |
|---------|-------|---------|----------|
| `postgres` | `postgres:18-alpine` | Audit + state; tuned for the audit-spine write pattern (`synchronous_commit=off`, larger `max_wal_size`, `wal_compression`) — see `airgap-guide.md` §4.2 for the WAL-fsync root-cause and tuning rationale. | `airgap-internal` |
| `mock-inference` | built from `../systemprompt-core/crates/tests/mock-inference` | Deterministic Anthropic-Messages-compatible responder. Counts hits at `GET /stats` so the egress proof can correlate. | `airgap-internal` |
| `app` | built from this repo's `Dockerfile` | The systemprompt binary, run with `SYSTEMPROMPT_PROFILE_DIR=/app/services/profiles/airgap`. **Profile dir mounted read-only.** | `airgap-internal` |
| `monitor` | `nicolaka/netshoot` | Sidecar with `ss`/`conntrack` for the egress assertion. Joined to `airgap-internal` so it sees every connection any other container makes. | `airgap-internal` |
| `ingress` | `alpine/socat` | Dumb TCP forwarder `:8090 → app:8080`. Runs no systemprompt code. The single demarcation point. | both |

**Read-only profile dir.** The secrets subsystem used to write a freshly
generated `manifest_signing_secret_seed` into `secrets.json` on first boot;
that required `:rw`. The subsystem now probes the directory's writability
(`dir_is_writable` in `crates/infra/config/src/bootstrap/manifest.rs`) and, on
a read-only mount, degrades to an ephemeral seed for the boot with a clear
warning. The air-gap `secrets.json.example` documents the
`manifest_signing_secret_seed` field; pre-supplying it makes the seed stable
across boots with no writes at all.

---

## 4. Boot sequence (what happens between `just airgap-up` and "healthy")

```
docker compose up
  │
  ├─► postgres        starts; healthcheck passes
  ├─► mock-inference  starts; listens on :8080
  │
  └─► app  (depends_on postgres healthy + mock-inference started)
        │
        ├─ entrypoint.sh:
        │    SYSTEMPROMPT_PROFILE_DIR=/app/services/profiles/airgap
        │    → load profile.yaml (validated by Profile::from_yaml)
        │
        ├─ systemprompt-config bootstrap:
        │    SecretsBootstrap::init() reads secrets.json
        │    ensure_manifest_signing_seed:
        │      Some(seed) supplied → return Ok
        │      else if !dir_is_writable → ephemeral seed + warn
        │      else → generate + persist_seed
        │
        ├─ DB migrations run on first boot
        │
        ├─ scheduler dispatches publish_pipeline (run_on_startup=true)
        │    → run_acl_yaml_load        (services/access-control/*.yaml)
        │    → run_gateway_policy_load  (services/gateway/policies.yaml)  ← WS1
        │         systemprompt_ai::load_gateway_policies_from_yaml(db, paths)
        │         GatewayPolicyIngestionService::ingest_config(
        │           override_existing: true, delete_orphans: true)
        │         → upsert / delete via AiGatewayPolicyRepository
        │    → ingestion, asset bundling, prerender, …
        │
        └─ healthcheck GET /api/v1/health → "healthy"
```

The publish_pipeline step that matters for this scenario is
**`run_gateway_policy_load`** in `extensions/web/jobs/src/publish.rs`. It calls
the core `load_from_yaml` entry in `crates/domain/ai/src/services/gateway/`,
which reads `services/gateway/policies.yaml`, deserialises it into
`GatewayPolicyConfig` (`#[serde(deny_unknown_fields)]`), validates names, and
reconciles the rows with the DB. `delete_orphans: true` means any policy
removed from the YAML is also removed from the table — the DB stays exactly
in sync with the committed config.

This is the structural fix for what used to be friction #10: the model
allow-list — a *security control* — used to exist only via raw SQL seeding
from a script. It now ships as version-controlled YAML alongside the rest of
the config.

---

## 5. The happy path: an allowed `/v1/messages` call

```
operator (host) ──curl──► localhost:8090
                              │
                          ingress (socat)
                              │  TCP forward
                              ▼
                          app:8080
                              │
                  ┌───────────┴─────────────────────────────────┐
                  │ /v1/messages handler (entry/api/.../extract.rs)
                  │                                              │
                  │  1. authenticate(jwt | x-api-key)            │
                  │  2. require x-session-id header  ──► 400 if absent
                  │  3. is_model_exposed(model)      ──► 403 if not in catalog
                  │  4. find_route(model)            ──► 404 if no match
                  │  5. enforce_authz_for_route       ──► 403 if denied
                  │       └─ message names route id+model+remedy (WS5 #9)
                  │  6. PolicyResolver.resolve()                 │
                  │       ai_gateway_policies (cached 60s)       │
                  │       quotas + safety; not model exposure    │
                  │  7. canonical → outbound adapter             │
                  │  8. POST to provider endpoint                │
                  │       provider.endpoint resolved from        │
                  │       expand_secrets(${ANTHROPIC_ENDPOINT})  │
                  │       → http://mock-inference:8080           │
                  └──────────────┬───────────────────────────────┘
                                 │
                          mock-inference (in-network)
                                 │
                            deterministic 200 + body
                                 ▼
                           response → app → ingress → operator
                                 │
                                 └─► 4 audit-spine writes:
                                       governance_decisions
                                       ai_requests INSERT
                                       logs
                                       ai_requests UPDATE
```

**Key code paths.**

- Header + route extraction:
  `../systemprompt-core/crates/entry/api/src/routes/gateway/messages/extract.rs`
  (`require_typed_header` for `x-session-id`, `enforce_authz_for_route`,
  `build_authz_request` — the route id, model, and remedy now appear in the
  denial message).
- Policy resolution:
  `../systemprompt-core/crates/entry/api/src/services/gateway/policy.rs`
  (`PolicyResolver::resolve` — 60s cache over `AiGatewayPolicyRepository::find_for_global`; merges multiple enabled policies).
- Provider endpoint resolution:
  `../systemprompt-core/crates/domain/ai/src/services/core/ai_service/service.rs`
  (`expand_secrets` — `${VAR}` interpolation for both `api_key` and `endpoint`;
  a provider with a custom endpoint stays enabled even without an upstream
  key, which lets the air-gap drop the placeholder Anthropic key entirely).

---

## 6. The deny path: an un-listed model

```
operator ──curl model=claude-opus-forbidden-99──► ingress ──► app
                                                                │
                                                  is_model_exposed(model)
                                                       catalog: [haiku, sonnet, opus-4-7, gpt-4-turbo, gemini-2.5-flash]
                                                  model NOT in catalog
                                                                │
                                                          HTTP 403
                                                  "model not permitted by gateway policy"
                                                                │
                                          ┌─ governance_decisions row written
                                          └─ mock-inference NEVER touched
                                                  (verified by mock /stats counter)
```

The deny path is the egress assertion's second measurement: the gateway must
reject *before* opening an upstream TCP connection. The mock's `/stats`
counter is the witness.

---

## 7. The four scripts

The `airgap-test` recipe runs them in order, stopping on the first failure.

### `01-egress-assert.sh` — egress closure (two independent proofs)

1. **Network proof.** From the `monitor` sidecar (which shares
   `airgap-internal`), capture `ss -tunp` (and `conntrack -L` if present)
   during a 25-request burst of *allowed* `/v1/messages` calls. Assert no
   remote endpoint outside the `airgap-internal` subnet. Any external
   address → fail.
2. **App-level proof.** Snapshot `mock-inference/stats.count` before, fire a
   25-request burst of *denied* requests (`model=claude-opus-forbidden-99`),
   snapshot again. The counter must be unchanged — denial precedes any
   upstream call.

### `02-load.sh` — sealed-network load + thresholds

**STEP 1 (changed in WS1).** No longer seeds the gateway policy — only
*verifies* the row landed. If `claude-haiku-4-5` is not in
`ai_gateway_policies`, the publish_pipeline ingestion failed and the run
aborts with a pointer at the logs.

**STEPs 2–5.** Ensure the demo admin user; run the core loadtest harness
(`../systemprompt-core/crates/tests/loadtest`) twice — once `gateway-inference`
(through to the mock), once `governance-only` (denial path). Parse JSON
results; assert `governance p95 ≤ 300 ms` and `error_rate ≤ 0.5 %`. Read the
mock `/stats` delta to confirm gateway-inference produced ≈ 1 mock hit per
request and governance-only added 0.

The loadtest now self-acquires a token against the air-gap profile thanks to
WS5 #13 (`acquire_token` honours `--profile`).

### `03-governance.sh` — four-stage pipeline, in isolation

Runs the existing `demo/governance/*.sh` checks against the air-gapped app
(port-rewritten to AIRGAP_HTTP_PORT) — pre-tool-call scope, secret scan,
blocklist, rate limit — and a routing proof:

- Allowed model → 200, with the response `model` field reflecting the mock's
  upstream remap (full engine → mock path confirmed).
- Denied model → 403 with `"not permitted by gateway policy"`.

### Results

JSON artifacts under `results/` (loadtest reports, mock `/stats` snapshots).
The summary tables in `docs-internal/evaluation/scenario1-airgap-template.md`
record the architecture proof matrix and security findings.

---

## 8. Profile and config inventory

The committed configs that close the system:

| File | What it pins | Why it matters |
|------|--------------|----------------|
| `.systemprompt/profiles/airgap/profile.yaml` | `gateway.routes[*].endpoint = http://mock-inference:8080`; route IDs aligned to the `gateway_route` grants in `services/access-control/roles.yaml`; `runtime.log_level: normal`; `cloud.tenant_id: null`; all three `api_*_url` pinned to in-container `:8080`. | Gateway never tries to dial Anthropic/OpenAI/Gemini; RBAC matches the routes so no `authz denied: not assigned`. |
| `.systemprompt/profiles/airgap/secrets.json.example` | `*_ENDPOINT` keys → mock URL; `manifest_signing_secret_seed`; no upstream provider api_key needed. | AI service providers (`config.yaml`) resolve their endpoints to the mock; missing api_key + present endpoint keeps the provider enabled (WS2). |
| `.systemprompt/profiles/airgap/catalog.yaml` | Declares the exposed models — referenced by `gateway.catalog_path` in profile.yaml. | Single source of truth for what `/v1/messages` accepts; dispatch and `/profile` both derive from it. |
| `services/gateway/policies.yaml` | Per-call ceilings, quotas, safety. Model exposure is NOT here. | Policy concerns kept separate from the model registry. |
| `services/ai/config.yaml` | Provider `endpoint: ${ANTHROPIC_ENDPOINT}` (and openai/gemini equivalents). | Endpoint interpolation lets the air-gap `secrets.json` point providers at the mock with no code changes. |

Validate any profile before boot:

```bash
systemprompt admin config validate .systemprompt/profiles/airgap/profile.yaml
# exits non-zero with the offending field/value when invalid
systemprompt admin config validate --schema > docs/profile.schema.json
```

---

## 9. Known limits / tech debt

- The cross-repo change set requires `systemprompt-core` ≥ 0.10.4. Until that
  releases, the template's `[patch.crates-io]` block must be uncommented for
  the template to build (see `Cargo.toml`).
- An operator must update the real `secrets.json` (git-ignored) to match
  `secrets.json.example` — the `*_ENDPOINT` keys and
  `manifest_signing_secret_seed` are required for a clean `:ro` boot.
- A production air-gap deployment on durable storage should keep
  `synchronous_commit=on` (the loosening in this scenario is for the WSL2
  evaluation host — see `airgap-guide.md` §4.2 for the rationale).
