# Systematic Debugging

## Overview
Random fixes waste time and mask underlying issues.
**Core principle:** Find root cause before fixing. Symptom fixes fail.
Violating this process violates the spirit of debugging.

## The Iron Law
```
NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST
```
Propose fixes only after completing Phase 1.

## When to Use
Use for ANY technical issue: failures, bugs, performance, or integration.
**Use ESPECIALLY when:** Under pressure, tempted by "quick fixes," or after failed attempts.
**Don't skip:** Even if the issue seems simple or urgent. Systematic is faster than thrashing.

## The Four Phases
Complete each phase before proceeding.

### Phase 1: Root Cause Investigation
**Before attempting any fix:**

1. **Read Error Messages:** Don't skip errors/warnings. Read full stack traces. Note line numbers, paths, and error codes.
2. **Reproduce Consistently:** Define exact steps to trigger it reliably. If not reproducible, gather more data; don't guess.
3. **Check Recent Changes:** Review git diffs, recent commits, and config/dependency changes.
4. **Gather Evidence in Multi-Component Systems:**
   Add diagnostic instrumentation before proposing fixes:
     - Log data at component boundaries (entry/exit).
     - Verify environment propagation and state at each layer.
     - Analyze evidence to isolate the failing component.
5. **Trace Data Flow:** Trace bad values backward to their source. Fix at the origin, not the symptom. See [root-cause-tracing](references/root-cause-tracing.md).

### Phase 2: Pattern Analysis
**Identify the pattern before fixing:**
1. **Find Working Examples:** Locate similar working code in the codebase.
2. **Compare References:** Read reference implementations fully before applying patterns.
3. **Identify Differences:** List every difference between working and broken code.
4. **Understand Dependencies:** Check required settings, configs, and environmental assumptions.

### Phase 3: Hypothesis and Testing
1. **Form Hypothesis:** State specifically: "X is the cause because Y."
2. **Test Minimally:** Change one variable at a time to test the hypothesis.
3. **Verify:** If it fails, form a new hypothesis. Do not stack unverified fixes.
4. **Research:** If you don't understand something, research or ask for help.

### Phase 4: Implementation
**Fix the root cause, not the symptom:**
1. **Create Failing Test:** Automate the simplest reproduction before fixing. Write the failing test before the fix (red → green).
2. **Implement Single Fix:** Address only the root cause. Avoid "while I'm here" changes.
3. **Verify:** Confirm the issue is resolved and no regressions exist.
4. **Handle Failures:** If 3+ fixes fail, STOP. Re-analyze Phase 1 or question the architecture.
5. **Question Architecture:** If fixes cause new issues or require massive refactoring, discuss with your partner. Don't force a broken pattern.

## The Diagnose → Fix → Verify Loop

Work every bug as three sequential stages. Do not blend them: finish the diagnosis before touching code, and finish the fix before judging it.

### Diagnose

Investigate the reported bug, identify the root cause, and state a hypothesis. Do not implement fixes yet.

1. **Analyze the issue:** review "Expected vs Actual" behavior and any ticket/docs context; search for the error messages, log patterns, or UI labels from the report; identify the code paths responsible for the feature.
2. **Investigate code and logic:** read the relevant source files, trace the data flow that leads to the "Actual Result", and look for off-by-one errors, null checks, race conditions, incorrect logic, and API mismatches. Compare the current implementation against the "Expected Result".
3. **Formulate the hypothesis:** determine why the bug happens, check whether existing tests cover the scenario, and identify which lines or functions are at fault. If multiple potential causes exist, list them with confidence levels, and differentiate "confirmed via code analysis" from "suspected". If the bug is a missing requirement rather than broken code, flag it.
4. **Record the findings** before moving on: the Root Cause Hypothesis, the evidence (code snippets or logic traces), the relevant files, a reproduction hint (inputs, conditions), and any missing information.

### Fix

Prove the bug exists with a test, fix it, and verify the result.

1. **Reproduce (mandatory first step):** create or modify a test that replicates the bug based on the Root Cause Hypothesis. Run it and CONFIRM it fails with the expected error/behavior. If the test passes initially, STOP — the bug is not reproduced; go back to Diagnose.
2. **Implement:** modify the code to address the identified root cause. Minimal changes only — no large refactors or style changes, to reduce regression risk. If the fix requires changing core architecture or high-risk shared components, stop and present options to the user first.
3. **Verify locally:** the reproduction test must now pass; run the relevant test suite for regressions; run lint and type checks and fix all errors. Use `git diff` to confirm the change set is minimal.

