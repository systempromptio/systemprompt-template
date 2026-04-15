#!/bin/bash
# DEMO: DATABASE OPERATIONS
#
# What this demonstrates:
#   1. Database connection info and health status
#   2. Schema introspection — tables, columns, indexes
#   3. Row counts and storage size
#   4. Read-only SQL queries
#   5. Migration status and schema validation
#
# CLI commands used:
#   - systemprompt infra db info
#   - systemprompt infra db status
#   - systemprompt infra db tables
#   - systemprompt infra db describe <table>
#   - systemprompt infra db indexes
#   - systemprompt infra db count <table>
#   - systemprompt infra db size
#   - systemprompt infra db query "<sql>"
#   - systemprompt infra db migrations
#   - systemprompt infra db validate
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "INFRASTRUCTURE: DATABASE" "Schema, tables, queries, indexes, migrations"

subheader "STEP 1: Database Info"
run_cli_indented infra db info

subheader "STEP 2: Connection Status"
run_cli_indented infra db status

subheader "STEP 3: List Tables"
run_cli_head 40 infra db tables

subheader "STEP 4: Describe a Table"
run_cli_indented infra db describe governance_decisions

subheader "STEP 5: Indexes"
run_cli_head 30 infra db indexes

subheader "STEP 6: Row Counts"
run_cli_indented infra db count governance_decisions

subheader "STEP 7: Database Size"
run_cli_indented infra db size

subheader "STEP 8: Read-Only Query"
cmd "systemprompt infra db query \"SELECT decision, COUNT(*) as count FROM governance_decisions GROUP BY decision\""
"$CLI" infra db query "SELECT decision, COUNT(*) as count FROM governance_decisions GROUP BY decision" --profile "$PROFILE" 2>&1 | sed 's/^/  /'

subheader "STEP 9: Migration Status"
run_cli_head 20 infra db migrations status

subheader "STEP 10: Schema Validation"
run_cli_indented infra db validate

echo ""
header "DATABASE DEMO COMPLETE" "Showed: info, status, tables, describe, indexes, count, size, query, migrations, validate"
