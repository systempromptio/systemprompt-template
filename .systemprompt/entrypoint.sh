#!/bin/sh
set -e

echo "Running database migrations..."
/app/bin/systemprompt services db migrate

echo "Starting services..."
exec /app/bin/systemprompt services serve --foreground
