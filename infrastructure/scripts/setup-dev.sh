#!/bin/bash
# SystemPrompt Template - First-Time Setup Script
# Run this after cloning the repository

set -e

echo "=========================================="
echo "  SystemPrompt Template Setup"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
DB_API_URL="${SYSTEMPROMPT_DB_API_URL:-http://localhost:8085}"

# Check prerequisites
echo "Checking prerequisites..."

check_command() {
    if command -v "$1" &> /dev/null; then
        echo -e "${GREEN}✓${NC} $1 found"
        return 0
    else
        echo -e "${RED}✗${NC} $1 not found"
        return 1
    fi
}

MISSING=0
check_command "cargo" || MISSING=1
check_command "docker" || MISSING=1
check_command "just" || { echo -e "${YELLOW}!${NC} 'just' not found - install with: cargo install just"; MISSING=1; }
check_command "curl" || MISSING=1
check_command "jq" || { echo -e "${YELLOW}!${NC} 'jq' not found - install with: sudo apt install jq"; MISSING=1; }

if [ $MISSING -eq 1 ]; then
    echo ""
    echo -e "${RED}Please install missing prerequisites and try again.${NC}"
    exit 1
fi

echo ""

# Install git hooks
echo "Installing git hooks..."
if [ -d ".git" ]; then
    ln -sf ../../infrastructure/scripts/pre-commit-core.sh .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo -e "${GREEN}✓${NC} Pre-commit hook installed (protects core/ from edits)"
else
    echo -e "${YELLOW}!${NC} Not a git repository - skipping hooks"
fi

echo ""

# Check systemprompt-db is running
echo "Checking systemprompt-db API..."
if ! curl -sf "$DB_API_URL/health" >/dev/null 2>&1; then
    echo -e "${RED}✗${NC} systemprompt-db API not running at $DB_API_URL"
    echo ""
    echo "Please start systemprompt-db first:"
    echo "  cd ../systemprompt-db && docker-compose -f docker-compose.local.yml up -d"
    echo ""
    echo "Or set SYSTEMPROMPT_DB_API_URL if using a different host."
    exit 1
fi
echo -e "${GREEN}✓${NC} systemprompt-db API available"

echo ""

# Open browser helper function
open_browser() {
    local url="$1"
    if command -v xdg-open &> /dev/null; then
        xdg-open "$url" 2>/dev/null &
    elif command -v open &> /dev/null; then
        open "$url" &
    elif command -v wslview &> /dev/null; then
        wslview "$url" &
    else
        echo -e "${YELLOW}!${NC} Could not open browser automatically"
        echo "Please open this URL manually: $url"
        return 1
    fi
    return 0
}

# Authentication
echo "Account Setup"
echo "-------------"
echo ""

# Check for existing credentials
CREDENTIALS_FILE="$HOME/.systemprompt/credentials"
if [ -f "$CREDENTIALS_FILE" ]; then
    SAVED_EMAIL=$(grep "^email=" "$CREDENTIALS_FILE" 2>/dev/null | cut -d= -f2-)
    SAVED_PASSWORD=$(grep "^password=" "$CREDENTIALS_FILE" 2>/dev/null | cut -d= -f2-)

    if [ -n "$SAVED_EMAIL" ] && [ -n "$SAVED_PASSWORD" ]; then
        echo "Found saved credentials for: $SAVED_EMAIL"
        read -p "Use saved credentials? [Y/n]: " USE_SAVED
        USE_SAVED="${USE_SAVED:-Y}"

        if [[ "$USE_SAVED" =~ ^[Yy]$ ]]; then
            EMAIL="$SAVED_EMAIL"
            PASSWORD="$SAVED_PASSWORD"
        fi
    fi
fi

# If no saved credentials or user declined
if [ -z "$EMAIL" ]; then
    echo "Enter your SystemPrompt account credentials."
    echo "If you don't have an account, one will be created."
    echo ""
    read -p "Email: " EMAIL
    read -s -p "Password: " PASSWORD
    echo ""

    if [ -z "$EMAIL" ] || [ -z "$PASSWORD" ]; then
        echo -e "${RED}✗${NC} Email and password are required"
        exit 1
    fi
