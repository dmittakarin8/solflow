-- ═══════════════════════════════════════════════════════════════════════
-- PHASE 9 MIGRATION: Add Price, Market Cap, and Token Age fields
-- ═══════════════════════════════════════════════════════════════════════
-- 
-- Adds fields for Deckscreener integration:
--   • price_usd          - Current token price in USD
--   • market_cap         - Current market capitalization
--   • token_age          - Token age in seconds (calculated from pair creation)
-- 
-- ═══════════════════════════════════════════════════════════════════════

-- Add price_usd column (nullable, updated via Deckscreener API)
ALTER TABLE token_metadata ADD COLUMN price_usd REAL DEFAULT NULL;

-- Add market_cap column (nullable, updated via Deckscreener API)
ALTER TABLE token_metadata ADD COLUMN market_cap REAL DEFAULT NULL;

-- Add token_age column (nullable, calculated from pair_created_at)
ALTER TABLE token_metadata ADD COLUMN token_age INTEGER DEFAULT NULL;

-- Create index for tokens with price data
CREATE INDEX IF NOT EXISTS idx_token_metadata_price
    ON token_metadata (price_usd) WHERE price_usd IS NOT NULL;

-- ═══════════════════════════════════════════════════════════════════════
-- TABLE: followed_tokens
-- ═══════════════════════════════════════════════════════════════════════
-- Tracks which tokens are being followed for continuous price polling

CREATE TABLE IF NOT EXISTS followed_tokens (
    mint                TEXT PRIMARY KEY,
    created_at          INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    last_fetch_at       INTEGER DEFAULT NULL
);

CREATE INDEX IF NOT EXISTS idx_followed_tokens_last_fetch
    ON followed_tokens (last_fetch_at);

-- ═══════════════════════════════════════════════════════════════════════
-- TABLE: blocklist
-- ═══════════════════════════════════════════════════════════════════════
-- Tracks blocked tokens (hidden from dashboard, ignored in GRPC ingestion)

CREATE TABLE IF NOT EXISTS blocklist (
    mint                TEXT PRIMARY KEY,
    created_at          INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    reason              TEXT DEFAULT NULL
);

CREATE INDEX IF NOT EXISTS idx_blocklist_created_at
    ON blocklist (created_at DESC);
