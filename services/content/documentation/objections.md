---
title: "Questions & Answers"
description: "Honest answers to common questions enterprises should ask before investing in an AI governance platform -- covering data sovereignty, model lock-in, cost justification, and enterprise scale."
author: "systemprompt.io"
slug: "objections"
keywords: "investment, AI governance, data sovereignty, model agnostic, enterprise, cost justification, vendor risk"
kind: "guide"
public: true
tags: ["enterprise", "sales", "governance", "security"]
published_at: "2026-03-02"
updated_at: "2026-03-02"
after_reading_this:
  - "Understand why AI provider features do not eliminate the need for a governance layer"
  - "Know the data sovereignty, licensing, and deployment options available"
  - "See how the platform supports multiple AI providers"
  - "Evaluate cost vs. build-in-house tradeoffs with a realistic timeline and risk model"
  - "Assess vendor continuity risk and mitigation strategies"
  - "Understand how the platform scales to enterprise teams and integrates with business systems"
---

# Questions & Answers

**These are the questions any enterprise should be asking.** Before investing in AI governance infrastructure, due diligence demands honest answers -- not sales deflection. This page addresses each concern directly, with evidence and counterarguments. Where the concern is valid, we say so.

---

## 1. Will AI Providers Build All of This?

*The question beneath the question: "Why invest in a governance layer that the model provider could eliminate?"*

This is a fair concern. AI providers are expanding their enterprise features -- SSO, audit logs, admin consoles. However, these features govern **their specific product only**. They do not and cannot:

- Govern multiple AI providers from a single control plane
- Run on your servers with your data never leaving the building
- Manage custom MCP servers for your internal systems (ERP, proprietary databases)
- Provide per-department agent governance with hook-driven automation across multiple AI providers
- Offer secret management and encrypted credential storage for arbitrary services

**The pattern is consistent across enterprise software:** platform providers move up the stack, and governance-and-integration layers remain a distinct, durable product category. AWS built compute -- enterprises still needed Terraform. Salesforce built CRM -- enterprises still needed integration platforms.

---

## 2. Code Ownership, Model Privacy, and Data Sovereignty

*The question beneath the question: "Our data must not leave our control. How do we know it does not?"*

**The platform is a library, not a SaaS product.** The core is a Rust codebase that runs on your infrastructure. Your data flows through your servers. The platform vendor does not receive prompts, responses, agent outputs, or any user data. You control the entire runtime.

| Concern | Position |
|---------|----------|
| Where does prompt data go? | To the AI provider you configure. The platform is the router, not a destination. |
| Who can read agent conversations? | Only principals you authorise. The audit log lives in your database. |
| Is the codebase auditable? | Yes. The BSL-1.1 licence permits full source inspection. |
| Can this run air-gapped? | Yes, with a locally-deployed model. |
| Who owns the customisation code? | You do. Skills, agents, hooks, and plugins you create are yours. |

---

## 3. Does This Only Work with One AI Provider?

*The question beneath the question: "We may not want to be locked into a single model provider."*

No. The platform is designed as a provider-agnostic control plane. The architecture separates governance (access control, audit, routing, hooks) from inference (who answers the question).

### Supported Provider Patterns

- **Anthropic** -- Claude models
- **OpenAI** -- GPT-4o, GPT-4 Turbo, o-series models
- **Google** -- Gemini models
- **Local / open-source** -- Ollama, LM Studio, any OpenAI-compatible endpoint
- **Azure OpenAI** -- via the OpenAI-compatible API surface

Switching providers means updating a configuration, not rewriting skills, agents, hooks, or governance rules.

---

## 4. Cost Justification vs. Building In-House

*The question beneath the question: "We have developers. Why not just build this?"*

You could. The question is whether you should.

### What "This" Actually Includes

- Role-based access control with department segmentation
- Plugin, skill, and agent management with marketplace distribution
- Audit logging for every AI interaction
- Secret management with AES-256 encryption, scoped per plugin
- MCP server orchestration with process lifecycle management
- Hook event system (pre/post execution, error handling, automation)
- Analytics pipeline (sessions, tokens, costs, tool usage, content performance)

