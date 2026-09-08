# Dashboard port coverage

Source: `systemprompt-astound` at `083df9e1` (changes reviewed August 31–September 8, 2026).
Destinations: template and internal. This document records the feature boundary;
build and runtime results are recorded below once validation completes.

| Feature | Source anchors | Integrated surface |
| --- | --- | --- |
| Dense shared design, charts, navigation | `9fd0af41`, `8e8a04ec`, `259c3374` | Shared partials, CSS, JS components, Overview landing, compatibility routes |
| Overview and analytics | `521f1ec6`, `7aa6d0c0`, `1eb4519a` | Six analytics tabs, request/p50/model charts, hook metrics, rollup/anomaly jobs |
| People and access | `625424ac`, `89099784`, `9250433b`, `2a724de2` | Groups, projects, per-person roles/devices, user access tab, custom role strings |
| Scope attribution | `970c4eef`, `53cd1a16` | Primary scope defaults, exclusive/member accounting, filters and recomputation |
| Requests and governance | `ecfd9ef6`, `6f832df9`, `e074847b`, `f8853281` | Rejected/unattributed requests, warning and secrets exports, approvals, time-window preservation |
| Account and connected clients | `74b745ed`, `38ea6926`, `4283cc0d`, `4306d94a` | Settings persistence/account closure, connection tabs, device rows, connector lifecycle |
| Conversations | `900c1168`, `7f23329d`, `4306d94a` | Session-derived identity, readable transcripts, folded tool results, side-call summaries |
| Query scaling | `e23331e1` | Bounded listings, batched membership/usage, page-before-title enrichment, retained totals |
| Generic platform catalog | `5ef33dc1`, `20987651`, `58fc9396` | MCP, marketplace, plugin, skill and gateway pages; authenticated connector APIs |

## Compatibility decisions

- Departments, organizations, plans, memberships and their ACLs remain intact.
  Groups/projects are independent additions; no department conversion is inferred.
- New migration numbers start at 051, above either destination's historical chain.
  Astound migration 046 and tenant seed/backfill migrations are not imported.
- Core facade dependencies use 0.48.0 in both Rust workspaces. Local path patches
  are development-only and must not be committed.
- The template retains passkeys, registration, magic links, evaluations and demos.
  Internal additionally retains Odoo/operator login, enterprise reporting and bridge consent.
- Known console roles control route permissions; entitlement roles remain free text.
  Existing `admin` operators can manage directory mappings without requiring a new bootstrap role.
- Astound's tenant identities, private marketplace content, SSO-only restrictions,
  knowledge ingestion and requirements workflows are excluded. Governance policy is unchanged.

## Acceptance checks

Asset references, imports and JavaScript syntax; coordinated builds; fresh/additive
schema installation; populated upgrade data preservation; owner/admin history;
read-only mutation denial; custom roles; settings persistence; connection tabs;
legacy routes and destination-specific pages. Full release gates remain part of
the existing promotion flow. No dashboard latency target is claimed without measurements.

## Schema validation — 2026-09-08

Both destinations passed isolated PostgreSQL 18 checks. Schema-only snapshots of
existing local databases were restored into disposable databases; no existing
rows, credentials, server processes or database objects were changed.

- Additive migrations 051–062 execute and replay successfully after core 0.48's
  request-session/classification migration. Custom user role arrays, users,
  departments, and internal organizations remain unchanged.
- A populated cross-user conversation retains the latest owner, counts two turns
  plus one probe, and totals the seeded cost correctly. A different context filter
  returns no rows. Migration replay preserves these results.
- Fresh web installation was exercised over a data-free core schema by removing
  web-owned tables and applying the registered definitions in the core installer's
  structural/dependent order: 141 statements for template, 153 for internal.
  PostgreSQL parsed and executed the definitions; statement AST categories matched
  the declarative allowlist. LOC columns, transcript search and conversation
  functions are present; the unassigned-group seed inserts once.
- Both repository schema lint checks pass. Imperative LOC/search alterations stay
  in migrations; fresh tables declare those columns directly. The unassigned
  group is an explicit boot seed, separate from declarative DDL.

This validates SQL installation and preservation, not the complete application
startup or migration-ledger orchestration. Those are covered by the isolated
runtime smoke below when the newly built binaries are available.

