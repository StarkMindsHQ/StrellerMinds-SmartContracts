#!/usr/bin/env bash
# backup.sh — Automated backup for StrellerMinds smart contracts
# Usage: ./scripts/backup.sh [backup_dir]
set -euo pipefail

BACKUP_ROOT="${1:-./backups}"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BACKUP_DIR="${BACKUP_ROOT}/${TIMESTAMP}"
LOG_FILE="${BACKUP_ROOT}/backup.log"
MANIFEST="${BACKUP_DIR}/manifest.sha256"

log() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"; }

mkdir -p "$BACKUP_DIR"
log "INFO  Starting backup → ${BACKUP_DIR}"

# ── 1. Contract source & build artifacts ────────────────────────────────────
CONTRACTS_ARCHIVE="${BACKUP_DIR}/contracts.tar.gz"
tar -czf "$CONTRACTS_ARCHIVE" \
  contracts/ src/ e2e-tests/ e2e/ \
  Cargo.toml Cargo.lock rust-toolchain.toml \
  --exclude='target' --exclude='*.wasm.bak'
log "INFO  Contracts archived → ${CONTRACTS_ARCHIVE}"

# ── 2. Scripts & configuration ───────────────────────────────────────────────
CONFIG_ARCHIVE="${BACKUP_DIR}/config.tar.gz"
tar -czf "$CONFIG_ARCHIVE" \
  scripts/ .cargo/ .github/ \
  docker-compose.yml Makefile package.json \
  mkdocs.yml lychee.toml rustfmt.toml
log "INFO  Config archived → ${CONFIG_ARCHIVE}"

# ── 3. Documentation ─────────────────────────────────────────────────────────
DOCS_ARCHIVE="${BACKUP_DIR}/docs.tar.gz"
tar -czf "$DOCS_ARCHIVE" docs/ README.md CHANGELOG.md
log "INFO  Docs archived → ${DOCS_ARCHIVE}"

# ── 4. Stellar localnet data (if running) ────────────────────────────────────
if docker volume inspect stellar-data &>/dev/null 2>&1; then
  STELLAR_ARCHIVE="${BACKUP_DIR}/stellar-data.tar.gz"
  docker run --rm \
    -v stellar-data:/data \
    -v "$(pwd)/${BACKUP_DIR}":/backup \
    alpine tar -czf /backup/stellar-data.tar.gz -C /data .
  log "INFO  Stellar volume archived → ${STELLAR_ARCHIVE}"
else
  log "WARN  stellar-data volume not found — skipping"
fi

# ── 5. Generate SHA-256 manifest ─────────────────────────────────────────────
sha256sum "${BACKUP_DIR}"/*.tar.gz > "$MANIFEST"
log "INFO  Manifest written → ${MANIFEST}"

# ── 6. Prune backups older than 30 days ──────────────────────────────────────
find "$BACKUP_ROOT" -maxdepth 1 -type d -mtime +30 | while read -r old; do
  log "INFO  Pruning old backup: ${old}"
  rm -rf "$old"
done

log "INFO  Backup complete ✓"
echo "$BACKUP_DIR"   # emit path for callers / CI
