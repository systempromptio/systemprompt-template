---
title: "Build & CI/CD Pipeline"
description: "Build pipeline, asset compilation, and deployment workflow for the Foodles AI governance platform. From source to production with just build and just publish."
author: "systemprompt.io"
slug: "ci-cd"
keywords: "build, CI/CD, pipeline, deployment, assets, compilation, Docker"
kind: "guide"
public: true
tags: ["devops", "build", "ci-cd"]
published_at: "2026-03-25"
updated_at: "2026-03-25"
after_reading_this:
  - "Understand the build pipeline from source to production binary"
  - "Know how to compile templates, bundle CSS/JS, and publish assets"
  - "Run database migrations as part of the deployment workflow"
related_docs:
  - title: "Architecture Overview"
    url: "/documentation/architecture"
  - title: "Deployment Models"
    url: "/documentation/deployment-models"
  - title: "Configuration & Profiles"
    url: "/documentation/configuration"
---

# Build & CI/CD Pipeline

The platform builds from source to a single binary with two primary commands. The build system uses `just` (a command runner) to orchestrate compilation, asset bundling, and deployment.

## Build Commands

### `just build`

Compiles the Rust binary and all extensions:

```bash
just build
```

This produces a single `systemprompt` binary containing the web server, API, job runner, template engine, and all governance logic. No external runtime dependencies.

### `just publish`

Compiles templates, bundles CSS/JS, and copies all assets to the web output directory:

```bash
just publish
```

The publish pipeline runs these steps in order:

1. **compile_admin_templates** — Compile Handlebars templates into prerendered HTML
2. **bundle_admin_css** — Concatenate and minify CSS files from `storage/files/css/`
3. **bundle_admin_js** — Concatenate and minify JavaScript from `storage/files/js/`
4. **copy_extension_assets** — Copy bundled files and static assets to `web/dist/`
5. **content_prerender** — Prerender documentation, feature pages, and content pages

Order matters — bundles must be built before `copy_extension_assets` copies them to the output directory.

## Asset Pipeline

### CSS

All CSS source files live in `storage/files/css/`. To add a new CSS file:

1. Create the file in `storage/files/css/`
2. Register it in `extensions/web/src/extension.rs` in the `required_assets()` function
3. Run `just publish` to compile and copy to output

### JavaScript

JavaScript source files live in `storage/files/js/`. The bundler concatenates them into `admin-bundle.js` for the admin interface.

### Static Files

Images, fonts, and other static assets in `storage/files/` are copied directly to `web/dist/` during the publish step.

## Database Migrations

Database schema migrations are SQL files in `extensions/web/schema/`, numbered sequentially:

```
extensions/web/schema/
  001_initial.sql
  002_users.sql
  ...
  026_usage_aggregations.sql
```

Migrations run automatically on startup. The platform tracks which migrations have been applied and only runs new ones.

## Deployment Workflow

A typical deployment follows these steps:

```bash
# 1. Pull latest changes
git pull origin main

# 2. Build the binary
just build

# 3. Compile and bundle assets
just publish

# 4. Start services (migrations run automatically)
just start
```

For Docker deployments, the Dockerfile handles steps 1-3 during the image build. Step 4 happens when the container starts.

## CI/CD Integration

The build pipeline is deterministic — given the same source, it produces the same output. This makes it straightforward to integrate with any CI/CD system:

1. **Build stage** — `just build` compiles the binary
2. **Asset stage** — `just publish` bundles CSS/JS and prerenders content
3. **Test stage** — Run tests against the compiled binary
4. **Package stage** — Build Docker image with the binary and assets
5. **Deploy stage** — Push image and restart services

## Health Checks

After deployment, verify the platform is running:

```bash
# Error check
systemprompt infra logs view --level error --since 5m
```
