-- Migration 004: Create Conversations Table for Multi-Agent Dialog Management
--
-- Creates the conversations table for tracking and managing multi-agent
-- conversations within the Caxton orchestration system. This table supports
-- FIPA-compliant conversation tracking with protocol awareness and archival
-- capabilities for long-term conversation management.
--
--
-- Schema Design:
-- - conversation_id: UUID primary key for unique conversation identification
-- - protocol_name: Optional FIPA protocol identifier (allows NULL for ad-hoc conversations)
-- - created_at: Unix timestamp of conversation initiation (immutable audit trail)
-- - last_activity: Unix timestamp of most recent message (updated automatically)
-- - message_count: Running count of messages in conversation (performance optimization)
-- - is_archived: Boolean flag for conversation lifecycle management
-- - Validation constraints ensure data integrity and UUID format compliance

CREATE TABLE IF NOT EXISTS conversations (
    conversation_id TEXT PRIMARY KEY,
    protocol_name TEXT,
    created_at INTEGER NOT NULL,
    last_activity INTEGER NOT NULL,
    message_count INTEGER NOT NULL DEFAULT 0,
    is_archived BOOLEAN NOT NULL DEFAULT FALSE,
    CHECK (length(conversation_id) = 36),
    CHECK (message_count >= 0)
);

-- Performance indexes for conversation management and analytics queries
-- These indexes support efficient conversation retrieval patterns:
--
-- - Archive status filtering: Separate active from archived conversations
-- - Temporal activity queries: Find conversations by recent activity
-- - Protocol-based grouping: Analyze conversations by FIPA protocol type
--

CREATE INDEX IF NOT EXISTS idx_conversations_archived ON conversations(is_archived);
CREATE INDEX IF NOT EXISTS idx_conversations_last_activity ON conversations(last_activity DESC);
CREATE INDEX IF NOT EXISTS idx_conversations_protocol ON conversations(protocol_name);
