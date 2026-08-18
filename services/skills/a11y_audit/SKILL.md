# Accessibility Audit (WCAG 2.2)

Lead Accessibility QA: pedantic code-level audit for WCAG 2.2 (Level A & AA) and WAI-ARIA authoring practices. Prioritize screen reader semantics, keyboard access, and focus management.

**Stack:** React (Storefront Next). **Patterns:** WAI-ARIA.

**Flow:** **SCOPE** → **CONTEXT** → **STATIC ANALYSIS** → **SPEC CROSS-CHECK** → **REPORT**

---

## Step 1 — Determine scope

Ask the user or infer: specific file, component, page route, or PR diff.

Use Grep / Glob to locate target files.

---

## Step 2 — Gather context

Find the component/page FSD or ISD — via the `atlassian` skill (Confluence search / `get-page`) or a
working copy committed in the repo — and read it. Note any requirements that constrain accessibility
behavior.

---

## Step 3 — Static code analysis

Read each target file. Check against the WCAG checklist below.

For **each** finding record: **location** (file + line or element), **WCAG criterion** (number + short name), **priority**, **impact**, **recommendation** (concise fix or pattern).

### WCAG checklist (A & AA focus)

**1. Perceivable**

- (1.1.1) Non-text Content — text alternatives for non-text content.
- (1.3.1) Info and Relationships — semantic markup; structure exposed to AT.
- (1.3.2) Meaningful Sequence — logical reading order when layout is unconventional.
- (1.4.3 / 1.4.11) Contrast — text and non-text contrast (including UI components, focus indicators).

**2. Operable**

- (2.1.1) Keyboard — all functionality operable via keyboard.
- (2.1.2) No Keyboard Trap — focus can always leave the component.
- (2.4.3) Focus Order — logical tab order.
- (2.4.4 / 2.4.6) Link Purpose & Headings — descriptive links; heading structure.
- (2.5.3) Label in Name — visible label is included in accessible name.

**3. Understandable**

- (3.2.1 / 3.2.2) On Focus / On Input — no unexpected context changes.
- (3.3.1 / 3.3.2) Errors & Labels — errors identifiable; labels/instructions for inputs.

**4. Robust**

- (4.1.2) Name, Role, Value — correct roles, states, properties for custom controls.
- (4.1.3) Status Messages — live regions for important dynamic updates.

**5. Complex patterns & focus**

- WAI-ARIA: dialogs, drawers, menus, popovers — `aria-modal`, labeling, focus trap, Escape, focus return.
- (2.4.11) Focus Not Obscured (AA) — focused item not fully hidden by sticky/overlay content (flag when visible in code/CSS).
- (2.5.5 / 2.5.8) Target Size — minimum touch target (and spacing) where applicable.

### Priority (default `Major` if unsure)

| Priority | When | Example |
|---|---|---|
| Blocker | Core flow blocked for keyboard/AT; no workaround | Keyboard trap in checkout; primary action is a `<div>` with no keyboard support |
| Critical | Severe semantic failure; AT cannot use or understand | Unlabeled `<input>` / `<select>`; interactive control with no accessible name |
| Major | Significant context loss or broken focus | Modal does not trap focus; wrong ARIA role for the pattern |
| Minor | Violation with limited impact on core tasks | Skipped heading levels; missing `aria-current` on active nav link |
| Trivial | Validational nit; no practical AT impact | Redundant `role="button"` on `<button>`; duplicate `aria-label` matching visible text |

---

## Step 4 — Cross-reference specifications

Before flagging: verify whether the implementation is driven by an FSD/ISD.
If a11y fix contradicts a functional requirement, flag the conflict.

---

## Step 5 — Compile findings

### Output format

**1. Executive summary**

- Total findings by priority (counts).
- Top 3 highest-impact issues (one line each).
- Overall a11y posture assessment.
- List any items marked **Manual verification required** (focus timing, contrast in real theme, runtime SR behavior).

**2. Findings table**

| ID | Location | WCAG | Priority | Issue | Recommendation | Spec conflict? |
|---:|---|---|---|---|---|---|
| 1 | `path/file.tsx:42` | 2.1.1 Keyboard | Major | … | … | No / Yes — cite doc |

**3. Manual verification required** (if any)

Bullet list: what to check in browser, and why static analysis was insufficient.

---

## Rules

- Default priority when uncertain: **Major**.
- Cite **WCAG success criterion numbers** in findings (e.g. 2.4.3), not only section names.
- Separate **code defect** from **spec conflict** from **needs manual check** — do not merge categories.
- Do not claim contrast or motion compliance from code alone when styles/theme/runtime states are unknown; mark for manual verification instead.
