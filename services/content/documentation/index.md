---
title: "Foodles AI Governance Platform"
description: "Documentation for the Foodles AI governance platform. Deploy, secure, distribute, and observe AI usage across your entire organisation."
author: "systemprompt.io"
slug: ""
keywords: "foodles, ai governance, deployment, security, marketplace, analytics, plugins, skills"
kind: "guide"
public: true
tags: ["documentation"]
published_at: "2026-02-18"
updated_at: "2026-03-25"
after_reading_this:
  - "Understand how the Foodles AI governance platform manages enterprise AI usage"
  - "Navigate to documentation for any platform capability"
  - "Know how deployment, governance, marketplace, and analytics work together"
---

# Foodles AI Governance Platform

> **Start with the presentation:** [Enterprise AI Governance Layer](/documentation/presentation)

Welcome to the Foodles AI Governance Platform documentation. This platform provides enterprise-grade governance infrastructure for AI usage — deploying, securing, distributing, and observing AI across your entire organisation.

## Start Here

- [Presentation](/documentation/presentation) — Interactive slide deck covering the full platform story

## Introduction & Integration

*Understand what this platform is, what it does, and how it integrates with Claude.*

- [Introduction to the Platform](/documentation/introduction) — What this site is and how the demo becomes your production deployment
- [Platform Overview](/documentation/platform-overview) — The three pillars: insight, governance, and integration
- [Anthropic Partnership](/documentation/anthropic-partnership) — How we work with the Claude ecosystem
- [Integration: Claude Code](/documentation/integration-claude-code) — Plugin system and marketplace distribution for the CLI
- [Integration: Claude Cowork](/documentation/integration-claude-cowork) — Governance for collaborative AI sessions (Research Preview)
- [Integration: Claude AI](/documentation/integration-claude-ai) — Extending governance to claude.ai via MCP

## Proposal

- [Partnership Proposal](/documentation/proposal) — What the platform does, licensing, and how to engage
- [Engagement Summary](/documentation/engagement-summary) — Partnership and cost breakdown
- [Licensing](/documentation/proposal-licensing) — Licence terms and IP ownership
- [Questions & Answers](/documentation/objections) — Data sovereignty, cost justification, and enterprise scale

## DevOps & Deployment

*Deploy the platform on your infrastructure.*

- [Architecture Overview](/documentation/architecture) — How the platform is structured and how components interact
- [Installation & Setup](/documentation/installation) — System requirements, setup, and deployment options
- [Deployment Models](/documentation/deployment-models) — Sidecar, gateway, embedded, and multi-tenant architectures
- [Configuration & Profiles](/documentation/configuration) — Profile-based config for dev, staging, and production
- [Scaling Architecture](/documentation/scaling) — Enterprise scaling strategies for high-volume deployments
- [Build & CI/CD Pipeline](/documentation/ci-cd) — Build pipeline, asset compilation, and deployment workflow

## Governance & Security

*Control what AI can do and who can use it.*

| Capability | Description |
|------------|-------------|
| [Access Control & RBAC](/documentation/access-control) | Role and department-based permissions governing all resources |
| [Authentication](/documentation/authentication) | Login, passkeys, magic links, and session management |
| [Secrets & Encryption](/documentation/secrets) | Environment variables, secret management, and encryption at rest |
| [MCP Servers](/documentation/mcp-servers) | Tool servers with per-server OAuth2 and port isolation |
| [Tool Governance](/documentation/tool-governance) | MCP tool access control, execution logging, and event hooks |
| [Hooks & Automation](/documentation/hooks) | Event-driven triggers that fire when things happen in the system |
| [Audit Trails & Events](/documentation/events) | System event log with filtering, audit trail, and compliance reporting |
| [Rate Limiting](/documentation/rate-limiting) | Per-role rate limits to control resource consumption |

## Marketplace & Skill Management

*Create, fork, and distribute AI capabilities.*

- [Plugins](/documentation/plugins) — Governed bundles of skills, agents, and tools
- [Skills](/documentation/skills) — Reusable capabilities that define what agents can do
- [Agents](/documentation/agents) — AI workers configured with instructions and tools
- [Marketplace](/documentation/marketplace) — Plugin discovery, ranking, and visibility management
- [Marketplace Versions](/documentation/marketplace-versions) — Version management and release history
- [Forking & Customization](/documentation/forking) — Fork any resource without affecting the baseline
- [Distribution Channels](/documentation/distribution-channels) — Git, JSON API, and Claude Code plugin format

## Analytics & Observability

*Know what AI you have deployed, how it is used, and what it costs.*

- [Dashboard](/documentation/dashboard) — Real-time metrics with charts, activity feed, and health indicators
- [Cost Tracking](/documentation/cost-tracking) — Token consumption and usage cost tracking
- [Activity Tracking](/documentation/activity-tracking) — 13 activity categories covering every AI interaction
- [Metrics Reference](/documentation/metrics-reference) — Complete reference of all dashboard metrics and data structures
- [Content Analytics](/documentation/content-analytics) — Link performance, campaign tracking, and content journeys
- [CLI Analytics](/documentation/cli-analytics) — Full CLI reference for querying metrics from the terminal
- [Gamification & Leaderboard](/documentation/gamification) — XP, ranks, achievements, streaks, and department scores

## Reference

- [Getting Started](/documentation/getting-started) — Quick start guide for new users
- [User Management](/documentation/users) — User administration, role assignment, and access control
- [Organizations](/documentation/organization) — Organisation and multi-tenancy configuration
- [Jobs & Background Tasks](/documentation/jobs) — Background job queue monitoring and management
