#!/usr/bin/env bash
# Focused regression coverage for the shared dashboard port. Use `just test-dashboard`.
set -euo pipefail
cd "$(dirname "$0")/.."
export SQLX_OFFLINE=true
if [ -z "${SYSTEMPROMPT_TEST_DATABASE_URL:-}" ] && [ -f .systemprompt/profiles/local/secrets.json ]; then
    SYSTEMPROMPT_TEST_DATABASE_URL=$(python3 - <<'PY'
import json
from urllib.parse import urlsplit, urlunsplit
with open('.systemprompt/profiles/local/secrets.json') as f:
    url = urlsplit(json.load(f)['database_url'])
print(urlunsplit((url.scheme, url.netloc, '/postgres', '', '')))
PY
)
    export SYSTEMPROMPT_TEST_DATABASE_URL
fi
stage=${1:-all}
case "$stage" in all|unit|contract|integration|registry) ;; *) echo "Unknown dashboard test stage: $stage" >&2; exit 2 ;; esac
if [[ "$stage" = all || "$stage" = unit ]]; then
    cargo nextest run --locked --manifest-path tests/Cargo.toml -p web-unit-tests \
        -E 'test(/conversation_|governance_gateway|governance_pages|governance_warnings|hooks_track_commits|session_registry_handle|dashboard_people|dev_login_pure/)'
fi
if [[ "$stage" = all || "$stage" = contract ]]; then
    cargo nextest run --locked --manifest-path tests/Cargo.toml -p admin-contract-tests \
        -E 'test(/self_service_contract|groups_contract|roles_contract|approvals_contract|dev_login_contract|write_boundaries|hook_track_deduplicates_and_rolls_up_the_session/)'
fi
if [[ "$stage" = registry ]]; then
    cargo nextest run --locked --manifest-path tests/Cargo.toml -p admin-contract-tests \
        -E 'test(hook_track_deduplicates_and_rolls_up_the_session)'
fi
if [[ "$stage" = all || "$stage" = integration ]]; then
    cargo nextest run --locked --manifest-path tests/Cargo.toml -p admin-db-core-tests \
        -E 'test(/usage_conversation_summary|users_access_matrix|users_lookups|analytics_requests|analytics_session_children|analytics_session_detail|traces_list|dashboard_session_summary|dashboard_usage_daily/)'
fi
