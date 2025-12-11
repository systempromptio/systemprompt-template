#!/bin/bash
# SystemPrompt Build System - Unified build script
# Usage: ./scripts/build.sh [debug|release] [--web] [--docker]
#
# Modes:
#   debug (default) - Debug build for local development
#   release - Optimized release build for Docker/GCP
#
# Options:
#   --web - Build web assets (React app)
#   --docker - Build Docker image (requires release binaries)

set -e

# Configuration
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

# Source environment files for sqlx compile-time checks
if [ -f "$REPO_ROOT/.env.local" ]; then
    set -a
    source "$REPO_ROOT/.env.local"
    set +a
fi
if [ -f "$REPO_ROOT/.env.secrets" ]; then
    set -a
    source "$REPO_ROOT/.env.secrets"
    set +a
fi
MODE="${1:-debug}"
BUILD_WEB=false
BUILD_DOCKER=false
ENVIRONMENT="${ENVIRONMENT:-docker-dev}"  # Default to docker-dev for Docker builds, can override with ENVIRONMENT=production
DOCKER_IMAGE="systemprompt-blog:latest"
DOCKER_IMAGE_API="systemprompt-blog-api:latest"
DOCKER_IMAGE_WEB="systemprompt-blog-web:latest"
GCR_IMAGE="gcr.io/vast-nectar-453310-d7/systemprompt-blog:latest"
GCR_IMAGE_API="gcr.io/vast-nectar-453310-d7/systemprompt-blog-api:latest"
GCR_IMAGE_WEB="gcr.io/vast-nectar-453310-d7/systemprompt-blog-web:latest"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse additional arguments
for arg in "${@:2}"; do
    case "$arg" in
        --web) BUILD_WEB=true ;;
        --docker) BUILD_DOCKER=true ;;
        --env)
            shift
            ENVIRONMENT="$1"
            ;;
        --env=*)
            ENVIRONMENT="${arg#*=}"
            ;;
        *) echo "Unknown option: $arg"; exit 1 ;;
    esac
done

# Validate mode
if [ "$MODE" != "debug" ] && [ "$MODE" != "release" ]; then
    echo -e "${RED}Error: MODE must be 'debug' or 'release', got: $MODE${NC}"
    exit 1
fi

# Set build flags
if [ "$MODE" = "release" ]; then
    BUILD_FLAG="--release"
    TARGET_DIR="release"
else
    BUILD_FLAG=""
    TARGET_DIR="debug"
fi

# If building Docker, require release mode
if [ "$BUILD_DOCKER" = true ] && [ "$MODE" != "release" ]; then
    echo -e "${RED}Error: Docker builds require release mode${NC}"
    echo -e "${YELLOW}Use: ./scripts/build.sh release --docker${NC}"
    exit 1
fi

echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  SystemPrompt Build System${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo -e "  Mode: ${YELLOW}${MODE}${NC}"
echo -e "  Build web: ${YELLOW}${BUILD_WEB}${NC}"
echo -e "  Build Docker: ${YELLOW}${BUILD_DOCKER}${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo ""

# Change to project root
cd "$REPO_ROOT"

# ──────────────────────────────────────────────────────────────
# Note: Prerendering and sitemap generation now run as scheduled jobs
# when the API starts, ensuring they use fresh database content
# ──────────────────────────────────────────────────────────────

# ──────────────────────────────────────────────────────────────
# Step 1: Build Rust binaries
# ──────────────────────────────────────────────────────────────

echo -e "${BLUE}Step 1: Building Rust binaries (${MODE})...${NC}"
echo ""

# Discover and build workspace members
WORKSPACE_MEMBERS=()

# Parse root workspace Cargo.toml
if [ -f "Cargo.toml" ]; then
    while IFS= read -r line; do
        member=$(echo "$line" | sed 's/[",]//g' | xargs)
        if [ -n "$member" ] && [ -d "$member" ]; then
            WORKSPACE_MEMBERS+=("$member/Cargo.toml")
        fi
    done < <(awk '/^\[workspace\]/,/^members[[:space:]]*=/{flag=1; next} /^\[/{flag=0} flag && /[a-zA-Z]/ {print}' Cargo.toml | grep -v "members" | grep -v "^\[" | grep -v "^#")
fi

# Build core workspace (submodule) - builds the systemprompt binary
if [ -f "core/Cargo.toml" ]; then
    echo -e "${YELLOW}→ Building: ${GREEN}core (systemprompt binary)${NC}"

    BUILD_OUTPUT=$(cargo build $BUILD_FLAG --manifest-path="core/Cargo.toml" --bins 2>&1)
    BUILD_EXIT_CODE=$?

    echo "$BUILD_OUTPUT" | grep -E "^(Compiling|Finished)" || true

    if [ $BUILD_EXIT_CODE -eq 0 ]; then
        echo -e "  ${GREEN}✓ Success${NC}"
    else
        echo -e "  ${RED}✗ Failed${NC}"
        echo ""
        echo -e "${RED}═══════════════════════════════════════════════════${NC}"
        echo -e "${RED}  Cargo build failed: core${NC}"
        echo -e "${RED}═══════════════════════════════════════════════════${NC}"
        echo ""
        echo -e "${YELLOW}Error summary:${NC}"
        echo "$BUILD_OUTPUT" | grep -E "^error(\[|:)" | head -10 || echo "  (error details below)"
        echo ""
        echo -e "${YELLOW}Full build output:${NC}"
        echo ""
        echo "$BUILD_OUTPUT"
        echo ""
        exit 1
    fi
fi

echo -e "${BLUE}Found ${#WORKSPACE_MEMBERS[@]} workspace member(s)${NC}"
echo ""

TOTAL=0
SUCCESS=0
FAILED=0
FAILED_TARGETS=()
declare -A BINARIES

for manifest in "${WORKSPACE_MEMBERS[@]}"; do
    if [ -f "$manifest" ]; then
        TOTAL=$((TOTAL + 1))
        manifest_name="$(basename "$(dirname "$manifest")")"

        echo -e "${YELLOW}→ Building: ${GREEN}${manifest_name}${NC}"

        # Run cargo build and capture output and exit code
        # Include --bins flag to ensure binary targets (like ingest, export) are built
        BUILD_OUTPUT=$(cargo build $BUILD_FLAG --manifest-path="$manifest" --bins 2>&1)
        BUILD_EXIT_CODE=$?

        # Show progress lines
        echo "$BUILD_OUTPUT" | grep -E "^(Compiling|Finished)" || true

        if [ $BUILD_EXIT_CODE -eq 0 ]; then
            SUCCESS=$((SUCCESS + 1))
            echo -e "  ${GREEN}✓ Success${NC}"
        else
            echo -e "  ${RED}✗ Failed${NC}"
            echo ""
            echo -e "${RED}═══════════════════════════════════════════════════${NC}"
            echo -e "${RED}  Cargo build failed: ${manifest_name}${NC}"
            echo -e "${RED}═══════════════════════════════════════════════════${NC}"
            echo ""

            # Extract and highlight error sections
            echo -e "${YELLOW}Error summary:${NC}"
            echo "$BUILD_OUTPUT" | grep -E "^error(\[|:)" | head -10 || echo "  (error details below)"
            echo ""

            # Show full output
            echo -e "${YELLOW}Full build output:${NC}"
            echo ""
            echo "$BUILD_OUTPUT"
            echo ""
            echo -e "${RED}Cargo build failed. See error details above.${NC}"
            echo -e "${YELLOW}Common issues:${NC}"
            echo "  • Check Rust compiler version: rustc --version"
            echo "  • Try cleaning build artifacts: cargo clean"
            echo "  • Check for circular dependencies or version conflicts"
            echo ""
            exit 1
        fi
    fi
done

echo ""
echo -e "${GREEN}✓ All binaries built successfully${NC}"
echo ""

# ──────────────────────────────────────────────────────────────
# Step 2: Build web assets (optional)
# ──────────────────────────────────────────────────────────────

if [ "$BUILD_WEB" = true ]; then
    echo -e "${BLUE}Step 2: Building web assets...${NC}"
    echo ""

    if [ ! -d "core/web" ]; then
        echo -e "${RED}Error: Web directory not found: core/web${NC}"
        exit 1
    fi

    # Sync blog images from canonical source to public folder
    echo -e "${CYAN}→${NC} Syncing blog images to public folder..."
    mkdir -p core/web/public/images/blog
    if [ -d "crates/services/web/assets/images/blog" ]; then
        cp -r crates/services/web/assets/images/blog/* core/web/public/images/blog/ 2>/dev/null || true
        echo -e "${GREEN}✓${NC} Blog images synced"
    fi

    cd core/web

    # Set SITEMAP_BASE_URL for production builds to ensure correct URLs in sitemap
    # Priority: explicit SITEMAP_BASE_URL > extract from .env GSC_PROPERTY_URL
    if [ "$MODE" = "release" ] && [ -z "$SITEMAP_BASE_URL" ]; then
        # Try to extract domain from GSC_PROPERTY_URL in .env files (format: sc-domain:tyingshoelaces.com)
        for env_file in "$REPO_ROOT/.env.production" "$REPO_ROOT/.env.docker" "$REPO_ROOT/.env"; do
            if [ -f "$env_file" ] && grep -q "GSC_PROPERTY_URL=.*sc-domain:" "$env_file"; then
                GSC_VALUE=$(grep "GSC_PROPERTY_URL=" "$env_file" | head -1 | cut -d'=' -f2- | tr -d ' "')
                DOMAIN_EXTRACTED=$(echo "$GSC_VALUE" | sed 's/sc-domain:\([^"]*\).*/\1/' | tr -d ' "')
                if [ -n "$DOMAIN_EXTRACTED" ] && [ "$DOMAIN_EXTRACTED" != "$GSC_VALUE" ]; then
                    export SITEMAP_BASE_URL="https://${DOMAIN_EXTRACTED}"
                    break
                fi
            fi
        done
    fi

    if [ -n "$SITEMAP_BASE_URL" ]; then
        echo -e "${CYAN}→${NC} Sitemap base URL: $SITEMAP_BASE_URL"
    fi

    # Set VITE_API_URL for production builds so sitemap generation can fetch from live database
    # This allows sitemap generation to pull content from production database instead of filesystem
    if [ "$MODE" = "release" ] && [ -z "$VITE_API_URL" ]; then
        # Try to extract API URL from .env.production
        if [ -f "core/web/.env.production" ]; then
            API_HOST=$(grep "VITE_API_BASE_HOST=" "core/web/.env.production" | cut -d'=' -f2- | tr -d ' "')
            if [ -n "$API_HOST" ]; then
                export VITE_API_URL="${API_HOST}/api/v1"
                echo -e "${CYAN}→${NC} Sitemap will fetch from production API: $VITE_API_URL"
            fi
        fi
    fi

    # For Docker builds, use appropriate npm script based on environment
    # Vite embeds environment variables at build time from mode-specific files
    # .env file priority: .env.{mode} > .env.local > .env
    NPM_SCRIPT="build:full"
    if [ "$BUILD_DOCKER" = true ]; then
        # Use environment-specific build script
        case "$ENVIRONMENT" in
            production)
                NPM_SCRIPT="build:prod:full"
                echo -e "${CYAN}→${NC} Building for production (uses --mode production, loads .env.production)"
                ;;
            docker-dev)
                NPM_SCRIPT="build:docker:full"
                echo -e "${CYAN}→${NC} Building for docker-dev (uses --mode docker, loads .env.docker-dev)"
                ;;
            docker)
                # Keep backward compatibility
                NPM_SCRIPT="build:docker:full"
                echo -e "${YELLOW}⚠${NC} 'docker' environment is deprecated, use 'docker-dev' instead"
                echo -e "${CYAN}→${NC} Building for docker (uses --mode docker, loads .env.docker-dev)"
                ;;
            *)
                echo -e "${YELLOW}⚠${NC} Unknown environment: $ENVIRONMENT, defaulting to production"
                NPM_SCRIPT="build:prod:full"
                ;;
        esac
        echo -e "${CYAN}→${NC} VITE variables will be embedded from core/web/.env.$ENVIRONMENT"
    else
        # Default build for local development (uses --mode development)
        echo -e "${CYAN}→${NC} Building for local development (uses --mode development, loads .env.local)"
    fi

    # Build with appropriate npm script
    echo -e "${YELLOW}→${NC} Running npm script: ${YELLOW}${NPM_SCRIPT}${NC}"
    WEB_BUILD_OUTPUT=$(npm run "$NPM_SCRIPT" 2>&1)
    WEB_BUILD_EXIT_CODE=$?

    if [ $WEB_BUILD_EXIT_CODE -eq 0 ]; then
        echo -e "${GREEN}✓ Web assets built (script: $NPM_SCRIPT)${NC}"
    else
        echo -e "${RED}Error: Web build failed${NC}"
        echo ""
        echo -e "${RED}═══════════════════════════════════════════════════${NC}"
        echo -e "${RED}  Web build failed: ${NPM_SCRIPT}${NC}"
        echo -e "${RED}═══════════════════════════════════════════════════${NC}"
        echo ""
        echo -e "${YELLOW}Full build output:${NC}"
        echo ""
        echo "$WEB_BUILD_OUTPUT"
        echo ""
        echo -e "${RED}Web build failed. See error above.${NC}"
        echo ""
        exit 1
    fi

    # Note: sitemap.xml and prerendered content generated by scheduled job (regenerate_static_content)
    # Job runs daily at midnight using fresh database content

    cd "$REPO_ROOT"
    echo ""
