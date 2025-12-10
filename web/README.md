# SystemPrompt Web Frontend + RAG Blog System

A modern React 19 + TypeScript + Vite frontend with integrated Retrieval-Augmented Generation (RAG) support for dynamic blog content.

## Features

✅ **Static Content** - Documentation, legal pages, marketing content (build-time)
✅ **Dynamic Blog System** - Auto-discovered, categorized blog posts from database (runtime)
✅ **SEO Optimized** - Dynamic sitemap generation with static + blog routes
✅ **Markdown Support** - Built-in markdown rendering with syntax highlighting
✅ **API Integration** - Seamless REST API integration for blog content
✅ **Production Ready** - Docker containerization, nginx configuration

## Architecture

### Technology Stack

- **Frontend**: React 19 + TypeScript + Vite 7
- **Routing**: React Router 7
- **Markdown**: react-markdown + remark-gfm
- **Styling**: Tailwind CSS 4 + custom theme
- **API Client**: Fetch API with type-safe service layer
- **Build**: Vite with sitemap generation

### Content Organization

```
/web/src/content/
  ├── documentation/     → Static docs (built-in)
  ├── pages/            → Marketing pages (built-in)
  └── legal/            → Legal pages (built-in)

/content/
  ├── blog/             → Blog posts (ingested to DB)
  ├── articles/         → Articles (ingested to DB)
  └── tutorials/        → Tutorials (ingested to DB)
```

## Development

### Prerequisites

- Node.js 20+
- npm or yarn

### Getting Started

```bash
# Install dependencies
npm install

# Start dev server with hot reload
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

The dev server runs on `http://localhost:5173`

### Available Scripts

```bash
# Development
npm run dev              # Start Vite dev server with hot reload

# Theme generation (called by build system)
npm run theme:generate   # Generate theme CSS from YAML

# Linting
npm run lint             # ESLint check

# Preview
npm run preview          # Preview production build

# Type checking
npx tsc --noEmit         # Type check without building
```

**Note**: Build orchestration is now handled by the Rust CLI (`systemprompt-build`). See "Building & Deployment" section below.

## Blog System

### Adding Blog Posts

1. Create markdown files in `/content/blog/`:

```markdown
---
title: "My Blog Post"
author: "Author Name"
published: "2025-01-15"
category: "tutorials"
tags: ["rust", "web", "beginner"]
excerpt: "Short description for previews"
---

# My Blog Post

Your content here...
```

2. Ingest content into the database:

```bash
# Ingest blog posts
just ingest-markdown /content/blog

# Or ingest all content types
just ingest-all
```

3. The blog will automatically appear:
   - `/blog` - Blog list page
   - `/blog/{slug}` - Individual blog post

### Frontmatter Format

| Field | Required | Description |
|-------|----------|-------------|
| `title` | Yes | Post title |
| `author` | No | Author name |
| `published` | No | Publication date (YYYY-MM-DD) |
| `category` | No | Content category |
| `tags` | No | Array of tags |
| `excerpt` | No | Short description for previews |

The slug is automatically generated from the filename (e.g., `2025-01-15-my-post.md` → `my-post`).

### Blog Routes

```
/blog                      # Blog list (all posts)
/blog/{slug}              # Individual blog post
/api/v1/blog              # API: List posts
/api/v1/blog/{slug}       # API: Get single post
/api/v1/rag/categories    # API: List categories
/api/v1/rag/tags          # API: List tags
```

## Building & Deployment

### Local Development with API

Start the API and web together:

```bash
# Terminal 1: Start API
just api

# Terminal 2: Start web dev server
just web-dev

# Terminal 3 (optional): Stream logs
just log
```

Then visit `http://localhost:5173`

### Docker Deployment

```bash
# Start entire stack (API + Web + Database)
just stack-up

# View logs
just stack-logs

# Stop stack
just stack-down
```

Accesses:
- Web: `http://localhost:5173`
- API: `http://localhost:8080`

### Production Build

```bash
# Build with Rust CLI (recommended)
systemprompt-build web --mode production

# Or use just command
just start --web

# Docker build (auto-builds web if needed)
./infrastructure/scripts/build.sh release --docker
```

## Sitemap Generation

Sitemap generation is handled by the Rust scheduler as a background job. It runs automatically and includes:

1. **Static routes** - From your documentation and pages
2. **Dynamic blog routes** - Fetched from the database

