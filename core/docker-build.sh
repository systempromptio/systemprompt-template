#!/bin/bash
set -e

# SystemPrompt OS Docker Build Script
# Builds locally, then creates Docker image

IMAGE_NAME="systemprompt-os"
IMAGE_TAG="${IMAGE_TAG:-latest}"
FULL_IMAGE="${IMAGE_NAME}:${IMAGE_TAG}"

echo "🔨 Building SystemPrompt OS"
echo "   Image: ${FULL_IMAGE}"
echo ""

# Step 1: Build Rust binaries locally
echo "🦀 Building Rust binaries locally (release mode)..."
cargo build --release --workspace

if [ $? -ne 0 ]; then
    echo "❌ Rust build failed!"
    exit 1
fi

echo "✅ Rust build complete"
echo ""

# Step 2: Build Docker image with pre-built binaries
echo "🐳 Building Docker image with pre-built binaries..."
docker build -f Dockerfile.prebuilt -t "${FULL_IMAGE}" .

if [ $? -ne 0 ]; then
    echo "❌ Docker build failed!"
    exit 1
fi

echo ""
echo "✅ Build complete!"
echo ""
echo "📊 Image details:"
docker images "${IMAGE_NAME}" --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}"

echo ""
echo "🚀 To run the container:"
echo "   docker-compose up -d"
echo ""
echo "   Or manually:"
echo "   docker run -d \\"
echo "     -p 8080:8080 \\"
echo "     -v \$(pwd)/database:/app/database \\"
echo "     -v \$(pwd)/.env:/app/.env:ro \\"
echo "     --name systemprompt \\"
echo "     ${FULL_IMAGE}"
echo ""
echo "🔍 To view logs:"
echo "   docker logs -f systemprompt"
echo ""
echo "🛠️  To access shell:"
echo "   docker exec -it systemprompt /bin/bash"
