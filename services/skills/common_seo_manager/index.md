
# SEO Manager Skill

## Descripción

Herramienta integral para gestión SEO que proporciona:

- **Dashboard interactivo**: Visualización de métricas SEO en tiempo real
- **Auditoría técnica**: Análisis on-page de URLs con detección de problemas
- **Generador de meta tags**: Creación optimizada de title, description, OG tags
- **Investigación de keywords**: Análisis de competencia y oportunidades
- **Reporting**: Informes profesionales de auditoría SEO

## Uso

### Dashboard SEO

Abre `templates/dashboard.html` en navegador para acceder al panel interactivo.
Funcionalidades:
- Toggle tema claro/oscuro (persistente en localStorage)
- Métricas clave en layout bento box
- Gráficos de evolución de posiciones
- Lista de tareas SEO pendientes

### Auditoría de URL

```bash
python scripts/seo-analyzer.py https://ejemplo.com
```

Analiza:
- Meta tags (title, description, robots)
- Encabezados (estructura H1-H6)
- Imágenes (alt text, tamaño)
- Enlaces (internos, externos, rotos)
- Rendimiento (tiempo de carga)
- Seguridad (HTTPS, headers)

### Generación de Meta Tags

```bash
python scripts/meta-generator.py --keyword "diseño web valencia" --type "service"
```

Genera:
- Title tag optimizado (50-60 caracteres)
- Meta description (150-160 caracteres)
- Open Graph tags
- Twitter Card tags
- Schema.org JSON-LD

## Templates

| Archivo | Propósito |
|---------|-----------|
| `templates/dashboard.html` | Panel principal con métricas y gráficos |
| `templates/audit-report.html` | Plantilla de informe de auditoría |
| `templates/keyword-report.html` | Informe de investigación de keywords |

## Scripts

| Archivo | Propósito |
|---------|-----------|
| `scripts/seo-analyzer.py` | Análisis técnico de URLs |
| `scripts/meta-generator.py` | Generación de meta tags optimizados |

## Diseño

El skill sigue el sistema de diseño Enterprise Demo:

- **Colores**: Warm Yellow (#E5B92B), Blue Lilac (#6B68FA), Blue Space (#1C265D), Light Sky (#8AC2DB)
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
