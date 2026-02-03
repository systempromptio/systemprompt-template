---
title: "Licensing"
description: "Understand how systemprompt.io licensing works: the core is BSL-1.1, the template is MIT and fully yours."
author: "systemprompt.io"
slug: "licensing"
keywords: "licensing, bsl, business source license, mit, open source, pricing, enterprise, self-hosted"
image: "/files/images/docs/licensing.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Licensing

The systemprompt.io core is licensed under the Business Source License 1.1 (BSL-1.1). The template you download and customize is MIT licensed—it's yours. This is an open-core model: you can view, modify, and self-host the code, but there are restrictions on competing with systemprompt.io directly.

**TL;DR:**

- The core is BSL-1.1, converts to Apache 2.0 after four years
- The template is MIT licensed—download it, edit it, it's yours
- CLI requires Google or GitHub signup (currently free, may change)
- Don't compete with or redistribute the core

## The Business Source License

The BSL-1.1 is a source-available license created by MariaDB. It allows you to use the software freely with one key restriction: you cannot offer systemprompt.io as a competing service.

**Key terms from the LICENSE file:**

| Term | Value |
|------|-------|
| License | Business Source License 1.1 |
| Additional Use Grant | You may use the Licensed Work in production |
| Change Date | Four years from each version's release |
| Change License | Apache License, Version 2.0 |

After four years, each version automatically converts to Apache 2.0—a permissive open-source license with no restrictions. This means code released today becomes fully open source in 2030.

## What You Can Do

The BSL-1.1 with our Additional Use Grant permits broad usage:

- **Use in production** — Deploy systemprompt.io for your business, products, and customers
- **Build proprietary products** — Create commercial software on top of the platform
- **Self-host** — Run on your own infrastructure
- **Modify the template** — The template is MIT licensed, customize it freely
- **View all code** — Full source transparency, no hidden components

## What You Cannot Do

The license restricts competitive use:

- **Compete with systemprompt.io** — Don't offer a substantially similar product or service
- **Redistribute the core as a service** — Don't resell hosting of the core to third parties
- **Remove license notices** — Keep the BSL-1.1 notice in the core

If your use case falls into a gray area, contact the licensing team (see Links section).

## Your Code Ownership

**The template is MIT licensed—it's yours.** When you download the systemprompt-template, you can modify it however you want. It becomes your code. We never see it.

- **MIT licensed** — The template is permissively licensed, no restrictions on your use
- **Completely private** — Your customizations never leave your infrastructure
- **No telemetry** — We don't collect or transmit your code
- **Full ownership** — Everything you build on the template is your intellectual property

Only the core (systemprompt-core) is protected under BSL-1.1. Everything wrapping and using the core—your services, agents, business logic, API routes, database schemas—is self-owned.

## Pricing Model

### Early Access (Current)

Using the CLI requires signing up with **Google or GitHub**. This authentication is currently free.

During early access, you can:

- Clone the template and build
- Self-host on your infrastructure
- Use all CLI features

**Pricing may change.** After early access, we may charge for commercial use. The BSL-1.1 license allows us to introduce pricing while ensuring you always have a path to Apache 2.0.

### Cloud Hosting

systemprompt.io Cloud provides managed infrastructure:

| Feature | Included |
|---------|----------|
| One-click deploy | Yes |
| Managed PostgreSQL | Yes |
| Automatic backups | Yes |
| Custom domains | Yes |
| Email support | Yes |

Cloud hosting starts at **$29/month**. This is optional—you can always self-host for free.

### Enterprise Licensing

Organizations needing predictable, stable licensing should contact us:

- **Volume licensing** — Pricing for large deployments
- **SLA guarantees** — Uptime and support commitments
- **Custom terms** — Licensing tailored to your compliance needs
- **Dedicated support** — Direct access to the team

Contact the enterprise team (see Links section) for inquiries.

## Frequently Asked Questions

### Can I use systemprompt.io for my SaaS product?

**Yes.** You can build and sell products powered by the platform. The restriction is on offering systemprompt.io itself as a service—not on building products that use it.

### What happens when the license converts to Apache 2.0?

After four years, that version becomes Apache 2.0 licensed with no restrictions. You can fork it, modify it, sell it, or do anything else Apache 2.0 permits.

### Do I need to open-source my code?

**No.** The template is MIT licensed—your code is yours. The BSL-1.1 only applies to the core, not to your customizations or business logic.

### Can I contribute to systemprompt.io?

Yes. Contributions to systemprompt-core are welcome under the project's contributor agreement. Your contributions will be licensed under BSL-1.1 initially and Apache 2.0 after conversion.

### What if I need different license terms?

Contact the licensing team (see Links section). We offer commercial licenses for use cases that don't fit the standard BSL-1.1 terms.

---

## Contact

| Purpose | Contact |
|---------|---------|
| Licensing questions | See Links section |
| Enterprise inquiries | See Links section |
| Sales | See Links section |