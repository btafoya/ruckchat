#!/usr/bin/env bash
set -euo pipefail

# Build the RuckChat server Docker image.
#
# This script builds the Web UI assets, refreshes the SQLx offline query cache,
# and builds a Docker image containing the single ruckchat-server binary.
#
# Environment variables:
#   IMAGE_TAG       Tag for the produced image (default: ruckchat-server:latest)
#   PUSH            Set to 1 to push the image after building (default: 0)
#
# Example:
#   IMAGE_TAG=ghcr.io/btafoya/ruckchat-server:0.1.0 PUSH=1 ./scripts/build-server.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

IMAGE_TAG="${IMAGE_TAG:-ruckchat-server:latest}"
PUSH="${PUSH:-0}"

cd "${PROJECT_ROOT}"

echo "Installing shared desktop frontend dependencies..."
cd desktop
pnpm install
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

if [[ "${PUSH}" == "1" ]]; then
    echo "Pushing ${IMAGE_TAG}..."
    docker push "${IMAGE_TAG}"
fi

echo "Build complete: ${IMAGE_TAG}"
