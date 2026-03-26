# Landing Page Creation Guide

Reference for creating conversion-focused and informational landing pages for the Foodles website (Odoo). All pages use Bootstrap 5 grid, `iw-*` classes from the design system, and follow foodles-brand guidelines.

## Prerequisites

Before creating a landing page, load these skills:
- **`foodles-brand`** -- Read `references/verbal-identity.md` for tone, vocabulary, and writing style. Read `references/visual-identity.md` for color and typography rules. Key constraints: professional "tu", zero emojis, value vocabulary.
- **`content-collection`** -- Read `references/sitemap-index.md` for interlinking URLs. Load `data/cases.json` for testimonials and social proof. Load `data/videos.json` for contextual video embeds.
- **`components-catalog.md`** -- All section components are copy-paste ready. This guide tells you which ones to use and in what order.

## Table of Contents
1. [Landing Page Types](#landing-page-types)
2. [Page Structure Patterns](#page-structure-patterns)
3. [Section Ordering Guidelines](#section-ordering-guidelines)
4. [CTA Placement Strategy](#cta-placement-strategy)
5. [Content Hierarchy & Messaging Flow](#content-hierarchy--messaging-flow)
6. [Brand Compliance](#brand-compliance)
7. [Content-Collection Integration](#content-collection-integration)
8. [SEO Considerations](#seo-considerations)
9. [Section Composition Guide](#section-composition-guide)
10. [Complete Landing Page Template](#complete-landing-page-template)
11. [Publishing to Odoo](#publishing-to-odoo)

---

## Landing Page Types

| Type | Goal | Key Sections | Typical Sections Count |
|------|------|-------------|----------------------|
| **Conversion** | Lead capture / demo request | Hero + Pain points + Solution + Social proof + CTA | 5-6 |
| **Informational** | Educate about a service/concept | Hero + Explanation + Features + FAQ + CTA | 6-8 |
| **Product Showcase** | Present a module/service | Hero + Feature details + Comparison + Pricing + CTA | 6-7 |
| **Event** | Webinar/event registration | Hero with date + Agenda + Speakers + Registration CTA | 4-5 |

### Conversion Page

Goal: get the visitor to request a demo, contact sales, or fill a form. Every element builds toward one action.
- Example pages: "Solicitar demo", "Contacto Foodles", "Evaluacion gratuita"
- Tone: confident, value-focused, results-oriented
- CTA: appears in hero AND as final section

### Informational Page

Goal: educate and position Foodles as an authority. Longer content, more detail.
- Example pages: "Que es Odoo", "ERP para fabricacion", "Implantacion Odoo"
- Tone: explanatory, authoritative, helpful
- May include FAQ section for SEO

### Product Showcase

Goal: present a specific Foodles Core module or Odoo service with features and pricing.
- Example pages: "Foodles Inventory", "Foodles MRP", "Odoo CRM"
- Tone: technical but accessible, feature-benefit oriented
- Includes comparison (before/after or vs competitors) and pricing

### Event Page

Goal: drive registrations for a specific event.
- Example pages: "Webinar Odoo", "Open Day Foodles", "Demo en vivo"
- Tone: exciting, time-sensitive, clear logistics
- Date/time prominent, agenda visible, single CTA (registration)

---

## Page Structure Patterns

### Conversion Page Pattern

```
1. Hero          -- Headline + subheadline + primary CTA button
                    (iw-hero + iw-bg-gradient-space or solid)

2. Pain Points   -- 3-column icon features showing problems solved
                    (iw-section iw-bg-light + iw-card iw-card-icon)

3. Solution      -- Text + image alternating (2 blocks)
                    (iw-section + iw-feature-list iw-feature-list-check)

4. Key Metrics   -- Stats row with impact numbers
                    (iw-section iw-bg-light + iw-stat iw-stat-gradient)

5. Social Proof  -- Testimonial + case study link
                    (iw-section + iw-testimonial)

6. Final CTA     -- Full-width gradient section
                    (iw-section iw-bg-gradient-space)
```

### Informational Page Pattern

```
1. Hero          -- Headline + introductory paragraph (no CTA or soft CTA)
                    (iw-hero)

2. Core Concept  -- Text + image explaining the main idea
                    (iw-section + 2-column layout)

3. Feature Grid  -- 3-column cards with key capabilities
                    (iw-section iw-bg-light + iw-card iw-card-accent)

4. Process       -- Timeline showing implementation steps
                    (iw-section + iw-timeline or iw-steps)

5. Stats         -- Impact numbers or market data
                    (iw-section iw-bg-light + iw-stat)

6. Video         -- Contextual YouTube embed
                    (iw-section + ratio ratio-16x9)

7. FAQ           -- 3-5 questions for SEO long-tail
                    (iw-section + h3 questions)

8. CTA           -- Final conversion section
                    (iw-section iw-bg-gradient-space)
```

### Product Showcase Pattern

```
1. Hero          -- Product name + value proposition + CTA
                    (iw-hero + badge + iw-btn-primary)

2. Features      -- Alternating text + image (2-3 blocks)
                    (iw-section + 2-column layouts)

3. Comparison    -- Before/after or vs alternatives
                    (iw-section + iw-comparison)

4. Key Figures   -- ROI or impact metrics
                    (iw-section iw-bg-light + iw-key-figure-box)

5. Pricing       -- Pricing cards (if applicable)
                    (iw-section + iw-pricing-card)

6. Testimonial   -- Client quote
                    (iw-section + iw-testimonial)

7. CTA           -- Request demo or contact
                    (iw-section iw-bg-gradient-space)
```

### Event Page Pattern

```
1. Hero          -- Event name + date/time + registration CTA
                    (iw-hero + iw-badge for date)

2. What You'll   -- Feature list of topics/learnings
   Learn            (iw-section + iw-feature-list iw-feature-list-check)

3. Agenda        -- Timeline of the event
                    (iw-section iw-bg-light + iw-timeline)

4. Speakers      -- Card grid with speaker info
                    (iw-section + iw-card)

5. Registration  -- Repeated CTA
   CTA              (iw-section iw-bg-gradient-space)
```

---

## Section Ordering Guidelines

### The Persuasion Flow

Every landing page follows the same psychological sequence:

```
Attention → Problem → Solution → Proof → Action
```

| Position | Purpose | What It Does | Component Type |
|----------|---------|-------------|---------------|
| 1st | **Attention** | Capture interest, state the promise | Hero section |
| 2nd | **Problem / Context** | Create relevance, show understanding | Icon features or text block |
| 3rd | **Solution / Features** | Present the answer | Text + image, feature grid |
| 4th | **Proof** | Build trust with evidence | Stats, testimonials, case studies |
| 5th | **Details** | Address objections | Comparison, FAQ, pricing |
| 6th | **Action** | Convert the visitor | CTA section (gradient) |

### Rules of Thumb

- **Hero is always first** -- no exceptions
- **CTA appears at least twice** -- after the hero (embedded) and as the final section
- **Social proof follows solution claims** -- never make claims without backing them
- **Stats precede or follow testimonials** -- quantitative + qualitative proof together
- **FAQ goes near the bottom** -- captures long-tail SEO, answers remaining objections
- **Never start with features** -- start with the problem or value proposition
- **Alternate backgrounds** -- use `iw-bg-light` on every other section for visual rhythm
- **Max 8 sections** -- beyond this, the page loses focus

---

## CTA Placement Strategy

### Above the Fold (Hero CTA)

Primary CTA in the hero section. Captures high-intent visitors who already know what they want.

```html
<div class="d-flex flex-wrap gap-3">
  <a href="/contacto-foodles" class="iw-btn iw-btn-primary iw-btn-lg">Solicitar demo</a>
  <a href="#funcionalidades" class="iw-btn iw-btn-ghost-light">Ver funcionalidades</a>
</div>
```

- Primary button: action-specific ("Solicitar demo", not "Enviar")
- Optional secondary ghost button: scrolls down for more info
- Two buttons max in the hero

### Mid-Page (Inline CTA)

Callout box after a section that builds strong interest (e.g., after stats or testimonial).

```html
<div class="iw-callout my-4">
  <p class="iw-heading-xs mb-2">Quieres ver estos resultados en tu empresa?</p>
  <p class="iw-body mb-3">Nuestros consultores analizan tu caso y te proponen un plan personalizado.</p>
  <a href="/contacto-foodles" class="iw-btn iw-btn-secondary">Hablar con un consultor</a>
</div>
```

- Use `iw-callout` box for visual distinction
- Text reinforces the value just presented
- Secondary button style (not competing with hero CTA)

### Bottom (Final CTA)

Full-width gradient section. Last chance to convert. High contrast, clear message.

```html
<section class="iw-section iw-bg-gradient-space text-center">
  <div class="container">
    <div class="row justify-content-center">
      <div class="col-lg-8">
        <h2 class="iw-heading-lg text-white mb-3">Listo para transformar tu empresa?</h2>
        <p class="iw-body-lg mb-4" style="color: rgba(255,255,255,0.85);">
          Solicita una demo personalizada y descubre como Odoo puede optimizar tus procesos.
        </p>
        <div class="d-flex flex-wrap gap-3 justify-content-center">
          <a href="/contacto-foodles" class="iw-btn iw-btn-primary iw-btn-lg">Solicitar demo gratuita</a>
          <a href="tel:+34XXXXXXXXX" class="iw-btn iw-btn-ghost-light">Llamar ahora</a>
        </div>
      </div>
    </div>
  </div>
</section>
```

- Gradient background (`iw-bg-gradient-space`) for maximum contrast
- White text on dark background
- Primary + secondary CTA options
- Centered layout, max 8 columns width

### CTA Text Rules

- **Action verbs**: "Solicitar demo", "Ver modulos", "Hablar con un consultor", "Registrarse"
- **Never generic**: avoid "Haz clic aqui", "Enviar", "Leer mas"
- **Value vocabulary** per foodles-brand: "Iniciar tu transformacion", not "Comprar ahora"
- **Max 4 words** on buttons
- **Primary**: yellow button (`iw-btn-primary`). **Secondary**: ghost/outline (`iw-btn-ghost`, `iw-btn-secondary`)

---

## Content Hierarchy & Messaging Flow

### Heading Hierarchy

| Level | Usage | SEO Role | CSS Class |
|-------|-------|----------|-----------|
| H1 | One per page, in hero | Primary keyword | `iw-heading-xl` or `iw-heading-lg` |
| H2 | Section titles (3-6 per page) | Secondary keywords | `iw-heading-lg` or `iw-heading-md` |
| H3 | Sub-features, card titles | Tertiary keywords | `iw-heading-sm` or `iw-heading-xs` |

- **H1 contains the primary keyword** in the first 60 characters
- **H2s are benefit-oriented**, not feature-oriented ("Reduce errores un 60%" not "Modulo de inventario")
- **H3s support H2 claims** with specific features or details

### Writing Rules (per foodles-brand)

- **Professional "tu"** throughout (never "usted", never "vosotros")
- **Zero emojis** -- no exceptions
- **Value vocabulary**: "inversion" not "gasto", "reto" not "problema", "solucion" not "arreglo", "resultado" not "output"
- **Each section**: headline + 2-3 sentences of supporting text + visual element (image, card, stat)
- **Argue every claim** -- never state a benefit without explaining why or showing proof
- **Short paragraphs** -- 2-3 sentences max per paragraph on landing pages (not blog-length)

### Messaging Flow Template

For each section, answer:
1. **What** -- What is this section about? (headline)
2. **Why** -- Why should the reader care? (supporting text)
3. **Proof** -- How do we back this up? (visual: stat, image, testimonial)
4. **Next** -- What should they do next? (CTA or scroll to next section)

---

## Brand Compliance

All landing pages follow `foodles-brand` guidelines. Quick reference:

### Colors

Use `iw-*` CSS classes (not inline styles -- landing pages use the full CSS design system).

| Usage | Class / Token |
|-------|--------------|
| Section backgrounds | `iw-bg-light`, `iw-bg-gradient-space`, `iw-bg-gradient-blue` |
| Text colors | `iw-text-space`, `iw-text-lilac`, `iw-text-yellow` |
| Cards | `iw-card-accent` (top border), `iw-card-gradient` (dark), `iw-card-elevated` |
| Buttons | `iw-btn-primary` (yellow), `iw-btn-secondary` (lilac), `iw-btn-ghost` |

### Typography

- **Dosis** loaded via Google Fonts (Odoo website theme handles this)
- Weights: 400 (body), 600 (semi-bold labels), 700 (headings, buttons)
- Fallback: Verdana, Arial, sans-serif

### Logo

- Logo is handled by Odoo's header/footer -- **do not place the logo in page content**
- If a standalone page requires branding, use `iw-overline` text ("FOODLES") instead of an image

### Voice

- Professional "tu" address
- Zero emojis
- Value vocabulary per foodles-brand

---

## Content-Collection Integration

### Case Studies as Social Proof

Load `data/cases.json` from `content-collection`. Select 1-2 relevant cases based on industry or module match.

**As a testimonial section:**

```html
<section class="iw-section">
  <div class="container">
    <div class="row justify-content-center">
      <div class="col-lg-8">
        <div class="iw-testimonial">
          <p class="iw-testimonial-text">
            "Gracias a la implantacion de Odoo con Foodles, hemos reducido un 40% el tiempo de procesamiento de pedidos y eliminado los errores de datos duplicados."
          </p>
          <p class="iw-testimonial-author">Maria Lopez</p>
          <p class="iw-testimonial-role">Directora de Operaciones, Empresa XYZ</p>
        </div>
        <div class="text-center mt-4">
          <a href="/blog/referencias-foodles-16/caso-empresa-xyz-456" class="iw-btn iw-btn-ghost iw-btn-sm">
            Ver caso de exito completo
          </a>
        </div>
      </div>
    </div>
  </div>
</section>
```

### YouTube Video Embed

Load `data/videos.json` from `content-collection`. Find the most relevant video by topic matching.

```html
<section class="iw-section">
  <div class="container">
    <div class="row justify-content-center">
      <div class="col-lg-8">
        <h2 class="iw-heading-md mb-3 text-center">Ve Odoo en accion</h2>
        <div class="ratio ratio-16x9 iw-img-rounded iw-img-shadow">
          <iframe
            src="https://www.youtube.com/embed/VIDEO_ID"
            title="Titulo del video"
            frameborder="0"
            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
            allowfullscreen
            loading="lazy"
          ></iframe>
        </div>
        <p class="iw-caption text-center mt-2">Titulo del video</p>
      </div>
    </div>
  </div>
</section>
```

### Internal Links (Sitemap)

Load `data/sitemap.json` from `content-collection`. Link to related services in feature descriptions and text blocks.

- Link to service pages when mentioning functionalities: `<a href="/odoo-crm">gestion de clientes</a>`
- Link to blog articles for deeper explanations: `<a href="/blog/blog-foodles-1/articulo-123">como funciona</a>`
- Link to case studies for real examples: `<a href="/blog/referencias-foodles-16/caso-456">caso de exito</a>`

**Distribution**: 3-6 internal links per landing page, naturally distributed across text sections.

---

## SEO Considerations

### Meta Title

- **Max 60 characters** (Google truncates beyond this)
- **Primary keyword first** + brand name
- Pattern: `[Keyword principal] | Foodles`
- Example: `Implantacion Odoo para Fabricacion | Foodles`

### Meta Description

- **150-160 characters**
- Include primary keyword + value proposition
- End with action language
- Example: `Implanta Odoo en tu empresa de fabricacion con resultados en 4 semanas. Consultores certificados, plan personalizado. Solicita tu demo gratuita.`

### URL Slug

- **Max 5 words**, no stop words
- Primary keyword
- Lowercase, hyphens only
- Example: `/implantacion-odoo-fabricacion`

### Heading Hierarchy for SEO

```html
<h1>Implantacion Odoo para Fabricacion</h1>          <!-- Primary keyword -->
  <h2>Los retos de la fabricacion sin ERP</h2>         <!-- Problem framing -->
  <h2>Como Odoo transforma tus procesos</h2>          <!-- Solution -->
    <h3>Planificacion de produccion</h3>               <!-- Feature detail -->
    <h3>Control de calidad integrado</h3>              <!-- Feature detail -->
  <h2>Resultados medibles en semanas</h2>              <!-- Proof -->
  <h2>Preguntas frecuentes</h2>                        <!-- FAQ for long-tail -->
```

### Schema JSON-LD (Optional)

For landing pages, use `WebPage` schema (simpler than blog's Article+FAQ):

```html
<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@type": "WebPage",
  "name": "Implantacion Odoo para Fabricacion",
  "description": "Meta description aqui (150-160 chars)",
  "url": "https://www.foodles.es/implantacion-odoo-fabricacion",
  "publisher": {
    "@type": "Organization",
    "name": "Foodles Business Solutions",
    "url": "https://www.foodles.es",
    "logo": {
      "@type": "ImageObject",
      "url": "https://www.foodles.es/web/image/website/1/logo"
    }
  },
  "inLanguage": "es"
}
</script>
```

If the page includes an FAQ section, add `FAQPage` as a second graph entry (same pattern as `blog-creation.md`).

---

## Section Composition Guide

Reference this table to build a page from `components-catalog.md`:

| Section Need | Component from Catalog | Catalog Section | Notes |
|---|---|---|---|
| Full-width header | Hero with gradient or Hero minimal | Hero Sections | Choose gradient for conversion, minimal for informational |
| Wave separator | Hero + `iw-section-wave` | Hero Sections | Decorative bottom edge |
| Problem / pain points | 3-column icon features | Feature Grids | Use `iw-card-icon` with problem-oriented icons |
| Detailed feature | 2-column text + image | Feature Grids | Use `iw-feature-list-check` for benefit lists |
| Service cards | Card grid with accent borders | Cards | Use `iw-card-accent` + badges |
| Key numbers | Stats row | Statistics / KPIs | Use `iw-stat-gradient` for colored numbers |
| Client quote | Single testimonial | Testimonials | Use `iw-testimonial` with left border |
| Bold statement | Large blockquote | Testimonials | Use `iw-blockquote` for emphasis |
| Pricing options | Pricing card grid | Pricing | Mark recommended with `iw-pricing-featured` |
| Process steps | Vertical timeline | Timeline / Process | Use `iw-timeline` for 3-5 steps |
| Horizontal steps | Step counter | Timeline / Process | Use `iw-steps` for parallel processes |
| Conversion section | CTA gradient | Call to Action | Use `iw-bg-gradient-space` + primary button |
| Mid-page callout | CTA inline | Call to Action | Use `iw-callout` + secondary button |
| Comparison | Two-column vs block | Blog Components | Use `iw-comparison` (positive/negative) |
| Before / After | Same as comparison | Blog Components | Label columns with before/after |
| FAQ section | Question + answer blocks | Blog Components | Use H3 questions + iw-body answers |

### Section Background Alternation

For visual rhythm, alternate between white and light backgrounds:

```
Section 1 (Hero):      iw-bg-gradient-space (dark)
Section 2 (Features):  (white, default)
Section 3 (Details):   iw-bg-light (light gray)
Section 4 (Stats):     (white)
Section 5 (Proof):     iw-bg-light
Section 6 (CTA):       iw-bg-gradient-space (dark)
```

---

## Complete Landing Page Template

Full conversion-focused landing page ready to paste into Odoo Website Builder. Replace all placeholder content.

```html
<!-- Hero Section -->
<section class="iw-hero">
  <div class="container">
    <div class="row align-items-center" style="min-height: 50vh;">
      <div class="col-lg-7">
        <p class="iw-overline mb-3" style="color: #E5B92B;">FOODLES BUSINESS SOLUTIONS</p>
        <h1 class="iw-heading-xl text-white mb-4">
          Titulo principal con keyword (max 60 chars)
        </h1>
        <p class="iw-body-lg mb-4" style="color: rgba(255,255,255,0.85);">
          Subtitulo que expande la propuesta de valor. Dos frases que explican el beneficio
          principal y por que el visitante deberia actuar ahora.
        </p>
        <div class="d-flex flex-wrap gap-3">
          <a href="/contacto-foodles" class="iw-btn iw-btn-primary iw-btn-lg">Solicitar demo</a>
          <a href="#funcionalidades" class="iw-btn iw-btn-ghost-light">Ver funcionalidades</a>
        </div>
      </div>
      <div class="col-lg-5 mt-4 mt-lg-0">
        <img src="/web/image/ir.attachment/hero-image" alt="Descripcion de la imagen"
          class="img-fluid iw-img-rounded iw-img-shadow">
      </div>
    </div>
  </div>
</section>

<!-- Pain Points / Problem Section -->
<section class="iw-section iw-bg-light" id="funcionalidades">
  <div class="container">
    <div class="row justify-content-center mb-5">
      <div class="col-lg-8 text-center">
        <p class="iw-overline mb-2">LOS RETOS</p>
        <h2 class="iw-heading-lg mb-3">Titulo de la seccion de problemas</h2>
        <p class="iw-body">
          Breve introduccion que empatiza con los retos del visitante.
        </p>
      </div>
    </div>
    <div class="row g-4 iw-stagger">
      <div class="col-lg-4">
        <div class="iw-card iw-card-icon h-100">
          <div class="iw-icon-wrapper mb-3">
            <img src="/web/image/ir.attachment/icon-1" alt="" width="32" height="32">
          </div>
          <h3 class="iw-heading-xs mb-2">Primer reto</h3>
          <p class="iw-body">
            Descripcion del problema que el visitante reconoce como propio.
          </p>
        </div>
      </div>
      <div class="col-lg-4">
        <div class="iw-card iw-card-icon h-100">
          <div class="iw-icon-wrapper mb-3">
            <img src="/web/image/ir.attachment/icon-2" alt="" width="32" height="32">
          </div>
          <h3 class="iw-heading-xs mb-2">Segundo reto</h3>
          <p class="iw-body">
            Otro problema comun que la solucion resuelve.
          </p>
        </div>
      </div>
      <div class="col-lg-4">
        <div class="iw-card iw-card-icon h-100">
          <div class="iw-icon-wrapper mb-3">
            <img src="/web/image/ir.attachment/icon-3" alt="" width="32" height="32">
          </div>
          <h3 class="iw-heading-xs mb-2">Tercer reto</h3>
          <p class="iw-body">
            Tercer punto de dolor que conecta con el publico objetivo.
          </p>
        </div>
      </div>
    </div>
  </div>
</section>

<!-- Solution Section (Text + Image) -->
<section class="iw-section">
  <div class="container">
    <div class="row align-items-center g-5">
      <div class="col-lg-6">
        <p class="iw-overline mb-2">LA SOLUCION</p>
        <h2 class="iw-heading-md mb-3">Como Odoo resuelve estos retos</h2>
        <p class="iw-body mb-3">
          Parrafo explicando la solucion de forma concreta. Conectar cada beneficio
          con un reto mencionado en la seccion anterior.
        </p>
        <ul class="iw-feature-list iw-feature-list-check">
          <li>Primer beneficio concreto y medible</li>
          <li>Segundo beneficio con resultado esperado</li>
          <li>Tercer beneficio diferenciador</li>
          <li>Cuarto beneficio relevante para el publico</li>
        </ul>
        <a href="/contacto-foodles" class="iw-btn iw-btn-secondary mt-3">Saber mas</a>
      </div>
      <div class="col-lg-6">
        <div class="iw-img-accent">
          <img src="/web/image/ir.attachment/screenshot-solution" alt="Captura de Odoo mostrando la funcionalidad"
            class="img-fluid iw-img-rounded">
        </div>
      </div>
    </div>
  </div>
</section>

<!-- Stats Section -->
<section class="iw-section iw-bg-light">
  <div class="container">
    <div class="row justify-content-center mb-4">
      <div class="col-lg-8 text-center">
        <h2 class="iw-heading-md mb-3">Resultados medibles</h2>
        <p class="iw-body">
          Datos reales de empresas que han implantado Odoo con Foodles.
        </p>
      </div>
    </div>
    <div class="row g-4 text-center">
      <div class="col-6 col-md-3">
        <div class="iw-stat iw-stat-gradient">
          <span class="iw-stat-value">+85%</span>
          <span class="iw-stat-label">Eficiencia operativa</span>
        </div>
      </div>
      <div class="col-6 col-md-3">
        <div class="iw-stat iw-stat-gradient">
          <span class="iw-stat-value">-40%</span>
          <span class="iw-stat-label">Costes de proceso</span>
        </div>
      </div>
      <div class="col-6 col-md-3">
        <div class="iw-stat iw-stat-gradient">
          <span class="iw-stat-value">4 sem.</span>
          <span class="iw-stat-label">Tiempo implantacion</span>
        </div>
      </div>
      <div class="col-6 col-md-3">
        <div class="iw-stat iw-stat-gradient">
          <span class="iw-stat-value">99%</span>
          <span class="iw-stat-label">Satisfaccion</span>
        </div>
      </div>
    </div>
  </div>
</section>

<!-- Testimonial Section -->
<section class="iw-section">
  <div class="container">
    <div class="row justify-content-center">
      <div class="col-lg-8">
        <div class="iw-testimonial">
          <p class="iw-testimonial-text">
            "Cita real de un cliente satisfecho describiendo el impacto concreto
            que Odoo ha tenido en su empresa. Dos o tres frases con datos."
          </p>
          <p class="iw-testimonial-author">Nombre del Cliente</p>
          <p class="iw-testimonial-role">Cargo, Empresa</p>
        </div>
        <div class="text-center mt-4">
          <a href="/blog/referencias-foodles-16/caso-cliente-123" class="iw-btn iw-btn-ghost iw-btn-sm">
            Ver caso de exito completo
          </a>
        </div>
      </div>
    </div>
  </div>
</section>

<!-- Final CTA Section -->
<section class="iw-section iw-bg-gradient-space text-center">
  <div class="container">
    <div class="row justify-content-center">
      <div class="col-lg-8">
        <h2 class="iw-heading-lg text-white mb-3">Listo para transformar tu empresa?</h2>
        <p class="iw-body-lg mb-4" style="color: rgba(255,255,255,0.85);">
          Solicita una demo personalizada y descubre como Odoo puede optimizar tus procesos.
          Nuestros consultores analizan tu caso y te proponen un plan a medida.
        </p>
        <div class="d-flex flex-wrap gap-3 justify-content-center">
          <a href="/contacto-foodles" class="iw-btn iw-btn-primary iw-btn-lg">Solicitar demo gratuita</a>
          <a href="tel:+34XXXXXXXXX" class="iw-btn iw-btn-ghost-light">Llamar ahora</a>
        </div>
      </div>
    </div>
  </div>
</section>
```

---

## Publishing to Odoo

### Option A: Paste into Website Builder (Recommended)

The simplest and most reliable method:

1. Navigate to **Website > Pages > New Page** in Odoo
2. Give the page a name and URL slug
3. In the Odoo Editor, switch to **Code View** (or use the HTML block)
4. Paste each section as a separate building block
5. Upload images via Odoo's media dialog and update `src` attributes
6. Set SEO fields: **Promote > SEO** tab for meta title and description
7. Save as draft, preview, publish when ready

### Option B: Programmatic via odoo-pilot (Advanced)

For automated page creation or bulk operations.

#### 1. Authenticate with Odoo

```bash
eval $(./scripts/auth.sh)
```

#### 2. Create the Page

Odoo website pages are stored as `website.page` records linked to `ir.ui.view` records.

```bash
./scripts/create_record.sh website.page '{
  "name": "Titulo de la pagina",
  "url": "/page-slug",
  "is_published": false,
  "website_meta_title": "Titulo SEO | Foodles",
  "website_meta_description": "Meta description con keyword principal y propuesta de valor (150-160 chars)"
}'
```

The `website.page` model creates the page with basic metadata. The page content (HTML) is typically edited through the Website Builder after creation.

**Note**: For programmatic content injection, the page architecture uses QWeb templates stored in `ir.ui.view`. This requires wrapping HTML in Odoo's template syntax:

```xml
<t t-name="website.page_slug">
  <t t-call="website.layout">
    <!-- Your HTML sections here -->
  </t>
</t>
```

This is complex and error-prone. **Prefer Option A** (paste into Website Builder) for most use cases.

#### 3. Confirm to User

```
Pagina creada como borrador en Odoo:
- ID: [record_id]
- URL de edicion: [ODOO_URL]/web#id=[record_id]&model=website.page&view_type=form
- URL de preview: [ODOO_URL]/page-slug
- Estado: No publicada (borrador para revision)
- Siguiente paso: Editar en el Website Builder para ajustar imagenes y contenido
```

### Important Notes

- **Always create as unpublished** -- the page must be reviewed by a human before publishing
- **Images**: Upload to Odoo first, then reference via `/web/image/` URLs in the HTML
- **CSS**: Ensure `foodles-frontend.css` is loaded in the Odoo website (see SKILL.md > CSS Installation)
- **Odoo theme**: The page inherits the Odoo website theme's header, footer, and base styles. The `iw-*` classes are additive
- **Paste-first is safer** -- programmatic creation via `website.page` is useful for batch operations but the Website Builder gives immediate visual feedback
