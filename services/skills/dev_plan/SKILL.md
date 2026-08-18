# Plan — Open Spec & Architecture

Plan before you build. Every non-trivial change to a Commerce Cloud / Storefront Next project starts with a short open spec that a reviewer can approve before any code exists. This skill defines how to write that spec, how to record architectural decisions, and how to slice the work into reviewable steps.

## When to Use

Use this skill at the start of any feature, refactor, or integration task — anything bigger than a one-file fix. It is the first stage of the Astound development workflow: **plan → build → release → test** (see `dev_build`, `dev_release`, `dev_test`).

## Workflow

1. **Restate the requirement.** One paragraph, in your own words, including what is explicitly out of scope. If the ticket and the code disagree, surface the conflict — do not guess.
2. **Gather context first.** Read the relevant cartridge/route/component code and check the project knowledge bank (workshop transcripts, Jira tickets, Confluence pages via the `knowledge-bank` MCP server) for prior decisions before proposing anything.
3. **Write the open spec.** A short markdown document containing:
   - **Problem** — what changes for the user or the business.
   - **Approach** — the shape of the solution: which routes, components, services, and data flows are touched.
   - **Alternatives rejected** — one line each and why.
   - **Risks & unknowns** — with how each will be resolved.
   - **Step plan** — an ordered list of independently reviewable steps, each small enough for a single pull request.
4. **Record decisions.** Any choice that constrains future work (library selection, data model, API contract) gets a dated decision record in the spec so it survives the ticket.
5. **Get the spec approved before building.** Share it in the pull request or ticket; do not start `dev_build` work on an unapproved spec unless the task is trivial.

## Planning Rules

- Prefer extending an existing pattern in the project over introducing a new one; name the file you are modelling on.
- Every step in the step plan must state its verification ("covered by Playwright spec X", "verified by unit test Y").
- Specs are living documents: when reality diverges during build, update the spec in the same pull request.
- Keep specs short — one page is the target. A spec nobody reads governs nothing.

## Planning method

Before writing the spec, gather context to ~90% confidence — enough to answer: which files and
functions are relevant, how the existing code works there, which patterns and conventions the
codebase follows, and which dependencies are involved. Breadth first, then drill down; document
concrete file paths and function names; note existing tests and similar implementations. Where
genuinely multiple approaches exist, present two or three with trade-offs and let the reviewer
pick — do not research to 100% certainty before surfacing options. When a spec needs a diagram
(system flow, sequence of calls), follow `drawio_diagrams` for a committable, re-generatable one.

## Where to go deeper

| Topic | Skill |
|---|---|
| Diagrams for specs and docs | `drawio_diagrams` |
| Jira / Confluence mechanics | `atlassian`, `atlassian_doc_templating` |
| Full house rules | `dev_rules` |

## Astound rules

The full rule text lives in `dev_rules`. Most relevant at the planning stage: **Autonomous
verification** (research, don't ask — verify capability claims before recording them), **No
unsolicited actions** (no Jira/Confluence writes or pushes without explicit approval), **Bug fix
planning** (reproduce before planning a fix), and **No symptom patches** (when the right fix is
architectural, plan the architectural fix).
