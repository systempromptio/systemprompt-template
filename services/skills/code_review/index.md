## When to Use

Invoke when reviewing a pull request, a diff, or a set of changed files.

## Checklist

Evaluate each category and assign PASS, WARN, or FAIL:

| # | Category | Key Questions |
|---|----------|---------------|
| 1 | Naming & Clarity | Are variables, functions, and files named descriptively? |
| 2 | Error Handling | Are errors caught, logged, and surfaced to the caller? |
| 3 | Test Coverage | Are new code paths covered by unit or integration tests? |
| 4 | Security | Are inputs validated? Are secrets handled safely? |
| 5 | Performance | Are there N+1 queries, unbounded loops, or missing caches? |

## Output Format

For each category:

### [#] [Category] — [PASS/WARN/FAIL]
**Evidence:** Specific observations with file:line references
**Gaps:** What is missing
**Fix:** Concrete remediation step

### Summary

| Category | Status |
|----------|--------|
| 1. Naming & Clarity | ... |
| 2. Error Handling | ... |
| 3. Test Coverage | ... |
| 4. Security | ... |
| 5. Performance | ... |

**Overall:** PASS / WARN / FAIL