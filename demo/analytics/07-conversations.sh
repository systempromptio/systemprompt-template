#!/bin/bash
# DEMO: CONVERSATION ANALYTICS — Stats, trends, listing
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "ANALYTICS: CONVERSATIONS" "Conversation statistics, trends, listing"

subheader "STEP 1: Conversation Stats"
run_cli_indented analytics conversations stats

subheader "STEP 2: Conversation Trends"
run_cli_indented analytics conversations trends --since 7d

subheader "STEP 3: Recent Conversations"
run_cli_head 20 analytics conversations list

header "CONVERSATION ANALYTICS DEMO COMPLETE"
