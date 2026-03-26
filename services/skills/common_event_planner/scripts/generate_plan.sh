#!/bin/bash
# Event Planner - Generate HTML Plan
# Usage: ./generate_plan.sh "Event Name" "2026-03-15" "output.html"
#
# Environment variables (optional):
#   EVENT_LOCATION - Event location (default: "Por definir")
#   EVENT_DESCRIPTION - Event description

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE_DIR="$SCRIPT_DIR/../templates"

# Arguments
EVENT_NAME="${1:-Nuevo Evento}"
EVENT_DATE="${2:-$(date -d '+30 days' +%Y-%m-%d 2>/dev/null || date -v+30d +%Y-%m-%d)}"
OUTPUT_FILE="${3:-event-plan.html}"

# Environment variables with defaults
EVENT_LOCATION="${EVENT_LOCATION:-Por definir}"
EVENT_DESCRIPTION="${EVENT_DESCRIPTION:-Plan de evento en desarrollo. Actualiza esta descripcion con los detalles especificos del evento, objetivos y publico objetivo.}"

# Calculate days to event
if command -v gdate &> /dev/null; then
  DATE_CMD="gdate"
else
  DATE_CMD="date"
fi

TODAY=$($DATE_CMD +%Y-%m-%d)
DAYS_TO_EVENT=$(( ( $($DATE_CMD -d "$EVENT_DATE" +%s) - $($DATE_CMD -d "$TODAY" +%s) ) / 86400 )) 2>/dev/null || DAYS_TO_EVENT="--"

GENERATION_DATE=$($DATE_CMD "+%d/%m/%Y %H:%M")
YEAR=$($DATE_CMD +%Y)

# Sample data (replace with actual data source integration)
TOTAL_TASKS=12
TASKS_COMPLETED=25
PENDING_TASKS=9
ATTENDEE_COUNT=24

# Generate objectives HTML
OBJECTIVES='
<li class="task-item">
  <div class="task-checkbox">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><polyline points="20 6 9 17 4 12"></polyline></svg>
  </div>
  <div class="task-content">
    <div class="task-title">Incrementar visibilidad de marca en el sector</div>
  </div>
</li>
<li class="task-item">
  <div class="task-checkbox">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><polyline points="20 6 9 17 4 12"></polyline></svg>
  </div>
  <div class="task-content">
    <div class="task-title">Generar leads cualificados para el equipo comercial</div>
  </div>
</li>
<li class="task-item">
  <div class="task-checkbox">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><polyline points="20 6 9 17 4 12"></polyline></svg>
  </div>
  <div class="task-content">
    <div class="task-title">Fortalecer relaciones con clientes actuales</div>
  </div>
</li>'

# Generate timeline HTML
TIMELINE_ITEMS='
<div class="timeline-item completed">
  <div class="timeline-marker"></div>
  <div class="timeline-content">
    <div class="timeline-date">Fase 1 - Planificacion</div>
    <div class="timeline-title">Definicion de objetivos y alcance</div>
    <p class="timeline-description">Establecer metas, presupuesto inicial y equipo responsable.</p>
  </div>
</div>
<div class="timeline-item active">
  <div class="timeline-marker"></div>
  <div class="timeline-content">
    <div class="timeline-date">Fase 2 - Preparacion</div>
    <div class="timeline-title">Logistica y proveedores</div>
    <p class="timeline-description">Reserva de venue, contratacion de catering y material promocional.</p>
  </div>
</div>
<div class="timeline-item">
  <div class="timeline-marker"></div>
  <div class="timeline-content">
    <div class="timeline-date">Fase 3 - Promocion</div>
    <div class="timeline-title">Campana de comunicacion</div>
    <p class="timeline-description">Invitaciones, redes sociales y confirmacion de asistentes.</p>
  </div>