### The Build vs. Buy Comparison

| Factor | Build In-House | This Platform |
|--------|---------------|-----------------|
| Time to first production agent | 6-18 months | Days to weeks |
| Ongoing maintenance burden | High (your team owns it) | Low (upstream handles platform) |
| Specialised AI tooling expertise | Depends on team | Included |
| Flexibility to customise | Total | High (extensions in Rust, config in YAML) |
| Risk of falling behind evolving AI tools | High | Low (platform tracks MCP spec, tool changes) |

**The strongest objection is cost justification for an organisation with strong engineering capability.** We acknowledge that openly.

---

## 5. What Happens If the Vendor Goes Away?

*The question beneath the question: "We do not want to be stranded by a vendor failure."*

This risk is structurally lower than for most SaaS vendors, because you run the code. If the vendor ceased to exist, you would still have:

- A running production instance on your infrastructure
- The full source code of everything deployed
- Skills, agents, hooks, and configurations in version-controlled YAML and Markdown
- A Rust codebase that can be forked and maintained internally

### Mitigations

- Contractual source escrow for the commercial licence
- A defined internal capability to maintain the fork if needed
- The core platform layer is deliberately thin -- an internal team could sustain it
- All data formats (YAML, Markdown, Git) are industry-standard and human-readable

---

## 6. Can This Scale to Our Teams?

*The question beneath the question: "We have people in multiple departments. Will this work at that scale?"*

The governance model is designed for this. The access control system operates on two axes -- **roles** and **departments** -- which are the natural organisational dimensions of most enterprises.

### Scaling Characteristics

- **User count** scales with the underlying database (PostgreSQL)
- **Concurrent agent sessions** depend on the AI provider's rate limits and your infrastructure sizing
- **Plugin distribution** works through an org-level marketplace with fork-on-write isolation
- **Audit and compliance** -- every action is logged with a comprehensive event trail

---

## 7. Business System Integration

*The question beneath the question: "Will this actually work with our existing data?"*

Yes. The MCP (Model Context Protocol) architecture provides a standard integration pattern:

1. An MCP server wraps your business system API, exposing operations as tools
2. That MCP server is registered in the platform and assigned to the relevant plugin
3. Agents gain access to those tools during their sessions
4. Access control governs which users can invoke agents with system access
5. The secret management system handles API credentials -- encrypted at rest, scoped per plugin

---

## 8. How Does This Relate to Other AI Tools?

*The question beneath the question: "Big providers already have plugin systems. Why another layer?"*

**This is not an alternative to those tools. It is the governance layer that sits above all of them.**

| Layer | What It Does | Examples |
|-------|-------------|----------|
| **AI Providers** | Supply the intelligence | Anthropic, OpenAI, Google, open-source models |
| **AI Surfaces** | Give users access through interfaces | Enterprise chat tools, coding assistants, productivity platforms |
| **AI Governance** | Controls who uses what, tracks everything, manages secrets, enforces policy | **This platform** |

Without a governance layer, each AI surface operates as an island -- fragmented visibility, no unified audit trail, no cross-provider policy enforcement.

---

## Summary: The Honest Assessment

| Concern | Risk Level | Mitigation |
|---------|-----------|------------|
| AI provider competes directly | Low | Different market segment -- governance vs. intelligence |
| Data sovereignty | Low | Self-hosted, source-available, your infrastructure |
| Model lock-in | Low | Provider-agnostic architecture, MCP open standard |
| Cost vs. build-in-house | **Medium** | 12-18 month acceleration; depends on priorities |
| Vendor continuity | Low-Medium | Self-hosted, BSL-1.1 source, forkable, standard formats |
| Enterprise scale | Low | Designed for role/department access control |
| Business system integration | Medium | MCP pattern is sound; server code requires development |
| vs. Big-vendor tools | Low | Not competing -- governance layer above all of them |

**The strongest objection is cost justification against building in-house** for an organisation with engineering capability. We say that openly because honest assessment builds more trust than a perfect sales pitch.