## Isolated runtime and browser smoke

The focused suite is `playwright/tests/dashboard-port.spec.ts`. Its dedicated
`dashboard-port.config.ts` never starts a server or seeds a database. It checks
19 scenarios: representative dashboard/catalog pages, navigation, connection-tab
mouse/keyboard behavior, account/history access, and the destination's login flow.
No screenshot baselines are used. Test discovery passed; browser execution is
recorded separately after the current binaries are built.

Prepare independent profiles, storage/services copies, signing keys, fixture-only
PostgreSQL databases and admin/user storage states under
`/tmp/dashboard-port-smoke/{template,internal}`. Use ports 18085 and 18086,
respectively. Scheduler jobs and service auto-start are disabled in the copied
services configuration. Original services, provider credentials and signing keys
are untouched. Run only the API service from each newly compiled binary with
`SYSTEMPROMPT_PROFILE` pointing to that smoke profile; never use the shared
`just start`/stop commands or the option to kill another port owner.

Publish using that same binary/profile (`infra jobs run publish_pipeline`) into
the temporary output tree before browsing. Each principal's cookie is signed
with the isolated key and names its own seeded session. Run from the destination's
`playwright/` directory, setting the matching explicit environment:

```bash
GATEWAY_URL=http://127.0.0.1:18085 \
E2E_ADMIN_STORAGE_STATE=/tmp/dashboard-port-smoke/template/admin.json \
E2E_USER_STORAGE_STATE=/tmp/dashboard-port-smoke/template/user.json \
npx playwright test --config dashboard-port.config.ts
```

For internal, substitute port 18086 and the `internal` state directory. Shut down
only the recorded smoke process, then drop only its recorded `_test_` database
and remove its temporary files. Browser startup, publish and HTTP results must be
reported independently of SQL checks; a build or SQL pass alone is insufficient.

## Desktop client artifact prerequisite

The template contains the dashboard and bridge enrollment APIs, not desktop
client binaries. `services/web/config/bridge.yaml` defaults to
`download_base: null`; Profile and workstation setup show “Desktop client
downloads are not configured” and omit installer/download links. Connection
codes and sign-in commands for an already-installed `systemprompt-bridge`
remain available when the gateway URL is configured.

To offer installers, build the generic client from the compatible
`systemprompt-core/bin/bridge` checkout (see `bridge/CORE_REF` and that client's
README; core's coordinated recipes are `just build-bridge` and
`just build-bridge-all`). Package the supported platform artifacts and publish
an installer that accepts `--download-base`, `--gateway`, `--code` and `--host`.
Configure `download_base` as the absolute HTTPS artifact directory, or as
`/files/downloads` after placing the files under `storage/files/downloads/`.
The directory must provide `install.sh`, `systemprompt-bridge-windows.exe`,
`systemprompt-bridge-macos.dmg`, `systemprompt-bridge-linux-x86_64.tar.gz` and
`systemprompt-bridge-linux-aarch64.tar.gz`, with each artifact's `.sha256`
companion. The installer must verify these checksums before installing.
The dashboard builds its download and checksum links from the configured base;
it does not generate or upload platform artifacts.

Internal continues to use its existing `bridge_downloads` release service,
`systemprompt-internal-bridge` artifact names, and Odoo-aware setup and consent.

## Final source-coverage decisions

The source's MCP runtime and marketplace layout styles, route-editor comment/ID
preservation, advertised-provider filtering, connector readiness intersection,
and read-only console search are included. Existing gateway file-path resolution
and public config response fields remain compatible with each destination.

The source's automatic 90-day raw-event deletion and fail-open gateway entitlement
guard are deliberately excluded as deployment-policy changes. Its external-agent
read repository has no UI consumers, and its legacy JavaScript bundler is not
registered; neither is needed by these dashboard pages. Astound-only authentication,
Slack alerting, knowledge and requirements integrations remain outside this port.

The final frozen-source audit covered dashboard Rust, jobs, templates, shared/page
CSS, JavaScript and their registrations. Remaining differences preserve destination
behavior or belong to the exclusions above. Raw transcripts and audit payloads keep
the existing admin/auditor boundary; ordinary read-only console access does not
expand access to these sensitive payloads. Destination department editing and
existing role-management APIs are retained alongside the new directory pages.

