---
title: "Welcome to Your Project"
description: "Get started with your new SystemPrompt-powered application"
author: "Author"
slug: "welcome"
keywords: "welcome, getting started, systemprompt"
image: ""
kind: "article"
public: true
tags: ["welcome", "getting-started"]
published_at: "2025-01-01"
updated_at: "2025-01-01"
---

# Welcome to Your Project

Welcome to your new SystemPrompt-powered application. This template provides everything you need to build a modern, AI-enhanced web application.

## What's Included

### AI Agents
Your project comes with a base **Assistant** agent ready to help users with their questions and tasks. Admin and Infrastructure agents are also included for system management.

### Blog System
A full-featured static blog with:
- Markdown content support
- Automatic sitemap generation
- Link tracking and analytics
- SEO optimization

### MCP Servers
Two MCP (Model Context Protocol) servers for extending AI capabilities:
- **Admin MCP**: Analytics, user management, and system monitoring
- **Infrastructure MCP**: Deployment and synchronization tools

## Getting Started

### 1. Configure Your Branding

Update the following files to match your project:

```
services/web/config.yaml      # Theme colors, fonts, branding
services/web/metadata.yaml    # SEO and social metadata
```

### 2. Customize Your Agent

Edit your assistant's system prompt:

```
services/agents/assistant.yaml
```

### 3. Add Your Content

Create new blog posts and documentation:

```
services/content/blog/       # Blog articles
services/content/docs/       # Documentation
services/content/legal/      # Legal pages
```

### 4. Run Your Application

```bash
# Start the database
just db-up

# Run migrations
just migrate

# Start the server
just start
```

## Next Steps

- Explore the `/docs` section for detailed documentation
- Check out the [SystemPrompt Core](https://github.com/systempromptio/systemprompt-core) repository
- Review the `CLAUDE.md` file for development guidelines

Happy building!
