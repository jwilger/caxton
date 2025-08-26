-- Migration 001: Foundation Migration for Caxton Database Schema
--
-- This migration establishes the SQLx migration tracking system for the Caxton
-- multi-agent orchestration server. It serves as the foundation for all subsequent
-- schema changes and ensures proper migration versioning.
--
-- Backward Compatibility Strategy:
-- All table creation migrations use CREATE TABLE IF NOT EXISTS to ensure
-- graceful handling of existing databases that may have been initialized
-- using legacy CREATE TABLE patterns.
--
-- Migration System Benefits:
-- - Versioned schema changes with rollback capability
-- - Automatic migration application during DatabaseConnection::initialize()
-- - Production-ready embedded migrations with zero external file dependencies
-- - Professional database administration with migration tracking

SELECT 1 AS migration_foundation_established;
