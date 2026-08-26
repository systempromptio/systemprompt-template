---
title: "Conversation History & Search"
description: "Search full AI conversation history — prompts, responses, tools, timestamps — scoped to your identity: yourself always, your organization's members as an owner, everything as an admin or auditor."
author: "Astound Digital"
slug: "enterprise-conversation-history"
keywords: "history, conversations, search, transcripts, audit, manager, fts"
kind: "guide"
public: true
tags: ["enterprise", "audit", "admin"]
published_at: "2026-08-26"
updated_at: "2026-08-26"
after_reading_this:
  - "Search your own AI conversation history from /admin/history"
  - "Understand who can see whose history: self, org owners, admins and auditors"
  - "Know that snippets are redacted and out-of-scope lookups are refused"
related_docs:
  - title: "Audit Trail, Traceability & Observability"
    url: "/documentation/enterprise-audit-observability"
  - title: "User & Access Management"
    url: "/documentation/enterprise-user-access"
---

# Conversation History & Search

**TL;DR:** `/admin/history` gives every signed-in user full-text search over their own AI conversations — prompts, responses, tools used, timestamps. Who you can search is decided by identity: yourself always, the members of your organization if you are its owner or admin, and everything if you hold the `admin` or `auditor` role. Snippets are redacted, and asking for a user outside your scope is refused with 403.

## Searching your history

Open `/admin/history` and search. Matching is full-text over session transcripts (a generated search index, not substring scanning), and results return the conversation with its timestamps and the tools that were used. The same capability is available programmatically at `/admin/api/history/search`.

## Who sees whose history

| You are | You can search |
|---|---|
| Any signed-in user | Your own conversations, always |
| An organization owner or admin | Your own, plus every member of your organization(s) |
| `admin` or `auditor` role | Everyone — the unrestricted audit view |

Scope is enforced at the query layer: a request naming a `user_id` outside your scope returns **403**, and the result set can never widen beyond the allowlist your identity resolves to.

## Redaction carries through

Snippets rendered in search results apply the same display-layer redaction as the rest of the audit surface — values the safety scanners flagged (secrets, PII) stay masked. Reviewing history does not re-expose the sensitive data that was caught on the way through. See [Content Safety & Guardrails](/documentation/enterprise-safety-guardrails).

## Relationship to the audit trail

History search reads the same conversation spine the [audit trail](/documentation/enterprise-audit-observability) records — it is the user- and manager-facing view over data that already exists, not a second store. The knowledge-platform search experience (reusing conversations as knowledge) builds on this same surface — see the [roadmap](/documentation/enterprise-roadmap).

## Verified evidence

Every capability on this page is proven by automated tests run against a seeded instance. To replicate: `just start`, then `just e2e-seed --reset`, then the command in the table.

| Ref | Verified behaviour | Replicate with |
|---|---|---|
| REQ-045 | Scope resolution: self always visible; org owner sees members; non-owner sees only self; admin/auditor unrestricted | `just test-unit` |
| REQ-045 | The history page is reachable signed-in and refused signed-out | `just e2e-req REQ-045` |

![The conversation history page with full-text search](/files/images/evidence/req-045-history.png)