Usage anomaly findings remain persisted and emit structured warning logs.
Tenant-specific Slack delivery is excluded; the port does not introduce an
external notification destination. Internal demo daily charts and business
report charts are adapted to the shared chart context while retaining their
existing data and routes.

## Isolated runtime and browser validation (2026-09-08)

Both newly compiled destination binaries activated the full API router on private
smoke ports (template 18085, internal 18086). Each used its own disposable database,
JWT key, storage copy and output directory; existing servers and databases were
not changed. Temporary services disabled inference agents, MCP subprocesses and
scheduled jobs. Each explicit publish pipeline completed all 11 steps successfully.

The focused Playwright suite passed all 19 tests in each destination (template
14.3 seconds, internal 13.7 seconds): 15 dashboard pages, sidebar navigation,
connection-tab selection and keyboard focus, ordinary-user account/history access
and directory denial, and the retained anonymous login flow. Page checks include
successful authenticated shell bootstrap and no browser console or page errors.

The actual extension installer applied all 12 dashboard migrations and web seeds
on the schema clones with their original migration-ledger metadata. Internal's
pre-existing default-department seed exposed an empty-organization failure on the
upgraded NOT NULL org_id schema. The seed now establishes the house organization
before inserting Default with an explicit house org_id. The new SQL passed an
empty-organization/department check in a rolled-back transaction. Internal's first
browser run used the equivalent house/default fixture. The rebuilt binary then
passed actual startup with organizations, departments and plans emptied: its
registered seeds created Default under house and marked house as the platform
organization, then activated the full API router without any fixture preseed.

Manual 1440-pixel desktop screenshot inspection covered Overview and Profile
in both forks. It caught and removed three obsolete internal dark-token files
whose cascade made new white cards unreadable; internal now uses the shared
primitive palette, with its own navigation and branding intact. Fresh screenshots
confirm readable cards, chart axes and connection controls. Seeded overview totals
reconcile to two requests and $0.000300; each user profile shows its own one
conversation and $0.000150. Browser checks use fixture data, not real provider
inference, external connector authorization or downloaded desktop installers.

A downstream preservation audit compared every internal extension module
registration with its pre-port HEAD, then checked modified Rust files outside
extensions. The only unintended missing module was the legacy session registry,
which is restored alongside the new telemetry. The other removals are accounted
for by replacement modules: contexts data loading, users roster/detail views,
and access-control matrix types. No additional lost module registrations were
found; business reports, demo charts and internal routes remain integrated.

Final rebuilt binaries were restarted after the last integration changes. Both
passed Profile HTTP 200, authenticated account bootstrap, generic connector
access copy and absence of browser errors. Only the owned smoke API processes
were stopped, both recorded disposable databases were dropped, and generated
signing keys, cookie states and database-secret files were removed. Test logs
and screenshots remain under `/tmp/dashboard-port-smoke/` for review.

## Focused Rust regression checks

`just test-dashboard` runs the focused checks under the existing build coordinator.
The optional `unit`, `contract`, `integration`, and `registry` stages allow a failed
stage to be resumed without repeating passing stages. Tests use disposable databases.

Template: 42 unit tests, 28 contract tests (including the hook rollup regression),
and 80 scoped database integration tests passed, including the populated
transcript upgrade lifecycle regression (46.6 seconds for the integration stage). Coverage includes department ACL
compatibility, custom roles, account closure/audit retention, read-only mutation and
raw-evidence denial, developer-login redemption, project-scoped traces, conversation
classification and session/daily counters.

Internal: 50 unit tests and 28 contract tests passed, including its eight legacy
session-registry tests and a real-handler regression verifying workspace/activity,
legacy live cost/context utilization, and new token/cost snapshots together.

The full-text-search schema follows the published core 0.48 installer phases:
structural tables, pending migrations, then dependent indexes. The generated
`search_tsv` column and GIN index therefore remain declarative; migration 059
adds the column to existing tables before the index phase. Fresh installs stamp
migrations without executing them, so moving the index exclusively to migration
059 would omit it. `usage_conversation_summary_schema` exercises this production
lifecycle against a populated transcript after removing the search column and
only its migration-ledger entry; it never pre-applies migration SQL.
