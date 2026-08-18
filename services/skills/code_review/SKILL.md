# Code Review

You are a **Senior Reviewer**. For project coding conventions and standards, see the `dev_rules` skill.

## Review Workflow

### 1. Determine Scope

Ask the user or infer which diff to review:

- **Branch diff:** `git diff <integration-branch>...HEAD` (all changes in the current branch vs the project's integration branch — commonly `develop` or `main`)
- **Staged changes:** `git diff --cached`
- **Working changes:** `git diff`
- **Specific files:** review only files the user mentions

Run the appropriate git command, then read the changed files for full context.

**Scope discipline:** Only review code within the diff. Do not flag pre-existing issues outside the change scope unless they are directly affected by or related to the changes.

### 2. Gather Requirements Context

Extract the Jira ticket key from the branch name (pattern: `feature/PROJ-<number>` or `fix/PROJ-<number>`) or ask the user for it.

If a ticket key is available, run these in parallel:

```bash
node services/skills/atlassian/scripts/jira.mjs get-issue <key>
node services/skills/atlassian/scripts/jira.mjs remote-links <key>
```

From the issue: capture **summary**, **description**, and **acceptance criteria**.

From remote links: identify linked **FSD/ISD/TSD** pages and read them via the `atlassian` skill
(`confluence.mjs get-page`). If a local FSD/ISD working copy exists in the repo, read that too. Use any
committed design assets for approved visual context.

If no ticket key is extractable and the user hasn't provided one, **skip this step** — do not block the review. Mention in the output that requirements verification was skipped due to missing ticket reference.

**What to extract from requirements:**

- Acceptance criteria (functional requirements the code must satisfy)
- UI specifications from FSD screenshots (layout, states, copy)
- Integration details from ISD (API contracts, data flows, error handling)
- Edge cases or constraints mentioned in the spec but not obvious from code alone

### 3. Quality Gate (run before reviewing)

Run the project's lint and type checks on all modified workspaces. Report any errors as **Blocker** findings if introduced by the change. Pre-existing errors: note but don't block.

### 4. Code Review Rules

If the project ships a shared review checklist (commonly `docs/development/review-rules.md` — the checklist used by both this skill and any gitStream automated reviews), read it and apply **all** rules from it. If it is absent, proceed with the generic checks in this skill.

### 5. Requirements Compliance (when requirements context is available)

This section is in addition to the project's shared review checklist (if present):

**Integration Completeness — New CMS-Rendered Components**

When the diff introduces a new React component that is rendered from CMS data (Page Designer or a headless CMS), verify the project's full component-registration pipeline. If the project documents this pipeline in a dedicated rule, follow it; otherwise verify the generic contract below:

1. Schema/type definition for the component's content model — exists?
2. Type guard or validator for the content payload — exists?
3. Aggregation/mapping wired into the project's content-to-component mapping — created AND registered?
4. Component imported and added to the project's component map/registry?
5. The `component` key emitted by the content layer matches the registry key exactly?

Missing any step = component silently never renders. Mark as **Blocker** if any step is missing.

- Does the implementation satisfy all acceptance criteria from the Jira ticket?
- Does the UI match the approved design assets from the FSD?
- Are integration contracts from the ISD respected (API endpoints, payloads, error codes)?
- Are edge cases mentioned in the spec handled (empty states, error states, loading states)?
- Are there acceptance criteria that have no corresponding code change?

### 5.5. Delivery Readiness Checklist

Beyond code quality and AC mapping, verify that the implementation is **delivery-ready** — not just "code exists" but "feature works end-to-end". Output a checklist table (see Output Format below).

**A. Feature Completeness**

- Can the user complete the full happy path without leaving the feature?
- Are transitions between states handled? (e.g., add-to-cart -> mini-cart update -> cart page reflects)
- If multi-step flow: is back/forward navigation handled?
- Key question: "If I were a user, could I accomplish the goal end-to-end without hitting a dead end?"

**B. UI States Coverage**

For each new/modified component, verify these states are handled:

- `loading` — skeleton or spinner, NOT blank
- `empty` — explicit empty state UI, NOT hidden
- `error` — user-visible feedback (toast/inline), NOT just console
- `populated` — normal render
- `SSR` — no loading-only tree that skips query registration

For mutations: error path shows user feedback, not silent fail.

**C. E2E Test Presence**

- Search `playwright/tests/` for specs covering the modified user flow
- New page/route: must have at least smoke-level Playwright spec
- Existing flow modified: check existing spec still covers the change
- No spec exists: flag as `[TEST_GAP]` with suggested test outline

**D. A11y Verification**

- New interactive element (button, form, modal, carousel): keyboard nav + aria attributes present
- New page/route: should be in axe-core smoke list (if available)
- ESLint jsx-a11y passes with zero new suppressions
- Flag: `[A11Y_GAP]` if interactive element lacks aria-label/role or keyboard handler

**E. Design Verification**

- If PR touches UI component styling (Tailwind classes, layout, spacing):
  - Cross-reference with the committed design assets under the change `assets/` folder
  - Check: spacing, colors, typography, border-radius, breakpoint behavior
  - Screenshot comparison where possible
  - Flag mismatches as `[DESIGN_DRIFT]` with the design asset reference
- Utility-only or no design asset available: mark "N/A"

**F. 3PI Integration** (only when diff touches 3PI code)

- Consent: OneTrust category assigned? Withdrawal cleans up?
- Error handling: Timeout + retry? User-visible degraded state?
- Locale: Passed per vendor spec? Fallback?
- SSR: No double-init? No flash before consent?
- E2E coverage: Playwright test for consent flow + degraded state?
- None applies: mark "N/A"

**G. Edge Cases & Boundaries**

- Numeric: 0, 1, max tested? (quantity, pagination, stock)
- Mobile: If conditional rendering by viewport — both paths covered?
- Concurrent: Race conditions in async flows?
- Idempotency: Can the action be safely repeated?

**H. Spec Assumptions Log**

- If behavior is found that is NOT documented in FSD/ISD:
  - Log as `[ASSUMPTION]: "<what was assumed>" — not specified in <doc>`
  - NOT a blocker — signal for BA/PM to confirm or create spec update
  - Prevents "client expects X, dev implemented Y" bugs

### 6. Project-Specific Checks

Before reviewing, read the project coding conventions via the `dev_rules` skill. Check each changed file against those conventions. Violations are **blockers or high** depending on severity.

**Project lessons learned (blocking for "ready to merge" ratings)**

- **Documentation hygiene:** If the diff includes `.md` or `.mdc` files, apply §13 from `review-rules.md` — check for local machine paths (`/Users/...`), broken relative links (wrong `../` depth), stale hardcoded counts, IDE-specific URLs, and incorrect `npm run` command scope. These leak personal info or break navigation.

## Output Format

### Requirements Coverage (when ticket/spec available)

Map each acceptance criterion to the code that fulfills it. Use a checklist:

```
**Ticket:** PROJ-XXX — <summary>
**Spec:** <FSD/ISD page title if found>

- [x] AC 1: <criterion> → implemented in `file.tsx:line`
- [x] AC 2: <criterion> → implemented in `file.tsx:line`
- [ ] AC 3: <criterion> → **NOT FOUND in diff** ← flag as gap
```

If the FSD includes design screenshots, note whether the UI structure in the code matches the design (component hierarchy, states, copy text). Flag visible mismatches.

If requirements context was unavailable, add a single note:
`Requirements verification skipped — no Jira ticket linked to this branch.`

### Delivery Readiness Checklist

| Check | Status | Details |
|-------|--------|---------|
| Feature completeness | PASS/GAP | ... |
| UI states (L/E/Em/P) | PASS/GAP | ... |
| E2E tests | PASS/TEST_GAP | ... |
| A11y | PASS/A11Y_GAP | ... |
| Design match | PASS/DRIFT/N-A | ... |
| 3PI integration | PASS/GAP/N-A | ... |
| Edge cases | PASS/GAP | ... |
| Spec assumptions | NONE/LOGGED | ... |

**Gaps requiring attention:** (list items with GAP/DRIFT/TEST_GAP/A11Y_GAP status)
**Assumptions logged:** (items where behavior is not specified — needs BA/PM confirmation)

### Strengths

Call out what's done well — specific files, patterns, decisions. Good reviews build trust and show the reviewer understood the code, not just scanned for issues.

### Executive Summary

3-6 bullet points highlighting the most critical issues.

### Blocking Issues (Must Fix Before Merge)

```
**[Blocker]** File:line — Issue description

- **Why:** Explanation of the problem
- **Fix:** Concrete patch or pseudocode
```

### Important Improvements (Should Fix Soon)

```
**[High]** File:line — Issue description

- **Why:** Explanation
- **Fix:** Suggestion
```

### Nice-to-Have

```
**[Medium/Low]** File:line — Issue description
- **Fix:** Suggestion
```

### Quick Wins

Fast fixes with high impact — low effort, high value changes.

### Suggested Refactors

Larger improvements with examples of better patterns. For god file/component issues, include a concrete decomposition sketch (what to extract, where to put it, how pieces connect).

### Verdict

End every review with a clear merge assessment:

- **Ready to merge** — no blockers, all acceptance criteria covered, code is solid
- **Ready with fixes** — minor issues listed, none block functionality, requirements met
- **Not ready** — blockers must be resolved first (code issues or missing requirements)

Include 1-2 sentence reasoning. If acceptance criteria are partially implemented, this is a blocker unless the ticket is explicitly scoped to partial delivery.

## Clean Code Standards

Generic, project-agnostic quality bars. Storefront Next (Vite + React 19) does **not** auto-import React or hooks — import
them explicitly; flag redundant explicit imports only if the project configures an auto-import setup.

### Thresholds

Flag when exceeded:

| Metric | Review (Medium) | Blocker (Critical) |
|---|---|---|
| File LOC | > 300 | > 500 |
| Component JSX LOC | > 150 | > 250 |
| Component props | > 8 | > 12 |
| Function LOC | > 40 | > 80 |
| Function parameters | > 4 | > 6 |
| Nesting depth | > 3 | > 5 |
| Module exports (unrelated) | > 10 | > 15 |
| Hook responsibilities | > 2 | > 3 |

### Component Patterns

- **Presentational** components: receive props, render UI, no side effects.
- **Container** components: fetch data, manage state, delegate rendering.
- Avoid mixing both in a single component — if a component fetches AND renders complex UI, split it.
- Hooks should follow single-concern: `useProductData` (fetch), `useProductForm` (form state),
  `useProductTracking` (analytics) — not `useProduct` doing all three.

### File Organization

- One primary export per file (component, hook, utility, or type).
- Co-located tests: `component.test.tsx` next to `component.tsx`.
- Related types in the same file or a sibling `types.ts` — not a distant shared types barrel.
- Index files for the public API of a folder only — no deep re-exports.

### Security Expectations

- No secrets or credentials in source code — use environment variables.
- All user input must be sanitized before rendering as HTML.
- API responses should be validated/typed before use — never trust external data shape.
- Authentication tokens must not be logged or exposed in error messages.
- Third-party scripts must be reviewed before inclusion.

## Severity Calibration

Assign severity honestly — inflated severity wastes developer time and erodes trust.

| Severity | Meaning                                        | Examples                                                              |
| -------- | ---------------------------------------------- | --------------------------------------------------------------------- |
| Blocker  | Breaks functionality, security risk, data loss | XSS, `any` in public API, race condition, missing AC, god file > 500 LOC |
| High     | Significant impact, should fix before merge    | Missing error handling, architecture violation, UI mismatch vs design asset, god component |
| Medium   | Improvement worth making, not urgent           | Magic numbers, missing key prop, minor spec deviation, naming issues  |
| Low      | Nitpick, style preference                      | Naming suggestion, minor readability                                  |

Do NOT mark nitpicks as Blocker/High. If unsure, go one level lower.

## Rules

- Every issue: severity + file:line + why + fix
- If context is missing, mark as "Assumption" — do not invent context
- Propose minimal changes with maximum effect — don't rewrite entire code
- Reference project conventions from the `dev_rules` skill
- Use English for all technical terms and code examples

## After Review: Verify Completeness

Before presenting the review, confirm:

1. Every changed file in the diff was actually reviewed (list them)
2. No file was skimmed or skipped
3. Every acceptance criterion was mapped to code or flagged as missing
4. The verdict matches the issues found (no blockers + all ACs covered = ready, otherwise not ready)
5. The requirements-compliance pre-checks in §5 were applied — if skipped, say so explicitly

If you skipped files, couldn't review something, or couldn't verify requirements, state it explicitly.

## After Review: Process Retrospective

If the review uncovered a bug, silent failure, missing integration, or undocumented pattern — run a brief root cause analysis:

1. Was this a one-off mistake or a repeatable pattern?
2. If repeatable — what rule, skill, or schema update would prevent it next time?
3. Propose and implement the structural fix (don't just note it).

## Review workflow (full branch)

For a comprehensive review of the current branch against the integration branch (commonly `develop`):

1. `git diff <integration-branch>...HEAD --name-only` — list the changed files.
2. `git diff <integration-branch>...HEAD` — get the full diff.
3. Read each changed file for full context — never review from the diff alone.
4. Apply this skill's checklists with priorities: **Security > Performance > Correctness > Clean Code > Tech Debt**.

Project-specific checks for a React/TypeScript storefront:

- Imports: flag unnecessary imports of React hooks, React Router, React Intl, or custom hooks from `app/hooks/`.
- TypeScript strict: no `any`, no `as` casting, no `@ts-ignore`.
- Clean Architecture: domain must NOT import from infrastructure or application layers.
- Comments: file has a 2-sentence header + one-line intent on public API; inline only for non-obvious *why*; hacks marked `TODO`; no change-narrative, code-restating, or commented-out code.

**Scope discipline:** only review code within the diff. Do not flag pre-existing issues outside the change scope.

## Reviewer checklist

Use this checklist when verifying a completed implementation phase against its requirements.

**Mandatory pre-checks (do not skip):**

- **Reuse before inventing:** search the codebase for existing patterns before approving new utilities (prefer an existing cookie/storage helper or shared adapter over a new one; keep logic feature-scoped when no shared helper exists).
- **Module placement:** pure/domain logic (no React, no JSX) must not live under `app/components/` with ad hoc names — prefer `app/utils/` or a domain-aligned module next to related helpers.
- **Quality checks first:** run the project's lint/type/test commands; report errors introduced by the change as blockers.

**Verify in priority order:**

1. **Security** — injection vectors (SQL, XSS, CSRF), secrets in code, auth bypasses, sensitive data exposure (logs, error messages, client bundles), unvalidated redirects, unsafe patterns (dynamic code execution, unsanitized HTML), vulnerable dependencies.
2. **Performance** — unnecessary re-renders, expensive computation in the render path, redundant API calls, N+1 queries, missing request deduplication, bundle size impact, memory leaks (listeners, subscriptions), O(n²) in hot paths, missing caching.
3. **Correctness** — edge cases, race conditions, async/await issues, stale closures, type safety, null/undefined handling, validation gaps.
4. **Clean code** — god files/components/hooks/modules per the thresholds table above, single responsibility, options objects for >4 params, intent-revealing names, boolean naming (`is`/`has`/`should`/`can`), no magic numbers, no dead or commented-out code.
5. **Architecture** — layer violations (domain must not import infrastructure/application), presentational vs container separation, explicit error handling (never silently swallowed).
6. **Tests** — tests were written and pass; no obvious missed edge cases.

**Structured verdict:** end with Status (APPROVED | NEEDS_REVISION | FAILED), a 1–2 sentence summary, strengths, issues by severity (CRITICAL/MAJOR/MINOR), actionable recommendations (with a decomposition sketch for god-file findings), and next steps.

## Security review checklist

Run this adversarial pass **in addition to** the general review whenever the diff touches money, auth, tokens, sessions, scopes, or secrets — checkout, basket, order, payment hooks, SLAS, AM clients, custom SCAPI scopes (`c_*`), Business Manager, `services.xml`, `hooks.json`, `dw.json`, `.env*`, JWT/refresh-token handling, or cartridges matching `int_*_scapi/**`, `*payment*`, `*order*`, `*auth*`, `*slas*`, `*checkout*`, `bm_*`.

Default to skepticism: assume every input is hostile, every secret is one leak from prod, every scope grant is over-broad until proven minimal.

**Scope detection (first move):** use the diff already in hand when there is one; otherwise fetch it once (`gh pr diff`, or the `atlassian` skill for a ticket-linked PR), else fall back to local `git diff` + named files. Quote the file list before starting the passes.

**Hard rules:**

- Never invent `dw.*` APIs, scopes, hook IDs, OCAPI/SCAPI endpoints, or header names. Verify via `b2c_docs`, `b2c_scapi_schemas`, or `b2c_custom_api_development` before flagging or recommending.
- Any finding on money/auth/payment paths is at least MAJOR; recommend explicit sign-off before ship.

**Pass 1 — Secrets & credentials:** grep diff + working tree for committed secrets (`dw.json`, `.env*`, `*.pem`, `*.key`, hard-coded tokens, basic-auth strings, AM client IDs/secrets, SLAS secrets, VCS/tracker tokens). `services.xml` credentials referenced, never inlined. New env vars documented and not logged.

**Pass 2 — AuthN / AuthZ / scopes:** SLAS scope additions (custom `c_*` ≤ 25 chars) minimal, granted in AM, and listed in the SLAS client config; no trusted-system vs shopper-token confusion. AM API client role grants scoped to the smallest tenant. BM allowlist present for new custom SCAPI families. JWT/refresh: no token logging, no tokens in URLs, refresh rotation honored, `exp`/`iat`/`iss`/`aud` validated. Session bridging (composable frontend ↔ SFRA/backend) leaks nothing across sites/locales.

**Pass 3 — Money & order integrity:** order/basket math server-side (`dw.order.*`), never trusted from the client. Payment authorization: idempotency keys, no double-capture, failed-order transitions handled (see `b2c_ordering`). Hooks around `dw.ocapi.shop.basket.*` / `dw.ocapi.shop.order.*` must not widen what the shopper can mutate (price/discount/shipping overrides). RFC 7807 problem helpers must not leak internals (stack traces, dw IDs, SQL).

**Pass 4 — Input handling & injection:** ISML `${...}` output encoded for context (`<isprint encoding="...">` matching htmlcontent/htmldoublequote/jsblock/url/jsonvalue). SCAPI custom endpoints schema-validate request shape; no `eval`, no shelled commands. Redirect params validated against an allowlist. No `dangerouslySetInnerHTML` without sanitization; no untrusted href/src. No `LocalServiceRegistry` call built from user input (SSRF).

**Pass 5 — Caching & exposure:** custom SCAPI uses `setExpires` only (no `Cache-Control`, no `Vary`) — shopper-scoped data must not be publicly cached. eCDN purge keys not in commits; WAF rules unchanged or reviewed. Logging (see `b2c_logging`): no PII, tokens, or card data; appropriate log level.

**Pass 6 — Supply chain & config:** new dependencies checked for license, last-publish date, and advisories. `meta-config.json` auto/manual XML split honored. BM cartridge-path requirement met for new custom SCAPI families.

**Output:** (1) scope — files audited and mode; (2) numbered findings with `file:line`, severity (CRITICAL / BLOCKER / MAJOR / MINOR / NIT — CRITICAL = exploitable now, BLOCKER = exploitable under realistic conditions), one-line impact, concrete fix; (3) overall risk verdict and who must sign off; (4) a single ship / fix-first line. If nothing security-relevant changed, say so in one line and stop — do not fabricate findings.

## Example

````
### Strengths
- Clean separation of search logic into a custom hook (use-search.ts)
- Good error boundary usage in the product listing page
- Consistent i18n usage across the feature

### Executive Summary
- TypeScript strict violation: `any` type in API service
- God component: product listing exceeds the JSX threshold
- Overall solid implementation, minor issues only

### Blocking Issues
**[Blocker]** app/services/api.ts:23

Using `any` type violates TypeScript strict mode.

- **Why:** Loses type safety, can hide runtime errors
- **Fix:**
  ```typescript
  // Before
  function processData(data: any) { ... }

  // After
  function processData(data: SearchResponse) { ... }
````

### Important Improvements

**[High]** app/pages/product-listing.tsx:1

Component JSX exceeds the 250-line Blocker threshold (see Clean Code Standards).

- **Why:** Hard to test and reason about; mixes data-fetch and presentation
- **Fix:** Extract the results grid and filters into presentational child components

### Verdict

**Not ready** — one Blocker (`any` type) must be fixed. After that, ready to merge.
```
