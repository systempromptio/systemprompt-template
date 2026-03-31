#!/usr/bin/env bash
# =============================================================================
# fetch_crm_data.sh - Extraccion de datos CRM de Odoo
# =============================================================================
# Conecta a Odoo via odoo-pilot y extrae datos de oportunidades (crm.lead),
# etapas (crm.stage) y comerciales (res.users) para analisis de salud del CRM.
#
# Uso:
#   ./fetch_crm_data.sh [--pilot-dir /path/to/odoo-pilot] [--months 3]
#
# Requisitos:
#   - Variables de entorno: ODOO_URL, ODOO_DB, ODOO_KEY, ODOO_USER
#   - odoo-pilot scripts accesibles (auth.sh, search_records.sh)
#   - Node.js disponible (para ensamblado JSON)
#
# Output: JSON estructurado a stdout
# Errores: stderr con mensajes descriptivos
# =============================================================================

set -euo pipefail

# --- Colores para stderr ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()  { echo -e "${BLUE}[INFO]${NC} $*" >&2; }
log_ok()    { echo -e "${GREEN}[OK]${NC} $*" >&2; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $*" >&2; }
log_error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }

# --- Parametros ---
PILOT_DIR=""
MONTHS=12

while [[ $# -gt 0 ]]; do
  case $1 in
    --pilot-dir)
      PILOT_DIR="$2"
      shift 2
      ;;
    --months)
      MONTHS="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

# --- Directorio temporal para datos ---
TMPDIR_DATA=$(mktemp -d)
trap 'rm -rf "$TMPDIR_DATA"' EXIT

# --- Localizar odoo-pilot ---
if [[ -z "$PILOT_DIR" ]]; then
  SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  POSSIBLE_DIRS=(
    "$SCRIPT_DIR/../../../../ai-claude-common/skills/odoo-pilot/scripts"
    "$HOME/miproyecto/ai-claude-common/skills/odoo-pilot/scripts"
  )
  for dir in "${POSSIBLE_DIRS[@]}"; do
    if [[ -f "$dir/auth.sh" ]]; then
      PILOT_DIR="$dir"
      break
    fi
  done
fi

if [[ -z "$PILOT_DIR" || ! -f "$PILOT_DIR/auth.sh" ]]; then
  log_error "No se encontro odoo-pilot. Usa --pilot-dir para indicar la ruta."
  exit 1
fi

log_info "Usando odoo-pilot en: $PILOT_DIR"

# --- Validar variables de entorno ---
for var in ODOO_URL ODOO_DB ODOO_KEY; do
  if [[ -z "${!var:-}" ]]; then
    log_error "Variable de entorno requerida: $var"
    exit 1
  fi
done

# --- Autenticar ---
log_info "Autenticando contra $ODOO_URL (DB: $ODOO_DB)..."
eval "$("$PILOT_DIR/auth.sh")"

if [[ -z "${ODOO_UID:-}" ]]; then
  log_error "Fallo la autenticacion. Verifica credenciales."
  exit 1
fi

log_ok "Autenticado como UID=$ODOO_UID (protocolo: ${ODOO_PROTOCOL:-unknown})"

# --- Calcular fecha limite (N meses atras) ---
if [[ "$(uname)" == "Darwin" ]]; then
  DATE_LIMIT=$(date -v-${MONTHS}m +%Y-%m-%d)
else
  DATE_LIMIT=$(date -d "-${MONTHS} months" +%Y-%m-%d)
fi

log_info "Extrayendo datos de los ultimos $MONTHS meses (desde $DATE_LIMIT)"

# --- Funcion auxiliar para busquedas ---
search_to_file() {
  local model="$1"
  local domain="$2"
  local fields="$3"
  local limit="${4:-500}"
  local outfile="$5"

  local result
  result=$("$PILOT_DIR/search_records.sh" "$model" "$domain" "$fields" "$limit" 2>/dev/null) || true

  if [[ -z "$result" || "$result" == "null" ]]; then
    echo "[]" > "$outfile"
  else
    echo "$result" > "$outfile"
  fi
}

# --- Campos comunes de crm.lead ---
LEAD_FIELDS='["name","partner_id","user_id","team_id","stage_id","probability","expected_revenue","date_deadline","create_date","write_date","date_last_stage_update","date_closed","priority","x_studio_fecha_demo","x_studio_last_update","active"]'

# --- 1. Oportunidades activas ---
log_info "Buscando oportunidades activas..."

search_to_file \
  "crm.lead" \
  '[["type","=","opportunity"],["company_id","=",1]]' \
  "$LEAD_FIELDS" \
  2000 \
  "$TMPDIR_DATA/leads_active.json"

