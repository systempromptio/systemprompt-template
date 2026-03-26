# HTML Presentations for Odoo

Guide for creating slide-deck presentations as self-contained HTML files. These can be used standalone or embedded in Odoo eLearning / Slide channels.

## Table of Contents
1. [Presentation Structure](#presentation-structure)
2. [Slide Types](#slide-types)
3. [Navigation System](#navigation-system)
4. [Style Discovery Workflow](#style-discovery-workflow)
5. [PPT Conversion](#ppt-conversion)

---

## Presentation Structure

Every presentation is a single HTML file with embedded CSS and JS. No external dependencies except Google Fonts (Dosis).

```html
<!DOCTYPE html>
<html lang="es">
<head>
  <meta charset="UTF-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
  <title>Presentation Title</title>
  <link href="https://fonts.googleapis.com/css2?family=Dosis:wght@400;600;700&display=swap" rel="stylesheet"/>
  <style>
    /* Design tokens from foodles-frontend.css (embed relevant subset) */
    /* Slide layout CSS */
    /* Animation CSS */
  </style>
</head>
<body>
  <div class="slides-container">
    <section class="slide" id="slide-1"><!-- Slide content --></section>
    <section class="slide" id="slide-2"><!-- Slide content --></section>
    <!-- ... more slides -->
  </div>
  <nav class="slide-nav"><!-- Navigation dots --></nav>
  <script>
    /* Keyboard, touch, scroll navigation */
    /* Intersection Observer for reveal animations */
  </script>
</body>
</html>
```

### Base slide CSS (embed in `<style>`)
```css
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

html {
  scroll-snap-type: y mandatory;
  scroll-behavior: smooth;
  overflow-y: scroll;
}

body {
  font-family: 'Dosis', Verdana, sans-serif;
  color: #1C265D;
  background: #FFFFFF;
}

.slides-container { width: 100%; }

.slide {
  width: 100%;
  min-height: 100vh;
  scroll-snap-align: start;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4rem;
  position: relative;
  overflow: hidden;
}

.slide-content {
  max-width: 1200px;
  width: 100%;
  z-index: 1;
}

/* Navigation dots */
.slide-nav {
  position: fixed;
  right: 1.5rem;
  top: 50%;
  transform: translateY(-50%);
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  z-index: 100;
}

.slide-nav a {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: rgba(28, 38, 93, 0.2);
  transition: background 0.3s, transform 0.3s;
  display: block;
}

.slide-nav a.active {
  background: #6B68FA;
  transform: scale(1.3);
}

/* Responsive */
@media (max-width: 768px) {
  .slide { padding: 2rem; }
  .slide-nav { right: 0.75rem; }
  .slide-nav a { width: 8px; height: 8px; }
}
```

## Slide Types

### Title slide
Dark gradient background, centered content, logo optional.
```html
<section class="slide" style="background: linear-gradient(160deg, #1C265D 0%, #6B68FA 50%, #8AC2DB 100%); color: #fff;">
  <div class="slide-content text-center">
    <p class="iw-overline mb-3" style="color: #E5B92B;">FOODLES BUSINESS SOLUTIONS</p>
    <h1 class="iw-heading-xl mb-4" style="color: #fff;">Titulo de la presentacion</h1>
    <p class="iw-body-lg" style="color: rgba(255,255,255,0.8);">Subtitulo o contexto</p>
  </div>
</section>
```

### Section divider
Marks a new section/topic within the presentation.
```html
<section class="slide" style="background: #6B68FA; color: #fff;">
  <div class="slide-content text-center">
    <span class="iw-overline d-block mb-3" style="color: #E5B92B;">SECCION 01</span>
    <h2 class="iw-heading-lg" style="color: #fff;">Nombre de la seccion</h2>
    <div class="iw-divider-accent iw-divider-accent-center mt-3" style="background: #E5B92B;"></div>
  </div>
</section>
```

### Content slide (text + bullets)
```html
<section class="slide">
  <div class="slide-content">
    <div class="row align-items-center">
      <div class="col-lg-6">
        <span class="iw-overline d-block mb-2">TEMA</span>
        <h2 class="iw-heading-md mb-4">Titulo del contenido</h2>
        <ul class="iw-feature-list iw-feature-list-check">
          <li>Punto principal uno con detalle</li>
          <li>Punto principal dos con detalle</li>
          <li>Punto principal tres con detalle</li>
        </ul>
      </div>
      <div class="col-lg-6">
        <img src="image.jpg" class="img-fluid iw-img-rounded iw-img-shadow" alt=""/>
      </div>
    </div>
  </div>
</section>
```

### Data / KPI slide
```html
<section class="slide iw-bg-light">
  <div class="slide-content">
    <h2 class="iw-heading-md text-center mb-5">Resultados clave</h2>
    <div class="row g-4 text-center">
      <div class="col-lg-3 col-6">
        <div class="iw-stat iw-stat-gradient">
          <span class="iw-stat-value">+85%</span>
          <span class="iw-stat-label">Eficiencia operativa</span>
        </div>
      </div>
      <!-- More stats -->
    </div>
  </div>
</section>
```

### Quote slide
```html
<section class="slide" style="background: #1C265D; color: #fff;">
  <div class="slide-content text-center">
    <div class="row justify-content-center">
      <div class="col-lg-8">
        <div class="iw-blockquote" style="color: #fff;">
          La tecnologia debe ser un acelerador, no una barrera.
        </div>
        <p class="iw-caption mt-3" style="color: #8AC2DB;">-- Nombre, Cargo</p>
      </div>
    </div>
  </div>
</section>
```

### Closing / CTA slide
```html
<section class="slide" style="background: linear-gradient(160deg, #1C265D 0%, #6B68FA 50%, #8AC2DB 100%); color: #fff;">
  <div class="slide-content text-center">
    <h2 class="iw-heading-lg mb-4" style="color: #fff;">Siguiente paso</h2>
    <p class="iw-body-lg mb-4" style="color: rgba(255,255,255,0.85);">Contacta con nuestro equipo para una demo personalizada.</p>
    <div class="d-flex gap-3 justify-content-center flex-wrap">
      <a href="mailto:info@foodles.es" class="iw-btn iw-btn-primary iw-btn-lg">Contactar</a>
      <a href="https://www.foodles.es" class="iw-btn iw-btn-ghost-light iw-btn-lg">foodles.es</a>
    </div>
  </div>
</section>
```

## Navigation System

Embed this JS at the end of the HTML for keyboard + scroll + touch navigation:

```javascript
(function() {
  const slides = document.querySelectorAll('.slide');
  const nav = document.querySelector('.slide-nav');

  // Create nav dots
  slides.forEach((slide, i) => {
    const dot = document.createElement('a');
    dot.href = '#slide-' + (i + 1);
    if (i === 0) dot.classList.add('active');
    nav.appendChild(dot);
  });

  // Intersection Observer for active slide
  const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        const index = Array.from(slides).indexOf(entry.target);
        nav.querySelectorAll('a').forEach((dot, i) => {
          dot.classList.toggle('active', i === index);
        });
        // Trigger reveal animations
        entry.target.querySelectorAll('.iw-reveal, .iw-reveal-left, .iw-reveal-right, .iw-reveal-scale').forEach(el => {
          el.classList.add('iw-visible');
        });
      }
    });
  }, { threshold: 0.5 });

  slides.forEach(slide => observer.observe(slide));

  // Keyboard navigation
  document.addEventListener('keydown', (e) => {
    const current = Math.round(window.scrollY / window.innerHeight);
    if (e.key === 'ArrowDown' || e.key === 'ArrowRight' || e.key === ' ') {
      e.preventDefault();
      const next = Math.min(current + 1, slides.length - 1);
      slides[next].scrollIntoView({ behavior: 'smooth' });
    }
    if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') {
      e.preventDefault();
      const prev = Math.max(current - 1, 0);
      slides[prev].scrollIntoView({ behavior: 'smooth' });
    }
  });
})();
```

## Style Discovery Workflow

When the user wants a presentation, generate 3 style previews as mini HTML files before building the full deck. Map mood to style:

| Mood | Styles to offer |
|------|----------------|
| Impressed/Confident | "Corporate Elegant" (Blue Space bg, subtle gradients), "Dark Executive" (full dark, yellow accents), "Clean Minimal" (white bg, lilac accents) |
| Excited/Energized | "Bold Gradients" (hero gradient bg, large type), "Kinetic Motion" (animated reveals, scale effects), "Vibrant Contrast" (yellow + lilac combos) |
| Calm/Focused | "Swiss Minimal" (lots of whitespace, thin type), "Soft Muted" (light sky tones, subtle), "Paper Clean" (off-white, minimal accents) |
| Inspired/Moved | "Cinematic Dark" (Blue Space, large images, fade reveals), "Warm Editorial" (yellow accents, serif quotes), "Atmospheric" (gradient overlays, depth) |

All styles use exclusively Foodles brand colors and Dosis font.

## PPT Conversion

When converting a .pptx file:

1. Extract content using `python-pptx` (extract text, images, notes per slide)
2. Map each PPT slide to the closest slide type above
3. Apply the user's chosen style
4. Preserve all images (save to assets folder, reference with relative paths)
5. Include speaker notes as HTML comments: `<!-- NOTES: ... -->`
