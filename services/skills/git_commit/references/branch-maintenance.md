# Keeping a feature branch up to date with the integration branch

If the integration branch moves fast, a feature branch that sat a few days without pulling it WILL
diverge — and the divergence often hides **runtime regressions that look like your bug** (e.g. a fix to a
shared adapter/util that landed on the integration branch but not in your branch). When a local
reproduction "should work" but doesn't, and the deployed environment shows the same page working,
**branch staleness is the first hypothesis to rule out — before touching code or restarting the server
for the Nth time.**

## When to check

- **Every time you resume work** on a feature branch after any gap (overnight, weekend, context switch).
- **Before any local repro / dev-server session** for debugging. A stale integration branch is a top-3
  source of "works on deployed, broken locally" false-positives.
- **Before running the full E2E/unit-test suite** — avoids chasing test failures caused by upstream fixes
  you just don't have yet.
- **When the symptom is in an unrelated module** (e.g. content adapter, slugify, SCAPI client) that your
  change did NOT touch — that module was very likely fixed upstream after you branched.

## Quick-check (1 command, 10 seconds)

```sh
git fetch origin <integration> --quiet && \
  echo "behind origin/<integration>: $(git rev-list --count HEAD..origin/<integration>) commits"
```

Heuristic:

- More than ~5 commits behind → update before continuing local work.
- Far behind (tens of commits) → additionally scan `git log origin/<integration> --stat --since=<branch-point>`
  for changes in your feature's blast radius (adapters, utils, middleware, shared hooks). Upstream fixes
  in those files may supersede your local workarounds — leave them in the merge, do not re-apply yours.

## Update procedure (safe, preserves WIP)

Run from the **feature branch worktree** with all uncommitted work intact.

```sh
# 1. Stash everything (tracked + untracked)
git stash push -u -m "WIP before integration merge"

# 2. Make sure the local integration ref is fresh
git fetch origin <integration>

# 3. Merge the integration branch into the feature branch
git merge --no-ff origin/<integration> -m "Merge branch '<integration>' into <your-feature-branch>"

# 4. Resolve any conflicts (`git status` → edit → `git add` → `git merge --continue`)
#    If no conflicts, step 3 auto-commits.

# 5. Restore WIP on top of the merge
git stash pop

# 6. Rebuild + resolve any post-merge issues:
#    - If the lockfile changed in the merge → reinstall deps (`pnpm install`)
#    - Restart dev server / rerun tests
#    - Check `git diff <pre-merge-HEAD> HEAD -- <file>` for surprises in areas you touched
```

Never use `git pull --rebase` across a big gap — it re-applies your commits one-by-one through the
upstream commits and multiplies conflict work.

## Post-merge sanity

- Run the project's typecheck and the focused test suite you care about.
- Quick smoke on the dev server: homepage + one representative page touched by your change.
- Look at `git diff <old-HEAD> HEAD --stat` — scan for surprise files in your feature's blast radius
  (adapters, utils, middleware). Upstream fixes there may supersede your own workarounds.
