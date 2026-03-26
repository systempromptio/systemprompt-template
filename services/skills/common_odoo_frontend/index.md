
# Odoo Frontend

Create HTML content for Odoo using Bootstrap 5 and the Foodles design system (`iw-*` classes). All output must be compatible with Odoo Editor and follow the foodles-brand guidelines.

**Before starting any task, read the `foodles-brand` skill** to load brand colors, typography, voice rules, and the zero-emoji policy.

## Quick Reference

| Token | Value |
|-------|-------|
| Font | Dosis (400, 600, 700) via Google Fonts |
| Warm Yellow | `#E5B92B` |
| Blue Lilac | `#6B68FA` |
| Blue Space | `#1C265D` |
| Light Sky | `#8AC2DB` |
| CSS file | `assets/css/foodles-frontend.css` |
| Reveal JS | `assets/js/iw-reveal.js` |

## Phase 0: Detect Mode

Determine the content type before producing anything:

| User wants... | Mode | Action |
|---------------|------|--------|
| New website page / landing page | A: New Page | Go to Phase 1 |
| New blog post content | A: Blog | Go to Phase 1 |
| New email template | A: Email | Go to Phase 1 |
| New HTML presentation | A: Presentation | Go to Phase 1 (with presentation flow) |
| Convert a .pptx file | B: PPT Conversion | Go to Phase 4 |
| Enhance existing HTML | C: Enhancement | Read existing file, then enhance with iw-* classes |
| Quick component | D: Component | Read `references/components-catalog.md`, produce component directly |

## Phase 1: Content Discovery

Ask the user via AskUserQuestion to understand requirements.

### Step 1.1: Context questions

**Question 1 -- Purpose:**
- Header: "Purpose"
- "What is this content for?"
- Options: "Website landing page", "Blog article", "Email campaign", "Presentation / Pitch deck"

**Question 2 -- Content readiness:**
- Header: "Content"
- "Do you have the content ready?"
- Options: "Content ready (text/images)", "Rough notes to organize", "Topic only -- help me create it"

**Question 3 -- Images:**
- Header: "Images"
- "Do you have images for this content?"
- Options: "Yes -- I'll provide URLs or paths", "Use free stock photos (Unsplash)", "Generate descriptive placeholders for now"

**Question 4 -- Audience:**
- Header: "Audience"
- "Who is the target audience?"
- Options: "Potential clients (commercial)", "Existing clients (retention)", "Internal team", "General public"

If the user has content or images, ask them to share before proceeding.

### Step 1.2: Image Discovery

Based on the user's image choice:

- **"Yes -- I'll provide URLs or paths"**: Ask user to provide image files or URLs, note their purpose (hero, section 1, etc.)
- **"Use free stock photos (Unsplash)"**: Read `references/image-workflow.md` > "Unsplash API" section, search for 3-5 content-relevant keywords, present top 3 results to user
- **"Generate descriptive placeholders"**: Use inline SVG placeholders with brand colors (see `references/image-workflow.md` > "Descriptive Placeholders")

**IMPORTANT:** Never use generic placeholder URLs (placeholder.com, lorempixel). Always use semantic inline SVG placeholders with Foodles brand colors and descriptive labels.

### For Presentations Only

**Additional question -- Length:**
- Header: "Length"
- "How many slides?"
- Options: "Short (5-10)", "Medium (10-20)", "Long (20+)"

**Additional question -- Mood:**
- Header: "Vibe"
- "What feeling should the audience have?"
- Options: "Impressed/Confident", "Excited/Energized", "Calm/Focused", "Inspired/Moved"
- multiSelect: true (up to 2)

## Phase 2: Style Selection (Presentations Only)

For presentations, generate 3 mini HTML previews in `/tmp/iw-preview/slide-previews/` (style-a.html, style-b.html, style-c.html). Create the directory with `mkdir -p /tmp/iw-preview/slide-previews`. Each preview is a single title slide showing the aesthetic.

**Map mood to style:**

| Mood | Style A | Style B | Style C |
|------|---------|---------|---------|
| Impressed/Confident | Corporate Elegant | Dark Executive | Clean Minimal |
| Excited/Energized | Bold Gradients | Kinetic Motion | Vibrant Contrast |
| Calm/Focused | Swiss Minimal | Soft Muted | Paper Clean |
| Inspired/Moved | Cinematic Dark | Warm Editorial | Atmospheric |

All styles use exclusively Foodles brand colors and Dosis font.

Present the previews, then ask user to pick via AskUserQuestion:
- Header: "Style"
- "Which style preview do you prefer?"
- Options: "Style A: [Name]", "Style B: [Name]", "Style C: [Name]", "Mix elements from multiple"

