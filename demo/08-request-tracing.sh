#!/bin/bash
# Backward-compatible wrapper — delegates to performance category
exec "$(dirname "$0")/performance/01-request-tracing.sh" "$@"