fi

# Get tenant name
DEFAULT_TENANT=$(basename "$(pwd)" | tr '[:upper:]' '[:lower:]' | tr '-' '_')
read -p "Project name [$DEFAULT_TENANT]: " TENANT_NAME
TENANT_NAME="${TENANT_NAME:-$DEFAULT_TENANT}"

# Validate: lowercase, alphanumeric, underscores only, 3-32 chars
if ! [[ "$TENANT_NAME" =~ ^[a-z][a-z0-9_]{2,31}$ ]]; then
    echo -e "${RED}✗${NC} Invalid project name."
    echo "  - Must start with a lowercase letter"
    echo "  - Only lowercase letters, numbers, and underscores"
    echo "  - 3-32 characters"
    exit 1
fi

echo ""

# Try login first
echo "Authenticating..."
LOGIN_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$DB_API_URL/api/v1/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}")

LOGIN_CODE=$(echo "$LOGIN_RESPONSE" | tail -n1)
LOGIN_BODY=$(echo "$LOGIN_RESPONSE" | sed '$d')

if [ "$LOGIN_CODE" = "200" ]; then
    # Check if database_url is available
    DATABASE_URL=$(echo "$LOGIN_BODY" | jq -r '.database_url // empty')
    if [ -n "$DATABASE_URL" ]; then
        echo -e "${GREEN}✓${NC} Logged in successfully"
        CONNECTION_STRING=$(echo "$DATABASE_URL" | sed 's/@postgres:/@localhost:/')

        # Save credentials
        mkdir -p "$(dirname "$CREDENTIALS_FILE")"
        cat > "$CREDENTIALS_FILE" << EOF
email=$EMAIL
password=$PASSWORD
EOF
        chmod 600 "$CREDENTIALS_FILE"
    else
        echo -e "${YELLOW}!${NC} Account exists but payment not completed"
        echo "Please complete payment in your browser to activate your account."
        # Could add checkout URL retrieval here if needed
        exit 1
    fi
elif [ "$LOGIN_CODE" = "403" ]; then
    # Payment required - account exists but not activated
    echo -e "${YELLOW}!${NC} Payment required to activate your account"
    echo ""
    echo "A browser window will open to complete payment."
    echo "After payment, this script will automatically continue."
    echo ""
    read -p "Press Enter to open checkout page..."

    # Get a fresh checkout URL via signup (will return existing checkout for same email)
    SIGNUP_RESPONSE=$(curl -s -X POST "$DB_API_URL/api/v1/auth/signup" \
        -H "Content-Type: application/json" \
        -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\",\"tenant_name\":\"$TENANT_NAME\"}")

    CHECKOUT_URL=$(echo "$SIGNUP_RESPONSE" | jq -r '.checkout_url // empty')
    if [ -z "$CHECKOUT_URL" ]; then
        echo -e "${RED}✗${NC} Could not get checkout URL"
        echo "$SIGNUP_RESPONSE" | jq . 2>/dev/null || echo "$SIGNUP_RESPONSE"
        exit 1
    fi

    open_browser "$CHECKOUT_URL"
    echo ""
    echo "Checkout URL: $CHECKOUT_URL"
    echo ""
    echo "Waiting for payment completion..."

    # Poll for login success
    MAX_ATTEMPTS=120  # 10 minutes
    ATTEMPT=0
    while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
        sleep 5
        ATTEMPT=$((ATTEMPT + 1))

        POLL_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$DB_API_URL/api/v1/auth/login" \
            -H "Content-Type: application/json" \
            -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}")

        POLL_CODE=$(echo "$POLL_RESPONSE" | tail -n1)
        POLL_BODY=$(echo "$POLL_RESPONSE" | sed '$d')

        if [ "$POLL_CODE" = "200" ]; then
            DATABASE_URL=$(echo "$POLL_BODY" | jq -r '.database_url // empty')
            if [ -n "$DATABASE_URL" ]; then
                echo ""
                echo -e "${GREEN}✓${NC} Payment completed! Account activated."
                CONNECTION_STRING=$(echo "$DATABASE_URL" | sed 's/@postgres:/@localhost:/')

                # Save credentials
                mkdir -p "$(dirname "$CREDENTIALS_FILE")"
                cat > "$CREDENTIALS_FILE" << EOF
