#!/bin/bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

DB_TOOL="cargo run --bin systemprompt-db --package systemprompt-core-system --"

info "=== HTTP Session Analytics Test ==="

info "Cleaning up test data..."
$DB_TOOL query "DELETE FROM http_sessions WHERE session_id LIKE 'http-test-%'" 2>/dev/null || true
$DB_TOOL query "DELETE FROM http_requests WHERE session_id LIKE 'http-test-%'" 2>/dev/null || true
$DB_TOOL query "DELETE FROM user_activity WHERE session_id LIKE 'http-test-%'" 2>/dev/null || true
info "Test data cleaned up"

info ""
info "======================================"
info "TEST 1: Authenticated User Session"
info "======================================"

AUTH_SESSION_ID="http-test-auth-$(date +%s)"
info "Using test session ID: $AUTH_SESSION_ID"

info ""
info "Step 1: Generate admin JWT token..."
ADMIN_TOKEN=$(cargo run --bin systemprompt-login-admin --package systemprompt-core-oauth 2>/dev/null | tail -1)
if [ -z "$ADMIN_TOKEN" ]; then
    error "Failed to generate admin token"
    exit 1
fi
info "✓ Admin token generated"
info "Token preview: ${ADMIN_TOKEN:0:50}..."

info ""
info "Step 2: Make authenticated HTTP request with session ID..."
RESPONSE=$(curl -s -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "X-Session-ID: $AUTH_SESSION_ID" \
    http://localhost:8080/api/v1/health)
info "Response: ${RESPONSE:0:100}..."

info ""
info "Step 3: Wait for async session creation..."
sleep 2

info ""
info "Step 4: Verify session was created in database..."
AUTH_SESSION_RESULT=$($DB_TOOL query "SELECT session_id, user_id, request_count FROM http_sessions WHERE session_id = '$AUTH_SESSION_ID'")
info "Session query result:"
echo "$AUTH_SESSION_RESULT"

if echo "$AUTH_SESSION_RESULT" | grep -q "$AUTH_SESSION_ID"; then
    info "✓ Session created in database"
else
    error "✗ Session was not created in database"
    exit 1
fi

info ""
info "Step 5: Verify session has user_id populated (from JWT)..."
USER_ID_RAW=$(echo "$AUTH_SESSION_RESULT" | grep "$AUTH_SESSION_ID")
USER_ID=$(echo "$USER_ID_RAW" | awk -F'|' '{print $2}' | tr -d ' ')

info "Raw line: $USER_ID_RAW"
info "Extracted user_id: '$USER_ID'"

if [ -z "$USER_ID" ] || [ "$USER_ID" = "NULL" ]; then
    error "✗ Session user_id is NULL (JWT extraction failed)"
    error "Expected: UUID from JWT 'sub' claim"
    error "Got: NULL or empty"
    exit 1
else
    info "✓ Session has user_id from JWT token: $USER_ID"
fi

info ""
info "Step 6: Make second request with same session..."
sleep 1
curl -s -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "X-Session-ID: $AUTH_SESSION_ID" \
    http://localhost:8080/api/v1/health > /dev/null
sleep 2

REQUEST_COUNT_RESULT=$($DB_TOOL query "SELECT request_count FROM http_sessions WHERE session_id = '$AUTH_SESSION_ID'" --format json)
REQUEST_COUNT=$(echo "$REQUEST_COUNT_RESULT" | grep -o '"request_count":[0-9]*' | grep -o '[0-9]*$')
info "Request count after second request: $REQUEST_COUNT"

if [ "$REQUEST_COUNT" -lt 2 ]; then
    warn "Request count may not have incremented properly (got: $REQUEST_COUNT)"
fi

info ""
info "✓ TEST 1 PASSED: Authenticated user session works"

info ""
info "======================================"
info "TEST 2: Anonymous User Session"
info "======================================"

ANON_SESSION_ID="http-test-anon-$(date +%s)"
info "Using anonymous session ID: $ANON_SESSION_ID"

info ""
info "Step 1: Make request without JWT token..."
curl -s -H "X-Session-ID: $ANON_SESSION_ID" \
    http://localhost:8080/api/v1/health > /dev/null
sleep 2

info ""
info "Step 2: Verify anonymous session created..."
ANON_SESSION_RESULT=$($DB_TOOL query "SELECT session_id, user_id FROM http_sessions WHERE session_id = '$ANON_SESSION_ID'")
info "Anonymous session query result:"
echo "$ANON_SESSION_RESULT"

if echo "$ANON_SESSION_RESULT" | grep -q "$ANON_SESSION_ID"; then
    info "✓ Anonymous session created"
else
    error "✗ Anonymous session was not created"
    exit 1
fi

info ""
info "Step 3: Verify anonymous session has NULL user_id..."
ANON_USER_ID_RAW=$(echo "$ANON_SESSION_RESULT" | grep "$ANON_SESSION_ID")
ANON_USER_ID=$(echo "$ANON_USER_ID_RAW" | awk -F'|' '{print $2}' | tr -d ' ')

info "Raw line: $ANON_USER_ID_RAW"
info "Extracted user_id: '$ANON_USER_ID'"

if [ -n "$ANON_USER_ID" ] && [ "$ANON_USER_ID" != "NULL" ]; then
    error "✗ Anonymous session should have NULL user_id, got: '$ANON_USER_ID'"
    exit 1
else
    info "✓ Anonymous session has NULL user_id (correct)"
fi

info ""
info "======================================"
info "✓ ALL TESTS PASSED"
info "======================================"
