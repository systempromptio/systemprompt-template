# Release — Commits & Pull Requests

How work leaves your machine: commit messages, pull request structure, and changelog discipline. It is the third stage of the Astound development workflow: plan → build → **release** → test.

## Commit messages

Use conventional commits:

```
<type>(<scope>): <imperative summary, ≤72 chars>

<body: what changed and why — wrap at 100 chars>
```

- **Types:** `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `chore`, `build`.
- **Scope** is the route, cartridge, or component touched (`feat(checkout): …`, `fix(cartridge/app_storefront): …`).
- One logical change per commit, matching a step from the `dev_plan` step plan. Never bundle a refactor with a behaviour change.
- Reference the ticket id in the body (`Refs: JIRA-1234`), not the summary line.
- The summary states what the change does, not what you did ("add guest checkout guard", not "added" / "adding").

## Pull requests

- **Title** follows the same conventional format as the lead commit.
- **Description** contains, in order:
  1. **What & why** — two or three sentences, linking the ticket and the approved spec.
  2. **How** — the approach, only where it isn't obvious from the diff.
  3. **Verification** — exactly what was run: Playwright suites (`dev_test`), unit tests, manual steps. A PR without a verification section is not ready for review.
  4. **Out of scope** — follow-ups deliberately deferred.
- Keep PRs reviewable: one spec step per PR where practical. If a PR exceeds ~400 changed lines, say in the description why it could not be split.
- Draft PRs early for visibility; mark ready only after `dev_test` verification passes.

## Changelog

- User-visible changes (`feat`, `fix`, behaviour-affecting `perf`) get a changelog entry in the project's changelog file in the same PR, written for the client's release notes: outcome first, no internal jargon.
- Internal-only changes (`refactor`, `chore`, `test`) do not.

## Where to go deeper

| Topic | Skill |
|---|---|
| Branching, commit format, PR flow (source of truth) | `git_commit` |
| Deploy cartridge code to an instance | `b2c_code` |
| Run and monitor instance jobs | `b2c_job` |
| Managed Runtime deploys | `b2c_mrt` |
| Site import/export bundles | `b2c_site_import_export` |

## Astound rules

The full rule text lives in `dev_rules`. Most relevant at the release stage: **Git workflow**
(follow `git_commit` before any branch, commit, push, or PR), **Pull request conventions**
(the pre-PR audit and description structure), **No unsolicited actions** (never commit, push, or
open a PR without explicit authorization), and **English-only artifacts**.
