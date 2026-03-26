---
title: "Installation"
description: "Install and configure the systemprompt.io agentic governance platform. Covers system requirements, building from source, database setup, and running the platform."
author: "systemprompt.io"
slug: "installation"
keywords: "installation, setup, build, configuration, database, requirements"
kind: "guide"
public: true
tags: ["installation", "setup"]
published_at: "2026-02-18"
updated_at: "2026-02-18"
after_reading_this:
  - "Set up the required system dependencies"
  - "Build the platform from source"
  - "Configure the database and environment"
  - "Start the platform and access the dashboard"
related_docs:
  - title: "Getting Started"
    url: "/documentation/getting-started"
  - title: "Dashboard"
    url: "/documentation/dashboard"
---

# Installation

**TL;DR:** systemprompt.io is a Rust-based platform that requires Rust, PostgreSQL, and the `just` command runner. Build with `just build`, configure your database, then start with `just start`.

## System Requirements

| Requirement | Minimum Version |
|-------------|----------------|
| Rust | Latest stable (with `cargo`) |
| PostgreSQL | 14+ |
| just | Latest ([casey/just](https://github.com/casey/just)) |
| Node.js | 18+ (for asset bundling) |
| Git | 2.x |

## Step 1: Clone the Repository

```bash
git clone <repository-url>
cd systemprompt-claude-marketplace
```

The repository uses a git submodule for the core library in `core/`. Initialize submodules:

```bash
git submodule update --init --recursive
```

## Step 2: Configure the Environment

The platform uses profile-based configuration stored in `.systemprompt/profiles/local/secrets.json`. This file contains your database URL and other secrets.

Create the local profile directory:

```bash
mkdir -p .systemprompt/profiles/local
```

Create `.systemprompt/profiles/local/secrets.json`:

```json
{
  "database_url": "postgresql://user:password@localhost:5432/systemprompt"
}
```

You can also use a `.env` file at the project root for additional environment variables.

## Step 3: Set Up the Database

Create the PostgreSQL database:

```bash
createdb systemprompt
```

Run database migrations:

```bash
just migrate
```

This applies all SQL migrations from the `extensions/web/schema/` directory.

## Step 4: Build the Platform

Build the entire workspace:

```bash
just build
```

The build system automatically detects whether the database is reachable. If it is, it compiles with live database verification. If not, it falls back to offline mode using cached query metadata.

For a release build:

```bash
just build --release
```

## Step 5: Publish Assets

Compile templates, bundle CSS/JS, and prerender static content:

```bash
just publish
```

This runs the following jobs in order:

1. `compile_admin_templates` — Compiles Handlebars admin templates
2. `bundle_admin_css` — Bundles CSS files from `storage/files/css/`
3. `bundle_admin_js` — Bundles JavaScript files from `storage/files/js/`
4. `copy_extension_assets` — Copies bundled assets to `web/dist/`
5. `content_prerender` — Prerenders documentation and other content pages

## Step 6: Start the Platform

```bash
just start
```

This starts the platform using the local profile. The server runs on `http://localhost:8080` by default.

## Verify the Installation

1. Open `http://localhost:8080` — You should see the homepage
2. Open `http://localhost:8080/documentation` — Documentation pages should render
3. Open `http://localhost:8080/admin/login` — The admin login page should appear

## CLI Discovery

The `systemprompt` CLI is your primary tool for managing the platform. Discover available commands:

```bash
systemprompt --help
systemprompt core --help
systemprompt infra --help
systemprompt admin --help
```

Key commands:

| Command | Purpose |
|---------|---------|
| `systemprompt infra services start` | Start the platform |
| `systemprompt infra db migrate` | Run database migrations |
| `systemprompt core skills list` | List all skills |
| `systemprompt core plugins generate` | Generate marketplace plugins |
| `systemprompt infra logs view --level error` | View error logs |

## Docker Deployment

Build and run with Docker:

```bash
just docker-build
just docker-run
```

This builds the Docker image and runs it on port 8080.

## Troubleshooting

**Build fails with database errors** — Ensure PostgreSQL is running and the `database_url` in your secrets file is correct. If the database is unavailable, the build falls back to offline mode automatically.

**`just` command not found** — Install just: `cargo install just`

**Assets not appearing** — Run `just publish` after any changes to templates, CSS, or JS files.

**Migrations fail** — Check that the database exists and the user has permissions. Review `extensions/web/schema/` for the migration files.

**Port 8080 already in use** — Stop any other services on port 8080 or configure an alternate port in your environment.