See `references/presentations.md` for slide type templates, navigation JS, and full structure.

## Phase 3: Generate Content

### HTML Structure Rules

Read `references/odoo-editor-rules.md` for the complete compatibility reference. Key rules:

1. **Use Bootstrap 5 grid** (`container`, `row`, `col-*`) for all layouts
2. **Use `iw-*` classes** from `assets/css/foodles-frontend.css` for styling
3. **Wrap in `<section>` tags** with Odoo snippet classes (`s_text_block`, etc.)
4. **No `<script>` tags** in Odoo page/blog/email content (JS stripped by editor)
5. **No inline `<style>` blocks** in email content (use inline styles instead)
6. **Images use `img-fluid`** class and Odoo `/web/image/` paths
7. **No `data-oe-*` attributes** (reserved by Odoo)

### Building a Page

Compose pages by combining components from `references/components-catalog.md`. Typical structure:

```
Hero section (iw-hero)
  |
Feature section (iw-section + cards or text+image)
  |
Stats section (iw-stat)
  |
Testimonial section (iw-testimonial)
  |
CTA section (iw-bg-gradient-space)
```

For complete landing page workflows, read `references/landing-page-creation.md`. It covers:
- Landing page types (conversion, informational, product showcase, event)
- Section ordering with persuasion flow rationale
- CTA placement strategy (above fold, mid-page, bottom)
- Content-collection integration for social proof
- SEO meta title/description guidelines
- Publishing to Odoo (paste into Website Builder or programmatic via odoo-pilot)

#### Self-contained output format (MANDATORY)

All website pages and landing pages MUST be generated as **self-contained HTML** ready to paste inside an Odoo QWeb template. The output file contains everything in a single block:

```html
<!-- Instructions for pasting into QWeb template -->

<style>
/* Full contents of assets/css/foodles-frontend.css inlined here */
@import url('https://fonts.googleapis.com/css2?family=Dosis:ital,wght@0,400;0,600;0,700;1,400;1,600&amp;display=swap');
/* ... all iw-* class definitions ... */
</style>

<section class="iw-hero iw-section-wave">
  <!-- Page content using Bootstrap 5 + iw-* classes -->
</section>
<!-- More sections... -->
```

**Rules for self-contained pages:**

1. **Inline the full CSS** from `assets/css/foodles-frontend.css` inside a `<style>` block at the top of the file. Do NOT reference external CSS files.
2. **No `<!DOCTYPE>`, `<html>`, `<head>`, `<body>` wrappers** -- Odoo's `website.layout` provides those.
3. **No `<script>` tags** -- Odoo Editor strips them from website pages.
4. **No `iw-reveal` animation classes** -- They require JS that cannot be included. Do not use `iw-reveal`, `iw-reveal-left`, `iw-reveal-right`, or `iw-reveal-scale`.
5. **XML-safe URLs** -- Use `&amp;` instead of `&` in `@import` and any other URL (the file is pasted inside a QWeb/XML template).
6. **Image placeholders** -- Use `/web/image/placeholder.png` for all images. Add an HTML comment above each `<img>` indicating the intended image: `<!-- IMAGEN: description_or_filename.png -->`.
7. **Include a header comment** explaining how to paste the content into the QWeb template structure:
   ```
   <t t-name="website.page_name">
     <t t-call="website.layout">
       ... PASTE HERE ...
     </t>
   </t>
   ```
8. **Preserve all accents and special characters** -- Spanish text must include proper tildes and accents (á, é, í, ó, ú, ñ, ü). Never strip diacritics.

### Building an Email

**Brand compliance**: Before writing any email, read the `foodles-brand` skill -- specifically `references/verbal-identity.md` (tone, vocabulary, writing style) and `references/visual-identity.md` (colors, typography). All email content must follow brand voice rules.

Read `references/email-creation.md` for the complete email workflow. Emails follow this structure:

1. **Preheader**: Hidden preview text (85-100 chars)
2. **Header**: Logo on dark background (`#1C265D`)
3. **Body**: Table-based layout, all inline styles, max 600px
4. **CTA**: Bulletproof button (Outlook-safe VML + CSS)
5. **Footer**: Company info, unsubscribe link, legal

Key constraints: inline styles only (Gmail strips `<style>` blocks), table-based layout (Outlook uses Word renderer), solid colors only (no gradients), all images with absolute URLs, Dosis with Verdana fallback.

**Email types**: Newsletter (multi-block digest), promotional (single offer), event invite (date + registration). See `references/email-creation.md` for templates of each type.

### Building a Blog Post

