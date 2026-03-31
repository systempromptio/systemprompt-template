# Rebranding Checklist

To rebrand this platform for a new client, edit the files below. After changes, run `just publish` to rebuild all assets.

---

## 1. Theme Configuration

**`services/web/config/theme.yaml`** (primary branding file)
- `branding.name` / `branding.display_name` — Brand name
- `branding.domain` — Client domain
- `branding.title` — Page title (e.g. "Brand — Tagline")
- `branding.description` — Meta description
- `branding.copyright` — Footer copyright
- `branding.themeColor` — Hex color for browser theme-color meta
- `branding.support_email` — Support contact
- `branding.session_storage_key` — Unique key per deployment
- `colors.light.accent` / `colors.dark.accent` — Accent hex values
- `card.gradient.*` — Card gradient rgba values (match brand color)
- `shadows.*.accent` — Accent shadow rgba values

## 2. CSS Tokens (Admin — OKLCH)

**`storage/files/css/admin/01-tokens-primitives.css`**
- `--sp-primitive-brand-*` (50–950) — 11-stop OKLCH brand scale
- Change the hue parameter (last number) to match your brand color
- Orange ~50, Purple ~300, Blue ~260, Teal ~175, Green ~155

## 3. CSS Tokens (Public — HSL)

**`storage/files/css/core/variables.css`**
- `--sp-brand-hue` — Single HSL hue value for the entire brand scale
- `--sp-color-primary` / `--sp-color-primary-hover` / `--sp-color-primary-dark` — Hex values
- `--sp-color-primary-rgb` — RGB triplet for rgba() usage
- `--sp-syntax-keyword` / `--sp-syntax-function` / `--sp-syntax-operator` — Code syntax colors
- `--sp-client-*` — Client-specific hex values
- `--sp-card-bg-gradient` / `--sp-card-shadow-highlight` — Brand-tinted gradients
- `--sp-brand-gradient` / `--sp-brand-gradient-reverse` — Full gradient definitions

## 4. Site Configuration

**`services/web/config.yaml`**
- `name`, `title`, `copyright`

**`services/web/metadata.yaml`**
- SEO and structured data brand references

**`services/content/config.yaml`**
- `structured_data.organization.name` and `.url`

## 5. Deployment Profiles

**`services/profiles/production/profile.yaml`**
- `site.name`, `api_server_url`

**`services/profiles/local/profile.yaml`**
- `site.name`

## 6. Agents

**`services/agents/*.yaml`** (3 files)
- `organization` field — set to brand name

## 7. Plugins

**`services/plugins/*/config.yaml`** (8 files)
- `author.name` and `author.email`

## 8. Workspace

**`Cargo.toml`**
- `workspace.package.authors`

## 9. Image Assets

**`storage/files/images/`**
- `logo.svg`, `logo-white.svg`, `logo.png`, `logo.webp`
- `favicon.ico`, `favicon-16x16.png`, `favicon-32x32.png`, `favicon-96x96.png`
- `apple-touch-icon.png`, `icon.svg`

## 10. Content and Documentation

- `README.md` — Project description
- `CLAUDE.md` — Brand name rule
- `demo/` — Demo scripts and architecture docs
- `services/content/documentation/` — Brand references in docs
- `services/skills/` — Brand references in skill examples

---

## Build After Rebranding

```bash
just publish    # Rebuild templates, CSS bundle, JS bundle, copy assets
just build      # Verify Rust compiles
just start      # Start and verify in browser
```
