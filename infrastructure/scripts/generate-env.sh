#!/usr/bin/env bash
# SystemPrompt Config Builder
# Merges YAML configurations and generates .env files

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$INFRA_DIR")"
ENVIRONMENTS_DIR="$INFRA_DIR/environments"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1" >&2
}

usage() {
    cat << EOF
SystemPrompt Config Builder

Usage: $0 [OPTIONS]

Options:
    -e, --environment ENV    Environment to build (local, docker, production)
    -o, --output FILE        Output .env file path (default: .env.<environment>)
    -v, --validate           Validate configuration only
    -h, --help               Show this help message

Examples:
    $0 --environment docker-dev
    $0 --environment docker-dev --output .env.docker-dev
    $0 --environment production --validate

EOF
    exit 1
}

# Parse command line arguments
ENVIRONMENT=""
OUTPUT_FILE=""
VALIDATE_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -e|--environment)
            ENVIRONMENT="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        -v|--validate)
            VALIDATE_ONLY=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            ;;
    esac
done

# Validate environment argument
if [ -z "$ENVIRONMENT" ]; then
    log_error "Environment is required"
    usage
fi

# Set default output file if not specified
if [ -z "$OUTPUT_FILE" ]; then
    OUTPUT_FILE="$PROJECT_ROOT/.env.$ENVIRONMENT"
fi

BASE_CONFIG="$ENVIRONMENTS_DIR/base.yml"
ENV_CONFIG="$ENVIRONMENTS_DIR/$ENVIRONMENT/config.yml"

log_info "Building configuration for environment: $ENVIRONMENT"

# Source .env.secrets if it exists to make API keys available for variable resolution
SECRETS_FILE="$PROJECT_ROOT/.env.secrets"
if [ -f "$SECRETS_FILE" ]; then
    log_info "Loading secrets from: $SECRETS_FILE"
    set -a  # Automatically export all variables
    source "$SECRETS_FILE"
    set +a
    log_success "Secrets loaded and available for variable resolution"
else
    log_warning "No .env.secrets file found at: $SECRETS_FILE"
    log_warning "API keys and secrets will not be resolved"
fi

# Check if base config exists
if [ ! -f "$BASE_CONFIG" ]; then
    log_error "Base config not found: $BASE_CONFIG"
    exit 1
fi

# Check if environment config exists
if [ ! -f "$ENV_CONFIG" ]; then
    log_error "Environment config not found: $ENV_CONFIG"
    exit 1
fi

# Check for yq and set path
YQ_BIN=""
if command -v yq &> /dev/null; then
    YQ_BIN="yq"
elif [ -x "$HOME/.local/bin/yq" ]; then
    YQ_BIN="$HOME/.local/bin/yq"
else
    log_error "yq is not installed. Please install it from https://github.com/mikefarah/yq"
    exit 1
fi

