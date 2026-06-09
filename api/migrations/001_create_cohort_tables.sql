-- Database Migration: Student Cohort Management
-- Issue #411: Implement Student Cohort Management
-- 
-- This script creates the necessary tables for cohort management functionality.
-- Run this migration after deploying the new API code.

-- ─────────────────────────────────────────────────────────────────────────────
-- Cohorts Table
-- Stores cohort/course group information
-- ─────────────────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS cohorts (
  id VARCHAR(100) PRIMARY KEY,
  name VARCHAR(200) NOT NULL,
  description TEXT,
  course VARCHAR(200) NOT NULL,
  instructor VARCHAR(200) NOT NULL,
  start_date BIGINT NOT NULL,
  end_date BIGINT NOT NULL,
  max_students INT NOT NULL DEFAULT 50,
  current_students INT NOT NULL DEFAULT 0,
  status VARCHAR(20) NOT NULL DEFAULT 'draft',
  created_at BIGINT NOT NULL,
  updated_at BIGINT NOT NULL,
  metadata JSONB DEFAULT '{}',
  
  -- Constraints
  CONSTRAINT chk_cohorts_status CHECK (status IN ('active', 'completed', 'archived', 'draft')),
  CONSTRAINT chk_cohorts_max_students CHECK (max_students > 0 AND max_students <= 1000),
  CONSTRAINT chk_cohorts_current_students CHECK (current_students >= 0 AND current_students <= max_students),
  CONSTRAINT chk_cohorts_dates CHECK (end_date > start_date)
);

-- ─────────────────────────────────────────────────────────────────────────────
-- Cohort Members Table
-- Tracks student enrollment in cohorts
-- ─────────────────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS cohort_members (
  cohort_id VARCHAR(100) NOT NULL REFERENCES cohorts(id) ON DELETE CASCADE,
  student_address VARCHAR(100) NOT NULL,
  enrolled_at BIGINT NOT NULL,
  status VARCHAR(20) NOT NULL DEFAULT 'active',
  progress INT NOT NULL DEFAULT 0,
  certificates_earned INT NOT NULL DEFAULT 0,
  last_activity BIGINT NOT NULL,
  
  -- Primary key and constraints
  PRIMARY KEY (cohort_id, student_address),
  CONSTRAINT chk_member_status CHECK (status IN ('active', 'completed', 'withdrawn')),
  CONSTRAINT chk_member_progress CHECK (progress >= 0 AND progress <= 100),
  CONSTRAINT chk_member_certificates CHECK (certificates_earned >= 0)
);

-- ─────────────────────────────────────────────────────────────────────────────
-- Cohort Messages Table
-- Group messaging system for cohorts
-- ─────────────────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS cohort_messages (
  id VARCHAR(100) PRIMARY KEY,
  cohort_id VARCHAR(100) NOT NULL REFERENCES cohorts(id) ON DELETE CASCADE,
  sender_address VARCHAR(100) NOT NULL,
  content TEXT NOT NULL,
  timestamp BIGINT NOT NULL,
  type VARCHAR(20) NOT NULL DEFAULT 'discussion',
  reply_to VARCHAR(100) REFERENCES cohort_messages(id) ON DELETE SET NULL,
  reactions JSONB DEFAULT '{}',
  
  -- Constraints
  CONSTRAINT chk_message_type CHECK (type IN ('announcement', 'discussion', 'question', 'resource')),
  CONSTRAINT chk_message_content CHECK (LENGTH(content) > 0 AND LENGTH(content) <= 5000)
);

-- ─────────────────────────────────────────────────────────────────────────────
-- Indexes for Performance Optimization
-- ─────────────────────────────────────────────────────────────────────────────

