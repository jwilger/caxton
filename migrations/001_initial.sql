-- Migration 001: Initial Migration
--
-- NOTE: This migration contains only a SELECT statement and creates no schema.
-- It exists to establish the migration tracking system. Future schema changes
-- begin with migration 002.
--
-- TODO: Consider removing in future major version when breaking changes are acceptable.

SELECT 1 AS migration_foundation_established;
