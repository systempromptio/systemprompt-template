#!/bin/bash

set -e

CONTENT_DIR="${1:-/content/blog}"

echo "📦 Ingesting markdown content from: $CONTENT_DIR"

cd /var/www/html/systemprompt-os-rust-2

if [ ! -f "target/debug/systemprompt" ]; then
    echo "🔨 Building systemprompt..."
    cargo build 2>&1 | tail -20
fi

echo "📝 Ingesting content..."
./target/debug/systemprompt ingest markdown --path "$CONTENT_DIR"

echo "✅ Content ingestion complete!"
