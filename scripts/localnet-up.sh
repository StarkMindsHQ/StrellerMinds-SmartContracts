#!/usr/bin/env bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "🚀 Starting Soroban Localnet..."

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
  echo "❌ Error: Docker is not running. Please start Docker and try again."
  exit 1
fi

# Navigate to project root
cd "${PROJECT_ROOT}"

# Start localnet with docker-compose
docker-compose up -d

# Wait for health check
echo "⏳ Waiting for localnet to be ready..."
MAX_RETRIES=30
RETRY_COUNT=0

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
  if docker-compose ps | grep -q "healthy"; then
    echo "✅ Soroban Localnet is ready!"
    echo ""
    echo "Network Information:"
    echo "  RPC URL: http://localhost:8000/soroban/rpc"
    echo "  Network Passphrase: Standalone Network ; February 2017"
    echo ""
    echo "To view logs: docker-compose logs -f"
    echo "To stop: ./scripts/localnet-down.sh"
    exit 0
  fi

  RETRY_COUNT=$((RETRY_COUNT + 1))
  sleep 2
done

echo "⚠️  Warning: Localnet may not be fully ready yet. Check logs with: docker-compose logs"
exit 0
