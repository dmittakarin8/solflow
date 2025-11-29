-- Phase 6: Enhanced Signals Engine
-- Adds new columns to support strength-based signals and metadata

-- Add strength column (0.0 - 1.0 intensity score)
-- Add metadata column (JSON) for storing signal-specific data
-- Update indexes for performance

-- Note: Using ALTER TABLE to avoid recreating existing table
-- SQLite will handle missing columns gracefully

ALTER TABLE token_signals ADD COLUMN strength REAL DEFAULT 0.0;
ALTER TABLE token_signals ADD COLUMN metadata TEXT;

-- Create optimized indexes for Phase 6 signal queries
CREATE INDEX IF NOT EXISTS idx_token_signals_mint ON token_signals(mint);
CREATE INDEX IF NOT EXISTS idx_token_signals_type ON token_signals(signal_type);
CREATE INDEX IF NOT EXISTS idx_token_signals_timestamp ON token_signals(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_token_signals_strength ON token_signals(strength DESC);
