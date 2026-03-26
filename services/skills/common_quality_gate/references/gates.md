# Gate Criteria & Thresholds

## Gate 1 · Requirements Alignment

| Criterion | Threshold |
|---|---|
| Every stated requirement maps to an implementation | 100% coverage |
| Requirements are testable (have measurable acceptance criteria) | All must be verifiable |
| No undocumented assumptions or implicit requirements | Zero tolerance |

**How to evaluate:**
- Compare the requirements (commit messages, PR description, task description, inline comments) against the implementation
- Flag any requirement that is unaddressed, ambiguous, or untestable
- If no explicit requirements are available, infer from the code's intent and flag as WARN

## Gate 2 · Code Quality & Coverage

| Criterion | Threshold |
|---|---|
| Unit test pass rate | 100% (zero failures) |
| Line/branch coverage on changed files | >= 80% |
| No critical static analysis violations | Zero high/critical severity |
| Functions <= 40 lines | Advisory (WARN if breached) |
| Cyclomatic complexity <= 10 | Advisory (WARN if breached) |

**How to evaluate:**
- Review code structure, naming, error handling, and separation of concerns
- Identify specific functions or modules that violate thresholds
- Check for: unused imports, dead code, overly broad exception handling, missing type hints (if project uses them)
- Run available linters/test commands if a test runner is configured
- For Python: check PEP 8 compliance, docstring presence on public APIs
- For JavaScript/TypeScript: check ESLint rules, proper async/await usage

## Gate 3 · Test Readiness

| Criterion | Threshold |
|---|---|
| Test environment reproducible from provided config | Single command setup |
| Test data/fixtures included or documented | 100% of tests runnable |
| No external service dependencies without mocks/stubs | Zero unmocked externals |

**How to evaluate:**
- Could a new developer clone, install, and run the full test suite in under 10 minutes with no tribal knowledge?
- Check for: hardcoded paths, missing fixture files, undocumented environment variables
- Verify test isolation — tests must not depend on execution order or shared mutable state

## Gate 4 · User Acceptance

| Criterion | Threshold |
|---|---|
| All acceptance criteria from Gate 1 demonstrated | 100% |
| No Severity 1 or Severity 2 open bugs | Zero S1/S2 |
| Edge cases and error states handled gracefully | Errors show actionable messages |

**Severity definitions:**
- **S1 (Critical):** System crash, data loss, security vulnerability, complete feature failure
- **S2 (Major):** Feature partially broken, significant UX degradation, workaround required

**How to evaluate:**
- Walk through each user-facing flow conceptually based on the code
- Flag broken paths or missing error handling with specific reproduction steps
- Check for: unhandled exceptions, missing validation, unclear error messages, silent failures

## Gate 5 · Production Readiness

| Criterion | Threshold |
|---|---|
| README / deployment docs updated | All changed components documented |
| Rollback plan defined | Must specify exact rollback steps |
| Environment variables documented (not hardcoded) | Zero hardcoded secrets |
| Database migrations reversible | Down migration exists (if applicable) |

**How to evaluate:**
- Could an on-call engineer deploy, monitor, and roll back this change at 3 AM without the original author?
- Check for: hardcoded URLs, API keys, passwords, connection strings
- Verify logging is adequate for debugging production issues
- Check for: missing migration files, irreversible schema changes