# Function to extract variable expansion with proper brace nesting
extract_var_expansion() {
    local str="$1"
    local pos="$2"

    # Parse ${VAR} or ${VAR:-default} with nested brace support
    if [[ "${str:pos:2}" == '${' ]]; then
        local start=$pos
        ((pos += 2))
        local brace_count=1
        local var_name=""
        local has_default=false
        local default_value=""
        local in_default=false

        while [ $pos -lt ${#str} ] && [ $brace_count -gt 0 ]; do
            local char="${str:pos:1}"

            if [ "$char" = '{' ]; then
                ((brace_count++))
                [ "$in_default" = true ] && default_value+="$char"
            elif [ "$char" = '}' ]; then
                ((brace_count--))
                [ $brace_count -gt 0 ] && [ "$in_default" = true ] && default_value+="$char"
            elif [ "$in_default" = true ]; then
                default_value+="$char"
            elif [ "$char" = ':' ] && [ "${str:pos+1:1}" = '-' ]; then
                has_default=true
                in_default=true
                ((pos++))  # Skip the '-'
            else
                var_name+="$char"
            fi

            ((pos++))
        done

        local full_match="${str:start:pos-start}"
        echo "$var_name|$default_value|$full_match"
        return 0
    fi

    return 1
}

# Function to resolve environment variables in a value (supports nested expansion)
resolve_env_var() {
    local value="$1"
    local max_iterations=10
    local iteration=0

    while [ $iteration -lt $max_iterations ]; do
        # Find first ${...}
        if [[ ! "$value" =~ \$\{ ]]; then
            break
        fi

        local pos=$(expr index "$value" '$')
        ((pos--))

        local extraction
        if extraction=$(extract_var_expansion "$value" "$pos"); then
            IFS='|' read -r var_name default_value full_match <<< "$extraction"

            # Resolve the variable
            if [ -n "${!var_name+x}" ]; then
                local var_value="${!var_name}"
                value="${value//"$full_match"/"$var_value"}"
            elif [ -n "$default_value" ]; then
                value="${value//"$full_match"/"$default_value"}"
            else
                # Can't resolve, stop trying
                break
            fi
        else
            # No valid expansion found
            break
        fi

        ((iteration++))
    done

    echo "$value"
}

# Function to flatten YAML and convert to env format
yaml_to_env() {
    local yaml_file="$1"

    # Use yq to convert YAML to flat key=value format
    # Process each line from yq output
    "$YQ_BIN" eval '.. | select(tag != "!!map" and tag != "!!seq") | {(path | join("_") | upcase): .} | to_entries | .[] | .key + "=" + .value' "$yaml_file" 2>/dev/null | while IFS='=' read -r key value; do
        # Resolve environment variables in the value
        resolved_value=$(resolve_env_var "$value")

        # Strip surrounding quotes if present (yq may include them from YAML)
        if [[ "$resolved_value" =~ ^\"(.*)\"$ ]]; then
            resolved_value="${BASH_REMATCH[1]}"
        fi

        # Quote value if it contains spaces or special characters
        if [[ "$resolved_value" =~ [[:space:]] ]]; then
            echo "${key}=\"${resolved_value}\""
        else
            echo "${key}=${resolved_value}"
        fi
    done
}

# Merge base and environment configs with multi-pass variable resolution
merge_configs() {
    local temp_merged=$(mktemp)

    log_info "Parsing base config: $BASE_CONFIG" >&2
    yaml_to_env "$BASE_CONFIG" "" > "$temp_merged"

    log_info "Parsing environment config: $ENV_CONFIG" >&2
    local temp_env=$(mktemp)
    yaml_to_env "$ENV_CONFIG" "" > "$temp_env"

    # Merge: environment overrides base
    declare -A config_map

    # Load base config
    while IFS='=' read -r key value; do
        config_map["$key"]="$value"
    done < "$temp_merged"

    # Override with environment config
    while IFS='=' read -r key value; do
        config_map["$key"]="$value"
    done < "$temp_env"

    # Multi-pass resolution: resolve variables that reference other variables in the map
    local max_passes=5
    local pass=1
    while [ $pass -le $max_passes ]; do
        local changes_made=false

        for key in "${!config_map[@]}"; do
            local value="${config_map[$key]}"

            # If value contains variable references, try to resolve them
            if [[ "$value" =~ \$\{([^:}]+) ]]; then
                # Export current config as env vars for resolution
                for export_key in "${!config_map[@]}"; do
                    export "$export_key"="${config_map[$export_key]}"
                done

                # Resolve with current environment
                local resolved=$(resolve_env_var "$value")

                if [ "$resolved" != "$value" ]; then
                    config_map["$key"]="$resolved"
                    changes_made=true
                fi
            fi
        done

        # Stop if no changes were made
        [ "$changes_made" = false ] && break

        ((pass++))
    done

    # Write merged config
    local temp_output=$(mktemp)
    for key in "${!config_map[@]}"; do
        local value="${config_map[$key]}"

        # Strip surrounding quotes if present
        if [[ "$value" =~ ^\"(.*)\"$ ]]; then
            value="${BASH_REMATCH[1]}"
        fi

        # Quote value if it contains spaces or special characters
        if [[ "$value" =~ [[:space:]] ]]; then
            echo "$key=\"$value\"" >> "$temp_output"
        else
            echo "$key=$value" >> "$temp_output"
        fi
    done

    # Sort output for consistency
    sort "$temp_output"

    rm -f "$temp_merged" "$temp_env" "$temp_output"
}

# Validate required fields
validate_config() {
    local config_content="$1"

    log_info "Validating configuration..."

    # Check for required flat key names (YAML now uses flat format directly)
    local required_vars=(
        "SYSTEM_PATH"
        "DATABASE_URL"
        "CARGO_TARGET_DIR"
        "AI_CONFIG_PATH"
        "CONTENT_CONFIG_PATH"
        "GEOIP_DATABASE_PATH"
        "STORAGE_PATH"
    )

    local missing_vars=()

    for var in "${required_vars[@]}"; do
        # Check if variable exists and doesn't contain unresolved placeholder
        if ! echo "$config_content" | grep -q "^${var}=" || \
           echo "$config_content" | grep "^${var}=" | grep -q '\${'; then
            missing_vars+=("$var")
        fi
    done

    if [ ${#missing_vars[@]} -gt 0 ]; then
        log_error "Missing or unresolved required variables:"
        for var in "${missing_vars[@]}"; do
            log_error "  - $var"
        done
        return 1
    fi

    log_success "Configuration validation passed"
    return 0
}

# Main execution
main() {
    local merged_config
    merged_config=$(merge_configs)

    if [ "$VALIDATE_ONLY" = true ]; then
        validate_config "$merged_config"
        exit $?
    fi

    # Validate before writing
    if ! validate_config "$merged_config"; then
        log_error "Configuration validation failed. .env file not created."
        exit 1
    fi

    # Write to output file
    echo "$merged_config" > "$OUTPUT_FILE"
    log_success "Configuration written to: $OUTPUT_FILE"

    # Generate web .env file with VITE_* variables
    local web_env_file="$PROJECT_ROOT/core/web/.env.$ENVIRONMENT"
    local vite_config=$(echo "$merged_config" | grep "^VITE_")

    if [ -n "$vite_config" ]; then
        echo "$vite_config" > "$web_env_file"
        log_success "Web configuration written to: $web_env_file"

        # Create symlink .env -> .env.local so dotenv can find it (for sitemap generation script)
        if [ "$ENVIRONMENT" = "local" ]; then
            local web_dir="$PROJECT_ROOT/core/web"
            local env_link="$web_dir/.env"
            if [ -L "$env_link" ] && [ "$(readlink "$env_link")" = ".env.local" ]; then
                log_success "Web .env symlink already points to .env.local"
            else
                rm -f "$env_link"
                ln -s ".env.local" "$env_link"
                log_success "Created symlink: $env_link -> .env.local (for dotenv in build scripts)"
            fi
        fi

        # For docker-dev environment, also create .env.docker for Vite --mode docker compatibility
        if [ "$ENVIRONMENT" = "docker-dev" ]; then
            local vite_docker_file="$PROJECT_ROOT/core/web/.env.docker"
            echo "$vite_config" > "$vite_docker_file"
            log_success "Web configuration also written to: $vite_docker_file (for Vite --mode docker)"
        fi
    else
        log_warning "No VITE_* variables found in configuration"
    fi

    # Show summary
    local var_count=$(echo "$merged_config" | wc -l | tr -d ' ')
    log_info "Generated $var_count environment variables"
}

main