email=$EMAIL
password=$PASSWORD
EOF
                chmod 600 "$CREDENTIALS_FILE"
                break
            fi
        fi

        printf "."
    done

    if [ -z "$CONNECTION_STRING" ]; then
        echo ""
        echo -e "${RED}✗${NC} Timed out waiting for payment"
        echo "Please complete payment and run this script again."
        exit 1
    fi
elif [ "$LOGIN_CODE" = "401" ]; then
    # Invalid credentials - could be wrong password OR new user
    # Try signup first, handle "email_exists" as wrong password
    echo "Creating new account..."

    SIGNUP_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$DB_API_URL/api/v1/auth/signup" \
        -H "Content-Type: application/json" \
        -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\",\"tenant_name\":\"$TENANT_NAME\"}")

    SIGNUP_CODE=$(echo "$SIGNUP_RESPONSE" | tail -n1)
    SIGNUP_BODY=$(echo "$SIGNUP_RESPONSE" | sed '$d')

    # Check if account exists (wrong password)
    SIGNUP_ERROR=$(echo "$SIGNUP_BODY" | jq -r '.error // empty')
    if [ "$SIGNUP_ERROR" = "email_exists" ] || [ "$SIGNUP_ERROR" = "tenant_exists" ]; then
        echo -e "${RED}✗${NC} Invalid password for existing account"
        echo "Please check your password and try again."
        exit 1
    fi

    if [ "$SIGNUP_CODE" = "201" ] || [ "$SIGNUP_CODE" = "200" ]; then
        CHECKOUT_URL=$(echo "$SIGNUP_BODY" | jq -r '.checkout_url // empty')

        if [ -n "$CHECKOUT_URL" ]; then
            echo -e "${GREEN}✓${NC} Account created"
            echo ""
            echo "A browser window will open to complete payment."
            echo "After payment, your database will be provisioned automatically."
            echo ""
            read -p "Press Enter to open checkout page..."

            open_browser "$CHECKOUT_URL"
            echo ""
            echo "Checkout URL: $CHECKOUT_URL"
            echo ""
            echo "Waiting for payment completion..."

            # Poll for login success
            MAX_ATTEMPTS=120  # 10 minutes
            ATTEMPT=0
            while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
                sleep 5
                ATTEMPT=$((ATTEMPT + 1))

                POLL_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$DB_API_URL/api/v1/auth/login" \
                    -H "Content-Type: application/json" \
                    -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}")

                POLL_CODE=$(echo "$POLL_RESPONSE" | tail -n1)
                POLL_BODY=$(echo "$POLL_RESPONSE" | sed '$d')

                if [ "$POLL_CODE" = "200" ]; then
                    DATABASE_URL=$(echo "$POLL_BODY" | jq -r '.database_url // empty')
                    if [ -n "$DATABASE_URL" ]; then
                        echo ""
                        echo -e "${GREEN}✓${NC} Payment completed! Database provisioned."
                        CONNECTION_STRING=$(echo "$DATABASE_URL" | sed 's/@postgres:/@localhost:/')

                        # Save credentials
                        mkdir -p "$(dirname "$CREDENTIALS_FILE")"
                        cat > "$CREDENTIALS_FILE" << EOF