**Brand compliance**: Before writing any blog, read the `foodles-brand` skill -- specifically `references/verbal-identity.md` (tone, vocabulary, writing style) and `references/visual-identity.md` (colors, typography). All blog content must follow brand voice rules.

Read `references/blog-creation.md` for the complete blog workflow. Blogs follow the BLUF structure:

1. **BLUF** (Bottom Line Up Front): 100-150 word executive summary answering the main question
2. **Table of contents**: Auto-generated from H2 headings
3. **Body**: 1200-1500+ words in 3-5 H2 sections with interlinking and 1 YouTube video embed
4. **FAQs**: 3-5 questions targeting long-tail keywords
5. **Schema JSON-LD**: Article + FAQPage combined

**Visual design blocks**: Include at least 2 design blocks per article to break text monotony. Available blocks (see `references/blog-creation.md` for HTML): bento grid, comparison table, pros/cons, key figures, summary box, timeline, stats row, data table.

**Interlinking**: Load `data/sitemap.json` from `content-collection` skill. Place 2-4 internal links per H2 section (mix of service pages, related blogs, and case studies).

**YouTube embed**: Load `data/videos.json` from `content-collection` skill. Find the most relevant video by topic matching and embed it contextually within the body.

**Minimum 1500 words** in the body (excluding BLUF, FAQs, schema). Spanish language, professional "tu" address, zero emojis, value vocabulary per foodles-brand.

### Building a Presentation

See `references/presentations.md` for complete slide types, navigation system, and structure. Presentations are self-contained HTML files with embedded CSS/JS.

## Phase 4: PPT Conversion

When converting PowerPoint files:

1. Extract content using `python-pptx` (pip install if needed):
   - Text from each slide (title, body, notes)
   - Images saved to an assets folder
   - Slide order and structure

2. Present extracted content summary to user for confirmation

3. Run Phase 2 (Style Selection) for the visual treatment

4. Generate HTML presentation mapping each PPT slide to the closest slide type from `references/presentations.md`

5. Preserve all images and speaker notes (as `<!-- NOTES: ... -->`)

## Phase 5: Preview & Delivery

After generating content, always preview in the browser before delivering.

### Step 5.1: Generate Preview

Read `references/preview-workflow.md` for the complete preview system. Summary:

1. Create the preview directory: `mkdir -p /tmp/iw-preview`
2. **Blogs / Landing pages** (HTML fragments): wrap in the preview template -- read `assets/css/foodles-frontend.css` and `assets/js/iw-reveal.js`, embed them in a full HTML document with Bootstrap 5 CDN, paste the content fragment in `<body>`
3. **Emails** (full HTML documents): save the generated HTML directly (already has inline styles and document structure)
4. **Presentations** (self-contained files): save as-is (already has embedded CSS/JS)
5. Write to `/tmp/iw-preview/[type]-preview.html`
6. Open in browser: `open /tmp/iw-preview/[type]-preview.html`

### Step 5.2: Review & Iterate

Ask the user via AskUserQuestion:
- Header: "Preview"
- "How does the preview look?"
- Options: "Looks good -- proceed to delivery", "Needs adjustments", "Start over with different approach"

If adjustments needed: ask what to change, apply edits, regenerate preview, re-open. Repeat until approved.

### Step 5.3: Create Image Manifest

Before final delivery, create an Image Manifest following the template in `references/image-workflow.md` > "Image Manifest Template". This manifest lists:

- All images in the content (numbered)
- Description and dimensions of each
- Source (user-provided, Unsplash with attribution, or placeholder)
- Status (✅ Ready or ⚠️ Replace)
- Replacement instructions for placeholders

Include the manifest as a markdown section in your delivery message to the user.

### Step 5.4: Deliver

Provide a summary of what was created:
- Content type and structure
- Word count / slide count
- Number of images (ready vs. placeholders)
- Any Unsplash attributions required

Then offer publishing options per content type.

**For Odoo website pages/landing pages**: The output is a single self-contained HTML file with CSS inlined in `<style>`. Tell the user:
- The file is ready to paste inside their QWeb template `<t t-call="website.layout">`
- They need to upload images to Odoo and replace `/web/image/placeholder.png` paths with actual Odoo attachment paths (e.g., `/web/image/ir.attachment/123/datas`)
- List all image placeholders with their intended screenshot/image descriptions

#### Blog Publishing via odoo-pilot

For blog posts, ask the user whether to publish the blog as a draft in Odoo:

1. Use the `odoo-pilot` skill to authenticate and create a `blog.post` record with `is_published: false`
2. Send the full HTML (BLUF through Schema) as the `content` field
3. Set `website_meta_title` and `website_meta_description` from the SEO data
4. Provide the user with the Odoo backend URL and preview URL for revision

