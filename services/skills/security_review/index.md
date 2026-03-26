## When to Use

Invoke when reviewing code changes that touch authentication, authorization, input handling, data storage, or external API calls.

## Checklist

Evaluate each category and assign PASS, WARN, or FAIL:

| # | Category | Key Questions |
|---|----------|---------------|
| 1 | Injection | Are all user inputs parameterized? No string concatenation in queries? |
| 2 | Authentication | Are credentials stored securely? No hardcoded secrets? |
| 3 | Authorization | Are access controls enforced at every entry point? |
| 4 | Data Exposure | Are sensitive fields redacted in logs and API responses? |
| 5 | Input Validation | Are inputs validated at system boundaries? Size limits enforced? |
| 6 | Dependencies | Are third-party packages pinned? Any known CVEs? |

## Output Format

For each category:

### [#] [Category] — [PASS/WARN/FAIL]
**Evidence:** Specific observations with file:line references
**Risk:** What could go wrong if unaddressed
**Fix:** Concrete remediation step

### Summary Table

| Category | Status | Severity |
|----------|--------|----------|
| 1. Injection | ... | ... |
| 2. Authentication | ... | ... |
| 3. Authorization | ... | ... |
| 4. Data Exposure | ... | ... |
| 5. Input Validation | ... | ... |
| 6. Dependencies | ... | ... |

**Overall:** PASS / WARN / FAIL