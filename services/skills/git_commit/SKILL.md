# Git Commit & Branch Workflow

Single source of truth for branch naming, commit format and body, and the commit-to-PR flow:
stage → branch → commit → (optional) push → draft PR → open in browser.

## Push and PR gate

Run **Steps 5–7** only after the user **explicitly** asks — for example "push", "push and draft PR", "create a draft PR", "open PR". After **Step 4** (commit), **stop** unless they request push/PR.

## Step 1 — Show uncommitted files

```bash
git status --short
```

Present the list to the user. Use `AskQuestion` (multi-select) to let them pick which files to include. Include only the files the user selects.

## Step 2 — Branch check

The **integration branch** is the shared base for feature/fix branches (commonly `develop` or `main`). Confirm the actual value with the user or from the remote branch list; the commands below use `<integration-branch>` as a placeholder.

```bash
git branch --show-current
```

**If current branch is the integration branch:** create a feature/fix branch (see [Branch naming](#branch-naming)). Confirm the name with the user first.

**If already on a feature/fix/requirements branch:** stay on it, skip branch creation.

## Step 3 — Stage selected files

```bash
git add <file1> <file2> ...
```

Stage only the files the user selected in Step 1. Never stage `.env`, credentials, or secrets.

## Step 4 — Commit message

1. Inspect staged changes: `git diff --cached`.
2. Look at recent commits for convention reference: `git log --oneline -20`.
3. Compose the header per [Commit format](#commit-format) and the body per [Commit body](#commit-body).
4. Show the proposed message to the user and ask for approval before committing.
5. Commit:

```bash
git commit -m "$(cat <<'EOF'
<message>
EOF
)"
```

If commitlint rejects the message, fix it and retry — never use `--no-verify`.

## Step 5 — Push

Only when the user asked to push (see [Push and PR gate](#push-and-pr-gate)).

```bash
git push -u origin HEAD
```

## Step 6 — Create draft PR

Only when the user asked for a PR / "push and draft PR" (see [Push and PR gate](#push-and-pr-gate)).

### Authenticate gh CLI

Homebrew-installed `gh` may not be in the shell PATH — always prepend:

```bash
export PATH="/opt/homebrew/bin:/usr/local/bin:$PATH"
```

Then check auth: `gh auth status`. If not authenticated and the remote is HTTPS with an embedded PAT:

```bash
GH_TOKEN=$(git remote get-url origin | sed -n 's|https://[^:]*:\([^@]*\)@.*|\1|p')
echo "$GH_TOKEN" | gh auth login --with-token
```

If the remote is SSH (`git@github.com:...`), the user must have `gh` pre-authorized (`gh auth login`). Verify with `gh auth status` first.

### Write PR body

Analyse the committed diff (`git diff origin/<integration-branch>...HEAD`) to fill the template. Save to a temp file, e.g. `/tmp/pr-body.md`:

```markdown
## Summary

[1–3 bullet points summarising what was done and why]

## Jira Ticket

- [PROJ-XXX](<atlassian-base-url>/browse/PROJ-XXX)

## Changes Made

- [Key change 1]

**Files affected:**
- `path/to/file.ts`

## Testing

[How to verify the change]

## Breaking Changes

None
```

### Create the PR

```bash
gh pr create \
  --base <integration-branch> \
  --draft \
  --title "<commit-message-title>" \
  --body-file /tmp/pr-body.md
```

PR title must follow the same `SCOPE:TYPE <TICKET> Description` format as the commit message.

## Step 7 — Open PR in browser

Only when the user asked to open the PR, or a browser handoff helps after Step 6.

```bash
gh pr view --web
```

---

## Branch naming

- **Always prefixed.** Never prefixless. Use `feature/` (new functionality, tooling, docs) or `fix/`
  (bug fixes). Requirements/BA work may also use `requirements/`.
- **With a ticket:** `feature/<TICKET>` / `fix/<TICKET>` — ticket key only (e.g. `feature/PROJ-1234`).
- **Without a ticket:** `feature/<slug>` / `fix/<slug>` — lowercase, hyphen-separated, ≤5 words
  (e.g. `feature/add-carousel-component`).
- **Never use `PROJ-000` as a branch name** — only in a commit message when a real ticket is absent.

### Creating branches from the integration branch (CRITICAL)

Let `<integration>` be the project's shared integration branch (commonly `develop`
or `main`). **Always use `--no-track`** when branching off it:

```sh
git fetch origin <integration>
git checkout -b feature/PROJ-1234 origin/<integration> --no-track
```

Without `--no-track`, the new branch tracks `origin/<integration>` and a later `git push` may update the
shared integration branch. A `pre-push` hook should block pushes to protected branches (`develop`,
`main`, `master`) as a safety net, but `--no-track` prevents the misconfiguration at the source.

**Never** push to a protected branch.

---

## Commit format

Format: `SCOPE:TYPE <TICKET> Subject` — validated by commitlint (Storefront
[conventional-changelog](https://github.com/conventional-changelog/conventional-changelog) preset).

```sh
SCOPE:TYPE PROJ-123, PROJ-124 Subject (sentence-case)
```

- **Scope:** UPPER CASE, from the project's `scope-enum` (see `commitlint.config.*`).
- **Type:** UPPER CASE, from the project's `type-enum` (commonly `FEAT`, `FIX`, `FEATFIX`, `RELEASE`,
  `DOCS`, `INT`, `REFACTOR`, `UNIT`, `DEPS`, `DEV-DEPS`).
- **Ticket:** at least one ticket key in the project's Jira prefix; comma-separate for multiple. When no
  real ticket exists, use the project's no-ticket convention (e.g. `PROJ-000`) — never invent one.
- **Subject:** one sentence, what was done.

**Validation:** scope and type must be from the configured enums; at least one ticket reference; body has
a leading blank line. Commits starting with `Merged` or `RELEASE` are ignored by commitlint.

**No AI footers** (`Co-Authored-By`, "Generated with …") in commit messages.

### Header examples

```sh
PLP:FEAT PROJ-1665 Restyled quick buy on PLP
3PI:FIX PROJ-1653 ApplePay. Fixed button appearing on variation change
CO:FEAT PROJ-1001, PROJ-1002 Implemented new checkout flow and payment validation
```

```sh
# Bad: missing scope, wrong case, missing ticket
FEAT PROJ-123 Added feature
plp:FEAT PROJ-123 Added feature
PLP:FEAT Added new feature
```

Scope and type enums are project configuration — extend them in the project's `commitlint.config.*`
(`scope-enum` / `type-enum`) and the Jira prefix in the parser options (`issuePrefixes`).

---

## Commit body

A commitlint-valid header is not a complete message. Anything that changes behaviour also carries a
body, so the history stays readable for the next developer and usable for QA regression scoping.

```
<header>

<1–2 sentences: what was done and why>

- <change>
- <change>

Impact:
- Area: <functional / integration surfaces, plus the WBS code where one exists>
- Effect: <1–3 bullets — observable behaviour, not implementation>
- Regression: <the flows QA must re-check>
- Migration: <metadata / config / reindex / flag to apply on rollout, or None>
```

- **Summary paragraph — required.** One or two sentences naming the problem and why this is the fix.
  Never open straight into the bullet list.
- **Bullets — required once more than one thing changed.** WHAT and WHY, never HOW; the diff shows how.
  Wrap the body at 72 characters.
- **`Impact` block — required** when storefront behaviour, an integration contract, SFCC metadata, or an
  LLM contract (rules, skills, artifact templates) changes. Omit it for pure refactors, docs, and
  test-only commits.
- **`Area` names surfaces, not modules** — `Add to Cart`, `Guest checkout`, `Cartora Maps`, not
  `cart-slice.ts`. Use the FSD/ISD vocabulary and add the WBS leaf code (`SLS.PDP.VAR`, `3PI.MAP.GEO`)
  whenever the commit implements a requirement, so a reader can join the commit to its requirement block
  and its Story.
- **`Regression` lists neighbouring flows** that could have been disturbed — not a restatement of
  `Effect`.
- **`Migration` is `None`** unless someone must do something for the change to take effect.
- A body may be omitted only for genuinely trivial commits (a typo in a doc, a lockfile bump).

### Body examples

A storefront behaviour change:

```
PDP:FEAT PROJ-1841 Preselect the single available variant on PDP

Shoppers had to pick a size even on one-size products before Add to Cart
enabled, which produced measurable drop-off on those categories.

- Preselect the variant when a master resolves to exactly one orderable variant
- Keep the selector visible so the choice stays explicit
- Leave multi-variant masters untouched

Impact:
- Area: PDP variant selection (SLS.PDP.VAR), Add to Cart, Mini-cart
- Effect: Add to Cart is enabled on first paint for single-variant masters
- Regression: add to cart from PDP and from quick-view, single- and
  multi-variant; variation switch on PDP; cart line item shows the right SKU
- Migration: None
```

An integration change:

```
3PI:FIX PROJ-1902 Retry Cartora geocoding on rate limits

Store search returned an empty list whenever Cartora rate-limited the geocode
call, so shoppers saw "no stores nearby" instead of a retry.

- Retry twice with exponential backoff on 429 and 503
- Fall back to the cached postal-code centroid when both retries fail
- Log the vendor request id on every failed attempt

Impact:
- Area: Store search (3PI.MAP.GEO), Cartora Maps integration, BOPIS store
  selection in checkout
- Effect: a rate-limited geocode degrades to cached coordinates instead of an
  empty result; worst-case search latency grows by ~1.2s on that path
- Regression: store search by postal code, by city and by geolocation; store
  selection in the checkout delivery step; behaviour with the vendor key revoked
- Migration: None — backoff bounds read from the existing CARTORA_* settings
```

A change that needs work on the instance:

```
INT:FEAT PROJ-1755 Add opening hours to the Store system object

The store detail card had no source for opening hours, so the locator showed
address and phone only while the client's feed already carried the data.

- Extend the Store system object with a localizable storeHours attribute
- Map the attribute in the store import feed
- Render it on the detail card when populated

Impact:
- Area: Store detail card (SLS.DTL), Business Manager store editing, store
  import feed
- Effect: the card renders opening hours; an empty attribute keeps today's layout
- Regression: store detail card with and without hours; store import job on a
  feed missing the new column; BM store editing
- Migration: import site-metadata.xml on every instance BEFORE deploy, then
  re-run the store import job
```

---

## Working the workflow

Operational guidance for every git operation in this skill. Optimize for cost and correctness — no design decisions, no code edits while running git operations.

**Scope discipline:** do only what the user asked. If they only asked to commit, do not push. Merge/rebase conflict resolution, source edits, and amending other people's commits are out of scope unless explicitly requested.

**Inspect (status / diff / log / branch list):** identify the target repo from the prompt or cwd and run git there. Return concise output — paths, hashes, branch names.

**Branch:** follow the project's branch naming convention (confirm the pattern from existing branches or the user), confirm the base branch from `git branch -a` or the user, then `git checkout -b` / `git switch -c` from the correct base. Never invent a branch name when the user supplied a ticket number.

**Commit (+ optional push):**

1. Run in parallel: `git status`, `git diff` (+ `git diff --staged` if needed), `git log -5 --oneline`.
2. Determine the commit convention: find `commitlint.config.*`, `.commitlintrc*`, or `commitlint` in `package.json`; no config → mirror recent `git log` style, using the [Commit format](#commit-format) above.
3. Draft a message focused on **why**. Never stage `.env`, credentials, or other secrets — warn if the user requested them.
4. Stage only relevant paths. Commit via HEREDOC. Run `git status` after commit to confirm.
5. Push with `-u origin HEAD` only when explicitly requested.

**Fetch / pull / stash / cherry-pick:** run only when explicitly asked. If pull or cherry-pick stops on conflicts, report the conflicted paths and stop — do not edit files to resolve as part of the git operation.

**Hard rules:**

- NEVER update git config. NEVER `--no-verify`, force-push, or rewrite history destructively unless the user explicitly asks.
- NEVER amend unless the user asked AND HEAD is yours and unpushed.
- No `Co-Authored-By`, "Generated with …", or any AI footer in commit messages.
- If nothing to commit, say so — do not create an empty commit.
- If a commit hook fails, fix the issue and create a **new** commit — never amend a failed commit.
- Never use interactive git flags (`-i`) — they require a TTY.

**Report back:** operation summary, branch name (if relevant), commit hash + message subject (if committed), push result (if any), and PR link if the remote prints one.

## Additional resources

- Issue-tracker ticket formatting (ADF descriptions, user lookup, transitions): use the
  `atlassian` skill — do not paste Markdown into Jira expecting it to render.
- Working on a git worktree: see [references/worktrees.md](references/worktrees.md).
- Keeping a feature branch up to date with a fast-moving integration branch (staleness debugging):
  see [references/branch-maintenance.md](references/branch-maintenance.md).

## Temp files

Write intermediate files (PR body, commit-message drafts) to a temp directory outside the repo. Clean up after.
