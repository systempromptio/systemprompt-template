---
name: "Event Planner"
description: "Marketing event planning with timeline, budget breakdown, attendee management, and task tracking"
---

----|-------------|
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
- Logo Foodles
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

- **Sin emojis**: Siguiendo politica Foodles
- **Lenguaje**: Profesional, orientado a valor
- **Glassmorphism**: Fondos translucidos con blur
- **Bento box**: Cuadricula modular para secciones
- **Animaciones**: Solo en hover/focus, sutiles
