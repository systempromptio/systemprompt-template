# Astound Development Rules

The house rules for all Astound storefront and B2C Commerce development. The four
entry-point skills (`dev_plan`, `dev_build`, `dev_release`, `dev_test`) reference the
rules relevant to their stage; this skill is the full text.

## Always-applied

These rules apply to every task, regardless of file or phase.

### Autonomous Verification & Action

*Be autonomous — try before asking, exhaust all self-service options, only escalate to user for decisions not actions*

**Try first, ask only if stuck.** The user is a decision maker, not an action executor.

#### Core Rules

- Read-only operations (browsing, API calls, log checks, screenshots) — just do them. No permission, no announcing.
- Authentication: try headless + persistent session first. Check **page content** (not URL) for login state. Only escalate if session is confirmed expired.
- Verification: use **multiple methods** before reporting "missing". A single 404 is not proof. Cross-verify delegated agents' claims via an independent channel.
- Capability claims: never state "X doesn't support Y" without verifying first. Research, don't ask.
- Multi-step tasks: execute sequentially without pausing. Pause only on failure requiring a decision.
- Errors: try alternatives before reporting. Report all attempts, not just the last.

#### When to Involve the User

**Decisions only:** choosing between approaches, approving destructive/visible changes (per the No Unsolicited External Actions rule below), providing credentials you cannot obtain, confirming requirements.

**Never ask:** "Should I check X?", "Should I try Y?", "Which verification method?" — just do it.

### English-Only Repository Artifacts

*All repository artifacts must be written in English. Chat language has no bearing on file content.*

Everything written to disk or pushed to remote **MUST be in English** — code, identifiers, comments, docs, rules, skills, tests, SFCC metadata, commit messages, branch names, PR titles/bodies, Jira, Confluence.

User "respond in \<language\>" applies to **chat only**, not files.

### Git Workflow

*Before any branch, commit, push, or PR, follow the `git_commit` skill (branch naming, --no-track, commitlint format, feature/requirements branch flow)*

Before creating any branch, commit, push, or PR, follow the **`git_commit` skill**. It is the source of
truth for branch naming, safe branching, commit-message format (commitlint), the
`feature/<name>` → `requirements/<name>` flow, and the push/PR gate.

**Never** push to a protected branch (`develop` / `main` / `master`). Branch off the integration branch
with `--no-track`.

### No Unsolicited External Actions

*No commits, pushes, PRs, Jira/Confluence/Slack actions, deploys, or staging/production SFCC/MRT config changes without explicit user authorization*

**NEVER** create/update/delete external resources (git commits/pushes/PRs, Jira, Confluence, Slack, deploys) unless explicitly authorized.

#### Default: finish local work, stop

Do NOT volunteer "want me to commit?" Present results, wait.

#### Explicit authorization = one of:

1. **Direct command** — "commit", "push", "open PR", etc.
2. **Pre-auth block** — scoped list of allowed actions. Honor scope exactly; ask for actions NOT covered.
3. **Inherent requirement** — "create a Jira ticket" inherently requires Jira; preview fields before executing.

None of the above → stay local.

#### Git discipline

- **Commit**: under (1)–(3). Always show files + diff summary.
- **Push**: ALWAYS requires separate explicit "push", even when commits pre-authorized.
- **Force-push, git config, destructive history**: only on direct specific request.
- **PRs**: always require explicit "open PR" / "create PR".

Pre-auth scope: narrow when in doubt — ask rather than assume.

#### Environment protection

**NEVER** modify staging or production SFCC instances (BM config, site preferences, code deploy, cartridge activation) or MRT environments (runtime config, env variables, deployments) without explicit authorization. Dev sandboxes are fair game for routine work; staging/production require a direct "deploy to staging" or equivalent command.

### Stuck-Loop Reflex

*Stop and micro-reflect when hitting the same wall repeatedly — don't keep trying variations*

**STOP** when you hit any of these:

- ≥3 attempts at the same fix without new information
- ≥2 retries of the same tool call producing the same error
- User repeated the same correction twice
- >15 tool calls on one sub-task without progress
- User questions a value from estimate/intuition — no debate, ask for the authoritative source

#### What to do

1. **Stop.** No more "let me try X".
2. **Micro-reflect**: What fact am I missing? Which rule/skill would have prevented this?
3. **Tell the user:** "I'm stuck on X. Root cause seems Y. Tried {list}. Options: {2–3}. Which?"
4. Wait for response before proceeding.

For deep investigations: use the `systematic_debugging` skill.

## Workflow

Rules for how work is planned, executed, reviewed, and verified.

### Bug Fix Planning

*MUST read when working on any Jira bug ticket (HC-*). Defines reproduce-first workflow, Jira reporting protocol, and fix verification sequence.*

#### Approach: Reproduce-First

**Reproduce first.** The bug might already be fixed or env-specific. Don't spend time on root cause analysis until you confirm the symptom exists.

1. **Read the ticket** — note which env, which version, when reported
2. **Reproduce + quick code check in parallel** (both are read-only — no user approval needed per the Autonomous Verification rule):
   - **(a) Reproduce:** Reproduce the symptom the same way QA saw it — Playwright browser on the reported env (or MRT preview / localhost with matching SFCC_ENV). QA reports what they see in a browser → verify in a browser.
   - **(b) Quick history check:** Lightweight — only `git log` for related fixes since report date, check if relevant code changed. NOT deep analysis, NOT writing tests, NOT reading implementation details.
3. **Decision point:**
   - If **not reproduced** + code already fixed → report findings to user, propose: comment with evidence + transition to Ready for QA (per the No Unsolicited External Actions rule — never write to Jira without explicit user command)
   - If **not reproduced** + code unchanged → dig deeper: check backend state via OCAPI/SCAPI Data API (promotion enabled? campaign active? preference value? endpoint responding?)
   - If **reproduced** → proceed to root cause analysis using the `systematic_debugging` methodology, then fix
4. **Assess full scope** — search for ALL occurrences of the same problem
5. **Trace the full lifecycle** — for async/state bugs, map out the complete state transition timeline (SSR → hydration → effects → data fetches → re-renders) **before** proposing a fix. Identify exactly which transition causes the bug, then fix at that layer — not at the consumer.
   - **When the symptom is visual, also diff the rendered subtree identity before/after the trigger, not just the visible pixels.** If the subtree swapped (e.g. component A unmounted and component B mounted in its place), the bug is at the layer driving that branch — not inside the components themselves. Polishing consumer-level rendering of a wrong subtree is a symptom patch.
   - **Anti-pattern:** taking the literal words of the bug title (e.g. "broken images", "wrong colour", "missing button") as the bug *layer*. Those describe the symptom's last visible step; the bug usually lives one or more state transitions upstream.
6. **Think architecturally** — before changing 12 consumer files, check if the fix can be applied at the infrastructure level (1-3 files)
7. **Draft a plan** — with specific files, line numbers, and code snippets
8. **Then code** — never write code before the plan is approved

##### SFCC Backend Issues — Diagnosis Priority

When the bug is **not reproduced** in the browser and you need to understand why, check backend state in this order:

1. **OCAPI Data API** — direct fact check (promotion exists? enabled? campaign active? preference value?)
2. **SCAPI direct call** — endpoint responds? what data does it return for a real basket?
3. **Logs / WebDAV** — server-side errors, script exceptions

This is for **diagnosis**, not reproduction. Reproduction = browser (same as QA).

#### Infrastructure-Level Fix Rule

If the bug manifests in 5+ consumer files, always try to fix at the level of:
- Utilities / helpers (paths.ts, url.ts)
- Navigation hooks (useNavigation)
- Base components (Link, Button)

Goal: **zero changes to consumers**. Consumers already call the correct API (`navigate(ROUTES.X)`, `<Link to={ROUTES.X}>`) — the problem is the API returning the wrong result.

#### Parallelism

Always batch work for parallel execution:

- **Batch 1**: foundational changes (everything else depends on these)
- **Batch 2**: independent changes on top of the foundation (worked in parallel)
- **Batch 3**: cleanup / refactor (parallel)
- **Batch 4**: type-check + lint (parallel)
- **Batch 5**: dev server + Playwright tests

#### Verification (mandatory sequence)

1. `tsc --noEmit` — types are not broken
2. `eslint` — lint passes
3. Local dev server
4. Playwright tests (specific scenarios from the ticket)
5. If tests fail: fix → repeat steps 1-4

#### After Fix: Process Retrospective

If the bug's root cause was a missed step, undocumented pattern, or silent failure — run a brief root cause analysis per the Process Retrospective rule below:

