# Foodles Frontend Components Catalog

Complete catalog of HTML components using Bootstrap 5 + `iw-*` design system classes. Copy-paste ready for Odoo Editor.

## Table of Contents
1. [Hero Sections](#hero-sections)
2. [Feature Grids](#feature-grids)
3. [Cards](#cards)
4. [Statistics / KPIs](#statistics--kpis)
5. [Testimonials](#testimonials)
6. [Pricing](#pricing)
7. [Timeline / Process](#timeline--process)
8. [Call to Action](#call-to-action)
9. [Content Blocks](#content-blocks)
10. [Tables](#tables)
11. [Blog Components](#blog-components)
12. [Email Components](#email-components)

---

## Hero Sections

### Hero with gradient background
```html
<section class="iw-hero">
  <div class="container">
    <div class="row align-items-center">
      <div class="col-lg-7">
        <span class="iw-overline mb-3 d-block">SOLUCIONES FOODLES</span>
        <h1 class="iw-heading-xl mb-4">Transforma tu negocio con Odoo</h1>
        <p class="iw-body-lg" style="color: rgba(255,255,255,0.85);">
          Te acompanamos en cada paso de la digitalizacion de tu empresa
          con soluciones a medida sobre la plataforma lider en ERP.
        </p>
        <div class="mt-4 d-flex gap-3 flex-wrap">
          <a href="#" class="iw-btn iw-btn-primary iw-btn-lg">Solicitar demo</a>
          <a href="#" class="iw-btn iw-btn-ghost-light iw-btn-lg">Saber mas</a>
        </div>
      </div>
      <div class="col-lg-5 d-none d-lg-block text-center">
        <img src="/web/image/..." class="img-fluid iw-img-rounded" alt="Dashboard Odoo"/>
      </div>
    </div>
  </div>
</section>
```

### Hero minimal (centered text)
```html
<section class="iw-hero text-center">
  <div class="container">
    <div class="row justify-content-center">
      <div class="col-lg-8">
        <span class="iw-badge iw-badge-lilac mb-3" style="background:rgba(255,255,255,0.15);color:#fff;">NUEVO</span>
        <h1 class="iw-heading-xl mb-4">Titulo del proyecto</h1>
        <p class="iw-body-lg mb-4" style="color: rgba(255,255,255,0.85);">
          Descripcion breve del contenido o propuesta de valor principal.
        </p>
        <a href="#" class="iw-btn iw-btn-primary iw-btn-lg">Accion principal</a>
      </div>
    </div>
  </div>
</section>
```

### Hero with wave separator
```html
<section class="iw-hero iw-section-wave">
  <div class="container">
    <!-- Same content as above -->
  </div>
</section>
```

---

## Feature Grids

### 3-column icon features
```html
<section class="iw-section iw-bg-light">
  <div class="container">
    <div class="text-center mb-5">
      <span class="iw-overline d-block mb-2">FUNCIONALIDADES</span>
      <h2 class="iw-heading-md">Por que elegir Foodles</h2>
      <div class="iw-divider-accent iw-divider-accent-center mt-3"></div>
    </div>
    <div class="row g-4 iw-stagger">
      <div class="col-lg-4 iw-reveal">
        <div class="iw-card iw-card-icon">
          <div class="iw-icon-wrapper">
            <i class="fa fa-cogs"></i>
          </div>
          <h3 class="iw-heading-xs mb-2">Configuracion a medida</h3>
          <p class="iw-body">Adaptamos cada modulo a los procesos especificos de tu empresa, sin desarrollo innecesario.</p>
        </div>
      </div>
      <div class="col-lg-4 iw-reveal">
        <div class="iw-card iw-card-icon">
          <div class="iw-icon-wrapper">
            <i class="fa fa-chart-line"></i>
          </div>
          <h3 class="iw-heading-xs mb-2">Resultados medibles</h3>
          <p class="iw-body">Implementamos KPIs y dashboards para que veas el retorno de tu inversion desde el primer dia.</p>
        </div>
      </div>
      <div class="col-lg-4 iw-reveal">
        <div class="iw-card iw-card-icon">
          <div class="iw-icon-wrapper">
            <i class="fa fa-users"></i>
          </div>
          <h3 class="iw-heading-xs mb-2">Equipo especializado</h3>
          <p class="iw-body">Consultores certificados con experiencia en tu sector industrial.</p>
        </div>
      </div>
    </div>
  </div>
</section>
```

### 2-column text + image feature
```html
<section class="iw-section">
  <div class="container">
    <div class="row align-items-center g-5">
      <div class="col-lg-6">
        <span class="iw-overline d-block mb-2">MODULO CORE</span>
        <h2 class="iw-heading-md mb-3">Gestion de inventario inteligente</h2>
        <div class="iw-divider-accent mb-4"></div>
        <p class="iw-body mb-4">
          Nuestro modulo de inventario te permite controlar el stock en tiempo real,
          optimizar las rutas de almacen y automatizar los procesos de reposicion.
        </p>
        <ul class="iw-feature-list iw-feature-list-check">
          <li>Control de stock en tiempo real multi-almacen</li>
          <li>Rutas de almacen configurables por producto</li>
          <li>Alertas automaticas de reposicion</li>
          <li>Trazabilidad completa por lote y numero de serie</li>
        </ul>
        <a href="#" class="iw-btn iw-btn-secondary mt-3">Ver modulo</a>
      </div>
      <div class="col-lg-6">
        <div class="iw-img-accent">
          <img src="/web/image/..." class="img-fluid iw-img-rounded iw-img-shadow" alt="Inventario Odoo"/>
        </div>
      </div>
    </div>
  </div>
</section>
```

---

## Cards

### Card grid (services / modules)
```html
<section class="iw-section">
  <div class="container">
    <div class="row g-4">
      <div class="col-lg-4 col-md-6">
        <div class="iw-card iw-card-accent h-100">
          <span class="iw-badge iw-badge-lilac mb-3">INVENTARIO</span>
          <h3 class="iw-heading-xs mb-2">Foodles Inventory</h3>
          <p class="iw-body mb-3">Control completo del stock con trazabilidad avanzada y rutas configurables.</p>
          <a href="#" class="iw-btn iw-btn-ghost iw-btn-sm mt-auto">Saber mas</a>
        </div>
      </div>
      <div class="col-lg-4 col-md-6">
        <div class="iw-card iw-card-accent h-100">
          <span class="iw-badge iw-badge-yellow mb-3">FABRICACION</span>
          <h3 class="iw-heading-xs mb-2">Foodles MRP</h3>
          <p class="iw-body mb-3">Planificacion y control de produccion con MRP avanzado y OEE integrado.</p>
          <a href="#" class="iw-btn iw-btn-ghost iw-btn-sm mt-auto">Saber mas</a>
        </div>
      </div>
      <div class="col-lg-4 col-md-6">
        <div class="iw-card iw-card-accent h-100">
          <span class="iw-badge iw-badge-sky mb-3">CALIDAD</span>
          <h3 class="iw-heading-xs mb-2">Foodles Quality</h3>
          <p class="iw-body mb-3">Gestion de calidad integrada con produccion, compras e inventario.</p>
          <a href="#" class="iw-btn iw-btn-ghost iw-btn-sm mt-auto">Saber mas</a>
        </div>
      </div>
    </div>
  </div>
</section>
```

### Glass cards on dark background
```html
<section class="iw-section iw-bg-gradient-space" style="color: #fff;">
  <div class="container">
    <div class="row g-4">
      <div class="col-lg-4">
        <div class="iw-card-glass h-100">
          <h3 class="iw-heading-xs mb-2" style="color:#fff;">Titulo</h3>
          <p style="color: rgba(255,255,255,0.8);">Contenido de la tarjeta con fondo translucido.</p>
        </div>
      </div>
      <!-- Repeat for more cards -->
    </div>
  </div>
</section>
```

---

## Statistics / KPIs

### Stats row
```html
<section class="iw-section iw-bg-light">
  <div class="container">
    <div class="row g-4 text-center">
      <div class="col-6 col-lg-3">
        <div class="iw-stat iw-stat-gradient">
          <span class="iw-stat-value">225+</span>
          <span class="iw-stat-label">Videos formativos</span>
        </div>
      </div>
      <div class="col-6 col-lg-3">
        <div class="iw-stat iw-stat-gradient">
          <span class="iw-stat-value">42</span>
          <span class="iw-stat-label">Modulos CORE</span>
        </div>
      </div>
      <div class="col-6 col-lg-3">
        <div class="iw-stat iw-stat-gradient">
          <span class="iw-stat-value">92</span>
          <span class="iw-stat-label">Casos de exito</span>
        </div>
      </div>
      <div class="col-6 col-lg-3">
        <div class="iw-stat iw-stat-gradient">
          <span class="iw-stat-value">15+</span>
          <span class="iw-stat-label">Anos de experiencia</span>
        </div>
      </div>
    </div>
  </div>
</section>
```

---

## Testimonials

### Single testimonial
```html
<section class="iw-section">
  <div class="container">
    <div class="row justify-content-center">
      <div class="col-lg-8">
        <div class="iw-testimonial">
          <p class="iw-testimonial-text">
            La implementacion de Odoo con Foodles transformo completamente nuestra
            gestion de almacen. En tres meses redujimos los errores de picking un 85%.
          </p>
          <p class="iw-testimonial-author">Maria Garcia</p>
          <p class="iw-testimonial-role">Directora de Operaciones, Empresa XYZ</p>
        </div>
      </div>
    </div>
  </div>
</section>
```

### Blockquote (large quote)
```html
<div class="iw-blockquote">
  No vendemos software, te acompanamos en la transformacion digital de tu negocio.
</div>
```

---

## Pricing

### Pricing cards (3-tier)
```html
<section class="iw-section">
  <div class="container">
    <div class="text-center mb-5">
      <h2 class="iw-heading-md">Propuestas de valor</h2>
      <div class="iw-divider-accent iw-divider-accent-center mt-3"></div>
    </div>
    <div class="row g-4 align-items-center justify-content-center">
      <div class="col-lg-4">
        <div class="iw-pricing-card">
          <h3 class="iw-heading-xs mb-3">Esencial</h3>
          <div class="mb-3">
            <span class="iw-pricing-amount">490</span>
            <span class="iw-pricing-period">/mes</span>
          </div>
          <ul class="iw-feature-list iw-feature-list-check text-start mb-4">
            <li>Hasta 10 usuarios</li>
            <li>3 modulos CORE incluidos</li>
            <li>Acompanamiento tecnico basico</li>
          </ul>
          <a href="#" class="iw-btn iw-btn-ghost w-100 justify-content-center">Seleccionar</a>
        </div>
      </div>
      <div class="col-lg-4">
        <div class="iw-pricing-card iw-pricing-featured">
          <span class="iw-pricing-label">RECOMENDADO</span>
          <h3 class="iw-heading-xs mb-3">Profesional</h3>
          <div class="mb-3">
            <span class="iw-pricing-amount">990</span>
            <span class="iw-pricing-period">/mes</span>
          </div>
          <ul class="iw-feature-list iw-feature-list-check text-start mb-4">
            <li>Hasta 50 usuarios</li>
            <li>Todos los modulos CORE</li>
            <li>Acompanamiento tecnico prioritario</li>
            <li>Formacion incluida</li>
          </ul>
          <a href="#" class="iw-btn iw-btn-primary w-100 justify-content-center">Seleccionar</a>
        </div>
      </div>
      <div class="col-lg-4">
        <div class="iw-pricing-card">
          <h3 class="iw-heading-xs mb-3">Enterprise</h3>
          <div class="mb-3">
            <span class="iw-pricing-amount">A medida</span>
          </div>
          <ul class="iw-feature-list iw-feature-list-check text-start mb-4">
            <li>Usuarios ilimitados</li>
            <li>Todos los modulos + personalizacion</li>
            <li>Consultor dedicado</li>
            <li>SLA garantizado</li>
          </ul>
          <a href="#" class="iw-btn iw-btn-ghost w-100 justify-content-center">Contactar</a>
        </div>
      </div>
    </div>
  </div>
</section>
```

---

## Timeline / Process

### Vertical timeline
```html
<section class="iw-section">
  <div class="container">
    <div class="row justify-content-center">
      <div class="col-lg-8">
        <h2 class="iw-heading-md text-center mb-5">Proceso de implementacion</h2>
        <div class="iw-timeline">
          <div class="iw-timeline-item">
            <h4 class="iw-heading-xs">Analisis de necesidades</h4>
            <p class="iw-body">Estudiamos tus procesos actuales y definimos los objetivos de la implementacion.</p>
          </div>
          <div class="iw-timeline-item">
            <h4 class="iw-heading-xs">Configuracion y desarrollo</h4>
            <p class="iw-body">Configuramos Odoo y desarrollamos las personalizaciones necesarias para tu negocio.</p>
          </div>
          <div class="iw-timeline-item">
            <h4 class="iw-heading-xs">Formacion y puesta en marcha</h4>
            <p class="iw-body">Formamos a tu equipo y te acompanamos durante las primeras semanas de uso.</p>
          </div>
          <div class="iw-timeline-item">
            <h4 class="iw-heading-xs">Evolucion continua</h4>
            <p class="iw-body">Seguimos a tu lado para optimizar y evolucionar la solucion con tu negocio.</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</section>
```

### Horizontal steps
```html
<section class="iw-section iw-bg-light">
  <div class="container">
    <div class="iw-steps">
      <div class="iw-step">
        <h4 class="iw-heading-xs">Analisis</h4>
        <p class="iw-caption">Definimos objetivos</p>
      </div>
      <div class="iw-step">
        <h4 class="iw-heading-xs">Diseno</h4>
        <p class="iw-caption">Planificamos la solucion</p>
      </div>
      <div class="iw-step">
        <h4 class="iw-heading-xs">Implementacion</h4>
        <p class="iw-caption">Configuramos y desarrollamos</p>
      </div>
      <div class="iw-step">
        <h4 class="iw-heading-xs">Evolucion</h4>
        <p class="iw-caption">Optimizamos continuamente</p>
      </div>
    </div>
  </div>
</section>
```

---

## Call to Action

### CTA section (gradient background)
```html
<section class="iw-section iw-bg-gradient-space" style="color: #fff;">
  <div class="container text-center">
    <div class="row justify-content-center">
      <div class="col-lg-8">
        <h2 class="iw-heading-lg mb-3" style="color: #fff;">Preparado para transformar tu negocio?</h2>
        <p class="iw-body-lg mb-4" style="color: rgba(255,255,255,0.85);">
          Solicita una demo personalizada y descubre como Odoo puede impulsar tu empresa.
        </p>
        <div class="d-flex gap-3 justify-content-center flex-wrap">
          <a href="#" class="iw-btn iw-btn-primary iw-btn-lg">Solicitar demo</a>
          <a href="#" class="iw-btn iw-btn-ghost-light iw-btn-lg">Hablar con un consultor</a>
        </div>
      </div>
    </div>
  </div>
</section>
```

### CTA inline (within content)
```html
<div class="iw-callout">
  <p class="iw-callout-title">Quieres saber mas?</p>
  <p class="iw-body mb-3">Nuestro equipo de consultores esta disponible para resolver cualquier duda sobre la implementacion.</p>
  <a href="#" class="iw-btn iw-btn-secondary iw-btn-sm">Contactar</a>
</div>
```

---

## Content Blocks

### Blog post header
```html
<section class="iw-section iw-bg-gradient-hero" style="color: #fff;">
  <div class="container">
    <div class="row justify-content-center">
      <div class="col-lg-8 text-center">
        <span class="iw-badge mb-3" style="background:rgba(255,255,255,0.15);color:#fff;">BLOG</span>
        <h1 class="iw-heading-lg mb-3" style="color:#fff;">Titulo del articulo</h1>
        <p class="iw-caption" style="color:rgba(255,255,255,0.7);">Publicado el 28 enero 2026 | 5 min lectura</p>
      </div>
    </div>
  </div>
</section>
```

### Callout box
```html
<div class="iw-callout">
  <p class="iw-callout-title">Punto clave</p>
  <p class="iw-body">Informacion destacada que el lector debe retener.</p>
</div>
```

### Warning callout
```html
<div class="iw-callout iw-callout-warning">
  <p class="iw-callout-title">Importante</p>
  <p class="iw-body">Nota de advertencia o informacion critica.</p>
</div>
```

---

## Tables

### Styled data table
```html
<div class="table-responsive">
  <table class="iw-table iw-table-striped">
    <thead>
      <tr>
        <th>Modulo</th>
        <th>Funcionalidad</th>
        <th>Inversion</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><strong>Foodles Inventory</strong></td>
        <td>Control de stock multi-almacen</td>
        <td>Desde 150/mes</td>
      </tr>
      <tr>
        <td><strong>Foodles MRP</strong></td>
        <td>Planificacion de produccion</td>
        <td>Desde 200/mes</td>
      </tr>
      <tr>
        <td><strong>Foodles Quality</strong></td>
        <td>Gestion de calidad integrada</td>
        <td>Desde 120/mes</td>
      </tr>
    </tbody>
  </table>
</div>
```

---

## Blog Components

### BLUF (Bottom Line Up Front)
```html
<div class="iw-callout mb-4">
  <p class="iw-body-lg">
    <strong>Respuesta directa a la pregunta del titulo.</strong>
    Contexto adicional que resume el valor del articulo y motiva
    a seguir leyendo para profundizar en el tema.
  </p>
</div>
```

### Table of contents
```html
<nav class="mb-5 p-4 iw-bg-light" style="border-radius: var(--iw-radius-lg, 12px);">
  <h2 class="iw-heading-xs mb-3">Indice de contenidos</h2>
  <ol class="iw-body" style="margin-bottom: 0;">
    <li><a href="#seccion-1" class="iw-text-lilac">Titulo seccion 1</a></li>
    <li><a href="#seccion-2" class="iw-text-lilac">Titulo seccion 2</a></li>
    <li><a href="#seccion-3" class="iw-text-lilac">Titulo seccion 3</a></li>
    <li><a href="#preguntas-frecuentes" class="iw-text-lilac">Preguntas frecuentes</a></li>
  </ol>
</nav>
```

### YouTube video embed (contextual)
```html
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
```

### FAQ section
```html
<section id="preguntas-frecuentes" class="mt-5 pt-4" style="border-top: 2px solid var(--iw-border-light, #e2e8f0);">
  <h2 class="iw-heading-md mb-4">Preguntas frecuentes</h2>
  <div class="mb-4">
    <h3 class="iw-heading-xs mb-2">Pregunta en formato natural?</h3>
    <p class="iw-body">Respuesta directa en 2-4 frases con datos concretos.</p>
  </div>
  <div class="mb-4">
    <h3 class="iw-heading-xs mb-2">Segunda pregunta?</h3>
    <p class="iw-body">Respuesta directa y util.</p>
  </div>
  <div class="mb-4">
    <h3 class="iw-heading-xs mb-2">Tercera pregunta?</h3>
    <p class="iw-body">Respuesta con valor anadido.</p>
  </div>
</section>
```

### Highlighted key point (inline)
```html
<div class="iw-callout-warning my-4">
  <p class="iw-callout-title">Punto clave</p>
  <p class="iw-body">Informacion importante que el lector no debe pasar por alto.</p>
</div>
```

### Bento grid (asymmetric feature layout)
```html
<div class="iw-bento my-4">
  <div class="iw-bento-item iw-bento-lg iw-bento-dark">
    <p class="iw-overline mb-2" style="color: #E5B92B;">PUNTO PRINCIPAL</p>
    <h3 class="iw-heading-xs">Concepto central del articulo</h3>
    <p class="iw-body mt-2" style="color: rgba(255,255,255,0.8);">Explicacion breve.</p>
  </div>
  <div class="iw-bento-item iw-bento-sm iw-bento-accent">
    <p class="iw-overline mb-2">DATO CLAVE</p>
    <p class="iw-key-figure-value">+85%</p>
    <p class="iw-caption">Mejora en eficiencia</p>
  </div>
  <div class="iw-bento-item iw-bento-sm">
    <h3 class="iw-heading-xs mb-2">Punto secundario</h3>
    <p class="iw-caption">Detalle complementario.</p>
  </div>
</div>
```

### Comparison table (vs)
```html
<div class="iw-comparison my-4">
  <div class="iw-comparison-col iw-comparison-negative">
    <div class="iw-comparison-header">Sin Odoo</div>
    <p class="iw-comparison-item">Procesos manuales</p>
    <p class="iw-comparison-item">Datos duplicados</p>
  </div>
  <div class="iw-comparison-col iw-comparison-positive">
    <div class="iw-comparison-header">Con Odoo</div>
    <p class="iw-comparison-item">Flujos automatizados</p>
    <p class="iw-comparison-item">Base de datos unica</p>
  </div>
</div>
```

### Pros / Cons list
```html
<div class="iw-pros-cons my-4">
  <div>
    <p class="iw-pros-title">Ventajas</p>
    <ul class="iw-pros-list">
      <li>Integracion nativa entre modulos</li>
      <li>Codigo abierto y personalizable</li>
    </ul>
  </div>
  <div>
    <p class="iw-cons-title">Consideraciones</p>
    <ul class="iw-cons-list">
      <li>Curva de aprendizaje inicial</li>
      <li>Requiere consultoria avanzada</li>
    </ul>
  </div>
</div>
```

### Key figures (boxed row)
```html
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
      <span class="iw-key-figure-label">Errores</span>
    </div>
  </div>
  <div class="col-md-4">
    <div class="iw-key-figure-box">
      <span class="iw-key-figure-value">4 sem.</span>
      <span class="iw-key-figure-label">Implantacion</span>
    </div>
  </div>
</div>
```

### Summary box (TL;DR)
```html
<div class="iw-summary-box my-4">
  <p class="iw-summary-box-title">Puntos clave</p>
  <ul>
    <li>Primer punto resumido</li>
    <li>Segundo punto con dato relevante</li>
    <li>Tercer punto con accion recomendada</li>
  </ul>
</div>
```

---

## Email Components

### Email template structure
```html
<!-- Email-safe: all inline styles, table layout -->
<div class="iw-email-container" style="max-width:600px;margin:0 auto;font-family:'Dosis',Verdana,Arial,sans-serif;color:#1C265D;">
  <div class="iw-email-header" style="background-color:#1C265D;padding:24px 32px;text-align:center;">
    <img src="https://www.foodles.es/logo-white.png" alt="Foodles" width="120" style="height:auto;"/>
  </div>
  <div class="iw-email-body" style="padding:32px;background-color:#FFFFFF;">
    <h1 style="font-family:'Dosis',Verdana,sans-serif;font-weight:700;font-size:24px;color:#1C265D;margin:0 0 16px;">
      Titulo del email
    </h1>
    <p style="font-family:'Dosis',Verdana,sans-serif;font-size:16px;line-height:1.6;color:#1C265D;margin:0 0 16px;">
      Contenido del email con estilo inline para maxima compatibilidad.
    </p>
    <a href="#" class="iw-email-btn" style="display:inline-block;background-color:#E5B92B;color:#1C265D;font-family:'Dosis',Verdana,sans-serif;font-weight:700;font-size:14px;text-transform:uppercase;letter-spacing:0.03em;padding:12px 28px;border-radius:100px;text-decoration:none;">
      Accion principal
    </a>
  </div>
  <div class="iw-email-footer" style="background-color:#f7f8fc;padding:24px 32px;text-align:center;font-size:12px;color:#4a5568;font-family:'Dosis',Verdana,sans-serif;">
    <p style="margin:0;">Foodles Business Solutions S.L. | foodles.es</p>
  </div>
</div>
```

### Email preheader (hidden preview text)
```html
<!-- Place immediately after <body> tag. Shows in inbox preview next to subject line. -->
<div style="display:none;font-size:1px;color:#f7f8fc;line-height:1px;max-height:0;max-width:0;opacity:0;overflow:hidden;mso-hide:all;">
  Texto de preheader que aparece en la bandeja de entrada (85-100 caracteres)
  &zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;
  &zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;
</div>
```

### Email two-column layout (table-based, Outlook-safe)
```html
<!-- Two-column table layout for Outlook compatibility -->
<tr>
  <td style="padding:24px 32px;">
    <table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0">
      <tr>
        <td width="48%" valign="top" style="padding-right:12px;">
          <h3 style="font-family:'Dosis',Verdana,sans-serif;font-size:18px;font-weight:700;color:#1C265D;margin:0 0 8px 0;">
            Titulo columna izquierda
          </h3>
          <p style="font-family:'Dosis',Verdana,sans-serif;font-size:14px;line-height:1.6;color:#334155;margin:0;">
            Texto descriptivo de la columna izquierda con informacion relevante.
          </p>
        </td>
        <td width="4%">&nbsp;</td>
        <td width="48%" valign="top" style="padding-left:12px;">
          <h3 style="font-family:'Dosis',Verdana,sans-serif;font-size:18px;font-weight:700;color:#1C265D;margin:0 0 8px 0;">
            Titulo columna derecha
          </h3>
          <p style="font-family:'Dosis',Verdana,sans-serif;font-size:14px;line-height:1.6;color:#334155;margin:0;">
            Texto descriptivo de la columna derecha. Puede ser texto o imagen.
          </p>
        </td>
      </tr>
    </table>
  </td>
</tr>
```

### Email bulletproof button (Outlook VML + modern CSS)
```html
<!-- Bulletproof CTA button: VML for Outlook, CSS for modern clients -->
<tr>
  <td align="center" style="padding:24px 32px;">
    <div style="text-align:center;">
      <!--[if mso]>
      <v:roundrect xmlns:v="urn:schemas-microsoft-com:vml"
        xmlns:w="urn:schemas-microsoft-com:office:word"
        href="https://www.foodles.es/contacto-foodles"
        style="height:44px;v-text-anchor:middle;width:220px;"
        arcsize="50%"
        strokecolor="#E5B92B"
        fillcolor="#E5B92B">
        <w:anchorlock/>
        <center style="color:#1C265D;font-family:'Dosis',Verdana,sans-serif;font-size:14px;font-weight:bold;text-transform:uppercase;">
          Solicitar demo
        </center>
      </v:roundrect>
      <![endif]-->
      <!--[if !mso]><!-->
      <a href="https://www.foodles.es/contacto-foodles"
        style="display:inline-block;background-color:#E5B92B;color:#1C265D;font-family:'Dosis',Verdana,sans-serif;font-size:14px;font-weight:700;text-transform:uppercase;text-decoration:none;padding:12px 28px;border-radius:100px;line-height:1;">
        Solicitar demo
      </a>
      <!--<![endif]-->
    </div>
  </td>
</tr>
```