ACTIVE_COUNT=$(node -e "
  const d = JSON.parse(require('fs').readFileSync('$TMPDIR_DATA/leads_active.json','utf8'));
  console.log(Array.isArray(d) ? d.length : 0);
" 2>/dev/null || echo "0")

log_ok "Oportunidades activas: $ACTIVE_COUNT"

# --- 2. Oportunidades ganadas (ultimos N meses) ---
log_info "Buscando oportunidades ganadas (ultimos $MONTHS meses)..."

search_to_file \
  "crm.lead" \
  "[[\"type\",\"=\",\"opportunity\"],[\"company_id\",\"=\",1],[\"stage_id\",\"=\",4],[\"date_closed\",\">=\",\"$DATE_LIMIT\"]]" \
  "$LEAD_FIELDS" \
  2000 \
  "$TMPDIR_DATA/leads_won.json"

WON_COUNT=$(node -e "
  const d = JSON.parse(require('fs').readFileSync('$TMPDIR_DATA/leads_won.json','utf8'));
  console.log(Array.isArray(d) ? d.length : 0);
" 2>/dev/null || echo "0")

log_ok "Oportunidades ganadas: $WON_COUNT"

# --- 3. Oportunidades perdidas/congeladas (ultimos N meses) ---
# En Enterprise Demo, las perdidas estan en la etapa "Congelado" (id=5).
# Si se configura otro lost_stage_id, se puede cambiar aqui.
log_info "Buscando oportunidades perdidas/congeladas (ultimos $MONTHS meses)..."

LOST_AVAILABLE=true

search_to_file \
  "crm.lead" \
  "[[\"type\",\"=\",\"opportunity\"],[\"company_id\",\"=\",1],[\"stage_id\",\"=\",5],[\"date_last_stage_update\",\">=\",\"$DATE_LIMIT\"]]" \
  "$LEAD_FIELDS" \
  2000 \
  "$TMPDIR_DATA/leads_lost.json"

LOST_COUNT=$(node -e "
  const d = JSON.parse(require('fs').readFileSync('$TMPDIR_DATA/leads_lost.json','utf8'));
  console.log(Array.isArray(d) ? d.length : 0);
" 2>/dev/null || echo "0")

if [[ "$LOST_COUNT" -eq 0 ]]; then
  log_warn "No se encontraron oportunidades en etapa Congelado."
  LOST_AVAILABLE=false
fi

log_ok "Oportunidades perdidas: $LOST_COUNT"

# --- 3b. TODAS las oportunidades (para win rate) ---
# Incluye activas Y archivadas (active=True OR active!=True)
# SIN filtro de create_date para obtener el total real de oportunidades asignadas
log_info "Buscando TODAS las oportunidades (activas + archivadas, sin limite de fecha)..."

search_to_file \
  "crm.lead" \
  "[\"|\",[\"active\",\"=\",true],[\"active\",\"!=\",true],[\"type\",\"=\",\"opportunity\"],[\"company_id\",\"=\",1]]" \
  "$LEAD_FIELDS" \
  10000 \
  "$TMPDIR_DATA/leads_all.json"

ALL_COUNT=$(node -e "
  const d = JSON.parse(require('fs').readFileSync('$TMPDIR_DATA/leads_all.json','utf8'));
  console.log(Array.isArray(d) ? d.length : 0);
" 2>/dev/null || echo "0")

log_ok "Oportunidades totales (activas+archivadas): $ALL_COUNT"

# --- 4. Etapas del CRM ---
log_info "Buscando etapas del CRM..."

search_to_file \
  "crm.stage" \
  '[]' \
  '["name","sequence","is_won","fold"]' \
  50 \
  "$TMPDIR_DATA/stages.json"

STAGES_COUNT=$(node -e "
  const d = JSON.parse(require('fs').readFileSync('$TMPDIR_DATA/stages.json','utf8'));
  console.log(Array.isArray(d) ? d.length : 0);
" 2>/dev/null || echo "0")

log_ok "Etapas CRM: $STAGES_COUNT"

# --- 5. Pedidos de venta confirmados (sale.order) ---
log_info "Buscando pedidos de venta confirmados (ultimos $MONTHS meses)..."

SALE_ORDER_FIELDS='["name","partner_id","user_id","date_order","amount_untaxed","state"]'

search_to_file \
  "sale.order" \
  "[[\"state\",\"in\",[\"sale\",\"done\"]],[\"company_id\",\"=\",1],[\"date_order\",\">=\",\"$DATE_LIMIT\"]]" \
  "$SALE_ORDER_FIELDS" \
  5000 \
  "$TMPDIR_DATA/sale_orders.json"

SALE_COUNT=$(node -e "
  const d = JSON.parse(require('fs').readFileSync('$TMPDIR_DATA/sale_orders.json','utf8'));
  console.log(Array.isArray(d) ? d.length : 0);
" 2>/dev/null || echo "0")

log_ok "Pedidos de venta confirmados: $SALE_COUNT"

# --- 6. Comerciales (extraer IDs unicos de oportunidades) ---
log_info "Extrayendo comerciales..."

# Extraer user_ids unicos de todas las oportunidades
USER_IDS=$(node -e "
  const fs = require('fs');
  const active = JSON.parse(fs.readFileSync('$TMPDIR_DATA/leads_active.json','utf8'));
  const won = JSON.parse(fs.readFileSync('$TMPDIR_DATA/leads_won.json','utf8'));
  const lost = JSON.parse(fs.readFileSync('$TMPDIR_DATA/leads_lost.json','utf8'));
  const all = [...(Array.isArray(active)?active:[]), ...(Array.isArray(won)?won:[]), ...(Array.isArray(lost)?lost:[])];
  const ids = new Set();
  all.forEach(l => {
    const uid = l.user_id;
    if (uid) {
      const id = Array.isArray(uid) ? uid[0] : uid;
      if (id) ids.add(id);
    }
  });
  console.log(JSON.stringify([...ids]));
" 2>/dev/null || echo "[]")

if [[ "$USER_IDS" != "[]" ]]; then
  search_to_file \
    "res.users" \
    "[[\"id\",\"in\",$USER_IDS]]" \
    '["name","login"]' \
    100 \
    "$TMPDIR_DATA/users.json"
else
  echo "[]" > "$TMPDIR_DATA/users.json"
fi

USERS_COUNT=$(node -e "
  const d = JSON.parse(require('fs').readFileSync('$TMPDIR_DATA/users.json','utf8'));
  console.log(Array.isArray(d) ? d.length : 0);
" 2>/dev/null || echo "0")

log_ok "Comerciales encontrados: $USERS_COUNT"

# --- 6. Ensamblar JSON unificado ---
log_info "Ensamblando datos..."

node -e "
const fs = require('fs');
const dir = '$TMPDIR_DATA';

const active = JSON.parse(fs.readFileSync(dir + '/leads_active.json', 'utf8'));
const won = JSON.parse(fs.readFileSync(dir + '/leads_won.json', 'utf8'));
const lost = JSON.parse(fs.readFileSync(dir + '/leads_lost.json', 'utf8'));
const allLeads = JSON.parse(fs.readFileSync(dir + '/leads_all.json', 'utf8'));
const stages = JSON.parse(fs.readFileSync(dir + '/stages.json', 'utf8'));
const users = JSON.parse(fs.readFileSync(dir + '/users.json', 'utf8'));
const saleOrders = JSON.parse(fs.readFileSync(dir + '/sale_orders.json', 'utf8'));

const activeLeads = Array.isArray(active) ? active : [];
const wonLeads = Array.isArray(won) ? won : [];
const lostLeads = Array.isArray(lost) ? lost : [];
const allLeadsList = Array.isArray(allLeads) ? allLeads : [];
const stageList = Array.isArray(stages) ? stages : [];
const userList = Array.isArray(users) ? users : [];
const saleOrderList = Array.isArray(saleOrders) ? saleOrders : [];

// Detectar si el campo x_studio_fecha_demo existe
const hasDemoField = activeLeads.some(l => 'x_studio_fecha_demo' in l);

// Pipeline total
const totalRevenue = activeLeads.reduce((s, l) => s + (l.expected_revenue || 0), 0);

// Comerciales unicos
const uniqueUsers = new Set();
activeLeads.forEach(l => {
  const uid = l.user_id;
  if (uid) uniqueUsers.add(Array.isArray(uid) ? uid[0] : uid);
});

const output = {
  metadata: {
    extracted_at: new Date().toISOString(),
    odoo_url: process.env.ODOO_URL,
    odoo_db: process.env.ODOO_DB,
    protocol: process.env.ODOO_PROTOCOL || 'unknown',
    months_lookback: $MONTHS,
    date_limit: '$DATE_LIMIT'
  },
  leads_active: activeLeads,
  leads_won: wonLeads,
  leads_lost: lostLeads,
  leads_all: allLeadsList,
  sale_orders: saleOrderList,
  stages: stageList,
  users: userList,
  summary: {
    active_count: activeLeads.length,
    won_count: wonLeads.length,
    lost_count: lostLeads.length,
    all_count: allLeadsList.length,
    lost_available: $LOST_AVAILABLE,
    stages_count: stageList.length,
    users_count: userList.length,
    unique_salespeople: uniqueUsers.size,
    total_pipeline_revenue: Math.round(totalRevenue * 100) / 100,
    has_demo_field: hasDemoField,
    sale_orders_count: saleOrderList.length
  }
};

console.log(JSON.stringify(output, null, 2));
"

log_ok "Datos extraidos correctamente"
log_info "Resumen: $ACTIVE_COUNT activas, $WON_COUNT ganadas, $LOST_COUNT perdidas, $ALL_COUNT totales, $SALE_COUNT pedidos venta, $STAGES_COUNT etapas, $USERS_COUNT comerciales"
log_info "Para calcular metricas: cat output.json | python3 scripts/calculate_crm_health.py"
