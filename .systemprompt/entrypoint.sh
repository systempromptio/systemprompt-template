#!/bin/sh
set -e

echo "Starting services..."
exec /app/bin/systemprompt services start --foreground