fi

# ──────────────────────────────────────────────────────────────
# Step 3: Stage binaries for Docker (release only)
# ──────────────────────────────────────────────────────────────

if [ "$MODE" = "release" ]; then
    echo -e "${BLUE}Step 3: Staging binaries for Docker...${NC}"
    echo ""

    STAGING_DIR="infrastructure/build-context/release"
    rm -rf "$STAGING_DIR"
    mkdir -p "$STAGING_DIR"

    # Copy main systemprompt binary
    if [ -f "core/target/release/systemprompt" ]; then
        cp "core/target/release/systemprompt" "$STAGING_DIR/"
        echo -e "  ${GREEN}→${NC} systemprompt"
    else
        echo -e "${RED}❌ ERROR: systemprompt binary not found at core/target/release/systemprompt${NC}"
        echo -e "${YELLOW}Build it first with: cargo build --release -p core${NC}"
        exit 1
    fi

    # Copy MCP servers (dynamically discovered from config)
    echo "Discovering MCP servers from config..."
    MCP_SERVERS=()

    # Parse MCP server configs from includes in main config
    if [ -f "crates/services/config/config.yml" ]; then
        # Extract MCP config file paths from includes
        MCP_CONFIGS=$(grep -E "^\s*-\s+\.\./mcp/.*\.yml" "crates/services/config/config.yml" | sed 's/.*\/\(.*\)\.yml/\1/')

        # For each MCP config, extract the binary name
        for mcp_name in $MCP_CONFIGS; do
            config_file="crates/services/mcp/${mcp_name}.yml"
            if [ -f "$config_file" ]; then
                # Extract binary field from YAML (format: "binary: \"name\"" or "binary: name")
                binary_name=$(grep -E "^\s*binary:\s*" "$config_file" | head -1 | sed -E 's/.*binary:\s*["]?([^"]+)["]?.*/\1/' | tr -d ' "')
                if [ -n "$binary_name" ]; then
                    MCP_SERVERS+=("$binary_name")
                    echo -e "  ${BLUE}→${NC} Found MCP server: $binary_name (from $config_file)"
                fi
            fi
        done
    fi

    if [ ${#MCP_SERVERS[@]} -eq 0 ]; then
        echo -e "${RED}❌ ERROR: No MCP servers found in config${NC}"
        exit 1
    fi

    echo ""
    echo "Copying ${#MCP_SERVERS[@]} MCP server binaries..."

    for server in "${MCP_SERVERS[@]}"; do
        # Try core/target/release first (main location)
        if [ -f "core/target/release/$server" ]; then
            cp "core/target/release/$server" "$STAGING_DIR/"
            echo -e "  ${GREEN}→${NC} $server"
        # Fall back to target/release (alternate location)
        elif [ -f "target/release/$server" ]; then
            cp "target/release/$server" "$STAGING_DIR/"
            echo -e "  ${GREEN}→${NC} $server"
        else
            echo -e "${RED}❌ ERROR: $server binary not found${NC}"
            echo -e "${YELLOW}Build it first with: cargo build --release -p $server${NC}"
            exit 1
        fi
    done

    echo ""
    echo -e "${GREEN}✓ Binaries staged in: ${STAGING_DIR}${NC}"
    echo ""
fi

# ──────────────────────────────────────────────────────────────
# Step 4: Build Docker image (optional, release only)
# ──────────────────────────────────────────────────────────────

if [ "$BUILD_DOCKER" = true ]; then
    echo -e "${BLUE}Step 4: Building Docker image...${NC}"
    echo ""

    # Validate all required binaries exist before Docker build (dynamically discovered from config)
    echo "Validating required binaries..."
    REQUIRED_BINARIES=("systemprompt")
    MISSING_BINARIES=()

    # Re-discover MCP servers from config for validation
    if [ -f "crates/services/config/config.yml" ]; then
        MCP_CONFIGS=$(grep -E "^\s*-\s+\.\./mcp/.*\.yml" "crates/services/config/config.yml" | sed 's/.*\/\(.*\)\.yml/\1/')
        for mcp_name in $MCP_CONFIGS; do
            config_file="crates/services/mcp/${mcp_name}.yml"
            if [ -f "$config_file" ]; then
                binary_name=$(grep -E "^\s*binary:\s*" "$config_file" | head -1 | sed -E 's/.*binary:\s*["]?([^"]+)["]?.*/\1/' | tr -d ' "')
                if [ -n "$binary_name" ]; then
                    REQUIRED_BINARIES+=("$binary_name")
                fi
            fi
        done
    fi

    for binary in "${REQUIRED_BINARIES[@]}"; do
        if [ ! -f "$STAGING_DIR/$binary" ]; then
            MISSING_BINARIES+=("$binary")
        fi
    done

    if [ ${#MISSING_BINARIES[@]} -gt 0 ]; then
        echo -e "${RED}❌ ERROR: Missing required binaries in $STAGING_DIR/${NC}"
        for binary in "${MISSING_BINARIES[@]}"; do
            echo -e "  ${RED}✗${NC} $binary"
        done
        echo ""
        echo -e "${YELLOW}Run 'just build' to build all binaries first${NC}"
        exit 1
    fi

    echo -e "${GREEN}✓ All required binaries present (${#REQUIRED_BINARIES[@]} total)${NC}"
    echo ""

    if [ ! -f "infrastructure/docker/app.Dockerfile" ]; then
        echo -e "${RED}Error: Dockerfile not found: infrastructure/docker/app.Dockerfile${NC}"
        exit 1
    fi

    # Build API image (Rust)
    echo "Building Rust API Docker image..."
    DOCKER_API_OUTPUT=$(docker build \
        -f infrastructure/docker/app.Dockerfile \
        -t "$DOCKER_IMAGE" \
        -t "$DOCKER_IMAGE_API" \
        -t "$GCR_IMAGE" \
        -t "$GCR_IMAGE_API" \
        . 2>&1)
    DOCKER_API_EXIT_CODE=$?

    if [ $DOCKER_API_EXIT_CODE -eq 0 ]; then
        echo ""
        echo -e "${GREEN}✓ Rust API Docker image built successfully${NC}"
        echo -e "  ${GREEN}→${NC} $DOCKER_IMAGE (compatibility)"
        echo -e "  ${GREEN}→${NC} $DOCKER_IMAGE_API"
        echo -e "  ${GREEN}→${NC} $GCR_IMAGE_API"
    else
        echo -e "${RED}Error: API Docker build failed${NC}"
        echo ""
        echo -e "${RED}═══════════════════════════════════════════════════${NC}"
        echo -e "${RED}  Docker API build failed${NC}"
        echo -e "${RED}═══════════════════════════════════════════════════${NC}"
        echo ""
        echo -e "${YELLOW}Full build output:${NC}"
        echo ""
        echo "$DOCKER_API_OUTPUT"
        echo ""
        echo -e "${RED}Docker build failed. See error above.${NC}"
        echo ""
        exit 1
    fi
    echo ""

    # Build Web image (nginx)
    echo "Building nginx web Docker image..."
    DOCKER_WEB_OUTPUT=$(docker build \
        -f core/web/Dockerfile \
        -t "$DOCKER_IMAGE_WEB" \
        -t "$GCR_IMAGE_WEB" \
        core/web 2>&1)
    DOCKER_WEB_EXIT_CODE=$?

    if [ $DOCKER_WEB_EXIT_CODE -eq 0 ]; then
        echo ""
        echo -e "${GREEN}✓ nginx Web Docker image built successfully${NC}"
        echo -e "  ${GREEN}→${NC} $DOCKER_IMAGE_WEB"
        echo -e "  ${GREEN}→${NC} $GCR_IMAGE_WEB"
    else
        echo -e "${RED}Error: Web Docker build failed${NC}"
        echo ""
        echo -e "${RED}═══════════════════════════════════════════════════${NC}"
        echo -e "${RED}  Docker Web build failed${NC}"
        echo -e "${RED}═══════════════════════════════════════════════════${NC}"
        echo ""
        echo -e "${YELLOW}Full build output:${NC}"
        echo ""
        echo "$DOCKER_WEB_OUTPUT"
        echo ""
        echo -e "${RED}Docker build failed. See error above.${NC}"
        echo ""
        exit 1
    fi
    echo ""
fi

# ──────────────────────────────────────────────────────────────
# Done
# ──────────────────────────────────────────────────────────────

echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  ✓ Build complete!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo ""

if [ "$MODE" = "debug" ]; then
    echo "Next steps:"
    echo -e "  ${YELLOW}just start${NC}          # Run API locally"
    echo -e "  ${YELLOW}just start --web${NC}   # Run API + rebuild web"
elif [ "$MODE" = "release" ]; then
    if [ "$BUILD_DOCKER" = true ]; then
        echo "Next steps:"
        echo -e "  ${YELLOW}just docker-run${NC}     # Run Docker container"
        echo -e "  ${YELLOW}just deploy${NC}        # Deploy to GCP"
    else
        echo "Next steps:"
        echo -e "  ${YELLOW}./scripts/build.sh release --docker${NC}  # Build Docker image"
    fi
fi
echo ""
