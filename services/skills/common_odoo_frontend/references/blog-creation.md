# Blog Creation Guide

Reference for generating SEO-optimized, AI-consumable blog articles for the Enterprise Demo website (Odoo). All blogs follow the BLUF + Body + FAQs + Schema structure.

## Prerequisites

Before creating a blog, load these skills:
- **`enterprise-demo-brand`** -- Read `references/verbal-identity.md` for tone, vocabulary, and writing style. Read `references/visual-identity.md` for color and typography rules. Key constraints: professional "tu", zero emojis, value vocabulary (inversion not gasto, reto not problema), long-form explanatory paragraphs, argue every recommendation.
- **`content-collection`** -- Read `references/sitemap-index.md` for interlinking URLs. Load `data/videos.json` for YouTube embeds. Load `data/cases.json` for case study references.

**Image handling:** Read `image-workflow.md` for the complete image system. Always ask the user about images in Phase 1 (AskUserQuestion with options: user-provided, Unsplash, or placeholders). Never use generic placeholder URLs -- use inline SVG placeholders with Enterprise Demo brand colors.

## Table of Contents
1. [Blog Structure (BLUF)](#blog-structure)
2. [SEO & AI Optimization](#seo--ai-optimization)
3. [Image Requirements](#image-requirements)
4. [Visual Design Blocks](#visual-design-blocks)
5. [Interlinking Strategy](#interlinking-strategy)
6. [YouTube Video Embedding](#youtube-video-embedding)
7. [FAQ Section](#faq-section)
8. [Schema JSON-LD](#schema-json-ld)
9. [Complete Blog Template](#complete-blog-template)
10. [Publishing to Odoo](#publishing-to-odoo)

---

## Blog Structure

Every blog follows the **BLUF** (Bottom Line Up Front) pattern: answer the question first, then develop.

### Mandatory Sections (in order)

```
1. BLUF (resumen ejecutivo)         ~100-150 words
2. Indice de contenidos             Auto-generated from H2/H3
3. Cuerpo del articulo              ~1200-1500 words (3-5 H2 sections)
   - Interlinking (2-4 internal links per section)
   - YouTube video embed (1 contextual video)
   - Keywords naturales
4. Seccion de FAQs                  3-5 preguntas con respuestas
5. Schema JSON-LD                   Article + FAQPage combined
```

### Minimum Length

Target: **1500+ words** (excluding BLUF, FAQs, and schema). This is critical for SEO depth and AI consumption. Each H2 section should be **250-400 words** of substantive content.

---

## SEO & AI Optimization

### HTML Heading Hierarchy

The blog MUST follow a strict heading hierarchy for both Google and LLM indexability:

```html
<h1>Single H1 -- the blog title (set by Odoo, not in body)</h1>

<!-- BLUF -->
<div class="iw-callout">
  <p>Resumen ejecutivo...</p>
</div>

<!-- Table of contents -->
<nav>
  <h2>Indice de contenidos</h2>
  <ol>
    <li><a href="#seccion-1">Seccion 1</a></li>
    ...
  </ol>
</nav>

<!-- Body -->
<h2 id="seccion-1">Seccion con keyword principal</h2>
<p>Paragraphs (not bullet-only)...</p>

<h3>Subtema</h3>
<p>More depth...</p>

<h2 id="seccion-2">Segunda seccion</h2>
...

<!-- FAQs -->
<h2 id="preguntas-frecuentes">Preguntas frecuentes</h2>
...

<!-- Schema -->
<script type="application/ld+json">...</script>
```

### Keyword Placement Rules

1. **Title (H1)**: Primary keyword in first 60 characters
2. **BLUF**: Primary keyword in first sentence
3. **First H2**: Primary keyword naturally included
4. **Body**: Secondary keywords distributed across sections (2-3 per H2 section)
5. **FAQs**: Long-tail keyword variants as questions
6. **URL slug**: Primary keyword, max 5 words, no stop words

### Meta Description (for Odoo SEO fields)

- 150-160 characters
- Include primary keyword
- Include a value proposition or answer
- End with action-oriented language

### Content Quality Rules

Per enterprise-demo-brand voice:
- Write in **long-form explanatory paragraphs** (never bullet-only)
- Use **professional "tu"** address
- Use **value vocabulary** (inversion, not gasto; reto, not problema)
- **Zero emojis**
- Every recommendation must be **argued with reasoning**
- Minimum 3 paragraphs per H2 section

---

## Image Requirements

Every blog must include **at least 2 images**:
1. **Hero image** (top of article, after title): 1200x600px, contextually relevant to topic
2. **Section images** (distributed across H2 sections): 800x600px, supporting visual content

### Image Discovery Process

Follow `image-workflow.md` > "Image Sources & Discovery":

1. **Ask user** via AskUserQuestion (already done in Phase 1): "Do you have images?"
   - If YES: use user-provided URLs or file paths
   - If "Use Unsplash": search Unsplash API with blog topic keywords, present top 3, get user selection
   - If "Placeholders": generate inline SVG placeholders with Enterprise Demo brand colors

2. **Never use** generic placeholder services (placeholder.com, lorempixel, etc.)

### Image Implementation

**Hero image** (required, top of article):
```html
<!-- Hero Image: [DESCRIPTION] -->
<div class="mb-4">
  <img src="[URL_OR_PLACEHOLDER]"
       alt="[Descriptive alt text matching topic]"
       class="img-fluid rounded"
       style="width: 100%; max-width: 1200px; height: auto;"/>
</div>
<!-- [If Unsplash: Photo by Name on Unsplash] -->
```

**Inline SVG placeholder example** (if no real image):
```html
<div class="mb-4">
  <img src="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 1200 600'%3E%3Crect fill='%231C265D' width='1200' height='600'/%3E%3Ctext x='50%25' y='45%25' dominant-baseline='middle' text-anchor='middle' font-family='Dosis' font-size='28' font-weight='600' fill='%23ffffff'%3EHero: Odoo Dashboard Overview%3C/text%3E%3Ctext x='50%25' y='58%25' dominant-baseline='middle' text-anchor='middle' font-family='Dosis' font-size='18' fill='%23E5B92B'%3E1200 x 600px • Replace via Odoo Media%3C/text%3E%3C/svg%3E"
       alt="Placeholder: Odoo dashboard hero image"
       class="img-fluid rounded"/>
</div>
<!-- Replace via: Odoo > Website > Edit > Click image > Upload -->
```

**Section images** (1-2 per article, within H2 sections):
- Smaller: 800x600px recommended
- Same strategy: user-provided, Unsplash, or inline SVG placeholder
- Place contextually within the most visual H2 sections

### Image Manifest

After generating the blog, create an Image Manifest (see `image-workflow.md` > "Image Manifest Template") listing all images, their status (ready vs. placeholder), dimensions, and replacement instructions.

---

## Visual Design Blocks

Blogs are not just text. Break visual monotony by inserting **at least 2 design blocks** per article from the list below. These use `iw-*` classes from `assets/css/enterprise-demo-frontend.css` and Bootstrap 5 grid -- fully compatible with Odoo Editor.

### When to Use Each Block

| Block | Best For | Where to Place |
|-------|----------|----------------|
| Bento grid | Feature highlights, multi-point overviews | After introductory paragraph of a section |
| Comparison table | Before/after, option A vs B, old vs new | When contrasting two approaches |
| Pros/Cons | Evaluating a tool, method, or strategy | Within analysis or evaluation sections |
| Key figures | Impressive metrics, ROI data | Opening of a data-heavy section |
| Summary box | TL;DR, key takeaways | End of a complex section or end of article |
| Timeline | Implementation phases, historical evolution | Process descriptions |
| Stats row | Multiple KPIs side-by-side | Results or impact sections |
| Data table | Structured comparisons, feature matrices | Technical or spec-heavy sections |

### Bento Grid

Asymmetric card layout for visual interest. Use to present 3-5 related points.

```html
<div class="iw-bento my-4">
  <div class="iw-bento-item iw-bento-lg iw-bento-dark">
    <p class="iw-overline mb-2" style="color: #E5B92B;">PUNTO PRINCIPAL</p>
    <h3 class="iw-heading-xs">El concepto mas importante del articulo</h3>
    <p class="iw-body mt-2" style="color: rgba(255,255,255,0.8);">Explicacion breve que desarrolla el punto.</p>
  </div>
  <div class="iw-bento-item iw-bento-sm iw-bento-accent">
    <p class="iw-overline mb-2">DATO CLAVE</p>
    <p class="iw-key-figure-value">+85%</p>
    <p class="iw-caption">Mejora en eficiencia</p>
  </div>
  <div class="iw-bento-item iw-bento-sm">
    <h3 class="iw-heading-xs mb-2">Punto secundario</h3>
    <p class="iw-caption">Detalle complementario que refuerza el argumento principal.</p>
  </div>
  <div class="iw-bento-item iw-bento-sm iw-bento-gradient">
    <h3 class="iw-heading-xs" style="color: #fff;">Beneficio clave</h3>
    <p class="iw-caption mt-2" style="color: rgba(255,255,255,0.85);">Resultado medible y concreto.</p>
  </div>
  <div class="iw-bento-item iw-bento-md">
    <h3 class="iw-heading-xs mb-2">Otro punto relevante</h3>
    <p class="iw-caption">Informacion adicional que aporta valor al lector.</p>
  </div>
</div>
```

### Comparison Table (vs)

Two-column comparison. Use `iw-comparison-negative` for the "before/without" side and `iw-comparison-positive` for the "after/with" side.

```html
<div class="iw-comparison my-4">
  <div class="iw-comparison-col iw-comparison-negative">
    <div class="iw-comparison-header">Sin Odoo</div>
    <p class="iw-comparison-item">Procesos manuales con hojas de calculo</p>
    <p class="iw-comparison-item">Datos duplicados entre departamentos</p>
    <p class="iw-comparison-item">Informes que tardan dias en generarse</p>
    <p class="iw-comparison-item">Sin visibilidad en tiempo real</p>
  </div>
  <div class="iw-comparison-col iw-comparison-positive">
    <div class="iw-comparison-header">Con Odoo</div>
    <p class="iw-comparison-item">Flujos automatizados de principio a fin</p>
    <p class="iw-comparison-item">Base de datos unica y centralizada</p>
    <p class="iw-comparison-item">Dashboards en tiempo real</p>
    <p class="iw-comparison-item">Trazabilidad completa de operaciones</p>
  </div>
</div>
```

### Pros / Cons

Vertical list with +/- indicators. Use when evaluating a single topic.

```html
<div class="iw-pros-cons my-4">
  <div>
    <p class="iw-pros-title">Ventajas</p>
    <ul class="iw-pros-list">
      <li>Integracion nativa entre modulos</li>
      <li>Coste predecible por usuario</li>
      <li>Codigo abierto y personalizable</li>
    </ul>
  </div>
  <div>
    <p class="iw-cons-title">Consideraciones</p>
    <ul class="iw-cons-list">
      <li>Curva de aprendizaje inicial</li>
      <li>Requiere consultoria para configuracion avanzada</li>
      <li>Personalizaciones complejas requieren desarrollo</li>
    </ul>
  </div>
</div>
```

### Key Figures (inline metrics)

Use to highlight a metric mid-text or as a row of boxed figures.

```html
<!-- Inline in text -->
<p class="iw-body mb-3">
  Los resultados fueron claros:
  <span class="iw-key-figure">
    <span class="iw-key-figure-value">3x</span>
    <span class="iw-key-figure-label">mas rapido</span>
  </span>
  en el procesamiento de pedidos.
</p>

<!-- Row of boxed figures -->
<div class="row g-3 my-4">
  <div class="col-md-4">
    <div class="iw-key-figure-box">
      <span class="iw-key-figure-value">+200%</span>
      <span class="iw-key-figure-label">Productividad</span>
    </div>
  </div>
  <div class="col-md-4">
    <div class="iw-key-figure-box">
      <span class="iw-key-figure-value">-60%</span>
      <span class="iw-key-figure-label">Errores manuales</span>
    </div>
  </div>
  <div class="col-md-4">
    <div class="iw-key-figure-box">
      <span class="iw-key-figure-value">4 sem.</span>
      <span class="iw-key-figure-label">Tiempo de implantacion</span>
    </div>
  </div>
</div>
```

### Summary Box (TL;DR / takeaways)

Dark box with key points. Place at the end of a complex section or as a recap before FAQs.

```html
<div class="iw-summary-box my-4">
  <p class="iw-summary-box-title">Puntos clave</p>
  <ul>
    <li>Primer punto importante resumido en una frase</li>
    <li>Segundo punto con el dato mas relevante</li>
    <li>Tercer punto con la accion recomendada</li>
  </ul>
</div>
```

### Timeline (within blog text)

Reuse the timeline component from the main CSS for process descriptions.

```html
<div class="iw-timeline my-4">
  <div class="iw-timeline-item">
    <h3 class="iw-heading-xs mb-1">Fase 1: Analisis</h3>
    <p class="iw-caption">Evaluacion de procesos actuales y definicion de objetivos.</p>
  </div>
  <div class="iw-timeline-item">
    <h3 class="iw-heading-xs mb-1">Fase 2: Configuracion</h3>
    <p class="iw-caption">Parametrizacion del sistema y migracion de datos.</p>
  </div>
  <div class="iw-timeline-item">
    <h3 class="iw-heading-xs mb-1">Fase 3: Formacion</h3>
    <p class="iw-caption">Capacitacion del equipo y puesta en marcha.</p>
  </div>
</div>
```

### Data Table

Use `iw-table` for structured data comparisons.

```html
<div class="table-responsive my-4">
  <table class="iw-table">
    <thead>
      <tr>
        <th>Funcionalidad</th>
        <th>Basico</th>
        <th>Profesional</th>
        <th>Empresa</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>Usuarios</td>
        <td>5</td>
        <td>25</td>
        <td>Ilimitados</td>
      </tr>
      <tr>
        <td>Modulos</td>
        <td>3</td>
        <td>10</td>
        <td>Todos</td>
      </tr>
      <tr>
        <td>Soporte</td>
        <td>Email</td>
        <td>Prioritario</td>
        <td>Dedicado</td>
      </tr>
    </tbody>
  </table>
</div>
```

### Stats Row (KPIs)

Horizontal row of large numbers. Reuses `iw-stat` from main CSS.

```html
<div class="row g-4 text-center my-4 py-3 iw-bg-light" style="border-radius: var(--iw-radius-lg, 12px);">
  <div class="col-6 col-md-3">
    <div class="iw-stat iw-stat-gradient">
      <span class="iw-stat-value">+85%</span>
      <span class="iw-stat-label">Eficiencia</span>
    </div>
  </div>
  <div class="col-6 col-md-3">
    <div class="iw-stat iw-stat-gradient">
      <span class="iw-stat-value">-40%</span>
      <span class="iw-stat-label">Costes operativos</span>
    </div>
  </div>
  <div class="col-6 col-md-3">
    <div class="iw-stat iw-stat-gradient">
      <span class="iw-stat-value">2x</span>
      <span class="iw-stat-label">Velocidad</span>
    </div>
  </div>
  <div class="col-6 col-md-3">
    <div class="iw-stat iw-stat-gradient">
      <span class="iw-stat-value">99%</span>
      <span class="iw-stat-label">Satisfaccion</span>
    </div>
  </div>
</div>
```

### Design Block Selection Guidelines

Choose blocks based on the blog topic:

| Blog Type | Recommended Blocks |
|-----------|-------------------|
| "How to" / Tutorial | Timeline + Key figures + Summary box |
| Product comparison | Comparison table + Pros/Cons + Data table |
| Case study / Results | Stats row + Bento grid + Key figures |
| Strategy / Analysis | Pros/Cons + Comparison + Summary box |
| Feature announcement | Bento grid + Stats row + Data table |

---

## Interlinking Strategy

### How to Interlink

Use the `content-collection` skill data for internal links:

1. **Load** `data/sitemap.json` from content-collection
2. **Filter** by category: `blog`, `service`, `page`, `case_study`
3. **Match** by topic relevance (keyword overlap with URL slug or title_hint)
4. **Place** 2-4 internal links per H2 section as natural inline links

### Link Types

| Type | When to Use | Example |
|------|-------------|---------|
| **Service page** | When mentioning a functionality | `<a href="/odoo-crm">gestion de relaciones con clientes</a>` |
| **Related blog** | When referencing a related topic | `<a href="/blog/blog-enterprise-demo-1/articulo-456">como configurar tu CRM</a>` |
| **Case study** | When citing a real example | `<a href="/blog/referencias-enterprise-demo-16/cliente-789">caso de exito de [Cliente]</a>` |
| **Landing page** | When mentioning a broad concept | `<a href="/que-es-odoo">plataforma ERP integral</a>` |

### Link Anchor Text Rules

- Use **descriptive anchor text** (never "click here" or "read more")
- Anchor text should include relevant keywords
- Vary anchor text for links to the same page
- Place links in the natural flow of text, not at the end of paragraphs

### Links Per Blog

- **Minimum**: 6 internal links total
- **Maximum**: 15 internal links total
- **Distribution**: 2-4 per H2 section
- **Variety**: Mix of service pages, blogs, and case studies

---

## YouTube Video Embedding

### Finding the Right Video

1. **Load** `data/videos.json` from content-collection
2. **Search** by topic matching (compare blog keywords with video `topics` array and `title`)
3. **Pick** the most relevant video for the blog's primary topic

### Embed Format

Place the video embed **within the body**, at a contextually relevant point (after the section that introduces the topic the video covers).

```html
<div class="ratio ratio-16x9 my-4 iw-img-rounded iw-img-shadow">
  <iframe
    src="https://www.youtube.com/embed/VIDEO_ID"
    title="Video title here"
    frameborder="0"
    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
    allowfullscreen
    loading="lazy"
  ></iframe>
</div>
<p class="iw-caption text-center">Video title here</p>
```

### Video Placement Rules

- **One video per blog** (contextual, not a separate section)
- Place it **after the relevant H2 section** that discusses the topic
- Always include `title` attribute for accessibility
- Always include `loading="lazy"` for performance
- Use Bootstrap `ratio ratio-16x9` for responsive sizing
- Add `iw-img-rounded iw-img-shadow` for consistent design treatment

---

## FAQ Section

### Structure

3-5 questions, each answering a long-tail search query related to the blog topic.

```html
<section id="preguntas-frecuentes">
  <h2 class="iw-heading-md mb-4">Preguntas frecuentes</h2>

  <div class="mb-4">
    <h3 class="iw-heading-xs mb-2">Pregunta en formato natural?</h3>
    <p class="iw-body">
      Respuesta completa en 2-4 frases. Debe responder la pregunta
      directamente sin rodeos, con datos concretos cuando sea posible.
    </p>
  </div>

  <div class="mb-4">
    <h3 class="iw-heading-xs mb-2">Segunda pregunta?</h3>
    <p class="iw-body">
      Respuesta directa y util.
    </p>
  </div>
</section>
```

### FAQ Rules

- Questions use natural language (how people actually search)
- Each answer: 2-4 sentences, direct, factual
- Include primary and secondary keywords naturally
- Questions should target **long-tail keywords** not covered in the body
- FAQs must match the Schema JSON-LD `FAQPage` exactly

---

## Schema JSON-LD

Every blog includes two schemas combined: `Article` + `FAQPage`.

Place this as the LAST element in the blog body:

```html
<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@graph": [
    {
      "@type": "Article",
      "headline": "Titulo del articulo (max 110 chars)",
      "description": "Meta description (max 160 chars)",
      "author": {
        "@type": "Organization",
        "name": "Enterprise Demo Business Solutions",
        "url": "https://www.enterprise-demo.es"
      },
      "publisher": {
        "@type": "Organization",
        "name": "Enterprise Demo Business Solutions",
        "url": "https://www.enterprise-demo.es",
        "logo": {
          "@type": "ImageObject",
          "url": "https://www.enterprise-demo.es/web/image/website/1/logo"
        }
      },
      "datePublished": "YYYY-MM-DD",
      "dateModified": "YYYY-MM-DD",
      "mainEntityOfPage": {
        "@type": "WebPage",
        "@id": "https://www.enterprise-demo.es/blog/blog-enterprise-demo-1/SLUG"
      },
      "image": "https://www.enterprise-demo.es/web/image/blog.post/ID/cover_properties",
      "articleSection": "Tecnologia",
      "inLanguage": "es"
    },
    {
      "@type": "FAQPage",
      "mainEntity": [
        {
          "@type": "Question",
          "name": "Pregunta 1?",
          "acceptedAnswer": {
            "@type": "Answer",
            "text": "Respuesta 1."
          }
        },
        {
          "@type": "Question",
          "name": "Pregunta 2?",
          "acceptedAnswer": {
            "@type": "Answer",
            "text": "Respuesta 2."
          }
        }
      ]
    }
  ]
}
</script>
```

### Schema Rules

- `headline`: Max 110 characters (Google truncates at 110)
- `description`: Same as meta description (150-160 chars)
- `datePublished` and `dateModified`: ISO 8601 format (YYYY-MM-DD)
- `image`: Must be a valid URL to the blog cover image
- FAQ questions and answers must **exactly match** the visible FAQ section
- Include `inLanguage: "es"` for all Spanish content

---

## Complete Blog Template

This is the full HTML structure to paste into Odoo Editor. Replace all placeholder content.

```html
<!-- BLUF -->
<div class="iw-callout mb-4">
  <p class="iw-body-lg">
    <strong>[Respuesta directa a la pregunta del titulo].</strong>
    [1-2 frases adicionales de contexto que resumen el valor del articulo
    y por que el lector deberia seguir leyendo].
  </p>
</div>

<!-- Indice de contenidos -->
<nav class="mb-5 p-4 iw-bg-light" style="border-radius: var(--iw-radius-lg, 12px);">
  <h2 class="iw-heading-xs mb-3">Indice de contenidos</h2>
  <ol class="iw-body" style="margin-bottom: 0;">
    <li><a href="#seccion-1" class="iw-text-lilac">Titulo seccion 1</a></li>
    <li><a href="#seccion-2" class="iw-text-lilac">Titulo seccion 2</a></li>
    <li><a href="#seccion-3" class="iw-text-lilac">Titulo seccion 3</a></li>
    <li><a href="#preguntas-frecuentes" class="iw-text-lilac">Preguntas frecuentes</a></li>
  </ol>
</nav>

<!-- Cuerpo del articulo -->
<section id="seccion-1">
  <h2 class="iw-heading-md mb-3">Titulo de la seccion 1 con keyword</h2>
  <p class="iw-body mb-3">
    Parrafo introductorio de la seccion (250-400 palabras por seccion).
    Incluir <a href="/odoo-crm">enlaces internos relevantes</a> de forma natural.
  </p>
  <p class="iw-body mb-3">
    Segundo parrafo con mas profundidad. Argumentar cada punto con razonamiento.
    Referenciar <a href="/blog/blog-enterprise-demo-1/articulo-relacionado-123">articulos relacionados</a>
    cuando sea pertinente.
  </p>
  <p class="iw-body mb-3">
    Tercer parrafo cerrando la seccion con una conclusion parcial o transicion
    a la siguiente seccion.
  </p>
</section>

<section id="seccion-2">
  <h2 class="iw-heading-md mb-3">Titulo de la seccion 2</h2>
  <p class="iw-body mb-3">
    Contenido de la seccion con enlaces internos y keywords secundarias.
    Mencionar un <a href="/blog/referencias-enterprise-demo-16/caso-cliente-456">caso de exito</a>
    para dar credibilidad.
  </p>
  <p class="iw-body mb-3">
    Mas contenido desarrollando el tema en profundidad.
  </p>

  <!-- Video YouTube contextual -->
  <div class="ratio ratio-16x9 my-4 iw-img-rounded iw-img-shadow">
    <iframe
      src="https://www.youtube.com/embed/VIDEO_ID"
      title="Titulo del video"
      frameborder="0"
      allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
      allowfullscreen
      loading="lazy"
    ></iframe>
  </div>
  <p class="iw-caption text-center mb-4">Titulo del video</p>

  <p class="iw-body mb-3">
    Contenido continuando despues del video, conectando lo mostrado
    con el argumento del articulo.
  </p>
</section>

<section id="seccion-3">
  <h2 class="iw-heading-md mb-3">Titulo de la seccion 3</h2>
  <p class="iw-body mb-3">
    Contenido final con conclusion del tema y llamada a la accion suave.
  </p>
  <p class="iw-body mb-3">
    Parrafo de cierre enlazando a la
    <a href="/contacto-enterprise-demo">pagina de contacto</a> o a una
    <a href="/casos-de-exito">pagina de conversion</a>.
  </p>
</section>

<!-- FAQs -->
<section id="preguntas-frecuentes" class="mt-5 pt-4" style="border-top: 2px solid var(--iw-border-light, #e2e8f0);">
  <h2 class="iw-heading-md mb-4">Preguntas frecuentes</h2>

  <div class="mb-4">
    <h3 class="iw-heading-xs mb-2">Pregunta 1 en formato natural?</h3>
    <p class="iw-body">
      Respuesta directa en 2-4 frases con datos concretos.
    </p>
  </div>

  <div class="mb-4">
    <h3 class="iw-heading-xs mb-2">Pregunta 2 en formato natural?</h3>
    <p class="iw-body">
      Respuesta directa y util.
    </p>
  </div>

  <div class="mb-4">
    <h3 class="iw-heading-xs mb-2">Pregunta 3 en formato natural?</h3>
    <p class="iw-body">
      Respuesta directa con valor anadido.
    </p>
  </div>
</section>

<!-- Schema JSON-LD -->
<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@graph": [
    {
      "@type": "Article",
      "headline": "Titulo del articulo",
      "description": "Meta description 150-160 chars",
      "author": {
        "@type": "Organization",
        "name": "Enterprise Demo Business Solutions",
        "url": "https://www.enterprise-demo.es"
      },
      "publisher": {
        "@type": "Organization",
        "name": "Enterprise Demo Business Solutions",
        "url": "https://www.enterprise-demo.es",
        "logo": {
          "@type": "ImageObject",
          "url": "https://www.enterprise-demo.es/web/image/website/1/logo"
        }
      },
      "datePublished": "YYYY-MM-DD",
      "dateModified": "YYYY-MM-DD",
      "mainEntityOfPage": {
        "@type": "WebPage",
        "@id": "https://www.enterprise-demo.es/blog/blog-enterprise-demo-1/SLUG"
      },
      "articleSection": "Tecnologia",
      "inLanguage": "es"
    },
    {
      "@type": "FAQPage",
      "mainEntity": [
        {
          "@type": "Question",
          "name": "Pregunta 1?",
          "acceptedAnswer": {
            "@type": "Answer",
            "text": "Respuesta 1."
          }
        },
        {
          "@type": "Question",
          "name": "Pregunta 2?",
          "acceptedAnswer": {
            "@type": "Answer",
            "text": "Respuesta 2."
          }
        },
        {
          "@type": "Question",
          "name": "Pregunta 3?",
          "acceptedAnswer": {
            "@type": "Answer",
            "text": "Respuesta 3."
          }
        }
      ]
    }
  ]
}
</script>
```

---

## Publishing to Odoo

After generating the blog HTML, use the **`odoo-pilot`** skill to create the blog post as **unpublished** (draft) in Odoo for human revision.

### Prerequisites

- The `odoo-pilot` skill must be available
- Odoo connection credentials configured (URL, DB, API key)
- The target blog must exist in Odoo (default: `blog-enterprise-demo-1`, blog_id = 1)

### Step-by-Step Workflow

#### 1. Authenticate with Odoo

Use the odoo-pilot `auth.sh` script to establish a session:

```bash
eval $(./scripts/auth.sh)
```

This auto-detects the Odoo protocol (JSON2 for v19+, JSON-RPC for older versions) and exports session variables.

#### 2. Find the Blog ID

If unsure of the blog_id, search for it:

```bash
./scripts/search_records.sh blog.blog '[]' '["id", "name"]'
```

The main Enterprise Demo blog (`blog-enterprise-demo-1`) has `blog_id = 1`.

#### 3. Prepare Blog Data

Build the JSON payload with these fields:

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Blog post title (the H1) |
| `blog_id` | Yes | Target blog ID (typically 1) |
| `content` | Yes | Full HTML body (everything from BLUF to Schema JSON-LD) |
| `is_published` | Yes | **Always `false`** -- draft for human revision |
| `subtitle` | No | Short subtitle shown in blog listing |
| `website_meta_title` | No | SEO title (max 60 chars) |
| `website_meta_description` | No | SEO meta description (150-160 chars) |
| `tag_ids` | No | Tag IDs as `[[6, 0, [id1, id2]]]` (many2many write syntax) |

#### 4. Create the Blog Post

```bash
./scripts/create_record.sh blog.post '{
  "name": "Titulo del articulo",
  "blog_id": 1,
  "content": "<div class=\"iw-callout mb-4\">...full HTML content...</div>",
  "is_published": false,
  "subtitle": "Subtitulo breve para el listado",
  "website_meta_title": "Titulo SEO (max 60 chars)",
  "website_meta_description": "Meta description con keyword principal y propuesta de valor (150-160 chars)"
}'
```

The script returns the new record ID.

#### 5. Confirm to User

After successful creation, provide:

```
Blog creado como borrador en Odoo:
- ID: [record_id]
- URL de edicion: [ODOO_URL]/web#id=[record_id]&model=blog.post&view_type=form
- URL de preview: [ODOO_URL]/blog/blog-enterprise-demo-1/[slug]-[record_id]
- Estado: No publicado (borrador para revision)
```

### Important Notes

- **Always set `is_published: false`** -- the blog must be reviewed by a human before publishing
- **Escape HTML in JSON**: The `content` field contains HTML that must be properly escaped inside the JSON string (double quotes become `\"`)
- **Schema JSON-LD**: Odoo may strip `<script>` tags from blog content. If so, add the schema via Odoo's SEO tab or a custom module instead
- **Images**: Any images referenced in the blog should be uploaded to Odoo first and use `/web/image/` URLs
- **Tags**: To assign tags, first search for existing tags with `search_records.sh blog.tag '[]'` and use the many2many write syntax `[[6, 0, [tag_id_1, tag_id_2]]]`
