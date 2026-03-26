## When to Use

Run at the start of the day to generate a standup update.

## Steps

1. Check `git log --oneline --since="yesterday"` for recent commits
2. Check `git diff --stat` for uncommitted work
3. Check any TODO/FIXME comments in recently changed files

## Output Format

**Yesterday:**
- [Completed items from git log]

**Today:**
- [Planned items based on uncommitted work and TODOs]

**Blockers:**
- [Any issues found, or "None"]