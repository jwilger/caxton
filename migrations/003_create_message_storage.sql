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
-- - expires_at: Optional message expiration (supports TTL message patterns)

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
