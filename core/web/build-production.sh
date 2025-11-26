#!/bin/bash

set -e

echo "🚀 Building SystemPrompt Web (Production)"

cd "$(dirname "$0")"

if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
fi

echo "🔨 Building production bundle..."
npm run build

echo "✅ Build complete! Output in dist/"
echo "📁 Build artifacts:"
ls -lh dist/ | tail -10
