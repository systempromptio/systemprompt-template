#!/bin/bash
# Backward-compatible wrapper — delegates to agents category
exec "$(dirname "$0")/agents/03-agent-messaging.sh" "$@"
