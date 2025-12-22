# Review Output Template

Generate this file as `status.md` in the reviewed crate.

---

```markdown
# {crate_name} Compliance

**Layer:** {Shared | Infrastructure | Domain | Application | Entry}
**Reviewed:** {YYYY-MM-DD}
**Verdict:** {COMPLIANT | NON-COMPLIANT}

---

## Checklist

| Category | Status |
|----------|--------|
| Boundary Rules | ✅ / ❌ |
| Required Structure | ✅ / ❌ |
| Code Quality | ✅ / ❌ |

---

## Violations

| File:Line | Violation | Category |
|-----------|-----------|----------|
| `src/foo.rs:42` | `unwrap()` usage | Code Quality |
| `src/bar.rs:15` | Direct SQL in service | Repository Pattern |

{Or: "None"}

---

## Commands Run

\`\`\`
cargo clippy -p {crate_name} -- -D warnings  # {PASS/FAIL}
cargo fmt -p {crate_name} -- --check          # {PASS/FAIL}
\`\`\`

---

## Actions Required

1. {Action to fix violation}
2. {Action to fix violation}

{Or: "None - fully compliant"}
```
