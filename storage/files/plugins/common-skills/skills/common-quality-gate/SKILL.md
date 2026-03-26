---
name: "Quality Gate"
description: "Structured quality gate review on code changes. Evaluates requirements, code quality, test readiness, and production readiness"
---

|
| **PASS** | All criteria met |
| **WARN** | Advisory thresholds breached. Resolve next sprint |
| **FAIL** | Hard thresholds breached. Deployment blocked |
| **BLOCKED** | Insufficient information. Specify what's needed |

## Output Format

For each gate:

```
### Gate [N] · [Name] → **[STATUS]**
**Evidence:** [Specific observations — quote code/docs]
**Gaps:** [What's missing, with file:line references]
**Remediation:** [Concrete steps to reach PASS]
```

Then produce:

### Summary Table

| Gate | Status | Key Finding |
|---|---|---|
| 1 · Requirements | ... | ... |
| 2 · Code Quality | ... | ... |
| 3 · Test Readiness | ... | ... |
| 4 · User Acceptance | ... | ... |
| 5 · Production Readiness | ... | ... |

**Overall Verdict:** PASS / WARN / FAIL / BLOCKED
(Overall = worst individual gate status)

### Top 3 Priority Fixes

List the 3 most impactful issues to resolve, ordered by severity.
Each fix must include: the gate it belongs to, the specific file:line, and
the concrete action to take.
