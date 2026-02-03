---
title: "Early Access"
description: "systemprompt.io is in open early access. Expect quirks—we're committed to fixing every bug and vulnerability you report."
author: "systemprompt.io"
slug: "early-access"
keywords: "early access, beta, open source, bugs, vulnerabilities, security, bsl, licensing, rust"
image: "/files/images/docs/early-access.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Early Access

systemprompt.io is in **open early access**. The platform is available for anyone to download, build, and deploy—but it's still evolving. This page explains what that means for you and how we're handling this phase.

**TL;DR:**

- Open early access: download, build, and use it today
- Expect quirks, bugs, and breaking changes
- We take every bug report and security vulnerability seriously
- Licenses are free under BSL-1.1 during early access (may change)

## Built in Rust

systemprompt.io is written in Rust. This matters for early access because Rust binaries are self-contained—once you have a working build, it stays working.

**It breaks on your terms.** Early access software changes frequently. But with Rust, you're not chasing runtime updates, package manager conflicts, or dependency hell. Your binary from six months ago still runs exactly as it did then.

**No migration pressure.** When we release breaking changes, you upgrade when you're ready—not when your infrastructure forces you. Pin to a version, deploy it, and forget about it until you choose to update.

**Rust binaries stay built.** No interpreter versions to manage. No virtual environments to recreate. No "works on my machine" mysteries. The binary you compiled is the binary you run, on any compatible system, indefinitely.

This gives you control during a period when the upstream project is evolving rapidly. Take the stability you need while we iterate.

## What is Open Early Access?

Open early access means the project is publicly available and actively developed, but not yet stable. We're building in the open—you can see every commit, every feature, and every fix as it happens.

This is different from a closed beta:

| Aspect | Closed Beta | Open Early Access |
|--------|-------------|-------------------|
| Access | Invite only | Anyone |
| Source | Hidden | Fully visible |
| Feedback | Private channels | Public issues |
| Stability | Curated experience | Raw development |

We chose open early access because we believe transparency builds trust. You can evaluate the code yourself, see how we handle issues, and decide if the project meets your standards.

## What to Expect

During early access, you should expect:

**Quirks and rough edges.** Some features work differently than documented. Some workflows feel clunky. The CLI might give unhelpful error messages. We're aware of many issues and actively fixing them.

**Bugs.** Things will break. Database migrations might fail. Services might crash. Features might not work as expected. This is normal for early-stage software—we're prioritizing shipping features over polish.

**Breaking changes.** APIs, configuration formats, and database schemas may change without deprecation periods. We'll document breaking changes in release notes, but you should expect to update your code when upgrading.

**Incomplete documentation.** Some features are documented sparsely or not at all. If you can't figure something out, open an issue—we'll either fix the docs or explain the feature.

## Our Commitment

This is a serious project with serious commitment. We take every bug report and vulnerability disclosure seriously.

**Bug reports.** When you report a bug, we commit to:

- Acknowledging the report within 48 hours
- Triaging and prioritizing based on severity
- Providing updates on fix progress
- Crediting reporters in release notes (if desired)

Report bugs via GitHub Issues (see Links section).

**Security vulnerabilities.** We treat security issues with urgency:

- Email security issues to the security team (see Links section)
- Do not disclose vulnerabilities publicly until patched
- We aim to patch critical vulnerabilities within 72 hours
- We'll coordinate disclosure timing with reporters

We follow responsible disclosure practices and appreciate researchers who give us time to fix issues before going public.

## Current Version

The latest published version is available on GitHub:

- **Releases:** See Related Code section for the releases page
- **Template:** See Related Code section for the project template

Check the releases page for:

- Version numbers and release dates
- Changelog with new features and fixes
- Breaking changes and migration guides
- Download links and installation instructions

We recommend pinning to specific versions in production rather than tracking `main`.

## Licensing During Early Access

During early access, systemprompt.io is free to use under the Business Source License 1.1 (BSL-1.1).

**Current terms:**

- Free for all use cases (commercial and non-commercial)
- CLI authentication via Google or GitHub (free)
- Self-host on your infrastructure at no cost
- No usage limits or feature restrictions

**This may change.** After early access concludes, we may introduce pricing for commercial use. The BSL-1.1 license structure allows us to evolve pricing while ensuring:

- You always have access to the source
- Each version converts to Apache 2.0 after four years
- Your existing deployments continue working

See the Licensing documentation for complete license terms and enterprise options.

## Get Involved

Early access is the best time to shape the project:

- **Report issues** — Every bug report improves the platform
- **Request features** — Tell us what you need
- **Contribute code** — PRs welcome for fixes and improvements
- **Share feedback** — Good and bad, we want to hear it

Start a conversation via GitHub Issues or email (see Links section).

---

## Contact

| Purpose | Contact |
|---------|---------|
| Bug reports | See Links section (GitHub Issues) |
| Security vulnerabilities | See Links section (security email) |
| General questions | See Links section (hello email) |
| Feature requests | See Links section (GitHub Issues) |