See `references/blog-creation.md` > "Publishing to Odoo" for the full workflow, field mapping, and odoo-pilot script usage.

#### Email Publishing via odoo-pilot

For emails, ask the user whether to create the mailing as a draft in Odoo:

1. Use the `odoo-pilot` skill to authenticate and create a `mailing.mailing` record
2. Send the full HTML as the `body_html` field
3. Set `subject` from the email subject line
4. Optionally link to a `mailing.list` contact list
5. Provide the user with the Odoo backend URL for testing and sending

See `references/email-creation.md` > "Publishing to Odoo" for the full workflow, field mapping, and odoo-pilot script usage.

#### Landing Page Publishing via odoo-pilot

For landing pages, ask the user whether to create the page in Odoo:

1. **Primary method**: Provide paste-ready self-contained HTML for the QWeb template (see Phase 3 self-contained format)
2. **Advanced method**: Use the `odoo-pilot` skill to create a `website.page` record programmatically with `is_published: false`

See `references/landing-page-creation.md` > "Publishing to Odoo" for both methods.

## CSS Loading

For **website pages**: the CSS is always inlined in the `<style>` block of the generated HTML (see Phase 3 self-contained format). No external CSS file installation is needed.

For **blog posts, email templates, and other content types** where a `<style>` block is not practical, the `iw-*` classes must be loaded in Odoo via one of these methods:

**Method 1: Website Customization Panel**
Odoo > Website > Customize > scroll to bottom > Custom CSS > paste contents of `assets/css/foodles-frontend.css`

**Method 2: Custom Odoo Module**
Add to a module's `assets.xml`:
```xml
<template id="assets_frontend" inherit_id="website.assets_frontend">
  <xpath expr="." position="inside">
    <link rel="stylesheet" href="/your_module/static/src/css/foodles-frontend.css"/>
  </xpath>
</template>
```

**Note on animations**: The `iw-reveal` scroll animations from `assets/js/iw-reveal.js` are only available when loaded via a custom Odoo module asset bundle. They are NOT included in self-contained page output (Odoo strips `<script>` tags).

## Resources

### assets/css/
- `foodles-frontend.css` -- Complete design system. 24 sections: tokens, typography, colors, sections, cards, buttons, badges, features, stats, testimonials, tables, pricing, timeline, dividers, callouts, images, animations, email utilities, bento grid, comparison table, pros/cons, key figures, summary box, responsive breakpoints.

### assets/js/
- `iw-reveal.js` -- Intersection Observer script for scroll-triggered entrance animations. Add `iw-reveal`, `iw-reveal-left`, `iw-reveal-right`, or `iw-reveal-scale` classes to elements.

### references/
- `odoo-editor-rules.md` -- Compatibility rules: allowed HTML elements, Bootstrap grid usage, Odoo snippet structure, forbidden patterns, image handling, class preservation, content type contexts.
- `components-catalog.md` -- Copy-paste ready HTML components: heroes, feature grids, cards, stats, testimonials, pricing, timeline, CTA, content blocks, tables, email templates.
- `blog-creation.md` -- Blog article workflow: BLUF structure, SEO/AI optimization, interlinking strategy with content-collection data, YouTube embedding, FAQ section, Schema JSON-LD (Article + FAQPage), complete HTML template, publishing via odoo-pilot.
- `email-creation.md` -- Email marketing workflow: email types, Odoo mass mailing compatibility, table-based layout for Outlook, inline style patterns, bulletproof CTA buttons, footer requirements, brand compliance, content-collection integration, complete newsletter and promotional templates, publishing via odoo-pilot.
- `landing-page-creation.md` -- Landing page workflow: page types (conversion, informational, product showcase, event), section ordering with persuasion flow, CTA placement strategy, content-collection integration for social proof, SEO meta fields, section composition guide referencing components-catalog, complete conversion page template, publishing via odoo-pilot or paste.
- `presentations.md` -- HTML presentation structure, slide types (title, section, content, data, quote, closing), navigation JS, style discovery workflow, PPT conversion guide.
- `preview-workflow.md` -- Browser preview system: preview architecture (self-contained vs fragment wrapping), wrapper template with embedded CSS/JS + Bootstrap 5 CDN, content-type handling table, iteration flow with user feedback loop.
- `image-workflow.md` -- Comprehensive image handling: image strategy by content type, Unsplash API integration for stock photos, inline SVG placeholder system with brand colors, Odoo Media Library upload guide, optimization requirements, image manifest template for delivery.
