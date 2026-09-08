# The Playwright harness

Browser coverage for the admin console: one deterministic dataset, two signed-in
principals plus an anonymous visitor, and a page spec per page.

Everything here runs against an **already-running** stack. It never boots a server:
this clone's server is shared between agents, so global setup pings `/health` and
fails fast instead of starting one.

```bash
just start                    # once, in another shell
just e2e-install              # once per clone
just e2e-seed --reset         # rebuild the dataset from scratch
just e2e                      # the whole suite
just e2e --project chromium   # what CI runs
just e2e-gate                 # the same, refusing loudly when no stack is up
```

`GATEWAY_URL` overrides the base URL, `E2E_DATABASE_URL` the database the seed
writes to, and `E2E_SIGNING_KEY` the key tokens are minted with. Without them the
harness reads the `local` profile, which is the right default for a lone
developer and for CI.

It is the wrong default when several agents share this checkout. The server on
the shared clone's own port belongs to whoever started it, and seeding or
restarting it disrupts their run. Point the harness at your own detached
worktree instead, and give it that worktree's database and signing key — all
three must come from the same place, or tokens will be minted against a key the
server does not know:

```bash
export GATEWAY_URL=http://localhost:8081
export E2E_DATABASE_URL=postgres://systemprompt:PASSWORD@localhost:5451/systemprompt
export E2E_SIGNING_KEY=/path/to/worktree/signing_key.pem
```

Two facts about the server make copying files into another checkout look
reasonable, and knowing them is what stops the next person reaching for it:

- **A template change needs a restart.** Partials register once when the admin
  template engine is constructed, not per request, so a `.hbs` edit sitting on
  disk is invisible until the process restarts.
- **A CSS or JS change needs a publish, not just a restart.** The browser is
  served from `web/dist`, not from `storage/files`, so the bundle has to be
  rebuilt for the change to exist at all.

**Run the suite from this checkout, whichever server you point it at.** Nothing
under `playwright/` is read by the server: it is test code, and the three
variables above are the entire coupling.

## The principals

Roles live in `users.roles`; the minted JWT's own role claim is ignored by the
server, so the seed is what decides what every authorization assertion sees.
This instance knows two roles.

| principal | id | roles | reaches |
|---|---|---|---|
| `admin` | `e2e-admin` | admin, user | every console page and the admin API |
| `user` | `e2e-user` | user | profile, settings and setup; every console page 303s them to their profile |
| `anon` | — | — | the sign-in page; every admin page 307s them to it |

Seven more accounts, `e2e-member-1` … `e2e-member-7`, hold the `user` role and
never sign in: they are the people the traffic is attributed to, spread over
three departments (Engineering: admin + members 1–3, Product: 4–5, Support:
6–7; `e2e-user` and `e2e-victim` stay in `Default`). `e2e-victim` is the account a
mutating spec may move without disturbing a row anything else counts.

The fixtures export a page per principal — `adminPage`, `userPage`, `anonPage` —
plus `cookieFor`, `apiAs`, `uniqueEmail`, `snapshot` and `dbRow` / `dbRows`.
Playwright resolves `storageState` per file, not per test, which is why
principals are page fixtures rather than an option override.

## The routes

`tests/support/paths.ts` mirrors `extensions/web/admin/src/routes/ssr.rs`. Specs
and page objects never write a literal admin URL.

| key | path | non-admin |
|---|---|---|
| `root` | `/admin` (308 → evals) | 308 |
| `users`, `user(id)` | `/admin/users`, `/admin/user?id=` | 303 |
| `departments`, `department(id)` | `/admin/departments[/{id}]` | 303 |
| `accessTokens` | `/admin/access-tokens` | 303 |
| `accessControl` | `/admin/access-control` | 303 |
| `requests`, `request(id)` | `/admin/requests[/{id}]` | 303 |
| `sessions`, `session(id)` | `/admin/sessions[/{id}]` | 303 |
| `traces`, `trace(id)` | `/admin/traces[/{id}]` | 303 |
| `contexts`, `context(id)` | `/admin/contexts[/{id}]` | 303 |
| `evals`, `evalRun(id)` | `/admin/evals`, `/admin/evals/runs/{id}` | 303 |
| `governance`, `governancePolicy(id)` | `/admin/governance`, `/admin/governance/policies/{id}` | 303 |
| `governanceDecisions` | `/admin/governance/decisions` | 303 |
| `governanceHooks` | `/admin/governance/hooks` | 303 |
| `demoTrace` | `/admin/demo/trace` | 303 |
| `models` | `/admin/models` | 303 |
| `profile`, `settings`, `setup` | `/admin/{profile,settings,setup}` | 200 |
| `login` | `/admin/login` | 200 |

