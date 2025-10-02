#!/usr/bin/env bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "🛑 Stopping Soroban Localnet..."

# Navigate to project root
cd "${PROJECT_ROOT}"

# Stop and remove containers, networks, volumes
docker-compose down -v

echo "✅ Soroban Localnet stopped and cleaned up"
