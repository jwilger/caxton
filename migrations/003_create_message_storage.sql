-- Migration 003: Create Message Storage Table for FIPA Message Persistence
--
-- Creates the message_storage table for persisting FIPA (Foundation for Intelligent
-- Physical Agents) compliant messages within the Caxton multi-agent system.
-- This table supports high-throughput message routing with optimized indexing
-- for temporal queries and agent-specific message retrieval.
--
-- BACKWARD COMPATIBILITY: Uses CREATE TABLE IF NOT EXISTS to gracefully
-- handle databases with pre-existing message_storage tables that may have
-- been created by legacy code or have different constraint definitions.
-- This migration will not conflict with existing data.
--
-- Schema Design:
-- - message_id: UUID primary key (exactly 36 characters with validation)
-- - sender_id/receiver_id: Agent UUIDs (36-character validation for referential integrity)
-- - conversation_id: Optional conversation grouping (allows NULL for standalone messages)
-- - message_content: FIPA message payload (stored as TEXT for flexibility)
-- - performative: FIPA performative type (inform, request, agree, etc.)
-- - created_at: Unix timestamp for message ordering and temporal queries
-- - expires_at: Optional message expiration (Unix timestamp) for TTL message cleanup
--
-- TTL (Time To Live) Message Patterns:
-- The expires_at field enables automatic message cleanup for temporary communications:
-- - NULL value: Message persists indefinitely (permanent storage)
-- - Future timestamp: Message expires at specified time for automatic cleanup
-- - Past timestamp: Message is eligible for immediate cleanup
--
-- Expected TTL Behavior:
-- 1. Message Creation: expires_at set based on message type and retention policies
-- 2. Message Retrieval: All messages returned regardless of expiration status
-- 3. Cleanup Process: Separate background process removes expired messages
-- 4. Query Filtering: Application logic may filter expired messages during retrieval
--
-- Cleanup Procedures for Expired Messages:
-- MANUAL CLEANUP (for development and testing):
--   DELETE FROM message_storage WHERE expires_at IS NOT NULL AND expires_at < strftime('%s', 'now');
--
-- AUTOMATED CLEANUP (recommended for production):
-- - Background task runs periodically (e.g., every hour) to remove expired messages
-- - Cleanup batch size should be limited (e.g., 1000 messages per run) to avoid blocking
-- - Consider using VACUUM INCREMENTAL after large cleanup operations
-- - Log cleanup statistics for monitoring: messages removed, storage reclaimed
--
-- Performance Considerations:
-- - Index on expires_at field supports efficient cleanup queries
-- - Batch cleanup operations to avoid long-running transactions
-- - Consider message archival before deletion for compliance/audit requirements
-- - Monitor database size growth and cleanup effectiveness

CREATE TABLE IF NOT EXISTS message_storage (
    message_id TEXT PRIMARY KEY,
    sender_id TEXT NOT NULL,
    receiver_id TEXT NOT NULL,
    conversation_id TEXT,
    message_content TEXT NOT NULL,
    performative TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER,
    CHECK (length(message_id) = 36),
    CHECK (length(sender_id) = 36),
    CHECK (length(receiver_id) = 36)
);

-- Performance indexes for high-throughput message routing operations
-- These compound indexes with DESC temporal ordering support efficient
-- message retrieval patterns optimized for recent-first queries:
--
-- - Agent message history: Retrieve recent messages for specific agents
-- - Conversation tracking: Access conversation messages in temporal order
-- - System-wide temporal queries: Support time-based message cleanup and analytics
--
-- BACKWARD COMPATIBILITY: Uses CREATE INDEX IF NOT EXISTS to avoid
-- conflicts with existing indexes while ensuring optimal performance
-- for all message routing operations.

CREATE INDEX IF NOT EXISTS idx_message_storage_sender ON message_storage(sender_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_message_storage_receiver ON message_storage(receiver_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_message_storage_conversation ON message_storage(conversation_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_message_storage_created_at ON message_storage(created_at DESC);

-- TTL cleanup optimization index: Supports efficient queries for expired message removal
-- This index enables fast identification of messages eligible for cleanup without
-- scanning the entire table, critical for maintaining performance during batch cleanup operations
CREATE INDEX IF NOT EXISTS idx_message_storage_expires_at ON message_storage(expires_at) WHERE expires_at IS NOT NULL;
