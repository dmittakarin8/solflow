-- Phase 5: Token Trades Table
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
