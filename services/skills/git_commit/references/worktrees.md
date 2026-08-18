# Working on a git worktree

When a feature lives in a sibling worktree (e.g. `<repo>-worktrees/PROJ-198-feature/`) but the session
was started in the main repo (`<repo>/`), project rules and configuration that resolve relative to the
working directory can silently stop applying to files in the worktree.

## Rule

First action on any worktree-based task: make the worktree the working directory for the session, and
use absolute paths into the worktree for every file operation. Confirm which checkout you are editing
with `git rev-parse --show-toplevel` before the first change.

## When to check

- Resuming or starting any feature where the branch is checked out in a worktree (not in the main repo).
- After creating a new worktree with `git worktree add`.
