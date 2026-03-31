# Odoo Editor Compatibility Rules

Rules and constraints for producing HTML that works inside Odoo's rich text editor (Website Builder, Blog, Email Marketing, eLearning slides).

## Table of Contents
1. [Allowed HTML Elements](#allowed-html-elements)
2. [Bootstrap 5 Grid in Odoo](#bootstrap-5-grid-in-odoo)
3. [Odoo Snippet Structure](#odoo-snippet-structure)
4. [Forbidden Patterns](#forbidden-patterns)
5. [Image Handling](#image-handling)
6. [Class Preservation](#class-preservation)
7. [Odoo Content Types & Contexts](#odoo-content-types--contexts)

---

## Allowed HTML Elements

Odoo Editor preserves these elements when editing content:

**Block-level:** `<div>`, `<section>`, `<p>`, `<h1>`-`<h6>`, `<ul>`, `<ol>`, `<li>`, `<table>`, `<thead>`, `<tbody>`, `<tr>`, `<th>`, `<td>`, `<blockquote>`, `<figure>`, `<figcaption>`, `<hr>`, `<br>`, `<pre>`, `<address>`

**Inline:** `<span>`, `<a>`, `<strong>`, `<b>`, `<em>`, `<i>`, `<u>`, `<s>`, `<small>`, `<sub>`, `<sup>`, `<img>`, `<br>`

**Semantic (preserved but not editable content):** `<header>`, `<footer>`, `<nav>`, `<main>`, `<article>`, `<aside>`

## Bootstrap 5 Grid in Odoo

Odoo uses Bootstrap 5 grid natively. Always structure layouts with:

```html
<section class="s_text_block pt40 pb40">
  <div class="container">
    <div class="row">
      <div class="col-lg-6">
        <!-- Content left -->
      </div>
      <div class="col-lg-6">
        <!-- Content right -->
      </div>
    </div>
  </div>
</section>
```

**Key Bootstrap classes safe in Odoo:**
- Grid: `container`, `container-fluid`, `row`, `col-*`, `offset-*`
- Spacing: `p-*`, `m-*`, `pt-*`, `pb-*`, `px-*`, `py-*`, `mt-*`, `mb-*`
- Flex: `d-flex`, `flex-column`, `flex-row`, `align-items-*`, `justify-content-*`, `gap-*`
- Text: `text-center`, `text-start`, `text-end`, `fw-bold`, `fst-italic`, `text-uppercase`
- Display: `d-none`, `d-md-block`, `d-lg-flex`
- Sizing: `w-100`, `w-50`, `h-100`
- Border: `border`, `border-0`, `rounded`, `rounded-circle`
- Background: `bg-*` (use `style` attribute for custom colors)
- Visibility: `visible`, `invisible`

## Odoo Snippet Structure

Odoo website pages are built with "snippets" (sections). Each snippet follows this pattern:

```html
<section class="s_snippet_name [spacing] [options]" data-snippet="s_snippet_name">
  <div class="container">
    <!-- Content using Bootstrap grid -->
  </div>
</section>
```

**Common Odoo section classes:**
- `s_text_block` - Generic text section
- `s_text_image` - Text with image
- `s_three_columns` - Three-column layout
- `s_banner` - Full-width banner / hero
- `s_features` - Feature grid
- `s_call_to_action` - CTA section
- `s_references` - Client logos / references
- `s_comparisons` - Comparison / pricing table
- `s_quotes_carousel` - Testimonials carousel

**Spacing classes (Odoo-specific):**
- `pt8`, `pt16`, `pt24`, `pt32`, `pt40`, `pt48`, `pt64`, `pt80`, `pt96`
- `pb8`, `pb16`, `pb24`, `pb32`, `pb40`, `pb48`, `pb64`, `pb80`, `pb96`

These map to pixel values and are preserved by Odoo Editor.

## Forbidden Patterns

These will be stripped or cause issues in Odoo Editor:

1. **`<script>` tags** -- Always stripped. Use Odoo's website builder JS injection or custom module instead.
2. **`<style>` tags inline in body** -- May be stripped in some contexts (emails, blog posts). Use Odoo's CSS customization panel or inject via custom SCSS.
3. **CSS `@import` in body** -- Stripped. Load external CSS via Odoo's asset bundle.
4. **`<iframe>` without Odoo wrapper** -- Use Odoo's media dialog to embed videos/maps.
5. **JavaScript event attributes** (`onclick`, `onload`, etc.) -- Stripped for security.
6. **Custom `data-*` attributes** -- Generally preserved but may conflict with Odoo's own data attributes. Avoid `data-oe-*` prefix (reserved by Odoo).
7. **`position: fixed` or `position: sticky`** -- Works in frontend but causes issues in editor mode.
8. **Complex CSS selectors with deep nesting** -- Keep selectors simple; Odoo editor may restructure DOM.

## Image Handling

**In Odoo website pages:**
```html
<img src="/web/image/..." class="img-fluid rounded" alt="Description"/>
```

**In emails (mass mailing):**
```html
<img src="https://full-url.com/image.jpg" width="600" style="max-width:100%; height:auto;" alt="Description"/>
```

Rules:
- Always use `img-fluid` class for responsive images in web pages
- Always include `alt` attribute
- For emails, use absolute URLs and inline `width`/`style`
- Odoo Editor handles image upload -- reference via `/web/image/model/id/field`

## Class Preservation

Odoo Editor preserves custom CSS classes on elements. This means `iw-*` classes from the Enterprise Demo Frontend CSS will persist through editing sessions.

**Safe to use:** Any `iw-*` class on `<section>`, `<div>`, `<p>`, `<h*>`, `<span>`, `<a>`, `<img>`, `<table>`, `<ul>`, `<li>`, `<blockquote>`

**Combine Bootstrap + iw-* freely:**
```html
<div class="col-lg-4 iw-card iw-card-accent">
  <h3 class="fw-bold iw-heading-sm">Title</h3>
  <p class="iw-body">Content here.</p>
</div>
```

## Odoo Content Types & Contexts

### Website Pages
Full HTML with Bootstrap 5 grid. All `iw-*` classes available. CSS loaded via Website > Customize > CSS or asset bundle.

### Blog Posts
Content inside `<div class="o_wblog_post_content">`. Full Bootstrap grid. Custom classes preserved. Limited to content area (no full-width sections unless blog layout supports it).

### Email Templates (Mass Mailing)
**Strict limitations:**
- No external CSS (inline styles only)
- Use `<table>` layout for email clients
- Use `iw-email-*` classes or inline equivalents
- No gradients (use solid colors)
- No `border-radius` over 8px (Outlook)
- Max width: 600px
- All images: absolute URLs

### eLearning / Slide Channels
Content slides in Odoo eLearning. Similar to website pages but within a narrower container. Bootstrap classes work. Custom CSS via website customization.

### Product Descriptions
Limited rich text. Bootstrap text utilities and basic formatting. Images inline. Keep it simple.

### Portal / Customer-facing HTML
Rendered in frontend. Full Bootstrap 5 available. Custom CSS via asset bundle.
