#!/bin/bash

# Build script for A2A web client

echo "🚀 Building A2A web client..."

# Navigate to web directory
cd "$(dirname "$0")" || exit 1

# Install dependencies if node_modules doesn't exist
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
fi

# Build the project
echo "🔨 Building production bundle..."
npm run build

if [ $? -eq 0 ]; then
    echo "✅ Build successful! Output in dist/"
    echo "📁 Files created:"
    ls -la dist/
else
    echo "❌ Build failed!"
    exit 1
fi