The sitemap is generated at `/dist/sitemap.xml` and includes:
- All static documentation pages
- All published blog posts
- Proper priority and change frequency for SEO

**Note**: Sitemap generation now runs as a scheduled Rust job, ensuring it uses fresh database content.

## API Integration

### Blog Service

Located in `/src/services/rag.service.ts`:

```typescript
import { RagService } from '@/services/rag.service';

// List blog posts
const posts = await RagService.listBlogPosts(limit, offset);

// Get single post by slug
const post = await RagService.getBlogPost('my-post');

// List categories
const categories = await RagService.listCategories();

// List tags
const tags = await RagService.listTags();
```

### Blog Components

- `<BlogListPage />` - Blog list at `/blog`
- `<BlogPostPage />` - Blog post detail at `/blog/:slug`
- `<MarkdownPage />` - Renders markdown content

## Project Structure

```
web/
├── src/
│   ├── components/
│   │   ├── pages/
│   │   │   └── MarkdownPage.tsx      # Generic markdown renderer
│   │   ├── markdown/
│   │   │   └── MarkdownContent.tsx   # Markdown rendering
│   │   └── ...
│   ├── pages/
│   │   ├── blog/
│   │   │   ├── BlogList.tsx          # Blog list page
│   │   │   └── BlogPost.tsx          # Blog post detail
│   │   ├── documentation/            # Existing docs
│   │   └── ...
│   ├── services/
│   │   ├── rag.service.ts            # Blog API client
│   │   └── ...
│   ├── layouts/
│   │   └── PageLayout.tsx            # Shared layout
│   ├── types/
│   │   └── markdown.ts               # Type definitions
│   ├── utils/
│   │   ├── env.ts                    # Environment helpers
│   │   └── seo.ts                    # SEO metadata
│   └── routes.tsx                    # Route definitions
├── scripts/
│   ├── generate-theme.js             # Theme generation (called by Rust)
│   └── theme-schema.json             # Theme validation schema
├── Dockerfile                         # Docker build
├── nginx.conf                         # Nginx config
└── package.json
```

## Environment Configuration

Set via `.env` file:

```env
# API URL for blog content fetching
VITE_API_URL=http://localhost:8080
```

## SEO & Performance

### Caching Strategy

**Static Assets** (1 year):
- JS, CSS, images, fonts are immutable

**HTML** (no cache):
- Dynamic content fetched each time

**API Responses** (1 hour with stale-while-revalidate):
- Blog posts cached for 1 hour
- Stale content served while fetching fresh data

### Dynamic Sitemap

Blog routes are fetched from the API during build, ensuring:
- All published posts appear in sitemap
- Correct lastmod timestamp
- Proper priority (0.8 for blog posts)

## Troubleshooting

### Blog posts not loading

1. Check API is running: `just api`
2. Verify content ingested: `just ingest-markdown /content/blog`
3. Check browser console for API errors
4. Verify `VITE_API_URL` environment variable

### Sitemap empty or missing dynamic routes

1. Sitemap is now generated by Rust scheduler
2. Check that scheduled jobs are running
3. Verify database has content

### Markdown not rendering properly

1. Check markdown syntax
2. Verify frontmatter format
3. Check console for parsing errors

## Contributing

When adding new features:

1. **Components** - Reuse `<MarkdownPage>` for consistency
2. **Types** - Add to `/src/types/` with clear interfaces
3. **Services** - Use `/src/services/` for API calls
4. **Routing** - Update `/src/routes.tsx` with new routes

## Performance Tips

- Use `VITE_API_URL` for production API endpoint
- Enable Docker caching for faster builds
- Use nginx gzip compression
- Implement CDN for static assets
- Rust build system provides faster, type-safe builds

## Just Commands

```bash
just start --web            # Build and start API with web assets
just dev                    # Run Vite dev server
just ingest path            # Ingest content from path
just docker-build           # Build Docker images
just docker-run             # Run Docker stack
```

**Build Commands** (via Rust CLI):
```bash
systemprompt-build web --mode development   # Dev build
systemprompt-build web --mode production    # Prod build
systemprompt-build web --mode docker        # Docker build
systemprompt-build theme                    # Theme only
systemprompt-build validate                 # Validate build
```

## License

Part of SystemPrompt OS - See root repository for details.
