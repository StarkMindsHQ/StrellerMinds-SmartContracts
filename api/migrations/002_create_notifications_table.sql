-- ── Notifications persistence layer ──────────────────────────────────────────
-- Migration: 002_create_notifications_table
-- Creates the notifications table and supporting indexes for the real-time
-- WebSocket notification system.

CREATE TABLE IF NOT EXISTS notifications (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  type            VARCHAR(64)  NOT NULL,
  priority        VARCHAR(16)  NOT NULL DEFAULT 'normal',
  recipient_id    VARCHAR(256) NOT NULL,
  title           TEXT         NOT NULL,
  body            TEXT         NOT NULL,
  data            JSONB,
  status          VARCHAR(16)  NOT NULL DEFAULT 'pending',
  retry_count     INTEGER      NOT NULL DEFAULT 0,
  created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
  expires_at      TIMESTAMPTZ,
  delivered_at    TIMESTAMPTZ,
  updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Index for fetching pending notifications for a user on reconnect
CREATE INDEX IF NOT EXISTS idx_notifications_recipient_status
  ON notifications (recipient_id, status, created_at DESC);

-- Index for expiry cleanup job
CREATE INDEX IF NOT EXISTS idx_notifications_expires_at
  ON notifications (expires_at)
  WHERE expires_at IS NOT NULL;

-- Index for status-based queries
CREATE INDEX IF NOT EXISTS idx_notifications_status_created
  ON notifications (status, created_at DESC);

-- Auto-update updated_at on row change
CREATE OR REPLACE FUNCTION update_notifications_updated_at()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_notifications_updated_at ON notifications;
CREATE TRIGGER trg_notifications_updated_at
  BEFORE UPDATE ON notifications
  FOR EACH ROW EXECUTE FUNCTION update_notifications_updated_at();

-- Cleanup function: remove expired and old delivered notifications
-- Call periodically (e.g. via pg_cron or a scheduled job)
CREATE OR REPLACE FUNCTION cleanup_notifications(
  p_max_age_days INTEGER DEFAULT 30
) RETURNS INTEGER AS $$
DECLARE
  deleted_count INTEGER;
BEGIN
  DELETE FROM notifications
  WHERE
    (expires_at IS NOT NULL AND expires_at < NOW())
    OR (status = 'delivered' AND delivered_at < NOW() - (p_max_age_days || ' days')::INTERVAL)
    OR (status = 'failed'    AND created_at  < NOW() - (p_max_age_days || ' days')::INTERVAL);

  GET DIAGNOSTICS deleted_count = ROW_COUNT;
  RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;