</div>
<div class="timeline-item">
  <div class="timeline-marker"></div>
  <div class="timeline-content">
    <div class="timeline-date">Fase 4 - Ejecucion</div>
    <div class="timeline-title">Dia del evento</div>
    <p class="timeline-description">Coordinacion en sitio y seguimiento post-evento.</p>
  </div>
</div>'

# Generate budget HTML
BUDGET_ITEMS='
<tr>
  <td>Venue</td>
  <td>Alquiler de espacio y equipamiento tecnico</td>
  <td class="amount">2.500,00 EUR</td>
  <td class="amount">--</td>
  <td class="amount">--</td>
</tr>
<tr>
  <td>Catering</td>
  <td>Coffee break y almuerzo para asistentes</td>
  <td class="amount">1.800,00 EUR</td>
  <td class="amount">--</td>
  <td class="amount">--</td>
</tr>
<tr>
  <td>Material</td>
  <td>Merchandising, folletos y presentaciones</td>
  <td class="amount">800,00 EUR</td>
  <td class="amount">--</td>
  <td class="amount">--</td>
</tr>
<tr>
  <td>Promocion</td>
  <td>Campanas digitales y diseno grafico</td>
  <td class="amount">600,00 EUR</td>
  <td class="amount">--</td>
  <td class="amount">--</td>
</tr>
<tr>
  <td>Contingencia</td>
  <td>Reserva para imprevistos (10%)</td>
  <td class="amount">570,00 EUR</td>
  <td class="amount">--</td>
  <td class="amount">--</td>
</tr>
<tr class="total-row">
  <td colspan="2"><strong>TOTAL INVERSION</strong></td>
  <td class="amount"><strong>6.270,00 EUR</strong></td>
  <td class="amount"><strong>--</strong></td>
  <td class="amount"><strong>--</strong></td>
</tr>'

# Generate tasks HTML
TASKS='
<li class="task-item completed">
  <div class="task-checkbox">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><polyline points="20 6 9 17 4 12"></polyline></svg>
  </div>
  <div class="task-content">
    <div class="task-title">Definir objetivos del evento</div>
    <div class="task-meta">
      <span>Marketing</span>
      <span>Completado</span>
    </div>
  </div>
</li>
<li class="task-item completed">
  <div class="task-checkbox">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><polyline points="20 6 9 17 4 12"></polyline></svg>
  </div>
  <div class="task-content">
    <div class="task-title">Aprobar presupuesto inicial</div>
    <div class="task-meta">
      <span>Direccion</span>
      <span>Completado</span>
    </div>
  </div>
</li>
<li class="task-item completed">
  <div class="task-checkbox">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><polyline points="20 6 9 17 4 12"></polyline></svg>
  </div>
  <div class="task-content">
    <div class="task-title">Seleccionar fecha y venue</div>
    <div class="task-meta">
      <span>Operaciones</span>
      <span>Completado</span>
    </div>
  </div>
</li>
<li class="task-item">
  <div class="task-checkbox">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><polyline points="20 6 9 17 4 12"></polyline></svg>
  </div>
  <div class="task-content">
    <div class="task-title">Contratar servicio de catering</div>
    <div class="task-meta">
      <span>Operaciones</span>
      <span class="badge badge-progress">En progreso</span>
    </div>
  </div>
</li>
<li class="task-item">
  <div class="task-checkbox">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><polyline points="20 6 9 17 4 12"></polyline></svg>
  </div>
  <div class="task-content">
    <div class="task-title">Disenar material promocional</div>
    <div class="task-meta">
      <span>Marketing</span>
      <span class="badge badge-pending">Pendiente</span>
    </div>
  </div>
</li>
<li class="task-item">
  <div class="task-checkbox">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><polyline points="20 6 9 17 4 12"></polyline></svg>
  </div>
  <div class="task-content">
    <div class="task-title">Enviar invitaciones</div>
    <div class="task-meta">
      <span>Marketing</span>
      <span class="badge badge-pending">Pendiente</span>
    </div>
  </div>
