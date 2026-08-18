# Build — Storefront Next Development

Build conventions for Salesforce Commerce Cloud work. The rules differ by context: **Storefront Next** (code-led, React Router based) and **classic B2C / SFRA** (cartridge-led). Identify which one the project uses before writing anything — the mounted project's own configuration and lockfile are the source of truth.

## When to Use

Use this skill while implementing an approved plan from `dev_plan`. It is the second stage of the Astound development workflow: plan → **build** → release → test.

## Identify the project shape first

- `react-router.config.*` / `app/routes` → **Storefront Next**: routes, loaders/actions, server-first data flow.
- `cartridges/*/cartridge/controllers` + ISML templates → **SFRA**: controller/pipeline extension model.
- PWA Kit (`pwa-kit.config`) → managed-runtime React storefront; follow its retail-react-app extension points.

Never mix idioms: no client-side data fetching where a route loader belongs, no new cartridge when an override in an existing site cartridge suffices.

## Storefront Next rules

- **Routes own data.** Fetch in route loaders/actions, not in components; components receive props. Mutations go through actions with progressive enhancement (forms work without JS).
- **Extend, don't fork.** Prefer the platform's documented extension points (route overrides, component slots, hooks) over copying platform source into the project.
- **Libraries are approved, not adopted.** Use what the project's `package.json` already ships. Proposing a new dependency is a `dev_plan` decision record, not a build-time choice.
- **Server-first.** Anything that can render on the server does; client components are the exception and say why.

## General build rules

- Match the surrounding code: naming, file layout, error handling, and comment density of the file you are editing.
- Small, single-purpose commits that map to the spec's step plan (format governed by `dev_release`).
- No dead flags or commented-out code left behind; delete, don't disable.
- Consult the `knowledge-bank` MCP server when a convention is unclear — prior project decisions outrank general best practice.
- Work is not complete until it passes verification under `dev_test` (Playwright).

## Executing a plan

Work an approved plan phase by phase, with a review gate between phases:

1. **Implement one phase at a time** — tests first (watch them fail), minimal code, tests pass.
   Run the individual test file first, then the full suite for regressions. When creating or
   modifying rendered UI, add stable `data-testid` selectors per `sfnext_test_ids`.
2. **Quality gate** — lint and type checks after every phase (see the Quality gate rule in
   `dev_rules`); fix all errors before review.
3. **Review** — check the phase against its objective and acceptance criteria per `code_review`.
   Needs revision → back to step 1 for that phase.
4. **Commit gate** — present what changed and a commit message per `dev_release` / `git_commit`;
   the user commits or requests changes. Then the next phase.

When uncertain about an implementation detail, stop and present 2–3 options with pros and cons
rather than guessing. Do not reset file changes without explicit instruction.

## Where to go deeper

| Topic | Skills |
|---|---|
| Storefront Next | `sfnext_routing`, `sfnext_components`, `sfnext_data_fetching`, `sfnext_state_management`, `sfnext_authentication`, `sfnext_i18n`, `sfnext_tailwind`, `sfnext_page_designer`, `sfnext_performance`, `sfnext_configuration`, `sfnext_extensions`, `sfnext_project_setup`, `sfnext_hybrid_storefronts`, `sfnext_mrt_data_store`, `sfnext_deployment`, `sfnext_test_ids` |
| B2C server-side | `b2c_hooks`, `b2c_scapi_custom`, `b2c_scapi_shopper`, `b2c_scapi_admin`, `b2c_scapi_schemas`, `b2c_custom_api_development`, `b2c_custom_job_steps`, `b2c_custom_objects`, `b2c_custom_caches`, `b2c_metadata`, `b2c_ordering`, `b2c_querying_data`, `b2c_webservices`, `b2c_slas`, `b2c_slas_auth_patterns`, `b2c_logging`, `b2c_localization`, `b2c_content`, `b2c_page_designer`, `b2c_business_manager_extensions`, `sfcc_general_development` |
| Offline `dw.*` API reference | `sfcc_api_classes` |
| Instance operations | `b2c_config`, `b2c_sandbox`, `b2c_logs`, `b2c_webdav`, `b2c_sites`, `b2c_users_roles`, `b2c_am`, `b2c_cap`, `b2c_cip`, `b2c_ecdn`, `b2c_onboarding`, `b2c_docs` |

## Astound rules

The full rule text lives in `dev_rules`. Most relevant while building: **Quality gate** (lint +
type checks after every change), **SFCC edit-then-verify** (confirm the edit reached the sandbox
before debugging symptoms), **Code comments** and **Coding standards** (comment discipline and
TS/React conventions), **Accessibility guidelines**, and **Stuck-loop reflex** (stop and
micro-reflect instead of retrying variations).
