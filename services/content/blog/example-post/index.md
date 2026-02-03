---
title: "Example Blog Post"
description: "This is an example blog post demonstrating the content system. Replace it with your own content."
slug: "example-post"
kind: "blog"
public: true
author: "Template Author"
published_at: "2025-01-01"
tags: ["example", "getting-started"]
category: "guide"
---

# Example Blog Post

This is a placeholder blog post demonstrating the content management system. **Delete this post and create your own content.**

## How Blog Posts Work

Blog posts are Markdown files stored in `services/content/blog/`. Each post lives in its own directory with an `index.md` file.

### Directory Structure

```
services/content/blog/
├── example-post/
│   └── index.md          # This file
└── your-new-post/
    ├── index.md          # Post content
    └── images/           # Optional images
        └── hero.png
```

### Frontmatter Fields

Every blog post needs frontmatter at the top:

```yaml
---
title: "Your Post Title"
description: "SEO description for the post"
slug: "url-slug"
kind: "blog"
public: true
author: "Your Name"
published_at: "2025-01-15"
tags: ["tag1", "tag2"]
category: "announcement"  # or "article", "guide"
---
```

### Publishing

After creating your post, publish it to the database:

```bash
systemprompt core content publish
```

## Next Steps

1. Delete this example post
2. Create your first real blog post
3. Run `systemprompt core content publish`
4. Visit `/blog` to see your content

For more details, see the playbook:

```bash
systemprompt core playbooks show guide_documentation
```