email=$EMAIL
password=$PASSWORD
EOF
                        chmod 600 "$CREDENTIALS_FILE"
                        break
                    fi
                fi

                printf "."
            done

            if [ -z "$CONNECTION_STRING" ]; then
                echo ""
                echo -e "${RED}✗${NC} Timed out waiting for payment"
                echo "Please complete payment and run this script again."
                exit 1
            fi
        else
            echo -e "${RED}✗${NC} No checkout URL in response"
            echo "$SIGNUP_BODY" | jq . 2>/dev/null || echo "$SIGNUP_BODY"
            exit 1
        fi
    else
        echo -e "${RED}✗${NC} Signup failed (HTTP $SIGNUP_CODE)"
        echo "$SIGNUP_BODY" | jq . 2>/dev/null || echo "$SIGNUP_BODY"
        exit 1
    fi
else
    echo -e "${RED}✗${NC} Authentication failed (HTTP $LOGIN_CODE)"
    echo "$LOGIN_BODY" | jq . 2>/dev/null || echo "$LOGIN_BODY"
    exit 1
fi

echo ""

# Setup .env.secrets
echo "Setting up .env.secrets..."
if [ ! -f ".env.secrets" ]; then
    cp .env.secrets.example .env.secrets
    echo -e "${GREEN}✓${NC} Created .env.secrets from template"
fi

# Update DATABASE_URL in .env.secrets (using printf to handle special chars in password)
grep -v "^DATABASE_URL=" .env.secrets > .env.secrets.tmp || true
printf 'DATABASE_URL=%s\n' "$CONNECTION_STRING" >> .env.secrets.tmp
mv .env.secrets.tmp .env.secrets
echo -e "${GREEN}✓${NC} DATABASE_URL saved to .env.secrets"

echo ""

# Create .env.local for local development
echo "Setting up .env.local..."
if [ ! -f ".env.local" ]; then
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

    cat > .env.local << EOF
# SystemPrompt Template - Local Development Environment
# Generated by setup-dev.sh

# Paths
SYSTEM_PATH=${PROJECT_ROOT}
WEB_DIR=${PROJECT_ROOT}/core/web/dist
SYSTEMPROMPT_CONFIG_PATH=${PROJECT_ROOT}/crates/services/config/config.yml
SYSTEMPROMPT_SERVICES_PATH=${PROJECT_ROOT}/crates/services
AI_CONFIG_PATH=${PROJECT_ROOT}/config/ai.yaml
CONTENT_CONFIG_PATH=${PROJECT_ROOT}/crates/services/content/config.yml

# Server
HOST=127.0.0.1
PORT=8080
API_SERVER_URL=http://localhost:8080

# Site
SITENAME=$TENANT_NAME
GITHUB_LINK=https://github.com/your-org/your-repo

# Logging
RUST_LOG=info

# JWT
JWT_ISSUER=systemprompt-$TENANT_NAME
JWT_ACCESS_TOKEN_EXPIRATION=86400
JWT_REFRESH_TOKEN_EXPIRATION=2592000

# CORS
CORS_ALLOWED_ORIGINS=http://localhost:8080,http://localhost:5173
EOF
    echo -e "${GREEN}✓${NC} Created .env.local"
else
    echo -e "${GREEN}✓${NC} .env.local already exists"
fi

echo ""

# Source environment for build (sqlx requires DATABASE_URL at compile time)
set -a
source .env.local
if [ -f .env.secrets ]; then source .env.secrets; fi
set +a

# Build
echo "Building project (this may take a few minutes on first run)..."
./infrastructure/scripts/build.sh debug
echo -e "${GREEN}✓${NC} Build complete"

echo ""

# Run migrations
echo "Running database migrations..."
./core/target/debug/systemprompt db migrate || {
    echo -e "${RED}✗${NC} Migration failed - check your DATABASE_URL"
    exit 1
}
echo -e "${GREEN}✓${NC} Migrations complete"

echo ""
echo "=========================================="
echo -e "${GREEN}  Setup Complete!${NC}"
echo "=========================================="
echo ""
echo "Your database: $TENANT_NAME"
echo ""
echo "Next steps:"
echo "  1. Run: just start"
echo "  2. Open: http://localhost:8080"
echo ""
