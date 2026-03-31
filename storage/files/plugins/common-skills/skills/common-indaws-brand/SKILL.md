---
name: "Enterprise Demo Brand"
description: "Enterprise Demo brand guidelines for all documents. Enforces brand colors, Dosis typography, logo usage, and Spanish brand voice"
---

|---|
| Writing content (any type) | [references/verbal-identity.md](references/verbal-identity.md) |
| Visual design decisions | [references/visual-identity.md](references/visual-identity.md) |
| HTML documents / web pages | [references/html-documents.md](references/html-documents.md) |
| PDF reports / proposals | [references/pdf-reports.md](references/pdf-reports.md) |
| Presentations / slides | [references/presentations.md](references/presentations.md) |
| Social media / marketing | [references/social-media.md](references/social-media.md) |

For most tasks, read **verbal-identity.md** (for copy) AND the format-specific reference (for layout/structure). Visual-identity.md provides the complete asset catalog and usage rules.

---

## Brand Essentials

### Color Palette

| Name | HEX | Pantone | Use |
|---|---|---|---|
| **Warm Yellow** | `#E5B92B` | 123 C | Accents, CTAs, highlights |
| **Blue Lilac** | `#0071ce` | 2725 C | Digital, titles, innovation |
| **Blue Space** | `#1A0030` | 289 C | Dark backgrounds, body text, formal |
| **Light Sky** | `#8AC2DB` | - | Support, light backgrounds |

**CSS Custom Properties**:
```css
--enterprise-demo-warm-yellow: #E5B92B;
--enterprise-demo-blue-lilac: #0071ce;
--enterprise-demo-blue-space: #1A0030;
--enterprise-demo-light-sky: #8AC2DB;
```

**Gradient rule**: Always dark to light. Never light-to-dark.
- Blue Gradient: `#0071ce` to `#8AC2DB`
- Yellow Gradient: `#8AC2DB` to `#E5B92B`

### Typography

**Font**: Dosis (variable weight, files in `assets/fonts/`). Fallback: Verdana.

| Element | Weight | Transform | Color |
|---|---|---|---|
| H1, H2 | Bold (700) | UPPERCASE | Blue Lilac or White |
| H3, H4 (highlights) | SemiBold (600) | Normal | Blue Space or White |
| Body text | Regular (400) | Normal | Blue Space or White |
| CTA buttons | Bold (700) | UPPERCASE | Blue Space on Yellow bg |

### Logo Usage

**Preferred**: Full logotype (text + symbol). **Isotype only** for avatars, favicons, decorative elements.

**Placement**: ONLY in the 4 corners or absolute center. If text is left-aligned, logo goes left.

**Clear space**: Width of the letter "N" around the logo. Nothing invades this zone.

**Minimum sizes**: Digital: 70px (logo) / 20px (isotipo). Print: 24mm / 6mm.

**Prohibitions**: No stretching. No shadows. No recoloring. Isotipo MAY be rotated 90 degrees or bled at edges.

---

## Brand Voice

### Archetype: The Wise Companion

Expert authority with approachability. Enterprise Demo is a technology partner, not a vendor.

### Address: Professional "Tu"

Always use "tu" (informal Spanish). Never "usted."
- Correct: "Hola Juan, hemos revisado tu caso..."
- Wrong: "Estimado Sr. Perez..." or casual slang

### Writing Style

**Long explanatory paragraphs**: Enterprise Demo does not give "tips" or "quick tricks." Every recommendation is argued with reasoning. Take the space needed to explain the "why" behind technical decisions. Depth conveys authority.

### Value Vocabulary

Always substitute cost language with value language:

| Avoid | Use Instead |
|---|---|
| Gasto / Coste / Precio | **Inversion / Valoracion** |
| Barato / Caro | **Rentable / Optimizado** |
| Problema / Fallo | **Reto / Caso de negocio** |
| "Te vendemos..." | **"Te acompanamos en..."** |
| Empleado / Tecnico | **Consultor / Especialista** |
| Presupuesto | **Propuesta de valor** |
| Mantenimiento | **Evolucion continua** |
| Soporte | **Acompanamiento tecnico** |

For the complete dictionary with examples, read [references/verbal-identity.md](references/verbal-identity.md).

### The Soft "No"