1. Was this a one-off mistake or a repeatable pattern?
2. If repeatable — what rule, skill, or schema update would prevent it next time?
3. Propose and implement the structural fix (don't just note it).

#### Anti-patterns

- Writing code before the plan
- Fixing a single occurrence instead of analyzing full scope
- Proposing a mapping utility when an infrastructure-level fix is possible
- Forgetting type-check in verification
- Running everything sequentially when parallelism is possible
- Fixing the code without analyzing why the process allowed the bug

### Documentation Lifecycle (FSD / ISD)

*Confluence is canonical for FSDs/ISDs — pull before editing, publish only with explicit user approval, keep statuses accurate*

Client-facing Functional Specification Documents (FSDs) and Integration Specification
Documents (ISDs) live in Confluence, managed via the `atlassian` skill. Rules:

- **The published Confluence page is canonical.** Before editing an existing FSD/ISD, pull
  the live page first and work from that — never from a stale local copy or from memory.
- **Edit only the touched sections** and preserve the full document body, so an update
  replaces the page in place without dropping existing sections.
- **Publishing is an explicit approval-gate action.** Never create or update a Confluence
  page without previewing the exact operation and getting an explicit yes from the user.
- **Keep statuses accurate.** FSD status runs `DRAFT -> ON REVIEW -> APPROVED`; a document
  goes to `ON REVIEW` when submitted for team/client review. `APPROVED` reflects client
  sign-off and is set manually in Confluence — tooling never sets it. Do not conflate
  submitting for review with client approval of the document.
- Page title convention: `<WBS> (PhaseN) - <Feature> FSD` / `... ISD`.
- Surface any publish warnings (e.g. inline comments that could not be re-anchored) to the
  user rather than swallowing them.

### FE Definition of Done — Self-Check

*FE self-check before requesting code review — feature completeness, edge cases, a11y, E2E, design, 3PI*

Run this checklist **before** requesting code review or marking FE work as complete. Every item must be PASS, N/A (with justification), or SKIP (with explicit reason logged).

#### When to apply

- All FE changes in the Storefront Next app (`src/` — routes, components, providers, hooks)
- Triggered: after implementation, before PR creation or code-review request

#### Checklist

##### 1. AC Coverage

Map every Jira acceptance criterion to file + location. If an AC has no corresponding code change — explain why (deferred, out of scope, covered by existing code).

##### 2. Full Flow Walkthrough

Trace the complete user journey, not isolated ACs:

- Can the user accomplish the goal **end-to-end** without hitting a dead end?
- Are transitions between states handled? (e.g., add-to-cart -> mini-cart update -> cart page reflects change)
- If multi-step flow: does back/forward navigation work?

##### 3. UI States

For each new/modified component, verify these states are explicitly handled:

| State | Requirement |
|-------|-------------|
| `loading` | Skeleton or spinner shown, NOT blank screen |
| `empty` | Explicit empty state UI, NOT hidden/collapsed |
| `error` | User-visible feedback (toast/inline), NOT just console |
| `populated` | Normal render |
| `SSR` | No loading-only tree that skips server-side data loading — the route `loader` must resolve during SSR |

For mutations: error path must surface user-visible feedback.

Reference: the FE Edge Cases Checklist rule below.

##### 4. A11y

- ESLint jsx-a11y passes with **zero new suppressions**
- New interactive elements have: aria-label/role, keyboard handler, focus management
- If `@axe-core/playwright` is available: run on new/modified pages

##### 5. E2E Tests

- Playwright spec exists in `playwright/tests/` for the modified user flow
- If new page/route: write at least smoke-level spec
- If existing flow modified: verify existing spec still covers the change
- `--list` output is NOT acceptable as proof — tests must actually run

##### 6. Design Check

- Compare implementation with the committed design assets under the change `assets/` folder
- Verify: spacing, colors, typography, border-radius, breakpoint behavior
- Document intentional deviations: "Design deviation: [reason]"

##### 7. 3PI Completeness (if applicable)

If third-party integration touched:

- Consent: OneTrust category assigned, withdrawal cleans up
- Error handling: timeout + retry, user-visible degraded state
- Locale: passed per vendor spec, fallback when unsupported

##### 8. SSR

For new pages/routes: `curl` or view-source confirms content in initial HTML.

##### 9. i18n

If strings changed: no hard-coded copy; translation keys resolve for every supported locale.

##### 10. Quality Gate

```bash
pnpm lint
pnpm typecheck
```

Run in all touched workspaces.

#### Output Format

After completing the checklist, produce an evidence block:

```
##### DoD Evidence

| # | Check | Status | Evidence |
|---|-------|--------|----------|
| 1 | AC coverage | PASS | 4/4 ACs mapped — see Requirements Coverage |
| 2 | Full flow | PASS | Traced: PLP → PDP → Add to cart → Mini-cart → Cart page |
| 3 | UI states | PASS | Loading skeleton + error toast + empty state in ProductGrid |
| 4 | A11y | PASS | jsx-a11y clean, keyboard nav verified for modal |
| 5 | E2E tests | PASS | playwright/tests/pdp/add-to-cart.spec.ts — 3 scenarios |
| 6 | Design | PASS | Matches assets/pdp-add-to-cart.png, spacing verified |
| 7 | 3PI | N/A | No 3PI changes |
| 8 | SSR | PASS | curl confirms product data in HTML |
| 9 | i18n | N/A | No string changes |
| 10 | Quality gate | PASS | lint + tsc clean |
```

#### Escape Hatch

If an item genuinely cannot be completed (e.g., design asset unavailable, axe-core not installed):

- Mark as `SKIP: [specific reason]`
- The `code_review` skill will flag skipped items for reviewer attention
- Do NOT mark as PASS without actually verifying

### FE Edge Cases Checklist

*Reference checklist for systematic edge case and UI state coverage during FE code review*

Reference rule for systematic edge case coverage. Used by the FE Definition of Done rule above and the `code_review` skill.

#### When to apply

When implementing or reviewing any FE component that fetches data, handles user input, or renders dynamic content.

#### Required States Per Component

Every data-driven component must explicitly handle all applicable states:

| State | What to render | Anti-pattern |
|-------|---------------|--------------|
| `undefined` (initial) | Nothing or parent skeleton | Rendering empty container |
| `loading` | Skeleton matching layout shape | Blank screen, spinner blocking entire page |
| `empty` | Dedicated empty state with guidance | Hiding component, showing nothing |
| `error` | User-visible message + retry action | `console.error` only, silent swallow |
| `populated` | Normal render | N/A |

##### SSR-specific

- Loading-only trees that skip the route `loader` break SSR — data must load server-side, not only after hydration
- Server must render meaningful content (not just skeleton) for SEO-critical pages
- Check: `curl <url> | grep <expected-content>` returns data

#### Good Patterns

<!-- Paths below are illustrative for a Storefront Next app (`src/`); adapt to the project's actual tree. -->

**PLP — product grid + empty state:**
- `src/components/product-grid/product-grid.tsx` — renders a skeleton (via `<Suspense>` fallback) while the loader data streams in
- `src/components/product-grid/empty-product-list.tsx` — dedicated empty-state component with user guidance, not a hidden/collapsed grid

**Content pages — page skeleton:**
- `src/components/page-skeleton/page-skeleton.tsx` — reusable page-level skeleton
- Pattern: resolve loader data server-side; on empty result render an explicit not-found/empty state, never an indefinite spinner

**Image gallery — skeleton variant:**
- `src/components/image-gallery/image-gallery-skeleton.tsx` — component-specific skeleton matching the final layout to avoid shift

#### Anti-Patterns to Flag

**Mini-cart:** Only header icon while loading — should show drawer skeleton to prevent layout shift

**Silent error swallowing:**
```typescript
// BAD
try { await addToCart(item) } catch (e) { console.error(e) }

// GOOD
try { await addToCart(item) } catch (e) {
  showToast({ type: 'error', message: formatMessage(messages.addToCartError) })
}
```

**Hidden empty state:**
```typescript
// BAD — user sees nothing
if (!items.length) return null

// GOOD — user gets guidance
if (!items.length) return <EmptyState message={...} action={...} />
```

#### Boundary Conditions

##### Numeric boundaries

| Context | Test values | Why |
|---------|-------------|-----|
| Quantity selector | 0, 1, max stock | Prevent negative, zero-add, over-stock |
| Pagination | page=0, page=1, page=last, page=last+1 | Off-by-one, empty last page |
| Search results | 0 results, 1 result, 1000+ results | Empty state, singular copy, performance |
| Cart items | 0, 1, max (99) | Empty cart, singular, overflow |
| Promo codes | empty string, valid, expired, max-length | Validation, error messages |

##### String boundaries

| Context | Test values |
|---------|-------------|
| User input | Empty, whitespace-only, max-length, special chars (`<script>`), unicode |
| Product names | Very long (overflow?), with HTML entities, with quotes |
| URLs | Missing slug, encoded chars, trailing slash |

##### Viewport/responsive

- If component has conditional rendering by breakpoint — test BOTH paths
- Mobile-specific: touch interactions, virtual keyboard pushing content
- Test at exact breakpoints (not just "mobile" and "desktop"): 390px, 768px, 1024px, 1280px

##### Concurrent/async

- Rapid clicks: Is the action debounced/disabled after first click?
- Unmount during fetch: Is the request aborted? Does setState on unmounted component occur?
- Stale closure: Does the callback reference current state?
- Optimistic update: Is it rolled back on server error?

##### Idempotency

- Can the user safely repeat the action? (double-submit prevention)
- Does refresh/back-nav cause unintended re-execution?

#### Per-Component-Type Checklist

##### Forms
- [ ] All inputs have validation (client + server-side)
- [ ] Submit disabled while processing
- [ ] Error messages per field + summary
- [ ] Server error shown (not just network error)
- [ ] Form state preserved on validation failure

##### Modals/Drawers
- [ ] Focus trapped inside
- [ ] Escape key closes
- [ ] Background scroll locked
- [ ] Content scrollable if overflow
- [ ] Close button accessible

##### Lists/Grids
- [ ] Empty state
- [ ] Loading skeleton matches grid layout
- [ ] Pagination/infinite scroll boundary
- [ ] Single item renders correctly (no plural bugs)
- [ ] Key prop uses stable ID (not array index)

##### Data fetching hooks
- [ ] Loading state returned
- [ ] Error state returned and typed
- [ ] Abort on unmount
- [ ] Cache invalidation after mutation
- [ ] Retry mechanism for transient failures

### No Symptom Patches

*When the right fix is architectural, do it — do not patch symptoms and call it pragmatism*

If root cause is **architectural** (wrong state owner, broken abstraction, leaking concerns), **fix it properly**. A symptom patch is acceptable only when you name a *concrete* cost exceeding the refactor.

#### Decision rule

1. State why it's wrong (1-2 sentences).
2. Describe what right looks like (1-2 sentences).
3. **Default: refactor now.**
4. Propose patch only with a real cost — and let the user decide.

Real costs: "touches auth client used by every page, no integration tests", "needs SCAPI schema change deployed first", "user said deadline is X". Generic "risk"/"scope"/"time" are NOT real costs.

#### Self-check

Before saying "let's leave it": Is the cost specific or generic? Would I argue this to a peer reviewer? Did the user ask to defer, or am I deferring without permission? If any answer is wrong — refactor.

### Process Retrospective

*Triggers process-level root cause analysis when discovering bugs, gaps, undocumented patterns, or silent failures. Applies during bug fixes, feature implementation, code reviews, and gap analyses. If something was harder than expected — analyze why and fix the process.*

When you discover a bug, a missing integration, a silent failure, or any gap between what was specified and what was implemented — **don't just fix the code**. Analyze why the process allowed this to happen and propose a structural fix.

#### When to Trigger

- **Bug fix**: root cause is a missed step, not a logic error
- **Feature implementation**: you hit an undocumented pattern or implicit step that wasn't in the task list
- **Code review**: finding that a feature was partially wired or a convention wasn't followed
- **Gap analysis**: comparing spec vs implementation and finding mismatches
- **Silent failures**: something doesn't work but produces no error
- **Repeated friction**: you had to figure out the same thing that a previous session already solved

#### 5 Whys — Quick Format

After identifying the code fix, run a brief root cause chain:

1. **What happened?** — the symptom (e.g., "PA carousel doesn't render")
2. **Why?** — the immediate cause (e.g., "not in COMPONENT_MAP")
3. **Why was it missed?** — the process gap (e.g., "no task in the plan for this step")
4. **Why wasn't there a task?** — the systemic cause (e.g., "the planning step didn't know about the Amplience pipeline")
5. **What would prevent this?** — the structural fix (e.g., "a `dev_rules` entry documenting the pipeline + a task-template step referencing it")

Stop at the level where a **concrete, automatable fix** exists. Not every chain goes to 5.

**Critical:** The root cause is systemic, not technical. "SDK strips fields" is a technical fact. "No documented integration contract between frontend and backend for this feature" is the systemic cause that explains why the developer used the SDK wrong, why the test couldn't detect it, and why the reviewer didn't catch it. Find the one missing thing that would have prevented multiple symptoms.

#### Structural Fix Categories

In order of preference (most durable first):

| Fix Type | Where | Prevents | Example |
| --- | --- | --- | --- |
| Rule update | `dev_rules` | Agent doesn't know about a pattern | Pipeline registration checklist |
| Skill update | the relevant skill | Review/QA misses a check | Code review integration completeness |
| Lint rule / CI check | `eslint` / `tsconfig` / CI | Mistake reaches PR without detection | Custom ESLint rule for orphan schemas |
| Documentation | `docs/` | Knowledge isn't preserved | Architecture decision record |

#### Output Format

When presenting a fix, always include both:

```markdown
###### Code Fix
[what to change in the codebase]

###### Process Fix
**Root cause chain:** [1-3 sentence chain from symptom to systemic cause]
**Structural fix:** [what rule/skill/schema to create or update]
**Prevents:** [class of similar issues this would catch]
```

#### Scope

- Focus on **repeatable patterns**, not one-off mistakes
- Only propose process changes when the same class of error could reasonably happen again
- Keep changes minimal — one targeted rule beats a sprawling checklist
- Don't propose process fixes for typos, simple logic errors, or well-understood patterns that were just accidentally missed once

#### Anti-Patterns

- Proposing a process fix without implementing it (just do it)
- Creating a rule so broad it triggers on every file (noise → ignored)
- Adding a 50-item checklist instead of a focused 3-5 item pipeline
- Blaming the agent instead of fixing the instruction that was missing

### Pull Request Conventions

*Pull request conventions and best practices for the storefront*

Generic PR discipline for the storefront. Project-specific values — the VCS host, the shared
integration (base) branch, the Jira project key and browse URL, the quality-gate commands — come
from the project's own docs and scripts. Use those as the source of truth; the structure below is
stack-agnostic.

#### Goal

Create pull requests that:

- Follow commit message conventions in the PR title.
- Are clearly linked with issue-tracker tickets.
- Provide comprehensive context for reviewers.
- Pass all quality checks (linting, TypeScript, tests).
- Enable an efficient code review process.
- Can be used for automatic changelog generation (title mirrors the squash-merge commit).

#### PR Title Format

Follow the commit message format from the `git_commit` skill:
`SCOPE:TYPE <TICKET> Description`. When the merge strategy is squash + changelog generation, the PR
title becomes the changelog entry — keep it commitlint-valid.

#### PR Description Template

Include the following sections in PR descriptions:

1. **Summary:** Start with the ticket link (e.g. `**Jira:** [PROJ-XXX](<browse-url>/PROJ-XXX)`),
   then a brief overview of changes.
2. **Changes Made:** Key changes + components/files affected.
3. **Testing:** How to test the changes, and test scenarios covered.
4. **Blocked Tests (fixme):** If any E2E tests are `test.fixme()`, list blockers and owners.
5. **Screenshots:** If UI changes are involved.
6. **Breaking Changes:** If any (clearly marked).

#### Linking with the issue tracker

- **Always link PRs with the corresponding ticket.**
- Use the ticket key in the PR title or description for automatic linking.
- Update the ticket status when the PR is ready (e.g. move to "On Review").

#### Review Process

1. **Self-review:** Review your own PR before requesting reviews.
2. **Request reviewers:** Assign appropriate team members.
3. **Address feedback:** Respond to comments and make requested changes.
4. **Update status:** Move the ticket to "On Review" when the PR is ready.
5. **QA:** Move to "Ready for QA" after approval.

#### Code Quality Requirements

Before creating a PR, ensure:

- All quality gates pass (build, lint, TypeScript, tests) via the project's PR-validation script.
- Commit messages follow conventions.
- No `console.log`s or debug code left.

#### Pre-PR Process Changes Audit

**Mandatory check** before opening any feature PR. Feature PRs should be reviewable for feature
intent; mixed AI-tooling changes either bloat the review or get skimmed. Extract them so the chore PR
merges fast and the team gets improved rules immediately.

Let `<integration>` be the project's shared integration branch. Run:

```bash
git diff --name-only $(git merge-base origin/<integration> HEAD)..HEAD | grep -E '^\.claude/|^services/skills/' || echo "clean"
```

**If output is `clean`** — proceed to open the feature PR normally.

**If output lists files** — the branch mixes feature code with AI tooling changes (rules / skills /
agents / hooks). Extract them into a separate chore PR first:

1. Identify the process-only commits (they should already use a `MISC:CHORE`-style prefix per
   the Coding Standards commit discipline).
2. Create a chore branch from the integration branch and cherry-pick the process commits:
    ```bash
    git checkout -b chore/agent-tooling-YYYYMMDD origin/<integration> --no-track
    git cherry-pick <sha-of-process-commit-1> <sha-of-process-commit-2> ...
    git push -u origin chore/agent-tooling-YYYYMMDD
    # open the chore PR against the integration branch with a MISC:CHORE-style title
    ```
3. Go back to the feature branch and open the feature PR. In the **Summary** section add a dependency
   note:
    ```markdown
    **Depends on:** #<chore-PR-number> (agent tooling/config process improvements).
    After the chore PR merges, this branch will be rebased on the integration branch — duplicate
    commits will drop automatically and the feature diff will be clean.
    ```
4. If the user explicitly declines extraction ("keep everything in one PR"), open the feature PR
   as-is and add to the PR description:
    ```markdown
    **Process improvements included:** This PR also touches agent tooling/config files (list them).
    Reviewers can skip these for code-review focus; they are scoped to AI tooling.
    ```

#### Examples

##### Good PR Title

```sh
#### Single commit PR - use the commit message as title
PLP:FEAT PROJ-1665 Restyled quick buy on PLP

#### Multiple commits PR - summarize changes
SS:FEAT PROJ-2001, PROJ-2002 Implemented search integration and query optimization

#### Fix with proper scope
SCS:FIX PROJ-123 Fixed issue with product bundle adding to the cart
```

##### Good PR Description

```markdown
##### Summary

**Jira:** [PROJ-2001](<browse-url>/PROJ-2001), [PROJ-2002](<browse-url>/PROJ-2002)

Implemented search integration for product search functionality with query optimization and caching.

##### Changes Made

- Added the search client adapter
- Implemented the search query builder with filters
- Added a caching layer for search results
- Added TypeScript types for search results

##### Testing

1. Navigate to the product list page
2. Enter a search query
3. Verify results are returned
4. Test filters (category, price range)
5. Verify caching works (check the network tab on the second search)

**Test scenarios:**

- Basic search query
- Search with filters
- Empty results handling
- Error handling (network failure)
- Cache invalidation

##### Blocked Tests (fixme)

<!-- Include this section only if any E2E tests are marked test.fixme() -->

| Test ID | Blocker | Owner | Required Action |
|---|---|---|---|
| LLT-005 | Missing non-VIP test account | @QA | Create account on sandbox |
| LLT-RGS-025 | CMS content not published | @FE | Publish the required content |

##### Screenshots

[Add screenshots if UI changes]

##### Breaking Changes

None
```

##### Bad PR Title

See bad examples in the `git_commit` skill.

#### Temp Files

When generating PR bodies, commit messages, or other disposable files, write them to a temp
directory (`/tmp/`). Clean up after use.

```bash
#### PR body
/tmp/pr-body.md

#### Commit message
/tmp/commit-msg.txt
```

**Never** write temp files to the repo root.

### Quality Gate

*Quality gate — lint and TypeScript compilation checks are mandatory after code changes*

After modifying TypeScript/React code, you **MUST** run lint and type checks before considering work done. This is not optional — unfixed lint or type errors mean the task is incomplete.

#### Commands

**Storefront Next app** (always, if `.ts`/`.tsx` files were changed in the storefront):
```bash
pnpm lint && pnpm typecheck
```

**sfcc** (only if files under `sfcc/` were changed):
```bash
cd sfcc && npm run lint
```

**playwright** (only if test files were changed):
```bash
cd playwright && npm run lint
```

#### Rules

- Run these AFTER implementation, BEFORE marking tasks complete or presenting results.
- Fix ALL errors. Auto-fixable lint issues: `pnpm lint --fix` in the Storefront Next app.
- IDE diagnostics are a supplement, not a replacement — always run the full CLI commands.
- If a lint/type error is pre-existing and unrelated to your changes, note it but do not block on it.

### Verify SFCC Deploy State Before Debugging the Running System

*After editing SFCC code or metadata, verify the change actually reached the target sandbox BEFORE debugging running-system symptoms. Triggers on b2c-related work or when a user reports an SFCC-side bug after a change.*

When you edit anything under `sfcc/cartridges/**`, `sfcc/meta/**`, or `sfcc/meta_delta/**`, the target sandbox is **not** automatically in sync. Local edits are invisible to a running Storefront Next / SCAPI / SFRA (hybrid) session until you actually push them. Skipping this verification turns every subsequent log tail and basket inspection into wasted time chasing an outdated runtime.

#### Mandatory pre-flight before debugging any SFCC-side symptom

Before tailing logs, inspecting basket JSON, hitting OCAPI, or asking "why doesn't the promo apply", run **all** of the following:

1. `git status` from the repo root — uncommitted changes under `sfcc/**` are almost certainly NOT deployed. Note them explicitly.
2. **Cartridge code (`sfcc/cartridges/**`)**: deploy via `b2c code:deploy` (see the `b2c_code` skill). After deploy, verify the new code version is **active** on the instance.
3. **Metadata XML (`sfcc/meta_delta/**`, `sfcc/meta/**`)**: import via `b2c job import` (see the `b2c_metadata` skill). For targeted single-attribute pushes, build a minimal site-archive under `/tmp/<name>/meta/` containing only the affected `system-objecttype-extensions.xml` block — see worked example at the bottom of this rule.
4. **Site preferences (`sfcc/meta_sandbox/site_template/sites/<SiteID>/preferences.xml`)**: import the same way. Note that the **definition** lives in `meta_delta`, the **value** in `meta_sandbox/site_template/`. Both must reach the instance. Reference: the SFCC Metadata File Locations rule below.

Only skip the verify-then-deploy step when the user **explicitly** confirms current state — e.g. "cartridge is on the instance", "deployed already", "skip the deploy". Even then, spot-check before debugging:

- `b2c webdav ls` on the cartridge to compare timestamps.
- `b2c logs get --since 5m --search "<new-symbol>"` to confirm the new code path is hit.

#### Deployment-induced silent failures to watch for

These are the failure modes that look like "feature is broken" but are really "deploy is incomplete":

| Symptom | Likely cause | Verification |
| --- | --- | --- |
| Promotion never applies | New customer-group rule / new product attr not deployed | Import `meta_delta` |
| `Unknown dynamic property '<attr>'` in customerror logs | New custom attribute referenced by code, not deployed | Import the attribute via minimal site-archive |
| Hook code edited but old behavior persists | Old code version still active | `b2c code:deploy` then activate |
| Site preference returns `null` and `if (pref) { ... }` short-circuits | Preference defined but value never imported | Import `preferences.xml` |
| New SCAPI custom-API returns 404 | `api.json` registered locally, endpoint not deployed | `b2c code:deploy` + check registration via `b2c scapi custom status` |

#### Frustration markers from the user

Any of the following user-input signals right after an SFCC edit is a **strong indicator** the agent skipped this rule. Treat them as a hard interrupt — stop debugging, run the deploy verification first.

- Imperatives like "deploy it", "where is it", "push it already"
- Profanity or impatience tokens together with deploy/cartridge/SFCC keywords
- The user re-asserting that the change "should be on the instance" / "is already deployed" when the log evidence does not support it

#### Worked example: minimal targeted metadata push

When a single new custom attribute is needed on an existing instance, do **not** import the full `meta_delta` archive (it overwrites unrelated attributes). Build a minimal one:

```bash
mkdir -p /tmp/attr-deploy/meta
cat > /tmp/attr-deploy/meta/system-objecttype-extensions.xml <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<metadata xmlns="http://www.demandware.com/xml/impex/metadata/2006-10-31">
    <type-extension type-id="ProductLineItem">
        <custom-attribute-definitions>
            <attribute-definition attribute-id="machinePLIUUID">
                <display-name xml:lang="x-default">Machine PLI UUID</display-name>
                <type>string</type>
                <mandatory-flag>false</mandatory-flag>
            </attribute-definition>
        </custom-attribute-definitions>
    </type-extension>
</metadata>
EOF

b2c job import /tmp/attr-deploy --show-log --config sfcc/dw.json
```

After import, the `Unknown dynamic property '<attr>'` errors stop on the next basket calculate. Confirm with `b2c logs get --since 5m --level ERROR --search "<attr>"`.

#### Cross-references

- The SFCC Metadata File Locations rule below — where to put metadata in the repo
- The `b2c_config` skill — `b2c` CLI auth, `dw.json` resolution
- The Autonomous Verification & Action rule above — verify environment state yourself; don't ask the user

### Verification Standards

*Standards for task verification — browser testing is mandatory, no code-review-only verification*

When implementing changes or marking verification tasks complete, follow these rules.

#### New Code Paths Require New Tests

When writing new logic (not just refactoring), tests for that logic are part of the deliverable — not a follow-up.

- **"Existing tests pass"** is regression coverage, not verification of new behavior. Both are required.
- Delegated code changes **must** include tests in the same task. A prompt that says "implement X" without "write tests for X" is incomplete.
- If the codebase has no test infrastructure for a layer (e.g. SFCC cartridge scripts), document manual verification steps instead: what to run, what to check, expected output.
- **Never claim changes are "tested"** based solely on existing test suite passing. State explicitly what new paths are covered and what isn't.

#### Browser Verification is MANDATORY

- **Never** mark visual verification or cross-browser tasks as done based on code review alone.
- Use the `playwright_cli` skill to navigate and verify.
- Ensure a target is available first: if no dev server is running, start one (`pnpm dev`, Vite default `:5173`) or pass a `--base-url=<MRT-URL>`. "Requires running dev server" is not an excuse — start it.
- For visual accuracy (measurements, colours, spacing, typography, component states), use the committed design assets under the change `assets/` folder — do not rely on FSD screenshots, which are outdated in the majority of cases.

> **Browser-auth caveat (playwright-cli).** Embedded credentials in the URL
> (`http://user:pass@host`) cause `ERR_INVALID_AUTH_CREDENTIALS` in headless Chrome. Set them on the
> context instead, and use `domcontentloaded` (not `networkidle`) — the dev server has persistent
> background requests that never let `networkidle` resolve:
>
> ```js
> await page.context().setHTTPCredentials({ username: '<user>', password: '<password>' });
> await page.goto('<url>', { waitUntil: 'domcontentloaded' });
> ```

#### No "Manual QA Needed" Cop-Out

Agents have the tools. Use them:

- **`playwright_cli`** — register users, log in, fill forms, take screenshots
- **b2c CLI** — WebDAV, OCAPI, code deploy, site config
- **Test credentials** — from goldenData, env vars, or registration flow

For authenticated views: register a test user via the registration page or SCAPI, then log in via `playwright_cli`. Do not skip with "requires authenticated session".

#### SFCC Deployment Blockers

If WebDAV 401 or OCAPI access fails:

1. Use the **`b2c_config`** skill to verify credentials and instance
2. Use the **`b2c_webdav`** skill to diagnose WebDAV permissions
3. Use the **`b2c_code`** skill for code deploy alternatives
4. Check BM permissions only when CLI confirms the issue — do not assume "needs BM config" without trying

#### Task Completion Honesty

- `[x]` only when acceptance criteria are **fully** met
- Partially done = `[ ]` with a note: what's done, what remains
- Do not mark verification tasks complete with caveats like "code-reviewed, visual check pending"

##### Partial completion = split into subtasks

If a task is partially done, **NEVER** mark it `[x]`. Instead split it into subtasks and mark only the truly completed ones. Example:

```
Bad:  - [x] 5.4 Accessibility audit — **VERIFIED via code review**: aria-live, labels...
Good: - [x] 5.4a Accessibility code patterns implemented — aria-live, labels, autoComplete...
      - [ ] 5.4b Manual keyboard navigation audit
      - [ ] 5.4c Screen reader testing (VoiceOver/NVDA)
```

##### "Code review" is not verification

If a task says "verify", "test", or "audit", code review alone is **not sufficient**. A real action is required: running tests, browser check, curl to SSR endpoint. If only code review was performed, split the task:
- `[x]` code patterns implemented
- `[ ]` real verification (run/test/check)

##### Test stubs are not passing tests

`test.skip()` stubs do **not** count as "tests pass". Stubs and real test execution are separate subtasks:
- `[x]` E2E test stubs created (N scenarios with `test.skip`)
- `[ ]` E2E tests implemented and passing (remove `test.skip`, write assertions)

##### Content tasks must include CMS configuration

When a feature requires CMS content (Amplience, SFCC content assets), always split into:
1. **Code integration** — delivery keys, API hooks, fallback values (`[x]` when code is merged)
2. **CMS configuration** — create/configure content items in Amplience/BM (`[ ]` until actually configured)

Both are frontend developer responsibility. Do not mark content tasks done when only code-side is complete.

Additionally, always include a **CMS content verification** task: open pages in browser and confirm CMS content renders instead of hardcoded fallback values. Content configuration without verification is not done.

##### Cross-browser during dev = Chrome only

During development, verify in **Chrome/Chromium only**. Do not claim Firefox/Safari verification unless actually tested. Cross-browser testing (Safari, Firefox, mobile browsers) is QA phase responsibility.

##### SSR requires a real server test

Reviewing code for SSR-safe patterns (`useEffect`-only redirects, no `window`/`document` access at module scope, client-only code behind lazy/`Suspense` boundaries or `.client.ts` modules) is necessary but not sufficient. To mark SSR verification done, start the local SSR server and verify the page renders an HTML shell via `curl` or `view-source:`.

### Delegation Discipline

*Discipline for delegating work — one methodology per delegated task, explicit artifact contract, no silent downgrade, cross-verify factual claims*

When work is delegated to another agent, the delegation prompt must follow these rules.
Sloppy delegation is the main source of "agent said PASS but didn't actually do the thing"
failures.

- **One methodology per delegated task.** Each delegated task targets one
  verification/execution method (e.g. one curl-only batch, one browser batch). Mixing
  methods in one prompt leaves the contract unclear and the executing agent picks the
  easiest. If a ticket needs both HTTP and browser checks, split it into two tasks.
- **Required artifacts contract.** Every delegation prompt lists the concrete output files
  the task must produce (screenshots, console logs, a report with Method/Verdict/Artifacts
  fields). No artifact list → no way to audit the claim. Missing artifacts in the return =
  automatic `INCONCLUSIVE`, not `PASS`.
- **No silent methodology downgrade.** If the prompted methodology cannot run (tool
  unavailable, server down, credentials missing), the delegated task stops and reports the
  blocker explicitly (e.g. `Verdict: BROWSER_UNAVAILABLE`) — it never switches to a weaker
  method (curl, code inspection, "looks fine from the source") and still claims success.
  The delegator retries with a different method in a new, explicit delegation. "Playwright
  if possible, otherwise curl" is the silent-downgrade escape hatch — do not write it.
- **Cross-verify external-system claims.** A delegated report describing the state of an
  external system (sandbox config, BM site preferences, deployed code version, third-party
  API state) is a claim, not a fact. Before relaying it as a conclusion, verify it through
  at least one independent channel that does not share the same network/auth/cache path.
  For SFCC, independent channels include the BM UI, the OCAPI Data API, the `b2c` CLI, and
  WebDAV — the storefront's own SCAPI path is not independent of the storefront. If
  verification is impossible, downgrade the claim to `UNVERIFIED` and say so.
- **Honest status vocabulary.** Reserve `PASS`/`FAIL` for the prompted methodology with all
  required artifacts present; use `HTTP-VERIFIED`, `CODE-REVIEWED`, `INCONCLUSIVE`, or
  `ENV_BLOCKED` where the evidence is weaker. This prevents green-checkmark inflation.

## Architecture

Coding and commenting standards for TypeScript/React work.

### Code Comments

*Comment discipline for code — one ≤200-char /** */ file header, a single ≤100-char // intent line above public API, single-line // notes for non-obvious logic only, TODO for hacks. Bans multi-line inline blocks, JSDoc prose, change-narrative and code-restating comments. Good comments boost both human and LLM effectiveness.*

Comments are load-bearing context — for humans AND for the LLM working in this repo. Write the useful
ones; never emit the noisy ones. Types and clear names carry the *what*; comments carry the *why* and
the *intent* the code cannot express. Be terse: the limits below are hard, not suggestions.

#### Hard limits

- **File header:** exactly one `/** … */` block at the top, **≤ 200 characters** total.
- **Everything else:** a single `//` line, **≤ 100 characters**. No multi-line `//` runs, no in-body
  `/** */` / `/* */` blocks (no JSDoc prose on functions — the one block in the file is the header).
- If a note does not fit in one ≤100-char line, the code needs a better name or a smaller function —
  fix that instead of writing a paragraph.

#### Always write these

1. **File header — one `/** */`, ≤200 chars.** What the file is and why it exists (its role / end
   purpose) — not how it works and not a change history. One block, at most two short sentences.
2. **One-line intent on public API.** Above every exported function / component / hook / class, a single
   `//` line (≤100 chars): what it is for, plus any non-obvious contract. Skip it only when the name and
   signature already say everything. Never a `/** */` block here.
3. **Inline notes for non-obvious logic only.** Race guards, ordering contracts, workarounds for a
   platform / Rhino / SFCC quirk (link the cause), regulatory constraints, surprising edge cases. One
   `//` line directly above the code.
4. **TODO for anything that looks like a hack.** A hardcoded value pending config, a temporary shim, a
   workaround — leave `// TODO: <why + what should replace it>` (link a ticket/issue when one exists)
   rather than leaving it silent. One line.

#### Never write these

- **Multi-line comment blocks** anywhere except the file header. Split the *why* into one sharp line, or
  drop it.
- **Change narrative.** No `Phase 2 …`, `now moved …`, `reduced height by 15%`, `previously used X`.
  The change story belongs in the commit / PR, never in the code.
- **Restating the code.** No `// import X`, `// increment counter`, `// handle error`, `// set state`.
- **Banner / section-divider comments.** No `// ── Report ──`, `// helpers`, `// ---`.
- **Ticket-number prefixes** when the comment already explains the why (`PROJ-123: …`). A ticket link is
  allowed only when it adds context the comment cannot (a third-party bug, a regulatory note).
- **Commented-out code / dead code.** Delete it.

#### Style

- English, present tense; describe intent / contract, not mechanics.
- One line, always. The file header is the only place a block is allowed.
- CLI `--help` / usage text belongs in the code that prints it (an `--help` handler or `usage` string),
  not in a giant header block.

#### Carve-outs

- **Storybook** story exports: keep the JSDoc describing the variant.
- **SFCC `.js` (Rhino / checkJs):** types-only `@param` / `@returns` are expected for the runtime type
  contract — this is the one place a per-function block is allowed. Prose still follows the rules above
  (no narrative, no restating; one line each).

#### Example

```ts
/** Client hook for the store-locator search box: debounces the query, fetches nearby stores, exposes results + status. */

const RADIUS_KM = 50; // TODO: read from merchant config; hardcoded until the BM setting exists

// Runs one lookup and commits its result only if it's still the latest call.
export function useNearbyStores(query: string) {
  // Race guard: tag each call; drop the response if a newer call started meanwhile.
  const ticket = ++seq.current;
}
```

#### Self-check (re-read the diff)

- File has exactly one `/** */` header ≤200 chars; each public symbol has a single-line intent.
- No multi-line `//` runs and no in-body `/** */` blocks (outside the SFCC carve-out).
- Every inline comment is one ≤100-char line recording a non-obvious *why* — delete any that restate
  code, narrate the change, or act as a banner.
- Hacks carry a `TODO`. No commented-out code left behind.

### Coding Standards (Core)

*Core coding standards for TypeScript, React, and project conventions — applies when editing code files*

Stack: Storefront Next — React 19 + React Router 7 + RSC + Vite + Tailwind CSS 4 + SCAPI on MRT.

#### Scripting Language

- **Always Node.js.** All ad-hoc scripts, one-off automation, HTTP requests, API calls, data transforms — write in Node.js. Never Python, never bash heredocs with curl for anything beyond trivial one-liners.
- This project is a Node.js monorepo. Every developer has Node; not everyone has Python configured. Keep the toolchain uniform.

#### Linter and Formatting

- Code must pass ESLint + Prettier. See the Quality Gate rule above for mandatory CLI checks.
- No format-only refactors.

#### Comments

Comments are required context, not clutter — write the useful ones, skip the noise. Full discipline in
the Code Comments rule above. In short:

- Every file gets a **2-sentence header** (what it is + why); every exported function/component/hook gets
  a **one-line intent**.
- **Inline comments only for a non-obvious *why*** (workaround with link, ordering contract, regulatory,
  Rhino/SFCC quirk): 1 line preferred (max ~3), English, on the line above the code.
- Mark hacks with `TODO: <why>`. **Never** narrate the change (`Phase 2`, `reduced by 15%`), restate the
  code, or leave commented-out / dead code.
- **No ticket-number prefixes** (`PROJ-1234:`) when the comment already explains the *why*; add a ticket
  link inline only when it conveys context the comment cannot (a third-party bug, a regulatory note).
- JSDoc is optional, reserved for non-obvious public API; Storybook story exports keep their variant JSDoc.

#### TypeScript

- No `any` — use type narrowing, generics, or `unknown`. No `as` casting — prefer type guards. No `@ts-ignore`.
- **Let TS infer function return types — never annotate explicitly** (including `async` functions and `Promise<…>`). Exceptions: type guard predicates (`value is T`) and interface/abstract method contracts.
- Prefer interfaces over type intersections (2-5x faster type resolution).
- Use `import type` for type-only imports. Avoid barrel file imports, circular dependencies.
- **Optional members and parameters:** when a property or parameter may be absent or `undefined`, prefer `name?: T` over `name: T | undefined` (same for optional function parameters). Does not apply when the key must be present with an explicit `undefined` value. (Separate from function return annotations: infer returns; type aliases / interfaces may still use unions like `T | undefined` where needed.)

#### React Core

- Composition over inheritance. Presentational components + logic in hooks/services.
- Memoization only when measured. Predictable data flow.

#### Clean Architecture

- **Domain** (pure) → **Infrastructure** (API, state, 3rd party) → **Application** (React, hooks, routing).
- Dependencies point inward only. Domain never imports from Infrastructure or Application.

#### Credentials & Secrets

- SFCC `services.xml` files with encrypted credentials (`common.export`) MUST be committed to the repo. This is intentional — they are required for site import and are instance-portable by design. Never flag these as a security issue.
- Storefront `.env` secrets (API keys, tokens) must NOT be committed — use environment variables.

#### Error Handling

- All async code handles errors explicitly. Never swallow errors. Validate external inputs.

#### Feature flags & SFCC kill-switches

Adding a boolean site preference to "make a feature toggleable just in case" is an anti-pattern in SFCC. The default state of a not-yet-deployed preference is `null`, and code like `if (pref !== true) { return; }` then silently disables the feature with **no error, no log, no signal**. This is how kill-switches turn into outages.

Do not add a kill-switch unless **all** of the following are true:

1. There is a concrete rollback scenario more likely than "we made a bug" (which is solved by reverting the deploy, not by a flag).
2. The preference value is committed in `sites/<SiteID>/preferences.xml` with an explicit default for **every** environment the code will reach (sandbox, dev, staging, prod), AND the preference *definition* is committed alongside it. Both must be deployed alongside the code.
3. The read site uses a helper that reads the preference with an explicit **default-on** value (i.e. the value that keeps the feature working), so the default encodes "what to do if this pref is missing" as code.
4. The disabled branch emits a `Logger.warn` so absence-of-feature is observable in logs, not silent.

If you can't satisfy all four — don't add the flag. Code-level revert is simpler, leaves an audit trail in git, and can't be undermined by a metadata import order bug.

Reference incident (tracked): a `...MigrationEnabled` flag was added as a "safety" kill-switch with default-off, deployed only to the local repo (never to the sandbox), evaluated `null !== true === true` → short-circuited the entire customer-group hook → silently disabled the promotion for every affected purchase. Removed entirely once diagnosed.

#### No Unverified Claims

- Never blame a tool, model, or platform limitation without evidence (link to issue, docs, or reproduction).
- If something doesn't work as expected — investigate the root cause first (wrong config, missing frontmatter, incorrect API call, etc.).
- If you suspect a platform bug — search for a known issue and provide a link. No link = don't claim it.

#### SSR / RSC Safety

- Follow existing project conventions. Prefer existing utilities and patterns.
- SSR-safe code: no `window`/`document` access outside `useEffect` or guards. Keep browser-only logic out of Server Components / server-rendered paths.

#### Institutional Learning

When you discover an undocumented pattern, implicit convention, or a silent failure mode during **any** task:

1. Fix the immediate problem
2. Ask: "Could another agent hit this same issue?" If yes — **note the observation**.
3. Decide whether to update rules/skills **now** vs **later**:
    - **Create/update now** if: the user explicitly asked for a rule change, OR the same pattern is already observed across 2+ independent features/sessions.
    - **Note only** (don't create) if: it's the first occurrence of a pattern in a single feature context. Tell the user what you noticed and suggest it as a future rule.

You **may** update `dev_rules` and the sibling skills during feature work — they are read from the working tree, so improvements are usable immediately in the same session.

**Commit discipline:**

- Keep agent tooling/config changes in **separate commits** with the `MISC:CHORE` prefix — never mix them with feature code in the same commit.
- Before opening the feature PR, run the pre-PR audit from the Pull Request Conventions rule — it will propose extracting the tooling commits into a dedicated chore PR and linking it as a dependency.

Full methodology: the Process Retrospective rule. Skip for one-off typos or simple logic errors.

### Coding Standards (Detailed)

*Detailed coding patterns for TypeScript, React, refactoring — loaded when editing code*

#### TypeScript

##### Type System Performance (CRITICAL)

- Prefer interfaces over type intersections (2-5x faster type resolution)
- Avoid deeply nested generic types (prevents exponential instantiation cost)
- Avoid large union types (quadratic O(n²) comparison cost)
- Extract conditional types to named aliases (enables compiler caching)

##### Async Patterns

- Avoid `await` inside loops — use `Promise.all` for independent operations
- Defer `await` until value is needed
- Avoid unnecessary `async/await`

##### Type Safety

- **No `as` casting** — use type guards (`'key' in obj`), derived types (`NonNullable<ReturnType<typeof hook>['data']>`), or widen function signatures. `as` hides real type mismatches.
- **Prefer `?` optional over `| undefined`** — use `name?: T` for optional properties and parameters, not `name: T | undefined`. Exception: when the key must be present with an explicit `undefined` value.
- Use `const` assertions for literal types
- Use exhaustive checks (`never`) for union types
- Use type guards for runtime type checking

##### Lint Discipline

- **Never suppress lint rules** (`eslint-disable`) to pass a check. If a rule fires, refactor the code to satisfy it — extract functions, simplify branches, reduce complexity.
- The only acceptable suppress is for a pre-existing violation unrelated to your changes, and even then prefer fixing it.
- If you believe a rule is genuinely wrong for a specific case, explain why in the PR — don't silently disable it.

---

#### React

##### Concurrent Rendering & Waterfalls (CRITICAL)

- Use `useTransition` for non-blocking updates
- Use `useDeferredValue` for derived expensive values
- Use `Promise.all()` for independent async operations
- Leverage automatic batching

##### Bundle Size (CRITICAL)

- Import directly, avoid barrel files
- Use dynamic imports for heavy components
- Defer third-party scripts after hydration
- Do not use `import { … } from 'lodash'` (pulls full `lodash.js`); use per-method imports such as `lodash/debounce` or `lodash/noop`, or inline trivial helpers

##### Re-render & State Optimization

- Functional state updates for stable callbacks
- Lazy initialization for expensive initial state
- Derive state during render, not in effects
- Split context to prevent unnecessary re-renders
- Use refs for transient frequent values

##### Memoization

- Avoid premature memoization (measure first)
- `React.memo` for expensive pure components
- `useCallback` for stable function refs passed to children
- `useMemo` for expensive calculations

##### Effects & Events

- Always clean up effect side effects
- Avoid effects for derived state and user events
- Avoid object/array dependencies in effects

##### Component Architecture

- Avoid boolean props — use composition
- Compound components with shared context for complex UIs
- Prefer `children` over render props
- No wrapper if the consumed component already exposes the surface — check its `className`/`children` slot before adding a div for positioning, sizing, or clipping

---

#### Refactoring Patterns

##### Structure & Decomposition

- Extract Method for long functions (under 20 lines)
- Single Responsibility per module
- Introduce Parameter Object for 3+ parameters

##### Coupling & Dependencies

- Dependency injection, hide delegates
- Fix feature envy, apply interface segregation

##### Conditional Logic

- Guard clauses over nested conditionals
- Decompose complex conditionals into named predicates
- Replace conditional with polymorphism when switching on type
- Lookup tables for value mappings

##### Naming

- Intention-revealing names, no abbreviations
- Consistent vocabulary, searchable names (no magic numbers)

##### Micro-Refactoring

- Remove dead code, inline trivial variables, simplify booleans

---

#### Modularity

- Small focused modules, single responsibility, clear boundaries
- Pure functions, explicit dependencies
- No deep nesting — early returns and guard clauses
- No god components — split UI, state, and domain logic

## Accessibility

WCAG 2.1 guidance for React components. For audits, follow the `a11y_audit` skill.

### Accessibility Guidelines

*Accessibility guidelines for React components (WCAG 2.1 compliance)*

Ensure all React components meet WCAG 2.1 accessibility standards.

#### Quick Reference

| Element    | Requirement      | Solution                                         |
| ---------- | ---------------- | ------------------------------------------------ |
| `<button>` | Discernible name | Text content, `aria-label`, or `aria-labelledby` |
| `<a>`      | Accessible name  | Descriptive text, `aria-label` for icon-only     |
| `<img>`    | Alt attribute    | `alt="description"` or `alt=""` for decorative   |
| `<input>`  | Associated label | `<label htmlFor>`, `aria-label`, or wrapper      |
| Headings   | Sequential order | h1 → h2 → h3, never skip levels                  |

#### Buttons

Buttons must have a discernible name for screen readers.

```tsx
// ✅ Good
<button>Submit</button>
<button aria-label="Close dialog"><XIcon /></button>

// ❌ Bad
<button><XIcon /></button>
<button></button>
```

#### Links

Links must describe their destination or action.

```tsx
// ✅ Good
<a href="/profile">Your Profile</a>
<a href="/settings" aria-label="Account settings"><SettingsIcon /></a>

// ❌ Bad
<a href="/docs">Click here</a>
<a href="/settings"><SettingsIcon /></a>
```

#### Images

All images must have `alt` attribute.

```tsx
// ✅ Good - informative image
<img src="product.jpg" alt="Dark chocolate truffles box" />

// ✅ Good - decorative image
<img src="decoration.png" alt="" role="presentation" />

// ❌ Bad
<img src="product.jpg" />
```

#### Form Inputs

Inputs must have associated labels.

```tsx
// ✅ Good
<label htmlFor="email">Email</label>
<input id="email" type="email" />

// ✅ Good - visually hidden label
<input type="search" aria-label="Search products" />

// ❌ Bad - placeholder is not a label
<input type="email" placeholder="Enter email" />
```

#### Heading Structure

Headings must follow sequential order for document structure.

```tsx
// ✅ Good
<h1>Main Title</h1>
<h2>Section One</h2>
<h3>Subsection</h3>
<h2>Section Two</h2>

// ❌ Bad - skipping levels
<h1>Main Title</h1>
<h4>Subsection</h4>  {/* Should be h2 */}
```

#### Automated Checks

These issues are also caught by:

- **ESLint:** `eslint-plugin-jsx-a11y` (configured in project)
- **Axe:** Browser extension for manual audits
- **Lighthouse:** Accessibility audits in DevTools

## SFCC

Platform-specific rules for B2C Commerce work.

### SFCC API Resource Routing

*OCAPI/SCAPI resource verification, 404 handling, and multi-method infrastructure checks. Use when making OCAPI/SCAPI Data API calls or verifying SFCC infrastructure exists.*

BEFORE making any OCAPI or SCAPI Data API call, verify against the official B2C Commerce API documentation that the resource exists and get the correct path — never guess paths or versions.

Quick reminders:
- **Price Books** — NOT in OCAPI. Use site archive export.
- **Assignments, Experiences, CDN, CORS, SEO** — NOT in OCAPI. SCAPI only.
- Never brute-force API versions. If a resource returns 404, check the documentation.

#### OCAPI 404 ≠ Resource Missing

An OCAPI Data API 404 does NOT reliably prove a resource is absent. Causes include: client_id lacks permissions, resource type not exposed in OCAPI, or wrong API version. **Never report infrastructure as "MISSING" based solely on an OCAPI 404.**

When verifying SFCC infrastructure exists, use multiple methods:
- **Services** — check WebDAV `/Logs/` for `service-<name>*` log files
- **Custom Object types** — try creating an instance via Shop API; `CustomObjectNotFoundException` = type exists (object not found), `CustomObjectTypeNotFoundException` = type genuinely missing
- **Jobs** — check WebDAV `/Logs/` for `custom-*` or job-specific log files
- **Cartridges** — list via WebDAV `/Cartridges/<code-version>/`

### Custom SCAPI Deployment Verification

*Verify Custom SCAPI endpoint deployment and handle caching gotchas*

After modifying files under `cartridge/rest-apis/`, follow this verification sequence:

#### Deploy

```bash
cd sfcc && b2c code deploy -v <version> --reload
```

#### Verify endpoint is reachable

```bash
#### Quick smoke test (expects 401 without auth, but proves the route is live)
curl -s -o /dev/null -w "%{http_code}" \
  "https://<hostname>/s/<SiteID>/dw/shop/v24_5/custom-objects/site-preferences/v1/preferences"
```

A `404` after deploy usually means the code version was not activated or the `api.json` route does not match. Check in this order:

1. **Cartridge is in the site's cartridge path** (the most commonly missed cause): `Administration → Sites → Manage Sites → <Site> → Settings → Cartridges`. If the cartridge containing `rest-apis/` is **not** in the site's `custom-cartridges`, SFCC does not scan it and **no** endpoint registers. Verify with `b2c scapi custom status --tenant-id <tenant> --short-code <sc>` — if it returns "No Custom API endpoints found" while the cartridge is deployed, cartridge path is almost certainly the cause. The cartridge path is defined in `sfcc/meta_sandbox/site_template/sites/<SiteId>/site.xml` under `<custom-cartridges>`.
2. Code version is active: `b2c code list`
3. `api.json` path matches the URL
4. Function name in `module.exports` matches `api.json` handler
5. If only **one** endpoint is missing (others work), check SFCC error logs for `Invalid API schema` — a single schema violation (e.g. custom query parameter missing `c_` prefix, `additionalProperties` in a request body) blocks **only that endpoint**, not the whole registration. The error log wording (`Fatal error detected during Custom API registration`) is misleading — it is per-endpoint, not global.

#### Isolate the cause quickly

Use `b2c scapi custom status --tenant-id <tenant> --short-code <sc>` to get gateway truth:
- **0 endpoints** registered → cartridge path or missing cartridge in code version
- **Some registered, target missing with status `not_registered`** → schema or api.json bug for that specific endpoint (check error logs)
- **Target registered `active`, still 404 via the storefront / MRT proxy** → SSR / browser cache, see below
- **Target registered `active`, gateway returns 401/400 on curl** → endpoint works, fix auth / payload

#### Caching

Custom Shopper SCAPI responses are cached by:
- **JWA (server-side gateway cache)** — the only reliable server-side cache for the endpoint. It honours `response.setExpires(...)` set in the handler; on a hit the handler is **not** re-invoked (a frozen `retrievedAt`-style timestamp is the tell). Do **not** call `response.setHttpHeader('Cache-Control', ...)` — the SCAPI gateway blocks it and logs an `IllegalArgumentException` per request.
- **eCDN / shared cache** — does **NOT** cache custom Shopper SCAPI: the platform forces `Cache-Control: no-store` on the wire. Public TTLs are configured at eCDN, not in script.
- **Storefront (Storefront Next on MRT)** — the storefront data-fetching / SSR layer may cache fetch results in the Node process during server render.
- **Browser** — standard HTTP cache.

After deploying a fix to a Custom SCAPI endpoint:
1. **Wait out / flush the JWA cache** — it keys off `setExpires`; a hit won't re-run the handler. Flush via BM → Administration → Operations → Custom Caches if a named `CacheMgr` cache is also involved.
2. **Restart the Storefront Next dev server** — clears the SSR fetch cache.
3. **Hard refresh browser** — `Cmd+Shift+R` or clear Application → Storage.

If you see stale data after deploy, it is almost certainly a cache layer. Try all three before investigating further.

#### Common pitfalls

| Symptom | Cause | Fix |
|---|---|---|
| `404` on endpoint, `b2c scapi custom status` shows 0 endpoints | Cartridge with `rest-apis/` not in site cartridge path | Add cartridge to `site.xml` `<custom-cartridges>` or via BM → Site Settings; then `code activate --reload` |
| `404` on one endpoint while others work | Schema validation for that endpoint fails | Check error logs for `Invalid API schema`; custom query params must begin with `c_`; no `additionalProperties` in request bodies |
| `404` on endpoint | Code version not active | `b2c code list` → activate correct version |
| Old response after deploy | JWA gateway cache or storefront SSR cache | Wait out `setExpires` TTL / restart the Storefront Next dev server |
| `TypeError` in SFCC logs | Java/Rhino interop | Inspect the types crossing the Java↔Rhino boundary at the failing call |
| Preference missing from response | Not in allowlist | See the SFCC Metadata File Locations rule — Exposing Preferences section |

### SFCC Metadata File Locations

*Where to place SFCC metadata XML (system-objecttype-extensions, custom objects, preferences) in the project*

New metadata definitions (custom attributes, custom object types, site preferences) go in `meta_delta` **ONLY** during feature development:

| Directory | When to modify | Purpose |
|---|---|---|
| `sfcc/meta_sandbox/site_template/meta/` | **At merge/release time only** — maintained by tech lead | Full metadata set for sandbox provisioning — the single source of truth |
| `sfcc/meta_delta/site_template/meta/` | **During feature work** — this is where YOU add new attributes | Incremental delta for deployments — only new/changed definitions |

⚠️ **CRITICAL**: When adding new custom attributes, add them ONLY to `meta_delta`. Do NOT modify `meta_sandbox/site_template/meta/system-objecttype-extensions.xml` — that file is the canonical full snapshot and is updated separately during release preparation.

#### Adding a New Site Preference

1. **`sfcc/meta_sandbox/site_template/meta/system-objecttype-extensions.xml`** — add `<attribute-definition>` inside `<type-extension type-id="SitePreferences">` in alphabetical order by `attribute-id`. Add to an existing or new `<attribute-group>`.
2. **`sfcc/meta_delta/site_template/meta/system-objecttype-extensions.xml`** — add the same `<type-extension type-id="SitePreferences">` block with only the new attribute.
3. **`sfcc/meta_sandbox/site_template/sites/<SiteID>/preferences.xml`** — optionally set the default value for the site.

#### Adding Custom Attributes on Other Objects

Same pattern: full set in `sfcc/meta_sandbox/`, delta in `sfcc/meta_delta/`. The `meta_delta` file already contains Product, Profile, Promotion, and ProductLineItem extensions — append new type-extensions there. The canonical `meta_sandbox/` file is updated at release time.

#### Exposing Preferences to the Storefront (Frontend)

The Storefront Next frontend has no direct access to `dw.system.Site` preferences — the XML definition alone is not enough. A preference the frontend must read at runtime has to be exposed through SCAPI:

1. **Expose via Custom SCAPI** — return the preference from a Custom SCAPI endpoint (e.g. a `site-preferences` endpoint). Keep an explicit **allowlist** of preference IDs the endpoint may return; never return the whole preference set. See the `b2c_config` skill.
2. **Consume in a loader** — fetch the value through the SCAPI client in a React Router loader and pass it to components as loader data; type the returned shape on the frontend.
3. **Deploy & cache** — after changing the allowlist or endpoint, deploy the cartridge (`b2c code deploy`). Custom SCAPI responses are cached via `setExpires`; account for that TTL when verifying a changed value.

**Checklist (copy into task):**

- [ ] XML attribute definition committed (definition + value locations)
- [ ] Preference ID added to the endpoint allowlist (never expose the full set)
- [ ] Frontend type for the returned shape
- [ ] Metadata imported to sandbox
- [ ] Cartridge code deployed
- [ ] Value verified through the SCAPI response / in the storefront

#### Importing to Sandbox

The archive root is `site_template/` (not `meta_delta/`/`meta/` itself — the parent folder is just a container that may also hold `jobs.xml.bak` and similar).

##### On a fresh sandbox — import the FULL archive first

`meta_delta` is an **incremental** delta. It does NOT contain the full schema nor the site `custom-cartridges` / preferences. Running only the delta on a fresh instance leaves Profile/Product/Order custom attributes undefined and the storefront cartridge-path missing integration cartridges — both will cause storefront requests to fail once rendering starts: SCAPI-backed Storefront Next data loads error out, and any SFRA/hybrid controllers (e.g. `Account-Show`) return `500 Internal Error`.

```bash
cd sfcc && b2c job import meta_sandbox/site_template --show-log
```

This imports the full snapshot: all custom object types, all `type-extension` definitions, the `<custom-cartridges>` chain in `sites/<SiteID>/site.xml`, jobs, services, preferences, OCAPI/SCAPI settings, etc.

##### On an already-bootstrapped sandbox — use the delta

For incremental changes (new attributes added to the repo since last deploy):

```bash
cd sfcc && b2c job import meta_delta/site_template --show-log
```

##### Targeted imports

When you need to apply only a specific file (e.g. just a new Profile attribute) without touching site prefs/jobs/services, build a minimal archive under `/tmp/<name>/` containing only the needed `meta/system-objecttype-extensions.xml` (or `sites/<siteId>/site.xml`, etc.) and import that folder directly.

#### CI Deployment: OCAPI Client Permissions

**Critical:** The system job `sfcc-site-archive-import` (triggered via OCAPI) **respects the OCAPI Data API permissions of the client that triggered it**. If the client lacks permission on a resource, the job silently skips that data — no error in the log, status still OK.

This is undocumented SFCC behavior discovered empirically. The job log will report "Processed N elements successfully" even when the client's permissions prevented the data from being written.

##### Required OCAPI Data API resources for full site import

The CI client (`4c047cce-...`) needs these resources in the **Global** OCAPI Data API configuration on every target instance (development, staging):

```json
{
    "methods": ["get", "put", "patch"],
    "read_attributes": "(**)",
    "write_attributes": "(**)",
    "resource_id": "/sites/**"
},
{
    "methods": ["get", "put", "patch", "post"],
    "read_attributes": "(**)",
    "write_attributes": "(**)",
    "resource_id": "/custom_objects/**"
}
```

| Resource                        | What it enables                                                                         |
| ------------------------------- | --------------------------------------------------------------------------------------- |
| `/sites/**`                     | Site preferences, search config, customer groups, stores, shipping, tax, slots, content |
| `/custom_object_definitions/**` | Custom object type definitions (`custom-objecttype-definitions.xml`)                    |
| `/system_object_definitions/**` | System object type extensions (`system-objecttype-extensions.xml`)                      |
| `/global_preferences/**`        | Global preferences (`preferences.xml` in archive root)                                  |
| `/custom_objects/**`            | Custom object instances (data records)                                                  |

##### Debugging "import OK but data missing"

1. Check which OCAPI client ID triggered the job — BM > Jobs > job history shows "by OCAPI Client" vs "by user@email"
2. Compare that client's OCAPI Data API permissions with the resource table above
3. Note: `sfcc-site-archive-import` (OCAPI) and `sfcc-site-import` (BM UI) are **different system jobs** — BM Site Import runs with BM user permissions, not OCAPI client permissions

## Search before you read

For a narrow factual lookup — find a file, locate a symbol, check whether a string
exists — grep first and read only the matching region. Do not open and scan whole
files for questions a targeted search answers in one step.
