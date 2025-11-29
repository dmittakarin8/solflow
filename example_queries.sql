-- Phase 5: Example SQL Queries for SolFlow Database
-- These queries demonstrate how to analyze the persisted data

-- ============================================
-- 1. ROLLING METRICS QUERIES
-- ============================================

-- Top 10 tokens by net flow (5 minute window)
SELECT 
    mint,
    net_flow_300s,
    unique_wallets_300s,
    bot_trades_300s,
    dca_ratio_300s,
    datetime(updated_at, 'unixepoch') as last_updated
FROM token_rolling_metrics
ORDER BY net_flow_300s DESC
LIMIT 10;

-- Tokens with highest bot activity
SELECT 
    mint,
    bot_wallets_300s,
    bot_trades_300s,
    bot_flow_300s,
    ROUND(bot_flow_300s / NULLIF(net_flow_300s, 0) * 100, 2) as bot_percentage
FROM token_rolling_metrics
WHERE bot_trades_300s > 0
ORDER BY bot_trades_300s DESC
LIMIT 10;

-- Tokens with highest DCA ratio
SELECT 
    mint,
    dca_ratio_300s,
    dca_flow_300s,
    dca_unique_wallets_300s,
    net_flow_300s
FROM token_rolling_metrics
WHERE dca_unique_wallets_300s > 0
ORDER BY dca_ratio_300s DESC
LIMIT 10;

-- Most active tokens (by unique wallets)
SELECT 
    mint,
    unique_wallets_300s,
    net_flow_300s,
    bot_wallets_300s,
    datetime(updated_at, 'unixepoch') as last_updated
FROM token_rolling_metrics
ORDER BY unique_wallets_300s DESC
LIMIT 10;

-- Net flow across all time windows
SELECT 
    mint,
    net_flow_60s as flow_1min,
    net_flow_300s as flow_5min,
    net_flow_900s as flow_15min,
    net_flow_3600s as flow_1hour,
    net_flow_7200s as flow_2hour,
    net_flow_14400s as flow_4hour
FROM token_rolling_metrics
WHERE net_flow_300s > 10.0
ORDER BY net_flow_300s DESC
LIMIT 10;

-- ============================================
-- 2. TRADE EVENT QUERIES
-- ============================================

-- Recent trades (last 50)
SELECT 
    mint,
    wallet,
    side,
    sol_amount,
    is_bot,
    is_dca,
    datetime(timestamp, 'unixepoch') as trade_time
FROM token_trades
ORDER BY timestamp DESC
LIMIT 50;

-- Trades for specific token
SELECT 
    wallet,
    side,
    sol_amount,
    is_bot,
    is_dca,
    datetime(timestamp, 'unixepoch') as trade_time
FROM token_trades
WHERE mint = 'YOUR_TOKEN_MINT_HERE'
ORDER BY timestamp DESC
LIMIT 100;

-- Bot trades analysis
SELECT 
    mint,
    COUNT(*) as bot_trade_count,
    COUNT(DISTINCT wallet) as unique_bot_wallets,
    SUM(CASE WHEN side = 'buy' THEN sol_amount ELSE 0 END) as bot_buys,
    SUM(CASE WHEN side = 'sell' THEN sol_amount ELSE 0 END) as bot_sells,
    SUM(CASE WHEN side = 'buy' THEN sol_amount ELSE -sol_amount END) as bot_net_flow
FROM token_trades
WHERE is_bot = 1
GROUP BY mint
ORDER BY bot_trade_count DESC
LIMIT 10;

-- DCA trades analysis
SELECT 
    mint,
    COUNT(*) as dca_trade_count,
    SUM(sol_amount) as total_dca_volume,
    COUNT(DISTINCT wallet) as unique_dca_wallets,
    AVG(sol_amount) as avg_dca_size
FROM token_trades
WHERE is_dca = 1
GROUP BY mint
ORDER BY total_dca_volume DESC
LIMIT 10;

-- Trade volume by side (buy vs sell)
SELECT 
    mint,
    COUNT(CASE WHEN side = 'buy' THEN 1 END) as buy_count,
    COUNT(CASE WHEN side = 'sell' THEN 1 END) as sell_count,
    SUM(CASE WHEN side = 'buy' THEN sol_amount ELSE 0 END) as buy_volume,
    SUM(CASE WHEN side = 'sell' THEN sol_amount ELSE 0 END) as sell_volume,
    SUM(CASE WHEN side = 'buy' THEN sol_amount ELSE -sol_amount END) as net_flow
FROM token_trades
GROUP BY mint
ORDER BY net_flow DESC
LIMIT 10;

-- ============================================
-- 3. TIME-BASED QUERIES
-- ============================================

-- Trades in last hour
SELECT 
    mint,
    COUNT(*) as trade_count,
    SUM(sol_amount) as total_volume,
    COUNT(DISTINCT wallet) as unique_wallets
FROM token_trades
WHERE timestamp >= (strftime('%s', 'now') - 3600)
GROUP BY mint
ORDER BY trade_count DESC;

-- Trades in last 5 minutes
SELECT 
    mint,
    COUNT(*) as trade_count,
    SUM(sol_amount) as total_volume,
    COUNT(DISTINCT wallet) as unique_wallets
