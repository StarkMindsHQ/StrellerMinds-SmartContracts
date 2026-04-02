# Backup Procedures

## Overview

Backups cover four areas: contract source & build artifacts, scripts/configuration, documentation, and the Stellar localnet Docker volume.

---

## Scripts

| Script | Purpose |
|---|---|
| `scripts/backup.sh [dir]` | Create a timestamped backup |
| `scripts/backup_verify.sh <dir>` | Verify integrity + smoke-test recovery |

---

## Running a Backup

```bash
chmod +x scripts/backup.sh scripts/backup_verify.sh

# Create backup (defaults to ./backups/)
BACKUP_DIR=$(./scripts/backup.sh)

# Verify it immediately
./scripts/backup_verify.sh "$BACKUP_DIR"
```

Each backup produces a directory under `./backups/<YYYYMMDD_HHMMSS>/`:

```
backups/20260402_150000/
├── contracts.tar.gz    # contracts/, src/, e2e-tests/, Cargo.*
├── config.tar.gz       # scripts/, .cargo/, .github/, docker-compose.yml, Makefile
├── docs.tar.gz         # docs/, README.md, CHANGELOG.md
├── stellar-data.tar.gz # Stellar localnet volume (if running)
└── manifest.sha256     # SHA-256 checksums for all archives
```

Backups older than **30 days** are pruned automatically.

---

## Automated Backups (CI)

The `.github/workflows/backup.yml` workflow runs **daily at 02:00 UTC** and:

1. Runs `scripts/backup.sh`
2. Runs `scripts/backup_verify.sh` on the new backup
3. Uploads archives as a GitHub Actions artifact (retained 30 days)
4. Fails the workflow (with an error annotation) if backup or verification fails

Trigger manually via **Actions → Backup → Run workflow**.  
To run verification only against the latest artifact, enable the `verify_only` toggle.

---

## Recovery

### Full recovery

```bash
BACKUP_DIR=./backups/<timestamp>

# Restore contracts
tar -xzf "${BACKUP_DIR}/contracts.tar.gz"

# Restore config
tar -xzf "${BACKUP_DIR}/config.tar.gz"

# Restore docs
tar -xzf "${BACKUP_DIR}/docs.tar.gz"
```

### Stellar localnet volume recovery

```bash
docker volume create stellar-data
docker run --rm \
  -v stellar-data:/data \
  -v "$(pwd)/${BACKUP_DIR}":/backup \
  alpine tar -xzf /backup/stellar-data.tar.gz -C /data
```

### Verify before restoring

Always verify checksums before restoring:

```bash
cd "$BACKUP_DIR"
sha256sum --check manifest.sha256
```

---

## Monitoring

- CI workflow status is visible under **Actions → Backup**.
- A failed run means backup or verification did not pass — investigate the logs immediately.
- For off-CI environments, check `backups/backup.log` for a persistent audit trail.

---

## Retention Policy

| Storage | Retention |
|---|---|
| Local `./backups/` | 30 days (auto-pruned by `backup.sh`) |
| GitHub Actions artifacts | 30 days |

Adjust the `mtime +30` value in `backup.sh` and `retention-days` in `backup.yml` to change retention.
