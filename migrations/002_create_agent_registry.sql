-- Migration 002: Create Agent Registry Table with Validation Constraints
--
-- Creates the agent_registry table for managing WebAssembly agent lifecycle
-- and metadata storage. This table supports the core agent management
-- functionality of the Caxton multi-agent orchestration server.
--
-- Schema Design:
-- - id: Unique agent identifier (validated non-empty string)
-- - name: Human-readable agent name (validated non-empty string)
-- - created_at: Unix timestamp of agent registration (immutable audit trail)
-- - updated_at: Unix timestamp of last agent modification (automatic refresh)
-- - CHECK constraints enforce domain validation at database level

CREATE TABLE IF NOT EXISTS agent_registry (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    CHECK (length(id) > 0),
    CHECK (length(name) > 0)
);
