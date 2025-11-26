# SystemPrompt Theme System

Single source of truth for all website styling, branding, and assets.

## Available Themes

- **`web.yaml`** - Active theme (currently: monochrome light theme)
- **`web-default.yaml`** - Warm orange theme (original default)
- **`web-blue.yaml`** - Cool cyberpunk blue theme
- **`web-light.yaml`** - Clean monochrome light theme (current active)

## Quick Start

### Switch Themes

```bash
# Switch to blue theme
cp templates/themes/web-blue.yaml crates/services/web.yaml
cd web && npm run build

# Switch to default (orange) theme
cp templates/themes/web-default.yaml crates/services/web.yaml
cd web && npm run build

# Switch to light (monochrome) theme
cp templates/themes/web-light.yaml crates/services/web.yaml
cd web && npm run build
```

### Development

```bash
# Edit theme
vim crates/services/web.yaml

# Auto-regenerate and hot reload
cd web && npm run dev
```

## Theme Structure

```yaml
branding:          # App metadata, logos, favicon
fonts:             # Font families and @font-face declarations
colors:            # Color palettes (light + dark modes)
typography:        # Font sizes and weights
spacing:           # Spacing scale (xs → xxl)
radius:            # Border radius scale
shadows:           # Box shadow definitions
animation:         # Animation durations
zIndex:            # Z-index layering
layout:            # Header/sidebar dimensions
card:              # Card-specific styling
mobile:            # Mobile responsive overrides
touchTargets:      # Touch target sizes
```

## Validation

Themes are validated against JSON Schema on every build:

```bash
npm run theme:generate
```

**Output:**
```
🎨 Generating theme from YAML...
🔍 Validating theme schema...
✅ Theme validation passed
📝 Writing CSS...
✅ Generated: src/styles/theme.generated.css
📝 Writing TypeScript config...
✅ Generated: src/theme.config.ts
✨ Theme generation complete!
```

**If validation fails:**
```
❌ Theme validation failed:
  - /colors/light/primary/rgb: must be array
    Details: {"type":"array"}
```

## Type Safety

TypeScript types are auto-generated from the theme:

```typescript
import { theme } from '@/theme.config'
import type { Theme, ColorPalette } from '@/types/theme.types'

// Fully typed theme access
const primaryColor = theme.colors.light.primary.hsl
const spacing = theme.spacing.md
```

## Creating Custom Themes

1. **Copy an existing theme:**
   ```bash
   cp templates/themes/web-default.yaml templates/themes/web-custom.yaml
   ```

2. **Edit values:**
   ```yaml
   colors:
     light:
       primary:
         hsl: "hsl(280, 85%, 65%)"  # Purple
         rgb: [178, 69, 237]
   ```

3. **Validate and build:**
   ```bash
   cp templates/themes/web-custom.yaml crates/services/web.yaml
   cd web && npm run build
   ```

4. **Check for errors:**
   - Schema validation catches structural errors
   - TypeScript compilation catches type errors
   - Visual inspection confirms design changes

## Theme Variables

All theme values become CSS variables:

```css
/* Auto-generated in theme.generated.css */
--color-primary: hsl(220, 13%, 18%);
--color-primary-rgb: 30, 41, 59;
--spacing-md: 16px;
--font-heading: 'OpenSans', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
--card-gradient-start: rgba(100, 116, 139, 0.08);
```

Use in components:

```tsx
// Via Tailwind classes
<div className="bg-primary text-white p-md rounded-lg">

// Via CSS variables
<div style={{ color: 'var(--color-primary)' }}>

// Via theme config
<div style={{ color: theme.colors.light.primary.hsl }}>
```

## Schema Reference

**Required top-level keys:**
- `branding`, `fonts`, `colors`, `typography`, `spacing`, `radius`
- `shadows`, `animation`, `zIndex`, `layout`, `card`, `mobile`, `touchTargets`

**Color structure:**
```yaml
colors:
  light:
    primary:
      hsl: "hsl(h, s%, l%)"    # Required HSL string
      rgb: [r, g, b]            # Required RGB array or null
```

**Font structure:**
```yaml
fonts:
  body:
    family: "FontName"          # Required
    fallback: "system-font"     # Required
    files:                      # Required array
      - path: "/path/to.ttf"
        weight: 400             # 100-900
        style: "normal"         # normal|italic|oblique
```

**Validation rules:**
- Theme color must be hex: `#1E293B` (example)
- Font weights: 100-900
- Durations: `150ms` or `0.5s`
- All sizes use valid CSS units

## Files Generated

**CSS Variables** (`web/src/styles/theme.generated.css`):
- All theme values as CSS custom properties
- @font-face declarations
- Dark mode overrides
- Mobile responsive overrides
- Keyframe animations
- Utility classes

**TypeScript Config** (`web/src/theme.config.ts`):
- Typed theme object
- Exported as `const` for tree-shaking
- Auto-updated on build

**Gitignored:**
- `theme.generated.css` - Regenerated from YAML
- `theme.config.ts` - Regenerated from YAML

## Hot Reload

In development, the Vite plugin watches `web.yaml`:

1. Edit `crates/services/web.yaml` (active theme)
2. Save file
3. Theme auto-regenerates
4. Browser hot reloads with new theme

Note: Theme presets are stored in `templates/themes/` and copied to `crates/services/web.yaml` when activated.

## Troubleshooting

**Theme not changing:**
```bash
# Hard refresh browser (Ctrl+Shift+R)
# Or clear cache and rebuild
rm -rf web/dist
cd web && npm run build
```

**Validation errors:**
- Check JSON Schema: `web/scripts/theme-schema.json`
- Compare against example themes
- Ensure all required fields present

**TypeScript errors:**
- Regenerate types: `npm run theme:generate`
- Check type imports: `@/types/theme.types`

## Advanced

**Custom validation:**
Edit `web/scripts/theme-schema.json` to add custom rules.

**Custom generators:**
Modify `web/scripts/generate-theme.js` to generate additional outputs.

**Theme switching UI:**
```typescript
import { theme } from '@/theme.config'

// Show active theme name
console.log(theme.branding.title)

// Detect theme from CSS variables
const primaryColor = getComputedStyle(document.documentElement)
  .getPropertyValue('--color-primary')
```
