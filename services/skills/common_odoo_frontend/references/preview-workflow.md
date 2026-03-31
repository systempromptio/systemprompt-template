# Preview Workflow

Generate a browser preview of all content types before delivering to the user. This allows visual review and iteration before pasting into Odoo.

## Table of Contents
1. [Preview Architecture](#preview-architecture)
2. [Preview Directory](#preview-directory)
3. [Content-Type Handling](#content-type-handling)
4. [Preview Wrapper Template](#preview-wrapper-template)
5. [Opening in Browser](#opening-in-browser)
6. [Iteration Flow](#iteration-flow)

---

## Preview Architecture

Content types fall into two categories:

**Self-contained (no wrapping needed):**
- **Emails** -- Full HTML documents with inline styles, `<!DOCTYPE>`, `<head>`, `<body>`
- **Presentations** -- Self-contained HTML files with embedded CSS and JS

**Fragments (wrapping required):**
- **Blog posts** -- HTML fragments meant for Odoo's content editor (no document structure)
- **Landing pages** -- HTML sections meant for Odoo's Website Builder (no document structure)

Fragments need wrapping in a full HTML document with:
- Bootstrap 5 CSS (via CDN -- Odoo provides this natively, but preview needs it explicitly)
- Enterprise Demo design system CSS (embedded from `assets/css/enterprise-demo-frontend.css`)
- Scroll reveal JS (embedded from `assets/js/iw-reveal.js`)
- Google Fonts (Dosis)

## Preview Directory

All preview files are saved to `/tmp/iw-preview/`. Create the directory before writing:

```bash
mkdir -p /tmp/iw-preview
```

Files are overwritten on each preview generation (no timestamps). The OS cleans `/tmp/` automatically.

## Content-Type Handling

| Content type | Wrapping | File name | Notes |
|-------------|----------|-----------|-------|
| Blog post | Yes -- wrap in full document | `blog-preview.html` | Embed CSS + JS, add Bootstrap CDN |
| Landing page | Yes -- wrap in full document | `landing-preview.html` | Embed CSS + JS, add Bootstrap CDN |
| Email | No -- save as-is | `email-preview.html` | Already has inline styles and full document structure |
| Presentation | No -- save as-is | `presentation-preview.html` | Already self-contained with embedded CSS/JS |

## Preview Wrapper Template

Use this template to wrap blog and landing page HTML fragments. Read the CSS and JS files at generation time and embed their contents inline.

```html
<!DOCTYPE html>
<html lang="es">
<head>
  <meta charset="UTF-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
  <title>Preview -- Enterprise Demo</title>
  <link href="https://fonts.googleapis.com/css2?family=Dosis:wght@400;600;700&display=swap" rel="stylesheet"/>
  <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css" rel="stylesheet"/>
  <style>
    /* === Enterprise Demo Frontend CSS (embedded from assets/css/enterprise-demo-frontend.css) === */
    /* READ the file and paste its entire contents here */
  </style>
</head>
<body>

  <!-- === Content Fragment === -->
  <!-- PASTE the generated HTML content here -->

  <script>
    // === Scroll Reveal (embedded from assets/js/iw-reveal.js) ===
    // READ the file and paste its entire contents here
  </script>
</body>
</html>
```

### How to build the preview file

1. Read `assets/css/enterprise-demo-frontend.css` -- copy entire contents
2. Read `assets/js/iw-reveal.js` -- copy entire contents
3. Replace the CSS placeholder comment with the CSS contents
4. Replace the JS placeholder comment with the JS contents
5. Replace the content fragment placeholder with the generated HTML
6. Write the complete file to `/tmp/iw-preview/[type]-preview.html`

## Opening in Browser

After saving the preview file, open it in the default browser:

**macOS:**
```bash
open /tmp/iw-preview/blog-preview.html
```

**Linux:**
```bash
xdg-open /tmp/iw-preview/blog-preview.html
```

Detect the platform and use the appropriate command. On macOS (Darwin), use `open`. On Linux, use `xdg-open`.

## Iteration Flow

After opening the preview, ask the user for feedback:

**AskUserQuestion:**
- Header: "Preview"
- Question: "How does the preview look?"
- Options:
  - "Looks good -- proceed to delivery" (continue to publishing step)
  - "Needs adjustments" (ask what to change, apply edits, re-preview)
  - "Start over with different approach" (return to content generation phase)

**On "Needs adjustments":**
1. Ask what specific changes are needed (free text via AskUserQuestion)
2. Apply changes to the generated HTML
3. Regenerate the preview file (overwrite the same path)
4. Re-open in browser
5. Ask for feedback again

Repeat until the user approves. Then proceed to the delivery/publishing step (Phase 5.3 in SKILL.md).
