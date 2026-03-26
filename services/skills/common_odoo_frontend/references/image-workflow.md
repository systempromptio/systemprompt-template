# Image Workflow

Comprehensive system for handling images in all Odoo content types. This workflow ensures images are properly sourced, optimized, and integrated.

## Table of Contents
1. [Image Strategy by Content Type](#image-strategy-by-content-type)
2. [Image Sources & Discovery](#image-sources--discovery)
3. [Image Optimization Requirements](#image-optimization-requirements)
4. [Image Placeholder System](#image-placeholder-system)
5. [Odoo Media Library Integration](#odoo-media-library-integration)
6. [Complete Image Checklist](#complete-image-checklist)

---

## Image Strategy by Content Type

| Content Type | Image Source Priority | URL Format | Max Width | Notes |
|--------------|----------------------|------------|-----------|-------|
| **Blog posts** | 1. User-provided<br>2. Unsplash via API<br>3. Descriptive placeholder | `/web/image/ir.attachment/ID/datas` | 1200px | Hero: 1200x600, inline: 800px max |
| **Landing pages** | 1. User-provided<br>2. Unsplash via API<br>3. Descriptive placeholder | `/web/image/ir.attachment/ID/datas` | 1920px | Hero: full-width, sections: 1200px |
| **Emails** | 1. User-provided<br>2. Hosted CDN URL<br>3. Unsplash direct URL | `https://` absolute URL | 600px | Must be hosted, no relative URLs |
| **Presentations** | 1. User-provided<br>2. Embedded base64 (small)<br>3. External URL | Embedded or `https://` | 1920px | Self-contained file preferred |

---

## Image Sources & Discovery

### Priority 1: User-Provided Images

**Always ask first:**
```
AskUserQuestion:
- Header: "Images"
- Question: "Do you have images for this content?"
- Options:
  - "Yes -- I'll provide URLs or paths"
  - "Use free stock photos (Unsplash)"
  - "Generate descriptive placeholders for now"
```

If user provides images:
- Accept file paths (will be uploaded to Odoo later)
- Accept URLs (validate they're accessible)
- Accept Odoo `/web/image/` URLs (if updating existing content)

### Priority 2: Unsplash API (Free Stock Photos)

When user selects "Use free stock photos", search Unsplash API for relevant images:

**Search strategy:**
1. Extract 3-5 keywords from content theme (e.g., "odoo", "business", "teamwork")
2. Use Unsplash API endpoint: `https://api.unsplash.com/search/photos?query={keywords}&per_page=5&orientation=landscape`
3. Present top 3 results to user via AskUserQuestion with image previews
4. Use selected image's `urls.regular` (1080px width)
5. **Include attribution** in HTML comment: `<!-- Photo by {photographer} on Unsplash -->`

**API access:**
```bash
# Set env var or use demo access key
UNSPLASH_ACCESS_KEY="your_key_here"
curl -H "Authorization: Client-ID $UNSPLASH_ACCESS_KEY" \
  "https://api.unsplash.com/search/photos?query=odoo+erp&per_page=5"
```

**Important:** Unsplash requires attribution. Add to footer or image caption:
```html
<!-- Photo by [Photographer Name] on Unsplash (https://unsplash.com) -->
```

### Priority 3: Descriptive Placeholders

When no real images are available, use semantic placeholders:

**DO NOT use generic placeholder services** (placeholder.com, lorempixel, etc.).

**INSTEAD, create descriptive placeholder with:**
- Width x height dimensions
- Descriptive label matching content
- Brand colors (Foodles Blue Space background, White text)
- Clear instruction for user replacement

**Example:**
```html
<div class="iw-image-placeholder" style="
  width: 100%;
  aspect-ratio: 16/9;
  background: linear-gradient(135deg, #1C265D 0%, #6B68FA 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
  border: 2px dashed #E5B92B;
">
  <div style="text-align: center; color: white; padding: 2rem;">
    <p style="font-size: 1.25rem; font-weight: 600; margin-bottom: 0.5rem;">
      Hero Image: Odoo Dashboard
    </p>
    <p style="font-size: 0.95rem; opacity: 0.8;">
      1200 x 600px • Replace via Odoo Media Library
    </p>
  </div>
</div>
```

This approach:
- Clearly shows what image belongs here
- Maintains layout during development
- Provides exact dimensions needed
- Uses brand colors (looks intentional, not broken)
- Includes replacement instructions

---

## Image Optimization Requirements

### File Size Targets

| Usage | Max File Size | Recommended Format | Dimensions |
|-------|--------------|-------------------|------------|
| Hero images | 300 KB | WebP (fallback: JPG) | 1920x1080 |
| Section images | 150 KB | WebP (fallback: JPG) | 1200x800 |
| Thumbnails | 50 KB | WebP (fallback: JPG) | 400x300 |
| Icons/logos | 20 KB | SVG or PNG | Vector or 2x |
| Email images | 100 KB | JPG only | 600x400 max |

### Optimization Commands

**If user provides large images, suggest optimization:**

```bash
# Using ImageMagick (if available)
convert input.jpg -resize 1200x -quality 85 -strip output.jpg

# Using cwebp for WebP
cwebp -q 85 input.jpg -o output.webp
```

**Provide guidance to user:**
```
The image you provided is [X] MB. For web performance, we should optimize it to under 300 KB.

Options:
1. I can provide the optimized image command (you run it locally)
2. Upload to Odoo as-is (Odoo may auto-resize)
3. Use a web optimizer like TinyPNG/Squoosh first

Which approach do you prefer?
```

---

## Image Placeholder System

### For Blogs & Landing Pages (Odoo Website)

Use structured placeholder with clear replacement path:

```html
<!-- Image Placeholder: [DESCRIPTION] -->
<div class="iw-img-placeholder" data-replace="true" data-description="Hero image showing Odoo dashboard interface">
  <img src="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 1200 600'%3E%3Crect fill='%231C265D' width='1200' height='600'/%3E%3Ctext x='50%25' y='50%25' dominant-baseline='middle' text-anchor='middle' font-family='Dosis' font-size='32' fill='%23E5B92B'%3EHero: Odoo Dashboard (1200x600)%3C/text%3E%3C/svg%3E"
       alt="Placeholder: Odoo dashboard hero image"
       class="img-fluid rounded"/>
</div>
<!-- Replace via: Odoo > Website > Edit > Click image > Upload -->
```

This creates an **inline SVG placeholder** that:
- Renders immediately (no external request)
- Shows exact dimensions and description
- Uses brand colors
- Provides replacement instructions
- Works in Odoo Editor preview

### For Emails (Absolute URLs Required)

Use Unsplash direct URLs or CDN-hosted placeholders:

```html
<!-- Email Image: [DESCRIPTION] -->
<img src="https://images.unsplash.com/photo-XXXXXX?w=600&q=80"
     alt="Business team collaboration"
     width="600"
     style="display: block; max-width: 100%; height: auto; border-radius: 8px;"
     />
<!-- Photo by [Name] on Unsplash -->
```

**Never use:** relative paths, `/web/image/`, `file://`, or placeholder services in emails.

### For Presentations (Self-Contained)

**Option A: Embed small images as base64** (< 50 KB)
```html
<img src="data:image/png;base64,iVBORw0KG..." alt="Logo" style="height: 60px;"/>
```

**Option B: External CDN URLs** (for larger images)
```html
<img src="https://cdn.example.com/image.jpg" alt="Slide background"/>
```

**Option C: Relative paths** (if presentation folder includes assets)
```html
<img src="./assets/slide-bg.jpg" alt="Background"/>
```

---

## Odoo Media Library Integration

### Uploading Images to Odoo

**Via UI (recommended for users):**
1. Odoo > Website > Site > Media (or Editor > Image icon)
2. Upload image files
3. Copy the generated `/web/image/ir.attachment/ID/datas` URL
4. Replace placeholder in HTML

**Via odoo-pilot (programmatic):**

```bash
# Step 1: Upload image file and create ir.attachment record
./scripts/create_record.sh ir.attachment '{
  "name": "hero-dashboard.jpg",
  "type": "binary",
  "datas": "'$(base64 -i hero-dashboard.jpg)'",
  "res_model": "ir.ui.view",
  "public": true,
  "mimetype": "image/jpeg"
}'

# Returns: {"id": 123}

# Step 2: Reference in HTML
<img src="/web/image/ir.attachment/123/datas" class="img-fluid" alt="Odoo Dashboard"/>
```

**Image URL formats in Odoo:**

| Format | Use Case | Example |
|--------|----------|---------|
| `/web/image/ir.attachment/ID/datas` | Uploaded media files | `/web/image/ir.attachment/456/datas` |
| `/web/image/product.product/ID/image_1920` | Product images | `/web/image/product.product/12/image_1920` |
| `/web/image/res.partner/ID/image_128` | Partner avatars | `/web/image/res.partner/8/image_128` |
| Absolute URL | External CDN or Unsplash | `https://images.unsplash.com/photo-...` |

---

## Complete Image Checklist

Before delivering any content with images, verify:

### ✅ Discovery Phase
- [ ] Asked user if they have images (AskUserQuestion)
- [ ] If no images: offered Unsplash search or descriptive placeholders
- [ ] If Unsplash: searched with content-relevant keywords
- [ ] Presented image options and got user selection

### ✅ Technical Implementation
- [ ] **Blogs/Landing pages**: Used inline SVG placeholders OR Odoo `/web/image/` URLs OR Unsplash URLs
- [ ] **Emails**: Used absolute `https://` URLs only (Unsplash or CDN)
- [ ] **Presentations**: Embedded base64 OR external URLs with offline fallback note
- [ ] All `<img>` tags have `alt` attributes with descriptive text
- [ ] Images use appropriate CSS classes (`img-fluid`, `rounded`, `iw-img-*`)

### ✅ Optimization
- [ ] Recommended dimensions match usage (hero: 1200x600, section: 800x600, etc.)
- [ ] If user provided large files: suggested optimization commands
- [ ] Email images ≤ 600px width
- [ ] Inline SVG placeholders use brand colors (#1C265D, #E5B92B)

### ✅ Documentation
- [ ] Added HTML comments above each image explaining purpose
- [ ] If placeholder: included replacement instructions
- [ ] If Unsplash: included photographer attribution
- [ ] Provided image upload guide if using Odoo Media Library

### ✅ Delivery
- [ ] Created image manifest (list of all images with descriptions, sources, dimensions)
- [ ] If placeholders: clearly communicated to user which images need replacement
- [ ] If Unsplash: verified attributions are present

---

## Image Manifest Template

When delivering content with images, include this manifest:

```markdown
## Image Manifest

### Images in This Content

| # | Location | Description | Dimensions | Source | Status |
|---|----------|-------------|------------|--------|--------|
| 1 | Hero section | Odoo dashboard overview | 1200x600 | Placeholder (inline SVG) | ⚠️ Replace |
| 2 | Feature block 1 | Team collaboration screenshot | 800x600 | Unsplash (@photographer) | ✅ Ready |
| 3 | Feature block 2 | Analytics graph | 800x600 | User-provided (analytics.jpg) | ✅ Ready |
| 4 | CTA section | Office workspace | 1200x400 | Placeholder (inline SVG) | ⚠️ Replace |

### Replacement Instructions

**For placeholders marked ⚠️ Replace:**

1. **Option A (Recommended):** Upload via Odoo UI
   - Odoo > Website > Edit page > Click placeholder > Upload image
   - Odoo auto-generates the `/web/image/` URL

2. **Option B:** Programmatic upload via odoo-pilot
   - See section "Odoo Media Library Integration" in `references/image-workflow.md`
   - Upload file as `ir.attachment`, get ID, update HTML

3. **Option C:** Use Unsplash search
   - Search for relevant keywords at unsplash.com
   - Use direct image URL in `<img src="https://images.unsplash.com/...">`
   - Add attribution comment

### Unsplash Attributions

- Image #2: Photo by [Photographer Name](https://unsplash.com/@username) on [Unsplash](https://unsplash.com)
```

This manifest ensures the user knows:
- How many images are needed
- Which ones are placeholders vs. ready
- Exact dimensions for each
- How to replace them
- Attribution requirements
