---
title: "Extensions"
description: "Build anything with SystemPrompt extensions: websites, MCP servers, background jobs, CLI tools. You own the binary, you control the code."
author: "SystemPrompt Team"
slug: "extensions"
keywords: "extensions, rust, ownership, binary, mcp, cli, jobs, web"
image: "/files/images/docs/extensions.svg"
kind: "guide"
public: true
tags: ["extensions", "development", "architecture"]
published_at: "2026-01-30"
updated_at: "2026-02-01"
after_reading_this:
  - "Understand you own a complete Rust binary, not a hosted service"
  - "Identify the four extension domains: library, web, MCP, CLI"
  - "Choose the right extension type for your use case"
  - "Navigate to specific extension documentation"
---

# Extensions

When you build with SystemPrompt, you own the binary. Not an account on a hosted service. A real Rust binary that compiles from source code you control, runs on infrastructure you choose, and executes functionality you define.

Extensions are how you add that functionality. Write Rust code that implements API routes, database schemas, background jobs, MCP tools, or custom CLI commands. Compile it into your binary. Deploy it wherever you want.

## Extension Domains

Extensions group into four primary domains:

| Domain | Binary Type | Use Case |
|--------|-------------|----------|
| [**Library**](/documentation/extensions/domains/library) | Compiles into main | Database, API, jobs, providers |
| [**Web**](/documentation/extensions/domains/web) | Compiles into main | Page data, templates, assets |
| [**MCP**](/documentation/extensions/domains/mcp) | Standalone binary | AI agent tools via MCP protocol |
| [**CLI**](/documentation/extensions/domains/cli) | Standalone binary | Command-line utilities |

### Library Extensions

Library extensions compile directly into your main binary. They implement the Extension trait and provide:

- Database schemas and migrations
- HTTP API routes
- Background jobs
- LLM and tool providers
- Configuration validation

See [Library Extensions](/documentation/extensions/domains/library) for details.

### Web Extensions

Web extensions handle content rendering and static site generation. They provide:

- Page data providers (template variables)
- Component renderers (HTML fragments)
- Content data providers (enrichment)
- Template data extenders
- Page prerenderers (static pages)
- RSS and sitemap generators
- Asset declarations

See [Web Extensions](/documentation/extensions/domains/web) for details.

### MCP Extensions

MCP extensions are standalone binaries that expose tools for AI agents via the Model Context Protocol. They:

- Run as separate processes
- Listen on TCP ports
- Serve tool requests via MCP
- Enable Claude and other AI clients

See [MCP Extensions](/documentation/extensions/domains/mcp) for details.

### CLI Extensions

CLI extensions are standalone binaries for automation:

- Custom command-line tools
- External integrations
- Utility commands
- Agent-invokable scripts

See [CLI Extensions](/documentation/extensions/domains/cli) for details.

## Documentation Structure

### Domains
- [Library Extensions](/documentation/extensions/domains/library) - Compiled into main binary
- [Web Extensions](/documentation/extensions/domains/web) - Content and rendering
- [MCP Extensions](/documentation/extensions/domains/mcp) - AI agent tools
- [CLI Extensions](/documentation/extensions/domains/cli) - Command-line tools

### Core Traits
- [Extension Trait](/documentation/extensions/traits/extension-trait) - Complete 30+ method reference
- [Schema Extension](/documentation/extensions/traits/schema-extension) - Database schemas
- [API Extension](/documentation/extensions/traits/api-extension) - HTTP routes
- [Job Extension](/documentation/extensions/traits/job-extension) - Background jobs
- [Provider Extension](/documentation/extensions/traits/provider-extension) - LLM/Tool providers
- [Config Extension](/documentation/extensions/traits/config-extension) - Configuration

### Web Traits
- [PageDataProvider](/documentation/extensions/web-traits/page-data-provider) - Template variables
- [ComponentRenderer](/documentation/extensions/web-traits/component-renderer) - HTML fragments
- [ContentDataProvider](/documentation/extensions/web-traits/content-data-provider) - Content enrichment
- [TemplateDataExtender](/documentation/extensions/web-traits/template-data-extender) - Final modifications
- [PagePrerenderer](/documentation/extensions/web-traits/page-prerenderer) - Static pages
- [FrontmatterProcessor](/documentation/extensions/web-traits/frontmatter-processor) - Frontmatter parsing
- [RSS & Sitemap](/documentation/extensions/web-traits/rss-sitemap-provider) - Feeds
- [Asset Declaration](/documentation/extensions/web-traits/asset-declaration) - CSS/JS/Fonts

### Lifecycle
- [Registration](/documentation/extensions/lifecycle/registration) - inventory crate, macros
- [Discovery](/documentation/extensions/lifecycle/discovery) - ExtensionRegistry
- [Dependencies](/documentation/extensions/lifecycle/dependencies) - Dependency management
- [Initialization](/documentation/extensions/lifecycle/initialization) - AppContext integration

### Internals
- [Typed Extensions](/documentation/extensions/internals/typed-extensions) - Compile-time safety
- [Extension Builder](/documentation/extensions/internals/extension-builder) - Type-safe registration
- [Error Handling](/documentation/extensions/internals/error-handling) - LoaderError, ConfigError

### MCP Deep Dives
- [Tool Structure](/documentation/extensions/mcp/tool-structure) - Modular tool organization
- [Resources](/documentation/extensions/mcp/resources) - MCP resources and templates
- [Skills](/documentation/extensions/mcp/skills) - Skill integration for AI tools
- [Responses](/documentation/extensions/mcp/responses) - Response formatting patterns
- [AI Integration](/documentation/extensions/mcp-ai-integration) - Full AI service guide

## Quick Start

1. Choose your extension domain based on use case
2. Create the extension crate
3. Implement required traits
4. Register with `register_extension!`
5. Link in the template's `src/lib.rs`
6. Build and test

## Directory Structure

```
extensions/
├── web/                    # Library extension
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── extension.rs
│   │   ├── api/
│   │   ├── jobs/
│   │   └── services/
│   └── schema/
├── mcp/
│   └── systemprompt/       # MCP server
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
└── cli/
    └── discord/            # CLI tool
        ├── Cargo.toml
        └── src/
            └── main.rs
```

## Ownership Model

The SystemPrompt template is your project. When you clone it:

- **Release cycle** - Upgrade core when you choose
- **Deployment** - Run anywhere
- **Modification** - Fork, customize, add features
- **Distribution** - Ship to customers, embed in products
