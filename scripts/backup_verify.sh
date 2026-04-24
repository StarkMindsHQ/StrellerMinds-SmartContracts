#!/usr/bin/env bash
# backup_verify.sh — Verify integrity of a backup and test recovery
# Usage: ./scripts/backup_verify.sh <backup_dir>
set -euo pipefail

BACKUP_DIR="${1:?Usage: $0 <backup_dir>}"
MANIFEST="${BACKUP_DIR}/manifest.sha256"
RESTORE_TMP=$(mktemp -d)
EXIT_CODE=0

log()  { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"; }
pass() { log "PASS  $*"; }
fail() { log "FAIL  $*"; EXIT_CODE=1; }

trap 'rm -rf "$RESTORE_TMP"' EXIT

# ── 1. Manifest exists ───────────────────────────────────────────────────────
[[ -f "$MANIFEST" ]] || { fail "Manifest not found: ${MANIFEST}"; exit 1; }

# ── 2. SHA-256 integrity check ───────────────────────────────────────────────
log "INFO  Verifying checksums…"
if (cd "$BACKUP_DIR" && sha256sum --check manifest.sha256 --quiet); then
  pass "All checksums match"
else
  fail "Checksum mismatch detected"
fi

# ── 3. Archive readability ───────────────────────────────────────────────────
for archive in "${BACKUP_DIR}"/*.tar.gz; do
  if tar -tzf "$archive" &>/dev/null; then
    pass "Readable: $(basename "$archive")"
  else
    fail "Corrupt archive: $(basename "$archive")"
  fi
done

# ── 4. Recovery smoke-test (contracts archive) ───────────────────────────────
CONTRACTS_ARCHIVE="${BACKUP_DIR}/contracts.tar.gz"
if [[ -f "$CONTRACTS_ARCHIVE" ]]; then
  log "INFO  Smoke-testing contract recovery…"
  tar -xzf "$CONTRACTS_ARCHIVE" -C "$RESTORE_TMP"

  # Verify key paths restored
  for path in contracts Cargo.toml Cargo.lock; do
    if [[ -e "${RESTORE_TMP}/${path}" ]]; then
      pass "Restored: ${path}"
    else
      fail "Missing after restore: ${path}"
    fi
  done

  # Verify Cargo.toml is parseable
  if grep -q '^\[workspace\]' "${RESTORE_TMP}/Cargo.toml"; then
    pass "Cargo.toml is valid"
  else
    fail "Cargo.toml appears corrupt"
  fi
fi

# ── 5. Summary ───────────────────────────────────────────────────────────────
if [[ $EXIT_CODE -eq 0 ]]; then
  log "INFO  Verification PASSED ✓"
else
  log "ERROR Verification FAILED — see above"
fi

exit $EXIT_CODE
