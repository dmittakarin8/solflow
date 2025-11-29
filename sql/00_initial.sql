-- ═══════════════════════════════════════════════════════════════════════
-- SOLFLOW - INITIAL DATABASE SCHEMA
-- ═══════════════════════════════════════════════════════════════════════
-- 
-- This file contains the complete, self-contained schema for SolFlow.
-- All tables, indexes, and constraints are defined here.
-- 
-- Schema includes:
--   • token_metadata           - Token information and launch data
--   • token_rolling_metrics    - Real-time rolling metrics (Phase 4/5)
--   • token_trades             - Append-only trade event log (Phase 5)
--   • token_signals            - Signal detection engine (Phase 6)
-- 
-- ═══════════════════════════════════════════════════════════════════════

-- Enable WAL mode for better concurrency
PRAGMA journal_mode=WAL;
PRAGMA synchronous = NORMAL;

-- ═══════════════════════════════════════════════════════════════════════
-- TABLE: token_metadata
-- ═══════════════════════════════════════════════════════════════════════
-- Stores basic token information and launch platform metadata
-- Frontend fetches price_usd, market_cap, and token_age from DexScreener

CREATE TABLE IF NOT EXISTS token_metadata (
    mint                TEXT PRIMARY KEY,
    symbol              TEXT,
    name                TEXT,
    decimals            INTEGER NOT NULL,
    launch_platform     TEXT,
    pair_created_at     INTEGER,
    price_usd           REAL,
    market_cap          REAL,
    token_age           INTEGER,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL,
    CHECK (decimals >= 0 AND decimals <= 18)
);

CREATE INDEX IF NOT EXISTS idx_token_metadata_created_at
    ON token_metadata (created_at);

CREATE INDEX IF NOT EXISTS idx_token_metadata_price_usd
    ON token_metadata (price_usd DESC);

CREATE INDEX IF NOT EXISTS idx_token_metadata_market_cap
    ON token_metadata (market_cap DESC);

-- ═══════════════════════════════════════════════════════════════════════
-- TABLE: token_rolling_metrics
-- ═══════════════════════════════════════════════════════════════════════
-- Real-time rolling metrics computed from Phase 4
-- UPSERT operations on each trade event

CREATE TABLE IF NOT EXISTS token_rolling_metrics (
    mint                        TEXT PRIMARY KEY,
    updated_at                  INTEGER NOT NULL,

    -- Net flow metrics (6 windows: 60s, 300s, 900s, 3600s, 7200s, 14400s)
    net_flow_60s                REAL NOT NULL DEFAULT 0.0,
    net_flow_300s               REAL NOT NULL DEFAULT 0.0,
    net_flow_900s               REAL NOT NULL DEFAULT 0.0,
    net_flow_3600s              REAL NOT NULL DEFAULT 0.0,
    net_flow_7200s              REAL NOT NULL DEFAULT 0.0,
    net_flow_14400s             REAL NOT NULL DEFAULT 0.0,

    -- Advanced metrics (300s window)
    unique_wallets_300s         INTEGER NOT NULL DEFAULT 0,
    bot_wallets_300s            INTEGER NOT NULL DEFAULT 0,
    bot_trades_300s             INTEGER NOT NULL DEFAULT 0,
    bot_flow_300s               REAL NOT NULL DEFAULT 0.0,

    -- DCA metrics (300s window)
    dca_flow_300s               REAL NOT NULL DEFAULT 0.0,
    dca_unique_wallets_300s     INTEGER NOT NULL DEFAULT 0,
    dca_ratio_300s              REAL NOT NULL DEFAULT 0.0
);

-- Index for time-based queries
CREATE INDEX IF NOT EXISTS idx_rolling_metrics_updated_at
    ON token_rolling_metrics (updated_at DESC);

-- Index for net flow queries (most active tokens)
CREATE INDEX IF NOT EXISTS idx_rolling_metrics_net_flow_300s
    ON token_rolling_metrics (net_flow_300s DESC);

-- ═══════════════════════════════════════════════════════════════════════
-- TABLE: token_trades
-- ═══════════════════════════════════════════════════════════════════════
-- Append-only trade event log for historical analysis
-- Stores all trades with bot/DCA flags from Phase 4

CREATE TABLE IF NOT EXISTS token_trades (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    mint                TEXT NOT NULL,
    timestamp           INTEGER NOT NULL,
    wallet              TEXT NOT NULL,
    side                TEXT NOT NULL,  -- 'buy' or 'sell'
    sol_amount          REAL NOT NULL,
    is_bot              INTEGER NOT NULL DEFAULT 0,  -- 0 = false, 1 = true
    is_dca              INTEGER NOT NULL DEFAULT 0   -- 0 = false, 1 = true
);

-- Index for mint-based queries (get trades for specific token)
CREATE INDEX IF NOT EXISTS idx_trades_mint
    ON token_trades (mint);

-- Index for time-based queries (recent trades)
CREATE INDEX IF NOT EXISTS idx_trades_timestamp
    ON token_trades (timestamp DESC);

-- Index for DCA analysis
CREATE INDEX IF NOT EXISTS idx_trades_is_dca
    ON token_trades (is_dca, timestamp);

-- Composite index for token + time range queries
CREATE INDEX IF NOT EXISTS idx_trades_mint_timestamp
    ON token_trades (mint, timestamp DESC);

-- ═══════════════════════════════════════════════════════════════════════
-- TABLE: token_signals
-- ═══════════════════════════════════════════════════════════════════════
-- Phase 6: Enhanced Signals Engine
-- Stores detected trading signals with strength, window, and metadata

CREATE TABLE IF NOT EXISTS token_signals (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    mint            TEXT NOT NULL,
    signal_type     TEXT NOT NULL,
    strength        REAL NOT NULL DEFAULT 0.0,
    window          TEXT NOT NULL,
    timestamp       INTEGER NOT NULL,
    metadata        TEXT,
    created_at      INTEGER DEFAULT (strftime('%s', 'now'))
);

-- Optimized indexes for signal queries
CREATE INDEX IF NOT EXISTS idx_token_signals_mint 
    ON token_signals(mint);

CREATE INDEX IF NOT EXISTS idx_token_signals_type 
    ON token_signals(signal_type);

CREATE INDEX IF NOT EXISTS idx_token_signals_timestamp 
    ON token_signals(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_token_signals_strength 
    ON token_signals(strength DESC);

CREATE INDEX IF NOT EXISTS idx_token_signals_mint_timestamp 
    ON token_signals(mint, timestamp DESC);

-- ═══════════════════════════════════════════════════════════════════════
-- TABLE: blocklist
-- ═══════════════════════════════════════════════════════════════════════
-- Stores token mints that should be filtered from dashboard queries
-- Used to hide spam tokens, rugs, or other unwanted tokens

CREATE TABLE IF NOT EXISTS blocklist (
    mint            TEXT PRIMARY KEY,
    reason          TEXT,
    added_at        INTEGER DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_blocklist_added_at 
    ON blocklist(added_at DESC);

-- ═══════════════════════════════════════════════════════════════════════
-- END OF SCHEMA
-- ═══════════════════════════════════════════════════════════════════════
