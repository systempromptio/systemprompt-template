---
name: "Email Campaign Builder"
description: "Email marketing campaign generator with landing pages, subscription forms, and transactional emails"
---

# Email Campaign Builder

## Descripción

Este skill genera activos de marketing por email:
- **Landing pages** para captación de leads
- **Formularios de suscripción** embebibles
- **Templates de email** transaccionales y promocionales
- **Páginas de confirmación** y thank you pages

## Sistema de Diseño

### Colores (variables CSS)

```css
--enterprise-demo-warm-yellow: #E5B92B;   /* CTAs, acentos, highlights */
--enterprise-demo-blue-lilac: #0071ce;    /* Títulos, elementos digitales */
--enterprise-demo-blue-space: #1A0030;    /* Fondos oscuros, texto principal */
--enterprise-demo-light-sky: #8AC2DB;     /* Fondos claros, soporte */
```

### Tipografía

- **Font**: Dosis (Google Fonts)
- **Headings**: Bold (700), uppercase
- **Body**: Regular (400)
- **CTAs**: Bold (700), uppercase

### Estilo Visual

- **Glassmorphism**: Fondos semitransparentes con blur
- **Bento Box**: Layout en grid con cards modulares
- **Animaciones**: Transiciones sutiles (0.3s ease)
- **Toggle tema**: Claro/oscuro controlado por usuario

## Templates Disponibles

### 1. Landing Page (`landing-page.html`)

Página completa de captación con:
- Hero section con CTA principal
- Grid de beneficios (bento box)
- Formulario de suscripción
- Testimonios
- Footer con enlaces

### 2. Formulario Embed (`subscribe-form.html`)

Widget embebible para integrar en cualquier sitio:
- Validación de email
- Estilos encapsulados
- Tema adaptable

### 3. Email Template (`email-template.html`)

Template de email compatible con clientes de correo:
- Diseño responsive
- Colores inline (compatibilidad)
- Estructura modular

## Uso

### Generar Landing Page

```bash
./scripts/generate-landing.sh \
  --title "Tu Título" \
  --subtitle "Subtítulo descriptivo" \
  --cta "Texto del botón" \
  --output output.html
```

### Personalizar Tema

Los templates incluyen toggle de tema. El usuario puede cambiar entre claro/oscuro.
El tema se persiste en localStorage.

## Estructura de Archivos

```
email-campaign-builder/
├── SKILL.md
├── templates/
│   ├── landing-page.html
│   ├── subscribe-form.html
│   └── email-template.html
├── assets/
│   ├── css/
│   │   └── campaign-styles.css
│   ├── js/
│   │   └── campaign-scripts.js
│   └── images/
└── scripts/
    └── generate-landing.sh
```

## Directrices de Contenido

### Tono de Voz

- Tuteo profesional ("tu", nunca "usted")
- Sin emojis (NUNCA)
- Vocabulario de valor (inversión, no gasto)
- Explicaciones razonadas, no tips rápidos

### Prohibiciones

1. ZERO emojis en cualquier output
2. No usar "gasto", "coste", "precio" - usar "inversión", "valoración"
3. No usar "problema" - usar "reto" o "oportunidad"
4. No CTAs genéricos ("click aquí") - usar acción específica

## Integración con Enterprise Demo Brand

Este skill extiende `enterprise-demo-brand`. Antes de modificar:
1. Lee `/skills/enterprise-demo-brand/SKILL.md`
2. Consulta `/skills/enterprise-demo-brand/references/html-documents.md`
3. Usa los assets de `/skills/enterprise-demo-brand/assets/`

## Ejemplos

### CTA Efectivo

```html
<a href="#" class="cta-button">Solicita tu consulta gratuita</a>
```

### Card Glassmorphism

```html
<div class="glass-card">
  <h3>Título de beneficio</h3>
  <p>Descripción del valor que aporta este servicio...</p>
</div>
```

### Bento Grid

```html
<div class="bento-grid">
  <div class="bento-item bento-large">Contenido principal</div>
  <div class="bento-item">Item 2</div>
  <div class="bento-item">Item 3</div>
</div>
```