-- Cohorts indexes
CREATE INDEX IF NOT EXISTS idx_cohorts_status ON cohorts(status);
CREATE INDEX IF NOT EXISTS idx_cohorts_course ON cohorts(course);
CREATE INDEX IF NOT EXISTS idx_cohorts_instructor ON cohorts(instructor);
CREATE INDEX IF NOT EXISTS idx_cohorts_created_at ON cohorts(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_cohorts_updated_at ON cohorts(updated_at DESC);

-- Cohort members indexes
CREATE INDEX IF NOT EXISTS idx_cohort_members_cohort ON cohort_members(cohort_id);
CREATE INDEX IF NOT EXISTS idx_cohort_members_student ON cohort_members(student_address);
CREATE INDEX IF NOT EXISTS idx_cohort_members_status ON cohort_members(status);
CREATE INDEX IF NOT EXISTS idx_cohort_members_progress ON cohort_members(cohort_id, progress DESC);
CREATE INDEX IF NOT EXISTS idx_cohort_members_last_activity ON cohort_members(cohort_id, last_activity DESC);

-- Cohort messages indexes
CREATE INDEX IF NOT EXISTS idx_cohort_messages_cohort ON cohort_messages(cohort_id);
CREATE INDEX IF NOT EXISTS idx_cohort_messages_timestamp ON cohort_messages(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_cohort_messages_type ON cohort_messages(cohort_id, type);
CREATE INDEX IF NOT EXISTS idx_cohort_messages_sender ON cohort_messages(sender_address);
CREATE INDEX IF NOT EXISTS idx_cohort_messages_reply_to ON cohort_messages(reply_to);

-- ─────────────────────────────────────────────────────────────────────────────
-- Comments for Documentation
-- ─────────────────────────────────────────────────────────────────────────────

COMMENT ON TABLE cohorts IS 'Student learning cohorts/groups for collaborative education';
COMMENT ON TABLE cohort_members IS 'Student enrollment records for cohorts';
COMMENT ON TABLE cohort_messages IS 'Group messaging system for cohort communication';

COMMENT ON COLUMN cohorts.metadata IS 'Flexible JSON storage for additional cohort attributes';
COMMENT ON COLUMN cohort_members.progress IS 'Student progress percentage (0-100)';
COMMENT ON COLUMN cohort_members.certificates_earned IS 'Number of certificates earned by student in this cohort';
COMMENT ON COLUMN cohort_messages.type IS 'Message type: announcement, discussion, question, or resource';
COMMENT ON COLUMN cohort_messages.reactions IS 'JSON object mapping emoji reactions to counts';

-- ─────────────────────────────────────────────────────────────────────────────
-- Sample Data (Optional - Remove in production)
-- ─────────────────────────────────────────────────────────────────────────────

-- Sample cohort
INSERT INTO cohorts (
  id, name, description, course, instructor,
  start_date, end_date, max_students, current_students,
  status, created_at, updated_at, metadata
) VALUES (
  'cohort_sample_001',
  'Blockchain Fundamentals Q1 2026',
  'An introductory course covering blockchain basics, smart contracts, and decentralized applications',
  'BLOCKCHAIN-101',
  'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA',
  1711929600000, -- April 1, 2026
  1719792000000, -- July 1, 2026
  50,
  0,
  'draft',
  EXTRACT(EPOCH FROM NOW()) * 1000,
  EXTRACT(EPOCH FROM NOW()) * 1000,
  '{"difficulty": "beginner", "prerequisites": []}'
) ON CONFLICT (id) DO NOTHING;

-- ─────────────────────────────────────────────────────────────────────────────
-- Rollback Instructions
-- ─────────────────────────────────────────────────────────────────────────────

/*
To rollback this migration, run:

DROP TABLE IF EXISTS cohort_messages CASCADE;
DROP TABLE IF EXISTS cohort_members CASCADE;
DROP TABLE IF EXISTS cohorts CASCADE;

Note: CASCADE will also drop all indexes automatically.
*/

-- ─────────────────────────────────────────────────────────────────────────────
-- Migration Complete
-- ─────────────────────────────────────────────────────────────────────────────

DO $$
BEGIN
  RAISE NOTICE 'Cohort management tables created successfully';
  RAISE NOTICE 'Tables: cohorts, cohort_members, cohort_messages';
  RAISE NOTICE 'Indexes: 15 performance indexes created';
END $$;
