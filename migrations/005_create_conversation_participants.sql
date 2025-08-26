-- Migration 005: Create Conversation Participants Junction Table
--
-- Creates the conversation_participants junction table for managing many-to-many
-- relationships between conversations and participating agents. This table enables
-- efficient tracking of which agents are involved in each conversation within
-- the Caxton multi-agent orchestration system.
--
-- BACKWARD COMPATIBILITY: Uses CREATE TABLE IF NOT EXISTS to handle
-- databases where this table may already exist from previous deployments.
-- Foreign key relationships are preserved and existing participant data
-- remains intact during migration.
--
-- Schema Design:
-- - conversation_id: References conversations table (36-character UUID validation)
-- - participant_id: Agent UUID participating in the conversation
-- - Composite primary key ensures unique participant-conversation pairs
-- - Foreign key constraint maintains referential integrity with conversations
-- - CHECK constraints validate UUID format compliance for both identifiers
--
-- Relationship Model:
-- This junction table supports the many-to-many relationship where:
-- - One conversation can have multiple participant agents
-- - One agent can participate in multiple conversations
-- - Referential integrity is maintained through foreign key constraints

CREATE TABLE IF NOT EXISTS conversation_participants (
    conversation_id TEXT NOT NULL,
    participant_id TEXT NOT NULL,
    PRIMARY KEY (conversation_id, participant_id),
    FOREIGN KEY (conversation_id) REFERENCES conversations(conversation_id),
    CHECK (length(conversation_id) = 36),
    CHECK (length(participant_id) = 36)
);

-- Performance indexes for participant relationship queries
-- These indexes optimize common query patterns for conversation management:
--
-- - Conversation membership: Find all participants in a specific conversation
-- - Agent participation: Find all conversations for a specific agent
--
-- BACKWARD COMPATIBILITY: Uses CREATE INDEX IF NOT EXISTS to ensure
-- no conflicts with existing indexes while providing optimal performance
-- for participant lookup operations.

CREATE INDEX IF NOT EXISTS idx_participants_conversation ON conversation_participants(conversation_id);
CREATE INDEX IF NOT EXISTS idx_participants_participant ON conversation_participants(participant_id);
