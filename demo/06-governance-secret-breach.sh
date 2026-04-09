#!/bin/bash
# Backward-compatible wrapper — delegates to governance category
exec "$(dirname "$0")/governance/06-secret-breach.sh" "$@"
