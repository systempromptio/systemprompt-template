#!/bin/bash
# Backward-compatible wrapper — delegates to governance category
exec "$(dirname "$0")/governance/$(basename "$0")" "$@"
