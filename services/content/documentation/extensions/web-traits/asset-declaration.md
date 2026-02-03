---
title: "Asset Declaration"
description: "Declare CSS, JavaScript, fonts, and images for your extension."
author: "SystemPrompt Team"
slug: "extensions/web-traits/asset-declaration"
keywords: "assets, css, javascript, fonts, images, extensions"
image: ""
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Asset Declaration

Extensions declare static assets (CSS, JS, fonts, images) via `declares_assets()` and `required_assets()`.

## The Methods

```rust
fn declares_assets(&self) -> bool {
    true
}

fn required_assets(&self, paths: &dyn AssetPaths) -> Vec<AssetDefinition> {
    vec![
        // Asset definitions
    ]
}
```

## AssetPaths

```rust
pub trait AssetPaths: Send + Sync {
    fn storage_files(&self) -> &Path;
    fn output_dir(&self) -> &Path;
}
```

- `storage_files()` - Source directory (`storage/files/`)
- `output_dir()` - Output directory (`web/dist/`)

## AssetDefinition

```rust
pub struct AssetDefinition {
    source: PathBuf,
    destination: &'static str,
    asset_type: AssetType,
    required: bool,
}

pub enum AssetType {
    Css,
    JavaScript,
    Font,
    Image,
}
```

## Creating Assets

### CSS

```rust
AssetDefinition::css(
    paths.storage_files().join("css/main.css"),
    "css/main.css"
)
```

### JavaScript

```rust
AssetDefinition::js(
    paths.storage_files().join("js/app.js"),
    "js/app.js"
)
```

### Fonts

```rust
AssetDefinition::font(
    paths.storage_files().join("fonts/inter.woff2"),
    "fonts/inter.woff2"
)
```

### Images

```rust
AssetDefinition::image(
    paths.storage_files().join("images/logo.svg"),
    "files/images/logo.svg"
)
```

## Builder Pattern

For advanced configuration:

```rust
AssetDefinition::builder(
    paths.storage_files().join("css/optional.css"),
    "css/optional.css",
    AssetType::Css
)
.optional()  // Don't fail if missing
.build()
```

## Complete Example

```rust
impl Extension for WebExtension {
    fn declares_assets(&self) -> bool {
        true
    }

    fn required_assets(&self, paths: &dyn AssetPaths) -> Vec<AssetDefinition> {
        let css = paths.storage_files().join("css");
        let js = paths.storage_files().join("js");
        let fonts = paths.storage_files().join("fonts");

        vec![
            // Core CSS
            AssetDefinition::css(css.join("core/variables.css"), "css/core/variables.css"),
            AssetDefinition::css(css.join("core/reset.css"), "css/core/reset.css"),
            AssetDefinition::css(css.join("core/typography.css"), "css/core/typography.css"),

            // Component CSS
            AssetDefinition::css(css.join("components/header.css"), "css/components/header.css"),
            AssetDefinition::css(css.join("components/footer.css"), "css/components/footer.css"),
            AssetDefinition::css(css.join("components/cards.css"), "css/components/cards.css"),

            // JavaScript
            AssetDefinition::js(js.join("main.js"), "js/main.js"),
            AssetDefinition::js(js.join("analytics.js"), "js/analytics.js"),

            // Fonts
            AssetDefinition::font(fonts.join("inter-regular.woff2"), "fonts/inter-regular.woff2"),
            AssetDefinition::font(fonts.join("inter-bold.woff2"), "fonts/inter-bold.woff2"),
        ]
    }
}
```

## Asset Pipeline

Assets flow through this pipeline:

```
storage/files/css/main.css
         |
    Extension declares asset
         |
    Asset validation
         |
    Copy to output
         |
web/dist/css/main.css
```

## URL Routing

Assets are served at these paths:

| Type | Source | URL |
|------|--------|-----|
| CSS | `storage/files/css/` | `/css/*` |
| JS | `storage/files/js/` | `/js/*` |
| Fonts | `storage/files/fonts/` | `/fonts/*` |
| Images | `storage/files/images/` | `/files/images/*` |

## CSS Organization

Recommended structure:

```
storage/files/css/
├── core/
│   ├── variables.css
│   ├── reset.css
│   └── typography.css
├── components/
│   ├── header.css
│   ├── footer.css
│   └── cards.css
├── pages/
│   ├── homepage.css
│   └── blog.css
└── features/
    ├── docs-nav.css
    └── search.css
```

## CLI Commands

```bash
# List declared assets
systemprompt web assets list

# Verify assets exist
systemprompt web assets verify

# Copy assets to output
systemprompt web assets publish
```