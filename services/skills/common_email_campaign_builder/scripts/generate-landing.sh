#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# ENTERPRISE DEMO EMAIL CAMPAIGN BUILDER - LANDING PAGE GENERATOR
# Genera landing pages a partir del template con datos personalizados
# ═══════════════════════════════════════════════════════════════════════════

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SKILL_DIR="$(dirname "$SCRIPT_DIR")"
TEMPLATE_DIR="$SKILL_DIR/templates"
ASSETS_DIR="$SKILL_DIR/assets"

# Default values
YEAR=$(date +%Y)
OUTPUT_FILE=""
CONFIG_FILE=""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

show_help() {
    cat << EOF
Uso: $(basename "$0") [OPTIONS]

Genera una landing page personalizada desde el template Enterprise Demo.

Opciones:
  -c, --config FILE      Archivo de configuracion JSON
  -o, --output FILE      Archivo de salida HTML
  -t, --template NAME    Template a usar (default: landing-page)
  -h, --help             Mostrar esta ayuda

Ejemplo:
  $(basename "$0") -c mi-campana.json -o output/mi-landing.html

Formato del archivo de configuracion (JSON):
{
  "PAGE_TITLE": "Mi Campaña",
  "HERO_TITLE": "Titulo Principal",
  "HERO_SUBTITLE": "Subtitulo",
  "HERO_DESCRIPTION": "Descripcion del hero",
  "CTA_PRIMARY": "Texto del boton",
  ...
}

Para ver todas las variables disponibles, revisa el template en:
$TEMPLATE_DIR/landing-page.html
EOF
}

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Parse arguments
TEMPLATE_NAME="landing-page"

while [[ $# -gt 0 ]]; do
    case $1 in
        -c|--config)
            CONFIG_FILE="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        -t|--template)
            TEMPLATE_NAME="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            log_error "Opcion desconocida: $1"
            ;;
    esac
done

# Validate inputs
if [ -z "$CONFIG_FILE" ]; then
    log_error "Se requiere un archivo de configuracion. Usa -c o --config"
fi

if [ ! -f "$CONFIG_FILE" ]; then
    log_error "Archivo de configuracion no encontrado: $CONFIG_FILE"
fi

TEMPLATE_FILE="$TEMPLATE_DIR/${TEMPLATE_NAME}.html"
if [ ! -f "$TEMPLATE_FILE" ]; then
    log_error "Template no encontrado: $TEMPLATE_FILE"
fi

if [ -z "$OUTPUT_FILE" ]; then
    OUTPUT_FILE="output-${TEMPLATE_NAME}-$(date +%Y%m%d-%H%M%S).html"
fi

# Create output directory if needed
OUTPUT_DIR=$(dirname "$OUTPUT_FILE")
if [ ! -d "$OUTPUT_DIR" ] && [ "$OUTPUT_DIR" != "." ]; then
    mkdir -p "$OUTPUT_DIR"
    log_info "Directorio creado: $OUTPUT_DIR"
fi

log_info "Generando landing page..."
log_info "Template: $TEMPLATE_FILE"
log_info "Config: $CONFIG_FILE"

# Read template
CONTENT=$(cat "$TEMPLATE_FILE")

# Add default YEAR
CONTENT="${CONTENT//\{\{YEAR\}\}/$YEAR}"

# Check if jq is available
if ! command -v jq &> /dev/null; then
    log_error "jq no esta instalado. Instala con: apt-get install jq"
fi

# Read config and replace placeholders
while IFS="=" read -r key value; do
    # Remove quotes from value
    value="${value%\"}"
    value="${value#\"}"
    
    # Replace placeholder in content
    CONTENT="${CONTENT//\{\{$key\}\}/$value}"
done < <(jq -r 'to_entries[] | "\(.key)=\(.value)"' "$CONFIG_FILE")

# Check for remaining placeholders
REMAINING=$(echo "$CONTENT" | grep -oE '\{\{[A-Z_]+\}\}' | sort -u || true)
if [ -n "$REMAINING" ]; then
    log_warn "Placeholders no reemplazados:"
    echo "$REMAINING" | while read -r placeholder; do
        echo "  - $placeholder"
    done
fi

# Write output
echo "$CONTENT" > "$OUTPUT_FILE"

log_info "Landing page generada: $OUTPUT_FILE"

# Show file size
SIZE=$(wc -c < "$OUTPUT_FILE")
log_info "Tamano del archivo: $(numfmt --to=iec-i --suffix=B $SIZE 2>/dev/null || echo "${SIZE} bytes")"

exit 0
