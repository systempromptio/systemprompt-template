---
title: "Web Assets"
description: "CSS, JavaScript, fonts, and images pipeline."
keywords:
  - assets
  - css
  - javascript
  - fonts
  - images
category: build
---

# Web Assets

CSS, JavaScript, fonts, and images pipeline.

> **Help**: `{ "command": "core playbooks show build_web-assets" }`

---

## CRITICAL: CSS File Location

**CSS files MUST be placed in `storage/files/css/`**

```
storage/files/css/           <- PUT CSS FILES HERE
├── core/
│   ├── variables.css
│   ├── fonts.css
│   └── reset.css
├── components/
│   ├── header.css
│   ├── footer.css
│   └── mobile-menu.css
├── feature-base.css
├── feature-rust.css
├── feature-cli.css
├── feature-memory.css
├── docs.css
├── blog.css
└── ...
```

**DO NOT put CSS in:**
- ~~`extensions/web/assets/css/`~~ - Not used by asset system
- ~~`web/dist/css/`~~ - Generated output, will be overwritten

---

## Asset Pipeline

```
storage/files/css/custom.css     <- SOURCE: Create file here
        ↓
extensions/web/src/extension.rs  <- REGISTER: Add to required_assets()
        ↓
systemprompt infra jobs run copy_extension_assets
        ↓
web/dist/css/custom.css          <- OUTPUT: Copied here
        ↓
/css/custom.css                  <- SERVED: Available at this URL
```

---

## URL Routing

| URL Path | Served From | Contains |
|----------|-------------|----------|
| `/css/*` | `web/dist/css/` | Stylesheets |
| `/js/*` | `web/dist/js/` | Scripts |
| `/fonts/*` | `web/dist/fonts/` | Fonts |
| `/files/*` | `storage/files/` | Images only |

> **Never** use `/files/css/` or `/files/js/`. CSS/JS are served from `/css/*` and `/js/*`.

---

## Adding CSS (Step-by-Step)

### Step 1: Create the CSS file

```bash
# Create file in the correct location
touch storage/files/css/my-page.css
```

### Step 2: Register in extension.rs

```rust
// File: extensions/web/src/extension.rs
// In required_assets() function:

fn required_assets(&self, paths: &SystemPaths) -> Vec<AssetDefinition> {
    let storage_css = paths.storage_files().join("css");
    vec![
        // ... existing assets ...
        AssetDefinition::css(
            storage_css.join("my-page.css"),
            "css/my-page.css",
        ),
    ]
}
```

### Step 3: Rebuild and publish

```bash
just build
systemprompt infra jobs run publish_pipeline
```

### Step 4: Reference in template

```html
<link rel="stylesheet" href="/css/my-page.css">
```

---

## CSS Organization

| Category | Files | Purpose |
|----------|-------|---------|
| Core | `core/variables.css`, `core/fonts.css`, `core/reset.css` | CSS variables, fonts, reset |
| Components | `components/header.css`, `components/footer.css` | Shared UI components |
| Pages | `feature-base.css`, `docs.css`, `blog.css` | Page-specific styles |
| Feature-Specific | `feature-rust.css`, `feature-cli.css`, `feature-memory.css` | Individual feature page styles |

### CSS Import Pattern

Feature page CSS files import the base:

```css
/* feature-rust.css */
@import url('/css/feature-base.css');

/* Page-specific styles below */
.rust-mesh-animation { ... }
```

---

## Adding JavaScript

1. **Create file**: `storage/files/js/custom.js`

2. **Register in extension**:
   ```rust
   AssetDefinition::js(storage_js.join("custom.js"), "js/custom.js")
   ```

3. **Rebuild and publish**:
   ```bash
   just build && systemprompt infra jobs run publish_pipeline
   ```

4. **Reference in template**:
   ```html
   <script src="/js/custom.js" defer></script>
   ```

---

## Adding Fonts

**Location**: `storage/files/fonts/`

```
storage/files/fonts/
├── OpenSans/
│   ├── OpenSans-Regular.ttf
│   └── OpenSans-Bold.ttf
└── Zepto/
    └── Zepto-Regular.ttf
```

**CSS reference**:
```css
@font-face {
  font-family: 'OpenSans';
  src: url('/fonts/OpenSans/OpenSans-Regular.ttf') format('truetype');
  font-weight: 400;
  font-display: swap;
}
```

---

## Adding Images

```bash
systemprompt core files upload ./my-image.webp
```

Or copy directly:
```bash
cp my-image.webp storage/files/images/blog/
```

Reference in content:
```yaml
image: "/files/images/blog/my-image.webp"
```

---

## Output Structure

```
web/dist/                   <- GENERATED (never edit directly)
├── index.html
├── sitemap.xml
├── css/                    <- CSS copied here on publish
├── js/                     <- JS copied here on publish
├── fonts/                  <- Fonts copied here on publish
├── features/{slug}/
├── blog/{slug}/
├── documentation/{slug}/
└── legal/{slug}/
```

> **Never** edit `web/dist/` directly. It's regenerated on every publish.

---

## Troubleshooting

### CSS not updating?

1. **Check file location**: Must be in `storage/files/css/`
2. **Check registration**: Must be in `extension.rs` `required_assets()`
3. **Rebuild**: `just build` (if you changed Rust code)
4. **Republish**: `systemprompt infra jobs run publish_pipeline`
5. **Hard refresh**: Ctrl+Shift+R in browser

### CSS 404 error?

1. File not in `storage/files/css/`
2. File not registered in `required_assets()`
3. Typo in filename or path

---

## Quick Reference

| Task | Command |
|------|---------|
| Full publish | `systemprompt infra jobs run publish_pipeline` |
| Assets only | `systemprompt infra jobs run copy_extension_assets` |
| Upload image | `systemprompt core files upload <path>` |

---

-> See [Web Content](web-content.md) for creating content.
-> See [Web Templates](web-templates.md) for template configuration.
-> See [Web Pages](web-pages.md) for feature pages.
