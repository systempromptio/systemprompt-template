# Test — Playwright Verification

**No change is done until Playwright proves it.** "It compiles" and "the unit tests pass" are necessary, not sufficient: the final stage of the Astound development workflow (plan → build → release → **test**) is demonstrating the change working in a real browser. Never declare a task complete, and never mark a pull request ready for review, without reporting a passing Playwright run for the affected flows.

## Workflow

1. **Find the project's Playwright setup.** Look for `playwright.config.ts` / `playwright.config.js` and an `e2e/` or `tests/` directory, and check `package.json` scripts (`test:e2e` or similar). The project's own configuration always wins over anything in this skill.
2. **If Playwright exists:**
   - Run the suite scoped to what you changed first (`npx playwright test <spec-or-grep>`), then the full suite if the project's conventions demand it.
   - A changed flow with no covering spec gets one **in the same change** — extend the nearest existing spec file, matching its fixtures and helpers.
3. **If the project has no Playwright:**
   - Scaffold the minimum: `npm i -D @playwright/test`, `npx playwright install chromium`, a `playwright.config.ts` with `baseURL` from the project's dev-server env, and one smoke spec covering the flow you changed.
   - Keep the scaffold small and note it in the PR — adopting Playwright project-wide is a `dev_plan` decision, the scaffold just proves your change.
4. **Report the evidence.** The PR's verification section (see `dev_release`) states the exact command run, the pass/fail counts, and names the specs covering the change. On failure, fix the change or the spec — never skip, quarantine, or `.skip()` a failing test to get green.

## Writing good specs

- Test user-visible behaviour through the UI (roles, labels, URLs), not implementation details; prefer `getByRole`/`getByLabel` over CSS selectors.
- Use Playwright's auto-waiting web-first assertions (`await expect(locator).toBeVisible()`); never hard-code `waitForTimeout`.
- Each spec is independent: sets up its own state (fixtures, API seeding), makes no assumptions about execution order.
- Storefront flows that matter most: navigation/PLP, product detail, cart, checkout, account. A change touching any of these runs that flow end to end.

## Environment notes

The Astound developer sandbox ships Node and Chromium, so `npx playwright test` works out of the box inside it. Cross-browser runs (WebKit/Firefox) follow the project's CI configuration — do not add browsers locally without need.

## Where to go deeper

| Topic | Skill |
|---|---|
| Browser automation mechanics | `playwright_cli` |
| Accessibility audits | `a11y_audit` |
| Reviewing a branch or PR | `code_review` |
| Root-causing a failure | `systematic_debugging` |

## Astound rules

The full rule text lives in `dev_rules`. Most relevant at the test stage: **Verification
standards** (browser testing is mandatory — code review alone is not verification),
**FE definition of done** and **FE edge cases checklist** (the self-check before requesting
review), and **Process retrospective** (a bug that slipped through triggers a process-level RCA,
not just a fix).