Anonymous gets 307 to `/admin/login?redirect=…` on every row except the last.
`tests/support/nav.ts` carries the sidebar's five sections by label so a spec
can assert its own item is current without a hard-coded href.

## The dataset

`setup/seed.ts` is the connection, the order and the reset; the data itself lives in
`setup/seed/`. Every row carries an `e2e-` id prefix or an `@e2e.local` email, and
`--reset` deletes exactly those rows — never a TRUNCATE, never a developer's data.
The `departments` rows are the one exception: they are upserted on name and never
deleted, because a developer may have hand-assigned a real user to one.

| module | what it owns |
|---|---|
| `kit.ts` | `T0`, the offsets, the seeded PRNG, the id vocabulary |
| `principals.ts` | the principals, their roles, their sessions, their department, the ten traffic actors |
| `departments.ts` | the three departments, upserted on name |
| `tokens.ts` | one personal access token per member: live, expired and revoked in a fixed mix |
| `traffic.ts` | 12 sessions, 6 contexts, 240 AI requests over a 6-model × 10-outcome wheel, 60 tool executions, 40 skill invocations |
| `governance.ts` | decisions at all four chain stages (`secret_scan`, `scope_check`, `tool_blocklist`, `rate_limit`), safety findings, approvals, session analyses and ratings, the secret audit log |
| `evals.ts` | one completed judge run with 24 results over the completed requests |

Times are fixed offsets from `T0`, the instant the seed started, so two runs an hour
apart produce the same shape shifted forward. Column shapes mirror
`tests/contract/admin/src/seed.rs` and the schema under `extensions/web/schema/`
(plus core's own schema for the shared tables); if an insert breaks, diff against
those first.

Mutating specs never edit a seeded row. They create their own subject with
`uniqueEmail()`, which the reset backstop cleans up.

## Writing a page spec

Copy `tests/pages/_template.spec.ts`. Four describe blocks, always these four names:

- **renders** — the page draws its content from the seeded dataset
- **actions** — every mutation the page offers, and its visible result
- **authorization** — `authorizationTable(PATH, rows)` from `tests/support/shared.ts`,
  with `CONSOLE_ACCESS` (admin 200, user 303, anon 307) or `ACCOUNT_ACCESS`
  (user 200 too) or the page's own literal rows
- **design language** — `designLanguageTests(PATH)` for the active nav item,
  breadcrumb, `h1`, `--sp-` tokens, a visible focus ring, no horizontal scroll at
  1440 or 1024 and zero serious or critical axe violations, plus the page's own
  `expectDensity(page, 'list' | 'detail')` call — the measured density bar
  (`tests/support/density.ts`), which the shape gate requires to appear in the file

`scripts/check-spec-shape.sh` fails a spec missing one of them. It runs in
`just lint-gates`, in `just e2e`, and in CI.

Never write a literal admin URL or a literal CSS class. Routes come from
`tests/support/paths.ts`, component classes from `tests/support/pages/selectors.ts`.
Both exist so a rename is one edit instead of a sweep.

`snapshot(page, name)` writes baselines under
`tests/<dir>/__screenshots__/<file>/chromium-linux/`. It asserts nothing off Linux, so
a developer on macOS never rewrites the baselines CI compares against.

## Scratch files

A probe you are using to answer one question goes in `tests/tmp-<something>/` or
is named `_something.ts`. Neither prefix is type-checked or run. Without that
mark a scratch file joins the type graph and the suite, and a file written to
answer one person's question for ten minutes turns the harness red for everyone
else.

## CI

The `e2e` job of `.github/workflows/ci.yml` runs the suite: debug build,
`setup-local` against its own Docker Postgres, `just start`, wait on `/health`,
then `just e2e --project chromium`, with `playwright-report/` and the server log
uploaded on every result. `just e2e-gate` is the same tier for a local
pre-release pass; it refuses, naming the command to run, when no stack is up.
