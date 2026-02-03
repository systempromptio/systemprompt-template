---
title: "Workflow Recipes"
description: "Complete workflow examples for common tasks."
author: "SystemPrompt"
slug: "guide-recipes"
keywords: "recipes, workflows, examples, tutorials"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Workflow Recipes

Complete workflow examples for common tasks.

---

## Create and Publish Blog Post

```bash
# 1. Create markdown file
cat > services/content/blog/my-post.md << 'EOF'
---
title: "My Article"
description: "Brief description for SEO"
author: "Author Name"
slug: "my-article"
keywords: "keyword1, keyword2"
kind: "article"
image: "/files/images/blog/featured.webp"
public: true
tags: ["topic"]
published_at: "2026-01-28"
updated_at: "2026-01-28"
---

# My Article

Content here...
EOF
```

```json
{ "command": "infra jobs run publish_pipeline" }
{ "command": "core content verify my-article --source blog" }
```

---

## Add Custom CSS

```bash
# 1. Create CSS file
cat > storage/files/css/custom.css << 'EOF'
.my-component {
  color: var(--text-primary);
  padding: 1rem;
}
EOF

# 2. Register in extension.rs (add to required_assets)
# AssetDefinition::css(storage_css.join("custom.css"), "css/custom.css"),
```

```json
{ "command": "infra jobs run copy_extension_assets" }
```

Reference in template: `<link rel="stylesheet" href="/css/custom.css">`

---

## Add Custom JavaScript

```bash
# 1. Create JS file
cat > storage/files/js/custom.js << 'EOF'
document.addEventListener('DOMContentLoaded', () => {
  console.log('Custom JS loaded');
});
EOF

# 2. Register in extension.rs (add to required_assets)
# AssetDefinition::js(storage_js.join("custom.js"), "js/custom.js"),
```

```json
{ "command": "infra jobs run copy_extension_assets" }
```

Reference in template: `<script src="/js/custom.js" defer></script>`

---

## Update Homepage

Edit `services/web/config/homepage.yaml` for homepage data, then:

```json
{ "command": "infra jobs run publish_pipeline" }
```

---

## Quick Content Refresh (No Prerender)

For quick content updates without full prerendering:

```json
{ "command": "infra jobs run publish_pipeline" }
```

---

## Quick Reference

| Recipe | Key Command |
|--------|-------------|
| Full publish | `infra jobs run publish_pipeline` |
| Quick publish (no prerender) | `infra jobs run publish_pipeline` |
| Add CSS/JS | `infra jobs run copy_extension_assets` |
| Ingest content only | `infra jobs run blog_content_ingestion` |
| Index images | `infra jobs run file_ingestion` |