</li>
<li class="task-item">
  <div class="task-checkbox">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><polyline points="20 6 9 17 4 12"></polyline></svg>
  </div>
  <div class="task-content">
    <div class="task-title">Confirmar ponentes y agenda</div>
    <div class="task-meta">
      <span>Contenido</span>
      <span class="badge badge-pending">Pendiente</span>
    </div>
  </div>
</li>'

# Generate attendees HTML
ATTENDEES='
<div class="attendee-card">
  <div class="attendee-name">Cliente Ejemplo 1</div>
  <div class="attendee-role">Director Tecnico</div>
  <span class="badge badge-confirmed">Confirmado</span>
</div>
<div class="attendee-card">
  <div class="attendee-name">Cliente Ejemplo 2</div>
  <div class="attendee-role">CTO</div>
  <span class="badge badge-confirmed">Confirmado</span>
</div>
<div class="attendee-card">
  <div class="attendee-name">Prospecto Ejemplo 1</div>
  <div class="attendee-role">IT Manager</div>
  <span class="badge badge-pending">Pendiente</span>
</div>
<div class="attendee-card">
  <div class="attendee-name">Prospecto Ejemplo 2</div>
  <div class="attendee-role">CEO</div>
  <span class="badge badge-pending">Pendiente</span>
</div>'

# Read template (prefer standalone for portability)
TEMPLATE_FILE="$TEMPLATE_DIR/event-plan-standalone.html"
if [[ ! -f "$TEMPLATE_FILE" ]]; then
  TEMPLATE_FILE="$TEMPLATE_DIR/event-plan.html"
fi

if [[ ! -f "$TEMPLATE_FILE" ]]; then
  echo "Error: Template not found at $TEMPLATE_DIR" >&2
  exit 1
fi

TEMPLATE=$(cat "$TEMPLATE_FILE")

# Replace placeholders
OUTPUT="$TEMPLATE"
OUTPUT="${OUTPUT//\{\{EVENT_NAME\}\}/$EVENT_NAME}"
OUTPUT="${OUTPUT//\{\{EVENT_DATE\}\}/$EVENT_DATE}"
OUTPUT="${OUTPUT//\{\{EVENT_LOCATION\}\}/$EVENT_LOCATION}"
OUTPUT="${OUTPUT//\{\{EVENT_DESCRIPTION\}\}/$EVENT_DESCRIPTION}"
OUTPUT="${OUTPUT//\{\{DAYS_TO_EVENT\}\}/$DAYS_TO_EVENT}"
OUTPUT="${OUTPUT//\{\{TOTAL_TASKS\}\}/$TOTAL_TASKS}"
OUTPUT="${OUTPUT//\{\{TASKS_COMPLETED\}\}/$TASKS_COMPLETED}"
OUTPUT="${OUTPUT//\{\{PENDING_TASKS\}\}/$PENDING_TASKS}"
OUTPUT="${OUTPUT//\{\{ATTENDEE_COUNT\}\}/$ATTENDEE_COUNT}"
OUTPUT="${OUTPUT//\{\{GENERATION_DATE\}\}/$GENERATION_DATE}"
OUTPUT="${OUTPUT//\{\{YEAR\}\}/$YEAR}"
OUTPUT="${OUTPUT//\{\{OBJECTIVES\}\}/$OBJECTIVES}"
OUTPUT="${OUTPUT//\{\{TIMELINE_ITEMS\}\}/$TIMELINE_ITEMS}"
OUTPUT="${OUTPUT//\{\{BUDGET_ITEMS\}\}/$BUDGET_ITEMS}"
OUTPUT="${OUTPUT//\{\{TASKS\}\}/$TASKS}"
OUTPUT="${OUTPUT//\{\{ATTENDEES\}\}/$ATTENDEES}"

# Write output
echo "$OUTPUT" > "$OUTPUT_FILE"
echo "Generated: $OUTPUT_FILE"
echo "Event: $EVENT_NAME"
echo "Date: $EVENT_DATE"
echo "Days to event: $DAYS_TO_EVENT"