FROM token_trades
WHERE timestamp >= (strftime('%s', 'now') - 300)
GROUP BY mint
ORDER BY trade_count DESC;

-- Hourly trade activity (last 24 hours)
SELECT 
    datetime(timestamp / 3600 * 3600, 'unixepoch') as hour,
    COUNT(*) as trade_count,
    SUM(sol_amount) as volume,
    COUNT(DISTINCT mint) as unique_tokens
FROM token_trades
WHERE timestamp >= (strftime('%s', 'now') - 86400)
GROUP BY hour
ORDER BY hour DESC;

-- ============================================
-- 4. WALLET ANALYSIS QUERIES
-- ============================================

-- Most active wallets
SELECT 
    wallet,
    COUNT(*) as trade_count,
    COUNT(DISTINCT mint) as tokens_traded,
    SUM(CASE WHEN side = 'buy' THEN sol_amount ELSE -sol_amount END) as net_flow,
    SUM(CASE WHEN is_bot = 1 THEN 1 ELSE 0 END) as bot_trades
FROM token_trades
GROUP BY wallet
ORDER BY trade_count DESC
LIMIT 20;

-- DCA wallets
SELECT 
    wallet,
    COUNT(*) as dca_trade_count,
    COUNT(DISTINCT mint) as tokens_dca,
    SUM(sol_amount) as total_dca_volume
FROM token_trades
WHERE is_dca = 1
GROUP BY wallet
ORDER BY dca_trade_count DESC
LIMIT 20;

-- Bot wallets
SELECT 
    wallet,
    COUNT(*) as bot_trade_count,
    COUNT(DISTINCT mint) as tokens_traded,
    SUM(CASE WHEN side = 'buy' THEN sol_amount ELSE -sol_amount END) as bot_net_flow
FROM token_trades
WHERE is_bot = 1
GROUP BY wallet
ORDER BY bot_trade_count DESC
LIMIT 20;

-- ============================================
-- 5. CORRELATION QUERIES
-- ============================================

-- Compare rolling metrics with recent trades
SELECT 
    m.mint,
    m.net_flow_300s as metrics_flow_5min,
    COALESCE(SUM(CASE WHEN t.side = 'buy' THEN t.sol_amount ELSE -t.sol_amount END), 0) as trades_flow_5min,
    m.unique_wallets_300s as metrics_wallets,
    COUNT(DISTINCT t.wallet) as trades_wallets
FROM token_rolling_metrics m
LEFT JOIN token_trades t 
    ON m.mint = t.mint 
    AND t.timestamp >= (strftime('%s', 'now') - 300)
GROUP BY m.mint, m.net_flow_300s, m.unique_wallets_300s
ORDER BY m.net_flow_300s DESC
LIMIT 10;

-- ============================================
-- 6. PERFORMANCE MONITORING QUERIES
-- ============================================

-- Database size and row counts
SELECT 
    'token_rolling_metrics' as table_name,
    COUNT(*) as row_count
FROM token_rolling_metrics
UNION ALL
SELECT 
    'token_trades' as table_name,
    COUNT(*) as row_count
FROM token_trades;

-- Oldest and newest data
SELECT 
    'oldest_trade' as metric,
    datetime(MIN(timestamp), 'unixepoch') as value
FROM token_trades
UNION ALL
SELECT 
    'newest_trade' as metric,
    datetime(MAX(timestamp), 'unixepoch') as value
FROM token_trades
UNION ALL
SELECT 
    'data_span_hours' as metric,
    ROUND((MAX(timestamp) - MIN(timestamp)) / 3600.0, 2) as value
FROM token_trades;

-- Trade rate (trades per minute over last hour)
SELECT 
    ROUND(COUNT(*) / 60.0, 2) as trades_per_minute
FROM token_trades
WHERE timestamp >= (strftime('%s', 'now') - 3600);

-- ============================================
-- 7. INDEX VERIFICATION
-- ============================================

-- List all indexes
SELECT 
    name,
    tbl_name,
    sql
FROM sqlite_master
WHERE type = 'index'
    AND tbl_name IN ('token_rolling_metrics', 'token_trades')
ORDER BY tbl_name, name;

-- ============================================
-- 8. DATA CLEANUP QUERIES
-- ============================================

-- Delete trades older than 7 days (for pruning)
-- WARNING: Uncomment only if you want to delete data
-- DELETE FROM token_trades WHERE timestamp < (strftime('%s', 'now') - 604800);

-- Count old trades (before deleting)
SELECT 
    COUNT(*) as trades_older_than_7_days,
    MIN(datetime(timestamp, 'unixepoch')) as oldest_trade
FROM token_trades
WHERE timestamp < (strftime('%s', 'now') - 604800);

-- Archive metrics for inactive tokens (no updates in 24 hours)
-- This is useful for cleaning up tokens that are no longer trading
SELECT 
    mint,
    datetime(updated_at, 'unixepoch') as last_updated,
    net_flow_300s
FROM token_rolling_metrics
WHERE updated_at < (strftime('%s', 'now') - 86400)
ORDER BY updated_at ASC;
