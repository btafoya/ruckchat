#!/usr/bin/env bash
set -euo pipefail

# Build the RuckChat server Docker image.
# This script builds the Web UI assets first, then invokes docker build.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

IMAGE_TAG="${IMAGE_TAG:-ruckchat-server:latest}"

cd "${PROJECT_ROOT}"

echo "Building Web UI assets..."
cd web
pnpm install
pnpm build
cd "${PROJECT_ROOT}"

echo "Preparing SQLx offline query data..."
cargo sqlx prepare --workspace

echo "Building Docker image ${IMAGE_TAG}..."
docker build \
    -t "${IMAGE_TAG}" \
    .

echo "Build complete: ${IMAGE_TAG}"