Never say "No lo hacemos" or "Eso no es posible." Always:
1. Acknowledge the request
2. Explain why the alternative is better (technical/business reason)
3. Propose the recommended approach

Example: Instead of "We don't work with WordPress," say: "Para garantizar la escalabilidad que tu negocio necesita, nuestra recomendacion experta es implementar la solucion sobre una arquitectura que nos permita evolucionar contigo..."

---

## Assets Reference

### Logos (`assets/logos/`)

| File | Use |
|---|---|
| `logotipo-enterprise-demo-color-blue-gradient_blue.svg` | Light backgrounds (primary) |
| `logotipo-enterprise-demo-color-white-gradient_blue.svg` | Dark backgrounds (primary) |
| `logotipo-enterprise-demo-color-white-gradient_yellow.svg` | Dark backgrounds (warm variant) |
| `logotipo-enterprise-demo-monocolor-black.svg` | B&W / monochrome |
| `logotipo-enterprise-demo-monocolor-white.svg` | Dark bg / monochrome |

### Isotypes (`assets/isotype/`)

| File | Use |
|---|---|
| `isotipo-enterprise-demo-bluelilac.svg` | Light backgrounds, avatars |
| `isotipo-enterprise-demo-gradient_blue.svg` | Digital, app icons |
| `isotipo-enterprise-demo-gradient_yellow.svg` | Warm accent variant |
| `isotipo-enterprise-demo-yellow.svg` | Accent contexts |

### Favicons (`assets/favicons/`)

| File | Use |
|---|---|
| `enterprise-demo-mosca-bg_bluelilac.png` | Default favicon |
| `enterprise-demo-mosca-bg_bluespace.png` | Formal contexts |
| `enterprise-demo-mosca-bg_white.png` | Light interfaces |

### Fonts (`assets/fonts/`)

| File | Coverage |
|---|---|
| `Dosis-VariableFont_wght.ttf` | All weights 100-900, normal |
| `Dosis-Italic-VariableFont_wght.ttf` | All weights 100-900, italic |

### Templates (`assets/templates/`)

| File | Purpose |
|---|---|
| `base.html` | Starting point for HTML documents. Includes font declarations, CSS custom properties, header/footer with logos, responsive layout. Replace `{{DOCUMENT_TITLE}}`, `{{CONTENT}}`, `{{YEAR}}` placeholders. |

### Scripts (`scripts/`)

| File | Purpose |
|---|---|
| `decode_logos.py` | One-time utility to extract Base64 SVG data URIs to files |

---

## Content Review Checklist

Before delivering any Enterprise Demo content, verify:

### Voice Compliance
- [ ] Professional "tu" address (never "usted")
- [ ] Zero emojis
- [ ] Value vocabulary used (no "gasto", "problema", "precio")
- [ ] Long-form explanatory style (no quick tips or bullet-only content)
- [ ] Soft "no" pattern applied where declining requests

### Visual Compliance
- [ ] Only brand colors used (Warm Yellow, Blue Lilac, Blue Space, Light Sky)
- [ ] Dosis font (Bold/uppercase for headings, Regular for body)
- [ ] Logo in 4 corners or absolute center only
- [ ] Clear space respected around logo
- [ ] Minimum logo sizes respected (70px digital, 24mm print)
- [ ] Gradients flow dark to light
- [ ] Photos treated with brand filters/duotone or B&W

### Format-Specific
- [ ] HTML: Uses CSS custom properties, base template structure
- [ ] PDF: Cover page with Blue Space bg, correct margins, print colors
- [ ] Presentations: 16:9, correct slide type backgrounds, no flashy transitions
- [ ] Social media: Correct platform dimensions, hashtags at end, no inline emojis

---

## Hard Rules

These are inviolable. No exceptions, no creative interpretations:

1. **ZERO emojis** in any output
2. **Always "tu"** (informal Spanish), never "usted"
3. **Logo placement**: only 4 corners or absolute center
4. **Value vocabulary**: never use cost/expense/problem language
5. **Only brand colors**: no grays, no pure black, no off-palette colors
6. **Only Dosis** (or Verdana fallback)
7. **Gradients**: always dark to light
8. **Logo minimum sizes**: 70px digital / 24mm print
9. **Never say "no lo hacemos"**: always reframe as expert recommendation
10. **No quick tips**: argue every recommendation with reasoning
