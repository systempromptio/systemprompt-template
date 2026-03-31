# Email Creation Guide

Reference for creating email marketing content (newsletters, promotions, event invites) compatible with Odoo Mass Mailing and major email clients. All emails use inline styles, table-based layout, and follow enterprise-demo-brand guidelines.

## Prerequisites

Before creating an email, load these skills:
- **`enterprise-demo-brand`** -- Read `references/verbal-identity.md` for tone, vocabulary, and writing style. Read `references/visual-identity.md` for color and typography rules. Key constraints: professional "tu", zero emojis, value vocabulary (inversion not gasto, reto not problema).
- **`content-collection`** -- Load `data/cases.json` for case study social proof. Load `data/videos.json` for video thumbnail links.

## Table of Contents
1. [Email Types](#email-types)
2. [Odoo Mass Mailing Compatibility](#odoo-mass-mailing-compatibility)
3. [Layout Patterns](#layout-patterns)
4. [Subject Line & Preheader](#subject-line--preheader)
5. [Table-Based Layout (Outlook)](#table-based-layout-outlook)
6. [Inline Style Requirements](#inline-style-requirements)
7. [Image Handling](#image-handling)
8. [CTA Button Patterns](#cta-button-patterns)
9. [Footer Requirements](#footer-requirements)
10. [Brand Compliance](#brand-compliance)
11. [Content-Collection Integration](#content-collection-integration)
12. [Complete Email Templates](#complete-email-templates)
13. [Publishing to Odoo](#publishing-to-odoo)

---

## Email Types

| Type | Purpose | Structure | CTA Style | Odoo Model |
|------|---------|-----------|-----------|------------|
| **Newsletter** | Monthly digest, multiple topics | Header → Hero → 2-3 content blocks → CTA → Footer | Multiple links + 1 primary CTA | `mailing.mailing` |
| **Promotional** | Single offer, urgency | Header → Hero image → Value prop → CTA → Footer | One strong primary CTA | `mailing.mailing` |
| **Event invite** | Webinar/event registration | Header → Date/time hero → Agenda → CTA → Footer | Registration button | `mailing.mailing` |
| **Transactional** | Confirmations, receipts | Header → Content → Action link → Footer | Informational link | `mail.template` |

### Which model to use

- **Marketing emails** (newsletter, promotional, event): Use `mailing.mailing` via Odoo Mass Mailing. Created as draft, sent to contact lists.
- **Transactional emails** (order confirmation, password reset): Use `mail.template`. These are triggered by actions, not sent in bulk. This guide focuses on marketing emails.

---

## Odoo Mass Mailing Compatibility

### What Odoo does to your HTML

When you create a mailing in Odoo, the mass mailing editor:

1. **Wraps content** in its own responsive table structure
2. **Re-inlines CSS** -- any `<style>` block gets converted to inline styles
3. **Adds tracking pixel** at the bottom for open tracking
4. **Adds unsubscribe link** automatically in the footer
5. **Converts relative URLs** to absolute URLs using the Odoo base URL
6. **Supports dynamic placeholders** -- `${object.name}`, `${object.email}`, etc.
7. **Preserves inline styles** that you write directly on elements

### What to avoid

| Pattern | Why | Alternative |
|---------|-----|-------------|
| `<style>` blocks | Stripped by Gmail, re-inlined by Odoo | Inline styles on every element |
| CSS media queries | Stripped by most email clients | Single-column responsive design |
| Background images | Not supported in Outlook | Solid background colors |
| CSS gradients | Not supported in Outlook/Gmail | Solid colors only |
| `border-radius > 8px` | Ignored in Outlook | Use 4-8px or none |
| Flexbox / CSS Grid | Not supported in Outlook | Table-based layout |
| `<div>` for layout | Unreliable in Outlook | `<table>` with `<tr>` / `<td>` |
| Web fonts via `@import` | Blocked by most clients | System-safe fonts with fallback |
| `position: absolute/relative` | Ignored in most clients | Table cell alignment |
| JavaScript | Stripped by all email clients | No interactivity in email |

### Dynamic placeholders

Odoo supports Jinja-style placeholders in email body:

```html
<p style="...">Hola ${object.name},</p>
<p style="...">Gracias por tu interes en ${object.company_id.name}.</p>
```

Common fields for `mailing.contact`:
- `${object.name}` -- Contact name
- `${object.email}` -- Contact email
- `${object.company_name}` -- Company name
- `${object.title.name}` -- Title (Sr., Sra.)

---

## Layout Patterns

### Single Column (Newsletter, Promotional)

The most reliable layout. Works in all email clients.

```
+----------------------------------+
|           HEADER (logo)          |
+----------------------------------+
|         HERO IMAGE / TEXT        |
+----------------------------------+
|                                  |
|          BODY CONTENT            |
|     (paragraphs, images)         |
|                                  |
+----------------------------------+
|          [ CTA BUTTON ]          |
+----------------------------------+
|            FOOTER                |
|   (company info, unsubscribe)    |
+----------------------------------+
```

### Two Column (Feature Highlight)

Use nested tables. Only for desktop -- collapses to single column on mobile.

```
+----------------------------------+
|           HEADER (logo)          |
+----------------------------------+
|         HERO IMAGE / TEXT        |
+----------------------------------+
|                |                 |
|  LEFT TEXT     |  RIGHT IMAGE   |
|  (description) |  (screenshot)  |
|                |                 |
+----------------------------------+
|          [ CTA BUTTON ]          |
+----------------------------------+
|            FOOTER                |
+----------------------------------+
```

### Multi-Block (Newsletter Digest)

Multiple content blocks separated by dividers.

```
+----------------------------------+
|           HEADER (logo)          |
+----------------------------------+
|         HERO (main topic)        |
+----------------------------------+
|     CONTENT BLOCK 1              |
|     (image + text + link)        |
+----------------------------------+
|          --- divider ---         |
+----------------------------------+
|     CONTENT BLOCK 2              |
|     (image + text + link)        |
+----------------------------------+
|          --- divider ---         |
+----------------------------------+
|     CONTENT BLOCK 3              |
|     (image + text + link)        |
+----------------------------------+
|          [ CTA BUTTON ]          |
+----------------------------------+
|            FOOTER                |
+----------------------------------+
```

---

## Subject Line & Preheader

### Subject Line Rules

- **Max 50 characters** (mobile truncates at ~40-50)
- **Primary keyword first** -- front-load the most important word
- **No all-caps** -- triggers spam filters
- **No exclamation marks** -- max 1, preferably none
- **Value-first** -- what the reader gets, not what you want
- **Brand voice** -- professional "tu", value vocabulary

### Subject Line Examples (Spanish)

| Email Type | Good | Bad |
|-----------|------|-----|
| Newsletter | "Tu resumen mensual: novedades Odoo enero" | "NEWSLETTER ENERO!!!" |
| Promotional | "Automatiza tu almacen en 4 semanas" | "OFERTA ESPECIAL - No te lo pierdas" |
| Event | "Webinar: ERP para fabricacion - 15 feb" | "Estas invitado a un evento especial!" |

### Preheader Text

Hidden text that appears in the inbox preview (next to subject line). Complements the subject -- does not repeat it.

- **85-100 characters** optimal
- **Pad with whitespace entities** to prevent email client from pulling body text

```html
<!-- Place immediately after <body> or at the top of your content -->
<div style="display:none;font-size:1px;color:#f7f8fc;line-height:1px;max-height:0;max-width:0;opacity:0;overflow:hidden;mso-hide:all;">
  Descubre como optimizar tu cadena de suministro con Odoo -- resultados reales de nuestros clientes.
  &zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;
  &zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;
</div>
```

---

## Table-Based Layout (Outlook)

### Why Tables

Microsoft Outlook (desktop) uses Word's rendering engine, not a browser engine. This means:
- No flexbox, no CSS grid, no float
- No `max-width` (use `width` attribute)
- No `background-image` (use `bgcolor` attribute)
- `border-radius` partially ignored
- `padding` only works on `<td>`, not on `<div>` or `<p>`

### Base Table Structure

Every email starts with this outer shell:

```html
<!DOCTYPE html>
<html lang="es" xmlns="http://www.w3.org/1999/xhtml" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:o="urn:schemas-microsoft-com:office:office">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="X-UA-Compatible" content="IE=edge">
  <meta name="x-apple-disable-message-reformatting">
  <title>Titulo del email</title>
  <!--[if mso]>
  <noscript>
    <xml>
      <o:OfficeDocumentSettings>
        <o:PixelsPerInch>96</o:PixelsPerInch>
      </o:OfficeDocumentSettings>
    </xml>
  </noscript>
  <![endif]-->
</head>
<body style="margin:0;padding:0;background-color:#f7f8fc;font-family:'Dosis',Verdana,Arial,sans-serif;">

  <!-- Outer wrapper table (full width, centered) -->
  <table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0" style="background-color:#f7f8fc;">
    <tr>
      <td align="center" style="padding:20px 10px;">

        <!-- Inner content table (600px max) -->
        <table role="presentation" width="600" cellspacing="0" cellpadding="0" border="0" style="background-color:#ffffff;border-radius:8px;">

          <!-- CONTENT ROWS GO HERE -->

        </table>

      </td>
    </tr>
  </table>

</body>
</html>
```

### Two-Column Pattern (Outlook-Safe)

```html
<!-- Two-column row -->
<tr>
  <td style="padding:24px 32px;">
    <table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0">
      <tr>
        <td width="48%" valign="top" style="padding-right:12px;">
          <!-- Left column content -->
          <h3 style="font-family:'Dosis',Verdana,sans-serif;font-size:18px;font-weight:700;color:#1C265D;margin:0 0 8px 0;">
            Titulo izquierda
          </h3>
          <p style="font-family:'Dosis',Verdana,sans-serif;font-size:14px;line-height:1.6;color:#334155;margin:0;">
            Texto descriptivo de la columna izquierda.
          </p>
        </td>
        <td width="4%">&nbsp;</td>
        <td width="48%" valign="top" style="padding-left:12px;">
          <!-- Right column content -->
          <img src="https://www.enterprise-demo.es/web/image/..." alt="Descripcion" width="260" style="display:block;max-width:100%;height:auto;border-radius:8px;">
        </td>
      </tr>
    </table>
  </td>
</tr>
```

### MSO Conditional Comments

For Outlook-specific overrides:

```html
<!--[if mso]>
  <table role="presentation" width="600"><tr><td>
<![endif]-->

  <!-- Your content here (uses max-width for modern clients) -->
  <div style="max-width:600px;margin:0 auto;">
    ...
  </div>

<!--[if mso]>
  </td></tr></table>
<![endif]-->
```

---

## Inline Style Requirements

### Why Inline

Gmail strips all `<style>` blocks entirely. Yahoo and Outlook.com partially strip them. The only reliable method is inline styles on every element.

### Design Token to Inline Style Mapping

| Design Token | Inline Style |
|-------------|-------------|
| `--iw-blue-space` | `color:#1C265D` |
| `--iw-warm-yellow` | `background-color:#E5B92B` |
| `--iw-blue-lilac` | `color:#6B68FA` |
| `--iw-light-sky` | `color:#8AC2DB` |
| `iw-heading-xl` | `font-family:'Dosis',Verdana,sans-serif;font-size:32px;font-weight:700;line-height:1.2;color:#1C265D;` |
| `iw-heading-md` | `font-family:'Dosis',Verdana,sans-serif;font-size:24px;font-weight:700;line-height:1.3;color:#1C265D;` |
| `iw-heading-xs` | `font-family:'Dosis',Verdana,sans-serif;font-size:18px;font-weight:700;line-height:1.4;color:#1C265D;` |
| `iw-body` | `font-family:'Dosis',Verdana,sans-serif;font-size:16px;line-height:1.6;color:#334155;` |
| `iw-body` (14px variant) | `font-family:'Dosis',Verdana,sans-serif;font-size:14px;line-height:1.6;color:#334155;` |
| `iw-caption` | `font-family:'Dosis',Verdana,sans-serif;font-size:12px;line-height:1.5;color:#64748b;` |
| `iw-overline` | `font-family:'Dosis',Verdana,sans-serif;font-size:12px;font-weight:600;letter-spacing:0.1em;text-transform:uppercase;color:#6B68FA;` |

### Common Inline Patterns

**Heading:**
```html
<h2 style="font-family:'Dosis',Verdana,sans-serif;font-size:24px;font-weight:700;color:#1C265D;margin:0 0 16px 0;">
  Titulo de seccion
</h2>
```

**Body paragraph:**
```html
<p style="font-family:'Dosis',Verdana,sans-serif;font-size:16px;line-height:1.6;color:#334155;margin:0 0 16px 0;">
  Texto del parrafo con informacion relevante.
</p>
```

**Link:**
```html
<a href="https://www.enterprise-demo.es/..." style="color:#6B68FA;text-decoration:underline;">texto del enlace</a>
```

**Divider:**
```html
<tr>
  <td style="padding:0 32px;">
    <hr style="border:none;border-top:1px solid #e2e8f0;margin:24px 0;">
  </td>
</tr>
```

---

## Image Handling

### Rules

| Rule | Value |
|------|-------|
| URLs | Always absolute (`https://www.enterprise-demo.es/web/image/...`) |
| Width attribute | Always set explicit `width` attribute on `<img>` |
| Max image width | 560px (600px container - 2x20px padding) |
| Inline style | `display:block;max-width:100%;height:auto;` |
| Alt text | Always present, descriptive (accessibility + image-blocking clients) |
| Format | JPEG for photos, PNG for logos/graphics |
| Retina | Serve 2x resolution, display at 1x size |
| Fallback | Set `bgcolor` on parent `<td>` behind images |

### Image Example

```html
<tr>
  <td style="padding:0;" bgcolor="#f7f8fc">
    <img
      src="https://www.enterprise-demo.es/web/image/ir.attachment/123/datas"
      alt="Dashboard de Odoo mostrando metricas en tiempo real"
      width="600"
      style="display:block;max-width:100%;height:auto;border:0;"
    >
  </td>
</tr>
```

### Logo in Header

```html
<tr>
  <td align="center" style="background-color:#1C265D;padding:24px 32px;">
    <img
      src="https://www.enterprise-demo.es/web/image/website/1/logo"
      alt="Enterprise Demo"
      width="140"
      style="display:block;max-width:140px;height:auto;border:0;"
    >
  </td>
</tr>
```

---

## CTA Button Patterns

### Bulletproof Button (Outlook + Modern)

This pattern works in all email clients including Outlook desktop. Uses VML (Vector Markup Language) for Outlook and standard HTML/CSS for modern clients.

```html
<tr>
  <td align="center" style="padding:24px 32px;">
    <div style="text-align:center;">
      <!--[if mso]>
      <v:roundrect xmlns:v="urn:schemas-microsoft-com:vml"
        xmlns:w="urn:schemas-microsoft-com:office:word"
        href="https://www.enterprise-demo.es/contacto-enterprise-demo"
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
      <a href="https://www.enterprise-demo.es/contacto-enterprise-demo"
        style="display:inline-block;background-color:#E5B92B;color:#1C265D;font-family:'Dosis',Verdana,sans-serif;font-size:14px;font-weight:700;text-transform:uppercase;text-decoration:none;padding:12px 28px;border-radius:100px;line-height:1;mso-hide:all;">
        Solicitar demo
      </a>
      <!--<![endif]-->
    </div>
  </td>
</tr>
```

### Simple CSS Button (Non-Outlook Fallback)

For environments where Outlook is not a concern (e.g., B2C audiences):

```html
<tr>
  <td align="center" style="padding:24px 32px;">
    <a href="https://www.enterprise-demo.es/contacto-enterprise-demo"
      style="display:inline-block;background-color:#E5B92B;color:#1C265D;font-family:'Dosis',Verdana,sans-serif;font-size:14px;font-weight:700;text-transform:uppercase;text-decoration:none;padding:12px 28px;border-radius:100px;">
      Solicitar demo
    </a>
  </td>
</tr>
```

### Padding-Based Button (Most Compatible)

Uses table cell padding for reliable rendering across all clients:

```html
<tr>
  <td align="center" style="padding:24px 32px;">
    <table role="presentation" cellspacing="0" cellpadding="0" border="0" align="center">
      <tr>
        <td style="background-color:#E5B92B;border-radius:100px;">
          <a href="https://www.enterprise-demo.es/contacto-enterprise-demo"
            style="display:block;padding:12px 28px;font-family:'Dosis',Verdana,sans-serif;font-size:14px;font-weight:700;color:#1C265D;text-transform:uppercase;text-decoration:none;text-align:center;">
            Solicitar demo
          </a>
        </td>
      </tr>
    </table>
  </td>
</tr>
```

### Button Rules

- **One primary CTA per email** -- yellow background (`#E5B92B`), dark blue text (`#1C265D`)
- **Optional secondary link** -- text link with `color:#6B68FA;text-decoration:underline;`
- **Text**: 14px uppercase, **max 4 words** ("Solicitar demo", "Ver modulos", "Registrarse ahora")
- **Minimum tap target**: 44x44px (padding ensures this)
- **Never use** "Haz clic aqui" or "Leer mas" -- use action-specific text

---

## Footer Requirements

### Legal Requirements (Spanish Law -- LSSI)

All commercial emails in Spain must include:
- Sender identification (company name)
- Physical address
- Unsubscribe mechanism (Odoo adds this automatically)
- Privacy policy link

### Footer HTML

```html
<!-- Footer row -->
<tr>
  <td style="background-color:#f7f8fc;padding:24px 32px;border-radius:0 0 8px 8px;">
    <table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0">
      <!-- Company info -->
      <tr>
        <td align="center" style="padding-bottom:16px;">
          <p style="font-family:'Dosis',Verdana,sans-serif;font-size:14px;font-weight:600;color:#1C265D;margin:0;">
            Enterprise Demo Business Solutions S.L.
          </p>
          <p style="font-family:'Dosis',Verdana,sans-serif;font-size:12px;color:#64748b;margin:4px 0 0 0;">
            Tu partner Odoo de confianza
          </p>
        </td>
      </tr>
      <!-- Social links -->
      <tr>
        <td align="center" style="padding-bottom:16px;">
          <a href="https://www.linkedin.com/company/enterprise-demo" style="text-decoration:none;margin:0 8px;">
            <img src="https://www.enterprise-demo.es/web/image/linkedin-icon" alt="LinkedIn" width="24" height="24" style="display:inline-block;border:0;">
          </a>
          <a href="https://www.youtube.com/@enterprise-demo-odoo" style="text-decoration:none;margin:0 8px;">
            <img src="https://www.enterprise-demo.es/web/image/youtube-icon" alt="YouTube" width="24" height="24" style="display:inline-block;border:0;">
          </a>
        </td>
      </tr>
      <!-- Legal -->
      <tr>
        <td align="center">
          <p style="font-family:'Dosis',Verdana,sans-serif;font-size:11px;color:#94a3b8;margin:0;line-height:1.6;">
            Recibes este email porque estas suscrito a nuestra lista de comunicaciones.
            <br>
            <a href="${unsubscribe_url}" style="color:#6B68FA;text-decoration:underline;">Darme de baja</a>
            &nbsp;|&nbsp;
            <a href="https://www.enterprise-demo.es/politica-de-privacidad" style="color:#6B68FA;text-decoration:underline;">Politica de privacidad</a>
          </p>
        </td>
      </tr>
    </table>
  </td>
</tr>
```

**Note**: Odoo automatically adds an unsubscribe link. The `${unsubscribe_url}` placeholder is resolved by Odoo when sending. You can omit the manual unsubscribe link if you prefer Odoo to handle it entirely.

---

## Brand Compliance

All emails must follow `enterprise-demo-brand` guidelines. Quick reference for email context:

### Colors (Solid Only)

| Usage | Color | Hex |
|-------|-------|-----|
| Header background | Blue Space | `#1C265D` |
| Body text | Dark | `#334155` |
| Headings | Blue Space | `#1C265D` |
| CTA button | Warm Yellow bg + Blue Space text | `#E5B92B` / `#1C265D` |
| Links | Blue Lilac | `#6B68FA` |
| Footer background | Light gray | `#f7f8fc` |
| Secondary text | Gray | `#64748b` |
| Dividers | Light border | `#e2e8f0` |

**No gradients in email** -- all solid colors.

### Typography

- **Font**: `'Dosis', Verdana, Arial, sans-serif`
- Dosis loads in Apple Mail, iOS Mail, and some Android clients
- **Verdana** is the primary fallback (similar proportions)
- Weights: 400 (body), 600 (semi-bold), 700 (headings, buttons)

### Voice

- Professional "tu" (never "usted", never "vosotros")
- Zero emojis in email body and subject line
- Value vocabulary: "inversion" not "gasto", "reto" not "problema", "solucion" not "arreglo"
- Action-oriented CTA text: "Solicitar demo", "Ver resultados", "Hablar con un consultor"

---

## Content-Collection Integration

### Case Study Social Proof

Load `data/cases.json` from `content-collection` to add real client references:

```html
<tr>
  <td style="padding:24px 32px;background-color:#f7f8fc;">
    <p style="font-family:'Dosis',Verdana,sans-serif;font-size:14px;font-style:italic;color:#334155;margin:0 0 8px 0;">
      "Gracias a Odoo e Enterprise Demo, hemos reducido un 40% el tiempo de procesamiento de pedidos."
    </p>
    <p style="font-family:'Dosis',Verdana,sans-serif;font-size:12px;font-weight:600;color:#1C265D;margin:0 0 4px 0;">
      Maria Lopez
    </p>
    <p style="font-family:'Dosis',Verdana,sans-serif;font-size:12px;color:#64748b;margin:0 0 12px 0;">
      Directora de Operaciones, Empresa XYZ
    </p>
    <a href="https://www.enterprise-demo.es/blog/referencias-enterprise-demo-16/caso-empresa-xyz-456"
      style="font-family:'Dosis',Verdana,sans-serif;font-size:13px;color:#6B68FA;text-decoration:underline;">
      Ver caso de exito completo
    </a>
  </td>
</tr>
```

### Video Thumbnail Links

Emails cannot embed iframes. Use a clickable thumbnail image linking to YouTube:

```html
<tr>
  <td style="padding:24px 32px;">
    <a href="https://www.youtube.com/watch?v=VIDEO_ID" style="text-decoration:none;">
      <img
        src="https://img.youtube.com/vi/VIDEO_ID/maxresdefault.jpg"
        alt="Titulo del video -- Ver en YouTube"
        width="536"
        style="display:block;max-width:100%;height:auto;border-radius:8px;border:0;"
      >
    </a>
    <p style="font-family:'Dosis',Verdana,sans-serif;font-size:12px;color:#64748b;text-align:center;margin:8px 0 0 0;">
      Titulo del video
    </p>
  </td>
</tr>
```

**Tip**: Add a "play button" overlay image on top of the thumbnail for better click-through rates.

---

## Complete Email Templates

### Newsletter Template

Full HTML for a monthly newsletter with header, hero, two content blocks, CTA, and footer.

```html
<!DOCTYPE html>
<html lang="es" xmlns="http://www.w3.org/1999/xhtml" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:o="urn:schemas-microsoft-com:office:office">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="X-UA-Compatible" content="IE=edge">
  <meta name="x-apple-disable-message-reformatting">
  <title>Newsletter Enterprise Demo -- Enero 2026</title>
  <!--[if mso]>
  <noscript><xml><o:OfficeDocumentSettings><o:PixelsPerInch>96</o:PixelsPerInch></o:OfficeDocumentSettings></xml></noscript>
  <![endif]-->
</head>
<body style="margin:0;padding:0;background-color:#f7f8fc;font-family:'Dosis',Verdana,Arial,sans-serif;">

<!-- Preheader -->
<div style="display:none;font-size:1px;color:#f7f8fc;line-height:1px;max-height:0;max-width:0;opacity:0;overflow:hidden;mso-hide:all;">
  Las novedades mas relevantes de Odoo y transformacion digital para tu empresa este mes.
  &zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;
</div>

<!-- Outer wrapper -->
<table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0" style="background-color:#f7f8fc;">
  <tr>
    <td align="center" style="padding:20px 10px;">

      <!-- Inner content (600px) -->
      <table role="presentation" width="600" cellspacing="0" cellpadding="0" border="0" style="background-color:#ffffff;border-radius:8px;overflow:hidden;">

        <!-- Header -->
        <tr>
          <td align="center" style="background-color:#1C265D;padding:24px 32px;">
            <img src="https://www.enterprise-demo.es/web/image/website/1/logo" alt="Enterprise Demo" width="140" style="display:block;max-width:140px;height:auto;border:0;">
          </td>
        </tr>

        <!-- Hero -->
        <tr>
          <td style="padding:32px 32px 24px 32px;">
            <p style="font-family:'Dosis',Verdana,sans-serif;font-size:12px;font-weight:600;letter-spacing:0.1em;text-transform:uppercase;color:#6B68FA;margin:0 0 8px 0;">
              NEWSLETTER ENERO 2026
            </p>
            <h1 style="font-family:'Dosis',Verdana,sans-serif;font-size:28px;font-weight:700;color:#1C265D;margin:0 0 16px 0;line-height:1.2;">
              Novedades del mes en transformacion digital
            </h1>
            <p style="font-family:'Dosis',Verdana,sans-serif;font-size:16px;line-height:1.6;color:#334155;margin:0;">
              Descubre las ultimas tendencias, casos de exito y funcionalidades de Odoo que pueden impulsar la eficiencia de tu empresa.
            </p>
          </td>
        </tr>

        <!-- Divider -->
        <tr>
          <td style="padding:0 32px;">
            <hr style="border:none;border-top:1px solid #e2e8f0;margin:0;">
          </td>
        </tr>

        <!-- Content Block 1 -->
        <tr>
          <td style="padding:24px 32px;">
            <h2 style="font-family:'Dosis',Verdana,sans-serif;font-size:20px;font-weight:700;color:#1C265D;margin:0 0 12px 0;">
              Titulo del primer articulo destacado
            </h2>
            <p style="font-family:'Dosis',Verdana,sans-serif;font-size:15px;line-height:1.6;color:#334155;margin:0 0 16px 0;">
              Resumen breve del articulo o novedad. Dos o tres frases que capten la atencion del lector y le motiven a leer mas sobre el tema.
            </p>
            <a href="https://www.enterprise-demo.es/blog/blog-enterprise-demo-1/articulo-1" style="font-family:'Dosis',Verdana,sans-serif;font-size:14px;font-weight:600;color:#6B68FA;text-decoration:underline;">
              Leer articulo completo
            </a>
          </td>
        </tr>

        <!-- Divider -->
        <tr>
          <td style="padding:0 32px;">
            <hr style="border:none;border-top:1px solid #e2e8f0;margin:0;">
          </td>
        </tr>

        <!-- Content Block 2 -->
        <tr>
          <td style="padding:24px 32px;">
            <h2 style="font-family:'Dosis',Verdana,sans-serif;font-size:20px;font-weight:700;color:#1C265D;margin:0 0 12px 0;">
              Titulo del segundo contenido
            </h2>
            <p style="font-family:'Dosis',Verdana,sans-serif;font-size:15px;line-height:1.6;color:#334155;margin:0 0 16px 0;">
              Segundo bloque de contenido. Puede ser un caso de exito, una guia practica o una novedad de producto.
            </p>
            <a href="https://www.enterprise-demo.es/blog/blog-enterprise-demo-1/articulo-2" style="font-family:'Dosis',Verdana,sans-serif;font-size:14px;font-weight:600;color:#6B68FA;text-decoration:underline;">
              Descubrir mas
            </a>
          </td>
        </tr>

        <!-- CTA -->
        <tr>
          <td align="center" style="padding:24px 32px;">
            <table role="presentation" cellspacing="0" cellpadding="0" border="0" align="center">
              <tr>
                <td style="background-color:#E5B92B;border-radius:100px;">
                  <a href="https://www.enterprise-demo.es/contacto-enterprise-demo"
                    style="display:block;padding:12px 28px;font-family:'Dosis',Verdana,sans-serif;font-size:14px;font-weight:700;color:#1C265D;text-transform:uppercase;text-decoration:none;text-align:center;">
                    Hablar con un consultor
                  </a>
                </td>
              </tr>
            </table>
          </td>
        </tr>

        <!-- Footer -->
        <tr>
          <td style="background-color:#f7f8fc;padding:24px 32px;border-radius:0 0 8px 8px;">
            <table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0">
              <tr>
                <td align="center" style="padding-bottom:12px;">
                  <p style="font-family:'Dosis',Verdana,sans-serif;font-size:13px;font-weight:600;color:#1C265D;margin:0;">
                    Enterprise Demo Business Solutions S.L.
                  </p>
                </td>
              </tr>
              <tr>
                <td align="center">
                  <p style="font-family:'Dosis',Verdana,sans-serif;font-size:11px;color:#94a3b8;margin:0;line-height:1.6;">
                    Recibes este email porque estas suscrito a nuestra lista de comunicaciones.
                    <br>
                    <a href="${unsubscribe_url}" style="color:#6B68FA;text-decoration:underline;">Darme de baja</a>
                    &nbsp;|&nbsp;
                    <a href="https://www.enterprise-demo.es/politica-de-privacidad" style="color:#6B68FA;text-decoration:underline;">Politica de privacidad</a>
                  </p>
                </td>
              </tr>
            </table>
          </td>
        </tr>

      </table>
    </td>
  </tr>
</table>

</body>
</html>
```

### Promotional Template

Full HTML for a single-offer promotional email.

```html
<!DOCTYPE html>
<html lang="es" xmlns="http://www.w3.org/1999/xhtml" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:o="urn:schemas-microsoft-com:office:office">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="X-UA-Compatible" content="IE=edge">
  <meta name="x-apple-disable-message-reformatting">
  <title>Automatiza tu almacen en 4 semanas</title>
  <!--[if mso]>
  <noscript><xml><o:OfficeDocumentSettings><o:PixelsPerInch>96</o:PixelsPerInch></o:OfficeDocumentSettings></xml></noscript>
  <![endif]-->
</head>
<body style="margin:0;padding:0;background-color:#f7f8fc;font-family:'Dosis',Verdana,Arial,sans-serif;">

<!-- Preheader -->
<div style="display:none;font-size:1px;color:#f7f8fc;line-height:1px;max-height:0;max-width:0;opacity:0;overflow:hidden;mso-hide:all;">
  Descubre como empresas como la tuya han transformado su logistica con Odoo -- resultados en semanas.
  &zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;
</div>

<table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0" style="background-color:#f7f8fc;">
  <tr>
    <td align="center" style="padding:20px 10px;">
      <table role="presentation" width="600" cellspacing="0" cellpadding="0" border="0" style="background-color:#ffffff;border-radius:8px;overflow:hidden;">

        <!-- Header -->
        <tr>
          <td align="center" style="background-color:#1C265D;padding:24px 32px;">
            <img src="https://www.enterprise-demo.es/web/image/website/1/logo" alt="Enterprise Demo" width="140" style="display:block;max-width:140px;height:auto;border:0;">
          </td>
        </tr>

        <!-- Hero Image -->
        <tr>
          <td style="padding:0;">
            <img src="https://www.enterprise-demo.es/web/image/ir.attachment/hero-almacen" alt="Almacen automatizado con Odoo" width="600" style="display:block;max-width:100%;height:auto;border:0;">
          </td>
        </tr>

        <!-- Value Proposition -->
        <tr>
          <td style="padding:32px 32px 16px 32px;">
            <h1 style="font-family:'Dosis',Verdana,sans-serif;font-size:28px;font-weight:700;color:#1C265D;margin:0 0 16px 0;line-height:1.2;">
              Automatiza tu almacen en 4 semanas
            </h1>
            <p style="font-family:'Dosis',Verdana,sans-serif;font-size:16px;line-height:1.6;color:#334155;margin:0 0 16px 0;">
              Las empresas que invierten en automatizacion logistica con Odoo reducen un promedio del 40% el tiempo de procesamiento de pedidos. Te mostramos como conseguirlo de forma predecible, con un plan de implantacion claro y resultados medibles.
            </p>
          </td>
        </tr>

        <!-- Key Metrics -->
        <tr>
          <td style="padding:0 32px 24px 32px;">
            <table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0">
              <tr>
                <td width="33%" align="center" style="padding:16px 8px;background-color:#f7f8fc;border-radius:8px;">
                  <p style="font-family:'Dosis',Verdana,sans-serif;font-size:28px;font-weight:700;color:#6B68FA;margin:0;">-40%</p>
                  <p style="font-family:'Dosis',Verdana,sans-serif;font-size:11px;color:#64748b;margin:4px 0 0 0;">Tiempo proceso</p>
                </td>
                <td width="2%">&nbsp;</td>
                <td width="33%" align="center" style="padding:16px 8px;background-color:#f7f8fc;border-radius:8px;">
                  <p style="font-family:'Dosis',Verdana,sans-serif;font-size:28px;font-weight:700;color:#6B68FA;margin:0;">+85%</p>
                  <p style="font-family:'Dosis',Verdana,sans-serif;font-size:11px;color:#64748b;margin:4px 0 0 0;">Eficiencia</p>
                </td>
                <td width="2%">&nbsp;</td>
                <td width="33%" align="center" style="padding:16px 8px;background-color:#f7f8fc;border-radius:8px;">
                  <p style="font-family:'Dosis',Verdana,sans-serif;font-size:28px;font-weight:700;color:#6B68FA;margin:0;">4 sem.</p>
                  <p style="font-family:'Dosis',Verdana,sans-serif;font-size:11px;color:#64748b;margin:4px 0 0 0;">Implantacion</p>
                </td>
              </tr>
            </table>
          </td>
        </tr>

        <!-- CTA -->
        <tr>
          <td align="center" style="padding:8px 32px 32px 32px;">
            <!--[if mso]>
            <v:roundrect xmlns:v="urn:schemas-microsoft-com:vml" xmlns:w="urn:schemas-microsoft-com:office:word"
              href="https://www.enterprise-demo.es/contacto-enterprise-demo" style="height:44px;v-text-anchor:middle;width:220px;"
              arcsize="50%" strokecolor="#E5B92B" fillcolor="#E5B92B">
              <w:anchorlock/>
              <center style="color:#1C265D;font-family:'Dosis',Verdana,sans-serif;font-size:14px;font-weight:bold;text-transform:uppercase;">
                Solicitar demo
              </center>
            </v:roundrect>
            <![endif]-->
            <!--[if !mso]><!-->
            <a href="https://www.enterprise-demo.es/contacto-enterprise-demo"
              style="display:inline-block;background-color:#E5B92B;color:#1C265D;font-family:'Dosis',Verdana,sans-serif;font-size:14px;font-weight:700;text-transform:uppercase;text-decoration:none;padding:12px 28px;border-radius:100px;line-height:1;">
              Solicitar demo
            </a>
            <!--<![endif]-->
          </td>
        </tr>

        <!-- Footer -->
        <tr>
          <td style="background-color:#f7f8fc;padding:24px 32px;border-radius:0 0 8px 8px;">
            <table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0">
              <tr>
                <td align="center" style="padding-bottom:12px;">
                  <p style="font-family:'Dosis',Verdana,sans-serif;font-size:13px;font-weight:600;color:#1C265D;margin:0;">
                    Enterprise Demo Business Solutions S.L.
                  </p>
                </td>
              </tr>
              <tr>
                <td align="center">
                  <p style="font-family:'Dosis',Verdana,sans-serif;font-size:11px;color:#94a3b8;margin:0;line-height:1.6;">
                    <a href="${unsubscribe_url}" style="color:#6B68FA;text-decoration:underline;">Darme de baja</a>
                    &nbsp;|&nbsp;
                    <a href="https://www.enterprise-demo.es/politica-de-privacidad" style="color:#6B68FA;text-decoration:underline;">Politica de privacidad</a>
                  </p>
                </td>
              </tr>
            </table>
          </td>
        </tr>

      </table>
    </td>
  </tr>
</table>

</body>
</html>
```

---

## Publishing to Odoo

After generating the email HTML, use the **`odoo-pilot`** skill to create the mailing as a **draft** in Odoo for testing and human revision.

### Prerequisites

- The `odoo-pilot` skill must be available
- Odoo connection credentials configured (URL, DB, API key)
- At least one mailing contact list must exist in Odoo

### Step-by-Step Workflow

#### 1. Authenticate with Odoo

```bash
eval $(./scripts/auth.sh)
```

#### 2. Find the Contact List

Search for existing mailing lists:

```bash
./scripts/search_records.sh mailing.list '[]' '["id", "name", "contact_count"]'
```

**Note**: The model is `mailing.list` in Odoo 15+. In older versions, use `mail.mass_mailing.list`.

#### 3. Find the Mailing Model ID

The mass mailing needs the target model ID (typically `mailing.contact`):

```bash
./scripts/search_records.sh ir.model '[["model","=","mailing.contact"]]' '["id"]'
```

#### 4. Prepare Email Data

Build the JSON payload with these fields:

| Field | Required | Description |
|-------|----------|-------------|
| `subject` | Yes | Email subject line (max 50 chars) |
| `body_html` | Yes | Full HTML email content |
| `mailing_type` | Yes | `"mail"` for email |
| `mailing_model_id` | Yes | Target model ID (from step 3) |
| `contact_list_ids` | No | `[[6, 0, [list_id]]]` (many2many syntax) |

#### 5. Create the Mailing

```bash
./scripts/create_record.sh mailing.mailing '{
  "subject": "Tu resumen mensual: novedades Odoo enero",
  "body_html": "<!DOCTYPE html><html lang=\"es\">...full HTML...</html>",
  "mailing_type": "mail",
  "mailing_model_id": MODEL_ID,
  "contact_list_ids": [[6, 0, [LIST_ID]]]
}'
```

**Note**: The model is `mailing.mailing` in Odoo 15+. In older versions, use `mail.mass_mailing`.

The script returns the new record ID.

#### 6. Confirm to User

```
Email creado como borrador en Odoo:
- ID: [record_id]
- URL de edicion: [ODOO_URL]/web#id=[record_id]&model=mailing.mailing&view_type=form
- Estado: Borrador (no enviado)
- Siguiente paso: Usar "Enviar prueba" en Odoo antes de enviar a la lista
```

### Important Notes

- **Never send automatically** -- always create as draft for human review
- **Test first** -- use Odoo's "Send Test" button to send a preview to yourself before sending to the list
- **Odoo re-processes HTML** -- Odoo will re-inline styles, add tracking, and add its own unsubscribe link
- **Escape HTML in JSON** -- the `body_html` field contains HTML that must be properly escaped inside the JSON string
- **Subject line encoding** -- special characters in the subject are handled by Odoo automatically
