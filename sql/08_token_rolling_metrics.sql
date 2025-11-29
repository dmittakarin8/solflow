-- Phase 5: Token Rolling Metrics Table
-- Real-time rolling metrics computed from Phase 4
-- UPSERT operations on each trade event

CREATE TABLE IF NOT EXISTS token_rolling_metrics (
    mint                        TEXT PRIMARY KEY,
    updated_at                  INTEGER NOT NULL,

    -- Net flow metrics (6 windows)
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
