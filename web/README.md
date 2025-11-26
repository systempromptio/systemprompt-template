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
# Theme generation
npm run theme:generate    # Generate theme CSS from YAML

# Build commands
npm run build            # Full production build
npm run build:web        # Web-only build

# Sitemap generation
npm run sitemap:generate # Generate sitemap.xml

# Linting
npm run lint             # ESLint check

# Type checking
npx tsc --noEmit         # Type check without building
```

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
# Build web for production
just web-build-prod

# Docker build
docker build -t systemprompt-web ./web
docker run -p 80:80 systemprompt-web
```

## Sitemap Generation

The sitemap is automatically generated during build, including both:

1. **Static routes** - From your documentation and pages
2. **Dynamic blog routes** - Fetched from the API

```bash
# Manual sitemap generation (requires API running)
just web-sitemap
```

The sitemap is generated at `/dist/sitemap.xml` and includes:
- All static documentation pages
- All published blog posts
- Proper priority and change frequency for SEO

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
│   ├── generate-theme.js             # Theme generation
│   └── generate-sitemap.js           # Sitemap generation
├── Dockerfile                         # Docker build
├── nginx.conf                         # Nginx config
├── build-production.sh                # Production build script
├── ingest-content.sh                  # Content ingestion
└── package.json
```

## Environment Configuration

Set via `.env` file:

```env
# API URL for blog content fetching
VITE_API_URL=http://localhost:8080

# Hostname for sitemap generation
# (currently hardcoded in script, configure in scripts/generate-sitemap.js)
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

1. Ensure API is running during build
2. Check `scripts/generate-sitemap.js` API URL
3. Build again: `npm run build`

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
- Monitor with `just web-sitemap` before deployment

## Just Commands

```bash
just web-build              # Build web frontend
just web-dev                # Run dev server
just web-build-prod         # Production build
just web-sitemap            # Generate sitemap
just ingest-markdown path   # Ingest content
just ingest-all             # Ingest all content types
just stack-up               # Docker compose up
just stack-down             # Docker compose down
just stack-logs             # View stack logs
```

## License

Part of SystemPrompt OS - See root repository for details.