### Verify the fix

Act as the quality gate on the completed fix before committing.

1. **Analyze the fix:** review the code changes, compare before/after logic, and confirm the change directly addresses the recorded Root Cause — not just the symptom.
2. **Verify integrity:** the requirement ("Expected Result") is met; the fix is contained and does not break related functionality; a valid reproduction test exists and passes; the changes are minimal and readable.
3. **Conclude explicitly:** state a status — VERIFIED, REGRESSION, or FAILED — with the pass/fail of the root-cause, requirements, and risk checks, any issues found (with file/line), and the next step: proceed to commit, or return to the Fix stage. If the code looks good but does not fix the bug, the status is FAILED.

## Red Flags - STOP
Stop and return to Phase 1 if you:
- Propose fixes without investigation or data tracing.
- Guess ("It's probably X", "Try changing X").
- Skip tests or batch multiple unverified changes.
- Encounter new problems with each fix.
- Reach 3+ failed attempts.

## Stuck Loop — Micro-Reflect

If you hit the red flags above (especially 3+ failed attempts), do **not** start Phase 1 over from scratch. Run a **micro-reflect** first — one focused causal chain to decide where to look next.

**Format:**

```
Symptom: {the exact failure, as observed just now — error message, status code, what user saw}
  ← BECAUSE: {immediate technical cause — NOT "because my fix was wrong", but "because Y happens before Z expects it"}
    ← BECAUSE: {why that cause is present in the system — code, config, environment}
      ← Missing fact: {ONE concrete piece of information you do not have}
      ← Missing rule/skill: {the rule or skill that would have prevented this loop, OR "no such rule exists — propose new"}
```

**Rules for the micro-reflect:**

- Exactly ONE chain, not a tree. Pick the most likely hypothesis; if wrong, run another micro-reflect, don't branch.
- The chain must terminate in either (a) a concrete missing fact you can look up, or (b) a concrete rule/file that would have prevented this.
- If the chain terminates in "I don't know why" — that IS the missing fact. Tell the user and ask for input.

**After the micro-reflect — pick ONE of three paths:**

| Outcome | Action |
|---|---|
| Identified a missing fact (e.g. "I don't know what format the API returns") | Stop fixing. Go find that fact. Read docs, run a probe call, ask the user. |
| Identified a missing rule or skill that would have saved this | Propose it to the user (1–2 line draft). If approved, add it — and the next session won't loop here. |
| Identified an architectural issue | Return to Phase 4 "Question Architecture" — discuss with partner before coding. |

**Never:** jump back into Phase 1 checklist mechanically after 3+ failures. The checklist didn't fail — your mental model did. Fix the model first.

## Partner Signals
Return to Phase 1 if your partner says:
- "Stop guessing."
- "Is that not happening?" (Unverified assumptions).
- "Will it show us...?" (Missing evidence).
- "Ultrathink this" or "We're stuck?".

## Common Rationalizations
| Excuse | Reality |
|---|---|
| "Issue is simple" | Even simple bugs need root causes. |
| "No time/Emergency" | Systematic debugging is faster than thrashing. |
| "Try this first" | First fixes set patterns. Do it right. |
| "Test later" | Untested fixes don't stick; test first. |
| "Multiple fixes" | You can't isolate what worked. |
| "Adapt reference" | Partial understanding guarantees bugs. |
| "I see the problem" | Symptoms != Root cause. |
| "One more fix" | 3+ failures = architectural issue. |

## Quick Reference
| Phase | Activities | Success Criteria |
|---|---|---|
| **1. Root Cause** | Read errors, reproduce, gather evidence | Understand WHAT and WHY |
| **2. Pattern** | Find examples, compare | Identify differences |
| **3. Hypothesis** | Form theory, test minimally | Confirmed hypothesis |
| **4. Implementation** | Create test, fix, verify | Issue resolved |

## Environmental/External Issues
If investigation shows the issue is truly external:
1. Document findings.
2. Implement handling (retries, timeouts).
3. Add monitoring.
*Note: 95% of "no root cause" cases are just incomplete investigations.*

## Supporting Techniques
- [root-cause-tracing](references/root-cause-tracing.md): Trace bugs to original trigger.
- [defense-in-depth](references/defense-in-depth.md): Add multi-layer validation.
- [condition-based-waiting](references/condition-based-waiting.md): Use polling, not timeouts.

## Real-World Impact
- **Fix time:** 15-30m vs. 2-3h thrashing.
- **Success rate:** 95% vs. 40%.
- **Regressions:** Near zero vs. common.
