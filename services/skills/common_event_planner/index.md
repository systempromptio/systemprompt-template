
# Event Planner

Herramienta de planificacion de eventos de marketing para Enterprise Demo. Genera documentos HTML interactivos con cronograma, presupuesto, gestion de asistentes y seguimiento de tareas.

## Caracteristicas

- **Cronograma visual**: Timeline interactivo con fases del evento
- **Presupuesto detallado**: Desglose por categorias con totales automaticos
- **Gestion de tareas**: Checklist con estados y responsables
- **Asistentes**: Listado con confirmaciones y detalles de contacto
- **Tema claro/oscuro**: Toggle para preferencia del usuario
- **Responsive**: Diseño adaptable a cualquier dispositivo
- **Exportable**: HTML autonomo listo para compartir

## Diseno

Sigue las directrices de marca Enterprise Demo:
- **Colores**: Warm Yellow (#E5B92B), Blue Lilac (#6B68FA), Blue Space (#1C265D), Light Sky (#8AC2DB)
- **Tipografia**: Dosis (Google Fonts)
- **Base**: Bootstrap 5
- **Estilo**: Glassmorphism + Bento box layout
- **Animaciones**: Transiciones CSS sutiles

## Estructura

```
event-planner/
├── SKILL.md                 # Este archivo
├── templates/
│   ├── event-plan.html      # Template principal
│   └── styles.css           # Estilos con variables CSS
└── scripts/
    └── generate_plan.sh     # Generador de planes
```

## Uso

### Generar plan de evento

```bash
./scripts/generate_plan.sh "Nombre del Evento" "2026-03-15" "output.html"
```

### Parametros del template

El template HTML acepta los siguientes datos:

| Variable | Descripcion |
|----------|-------------|
| `{{EVENT_NAME}}` | Nombre del evento |
| `{{EVENT_DATE}}` | Fecha del evento |
| `{{EVENT_LOCATION}}` | Ubicacion |
| `{{EVENT_DESCRIPTION}}` | Descripcion detallada |
| `{{BUDGET_ITEMS}}` | Items de presupuesto (HTML) |
| `{{TIMELINE_ITEMS}}` | Fases del cronograma (HTML) |
| `{{TASKS}}` | Lista de tareas (HTML) |
| `{{ATTENDEES}}` | Lista de asistentes (HTML) |

## Secciones del Plan

### 1. Cabecera
- Logo Enterprise Demo
- Nombre del evento
- Fecha y ubicacion
- Toggle tema claro/oscuro

### 2. Resumen Ejecutivo
- Descripcion del evento
- Objetivos principales
- Metricas de exito

### 3. Cronograma (Timeline)
- Fases: Planificacion, Preparacion, Ejecucion, Seguimiento
- Fechas clave
- Responsables por fase

### 4. Presupuesto
- Categorias: Venue, Catering, Material, Promocion, Contingencia
- Inversion estimada vs real
- Totales y porcentajes

### 5. Tareas
- Checklist con estados: Pendiente, En progreso, Completada
- Asignacion de responsables
- Fechas limite

### 6. Asistentes
- Lista de invitados
- Estado de confirmacion
- Notas especiales

## Integracion

- **Odoo**: Sincroniza con proyectos y tareas
- **Calendar**: Exporta fechas clave a Google Calendar
- **Email**: Envia invitaciones automatizadas

## Notas de Diseno

- **Sin emojis**: Siguiendo politica Enterprise Demo
- **Lenguaje**: Profesional, orientado a valor
- **Glassmorphism**: Fondos translucidos con blur
- **Bento box**: Cuadricula modular para secciones
- **Animaciones**: Solo en hover/focus, sutiles
