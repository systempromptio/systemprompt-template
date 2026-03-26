#!/bin/sh
set -e

echo "Running database migrations..."
/app/bin/systemprompt infra db migrate

echo "Starting services..."
exec /app/bin/systemprompt infra services serve --foreground
