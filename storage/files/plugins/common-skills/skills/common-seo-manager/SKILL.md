---
name: "SEO Manager"
description: "SEO management and audit tool with keyword analysis, meta tag generation, technical on-page audit, and reporting"
---

---|-----------|
| `templates/dashboard.html` | Panel principal con métricas y gráficos |
| `templates/audit-report.html` | Plantilla de informe de auditoría |
| `templates/keyword-report.html` | Informe de investigación de keywords |

## Scripts

| Archivo | Propósito |
|---------|-----------|
| `scripts/seo-analyzer.py` | Análisis técnico de URLs |
| `scripts/meta-generator.py` | Generación de meta tags optimizados |

## Diseño

El skill sigue el sistema de diseño Foodles:

- **Colores**: Warm Yellow (#E5B92B), Blue Lilac (#0071ce), Blue Space (#1A0030), Light Sky (#8AC2DB)
- **Tipografía**: Dosis (Google Fonts)
- **Framework**: Bootstrap 5
- **Estilo**: Glassmorphism con layout bento box
- **Tema**: Toggle claro/oscuro con persistencia
- **Animaciones**: Transiciones sutiles CSS

## Dependencias

### Python (scripts)
- requests
- beautifulsoup4

### HTML (templates)
- Bootstrap 5.3 (CDN)
- Dosis font (Google Fonts)
- Chart.js (para gráficos)

## Instalación de dependencias Python

```bash
pip install requests beautifulsoup4
```
