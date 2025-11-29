-- Phase 6: Example Signal Queries
-- Comprehensive SQL queries for analyzing trading signals

-- ============================================
-- 1. RECENT SIGNALS FOR A SPECIFIC TOKEN
-- ============================================
-- Get the 10 most recent signals for a token
SELECT 
    signal_type,
    strength,
    window,
    datetime(timestamp, 'unixepoch') as signal_time,
    metadata
FROM token_signals
WHERE mint = 'YOUR_MINT_ADDRESS_HERE'
ORDER BY timestamp DESC
LIMIT 10;

-- ============================================
-- 2. TOP STRENGTH SIGNALS (LAST HOUR)
-- ============================================
-- Find highest strength signals in the last hour
SELECT 
    mint,
    signal_type,
    strength,
    window,
    datetime(timestamp, 'unixepoch') as signal_time,
    json_extract(metadata, '$.net_flow_300s') as net_flow
FROM token_signals
WHERE timestamp >= strftime('%s', 'now') - 3600
ORDER BY strength DESC
LIMIT 20;

-- ============================================
-- 3. BREAKOUT SIGNALS WITH DETAILS
-- ============================================
-- Analyze breakout signals with key metrics
SELECT 
    mint,
    strength,
    datetime(timestamp, 'unixepoch') as signal_time,
    json_extract(metadata, '$.net_flow_60s') as flow_60s,
    json_extract(metadata, '$.net_flow_300s') as flow_300s,
    json_extract(metadata, '$.net_flow_900s') as flow_900s,
    json_extract(metadata, '$.unique_wallets') as wallets,
    json_extract(metadata, '$.bot_ratio') as bot_ratio
FROM token_signals
WHERE signal_type = 'BREAKOUT'
  AND timestamp >= strftime('%s', 'now') - 3600
ORDER BY strength DESC;

-- ============================================
-- 4. FOCUSED BUYERS (WHALE ACTIVITY)
-- ============================================
-- Find tokens with concentrated buying patterns
SELECT 
    mint,
    strength,
    datetime(timestamp, 'unixepoch') as signal_time,
    ROUND(json_extract(metadata, '$.f_score'), 3) as f_score,
    json_extract(metadata, '$.wallets_needed') as whales,
    json_extract(metadata, '$.total_wallets') as total_wallets,
    ROUND(json_extract(metadata, '$.total_inflow'), 2) as total_inflow
FROM token_signals
WHERE signal_type = 'FOCUSED_BUYERS'
  AND timestamp >= strftime('%s', 'now') - 1800
ORDER BY strength DESC;

-- ============================================
-- 5. DCA REACCUMULATION SIGNALS
-- ============================================
-- Tokens with strong DCA accumulation
SELECT 
    mint,
    strength,
    datetime(timestamp, 'unixepoch') as signal_time,
    ROUND(json_extract(metadata, '$.dca_flow'), 2) as dca_flow,
    json_extract(metadata, '$.dca_wallets') as dca_wallets,
    ROUND(json_extract(metadata, '$.dca_ratio'), 3) as dca_ratio,
    ROUND(json_extract(metadata, '$.net_flow_300s'), 2) as net_flow
FROM token_signals
WHERE signal_type = 'REACCUMULATION'
  AND timestamp >= strftime('%s', 'now') - 3600
ORDER BY strength DESC;

-- ============================================
-- 6. PERSISTENCE SIGNALS (SUSTAINED MOMENTUM)
-- ============================================
-- Tokens with sustained positive flow
SELECT 
    mint,
    strength,
    datetime(timestamp, 'unixepoch') as signal_time,
    ROUND(json_extract(metadata, '$.net_flow_60s'), 2) as flow_60s,
    ROUND(json_extract(metadata, '$.net_flow_300s'), 2) as flow_300s,
    ROUND(json_extract(metadata, '$.net_flow_900s'), 2) as flow_900s,
    json_extract(metadata, '$.unique_wallets') as wallets
FROM token_signals
WHERE signal_type = 'PERSISTENCE'
  AND timestamp >= strftime('%s', 'now') - 3600
ORDER BY strength DESC;

-- ============================================
-- 7. FLOW REVERSAL (EARLY WARNING)
-- ============================================
-- Tokens showing momentum exhaustion
SELECT 
    mint,
    strength,
    datetime(timestamp, 'unixepoch') as signal_time,
    ROUND(json_extract(metadata, '$.net_flow_60s'), 2) as flow_60s,
    ROUND(json_extract(metadata, '$.net_flow_300s'), 2) as flow_300s,
    json_extract(metadata, '$.unique_wallets') as wallets,
    ROUND(json_extract(metadata, '$.wallets_per_trade'), 2) as wallets_per_trade
FROM token_signals
WHERE signal_type = 'FLOW_REVERSAL'
  AND timestamp >= strftime('%s', 'now') - 1800
ORDER BY timestamp DESC;

-- ============================================
-- 8. MULTIPLE SIMULTANEOUS SIGNALS
-- ============================================
-- Tokens triggering multiple signals (high conviction)
SELECT 
    mint,
    COUNT(DISTINCT signal_type) as signal_count,
    GROUP_CONCAT(DISTINCT signal_type) as signals,
    ROUND(AVG(strength), 3) as avg_strength,
    ROUND(MAX(strength), 3) as max_strength,
    datetime(MAX(timestamp), 'unixepoch') as latest_signal
FROM token_signals
WHERE timestamp >= strftime('%s', 'now') - 300
GROUP BY mint
HAVING signal_count >= 2
ORDER BY signal_count DESC, avg_strength DESC;

-- ============================================
-- 9. SIGNAL FREQUENCY BY TYPE (LAST 24H)
-- ============================================
-- Analyze which signals are most common
SELECT 
    signal_type,
    COUNT(*) as signal_count,
    ROUND(AVG(strength), 3) as avg_strength,
    ROUND(MIN(strength), 3) as min_strength,
    ROUND(MAX(strength), 3) as max_strength
FROM token_signals
WHERE timestamp >= strftime('%s', 'now') - 86400
GROUP BY signal_type
ORDER BY signal_count DESC;

-- ============================================
-- 10. SIGNAL TIMELINE FOR A TOKEN
-- ============================================
-- Chronological signal history for pattern analysis
SELECT 
    signal_type,
    strength,
    window,
    datetime(timestamp, 'unixepoch') as signal_time,
    CASE 
        WHEN strength >= 0.8 THEN 'VERY_STRONG'
        WHEN strength >= 0.5 THEN 'STRONG'
        WHEN strength >= 0.2 THEN 'MODERATE'
        ELSE 'WEAK'
    END as classification
FROM token_signals
WHERE mint = 'YOUR_MINT_ADDRESS_HERE'
  AND timestamp >= strftime('%s', 'now') - 7200
ORDER BY timestamp ASC;

-- ============================================
-- 11. TOP MINTS BY SIGNAL STRENGTH
-- ============================================
-- Tokens with strongest signals overall
SELECT 
    mint,
    MAX(strength) as max_strength,
    COUNT(*) as signal_count,
    GROUP_CONCAT(DISTINCT signal_type) as signal_types,
    datetime(MAX(timestamp), 'unixepoch') as latest_signal
FROM token_signals
WHERE timestamp >= strftime('%s', 'now') - 3600
GROUP BY mint
ORDER BY max_strength DESC
LIMIT 20;

-- ============================================
-- 12. SIGNAL STRENGTH DISTRIBUTION
-- ============================================
-- Histogram of signal strengths
SELECT 
    signal_type,
    CASE 
        WHEN strength < 0.2 THEN '0.0-0.2'
        WHEN strength < 0.4 THEN '0.2-0.4'
        WHEN strength < 0.6 THEN '0.4-0.6'
        WHEN strength < 0.8 THEN '0.6-0.8'
        ELSE '0.8-1.0'
    END as strength_range,
    COUNT(*) as count
FROM token_signals
WHERE timestamp >= strftime('%s', 'now') - 3600
GROUP BY signal_type, strength_range
ORDER BY signal_type, strength_range;

-- ============================================
-- 13. RECENT STRONG SIGNALS (ACTIONABLE)
-- ============================================
-- High-confidence signals from last 30 minutes
SELECT 
    mint,
    signal_type,
    strength,
    window,
    datetime(timestamp, 'unixepoch') as signal_time,
    metadata
FROM token_signals
WHERE timestamp >= strftime('%s', 'now') - 1800
  AND strength >= 0.6
ORDER BY timestamp DESC, strength DESC
LIMIT 30;

-- ============================================
-- 14. FLOW DIVERGENCE ANALYSIS
-- ============================================
-- Compare 60s vs 300s flow across breakouts
SELECT 
    mint,
    strength,
    ROUND(json_extract(metadata, '$.net_flow_60s'), 2) as flow_60s,
    ROUND(json_extract(metadata, '$.net_flow_300s'), 2) as flow_300s,
    ROUND(
        (json_extract(metadata, '$.net_flow_60s') - json_extract(metadata, '$.net_flow_300s')) 
        / json_extract(metadata, '$.net_flow_300s') * 100, 
        1
    ) as divergence_pct
FROM token_signals
WHERE signal_type = 'BREAKOUT'
  AND timestamp >= strftime('%s', 'now') - 3600
ORDER BY divergence_pct DESC;

-- ============================================
-- 15. BOT ACTIVITY CORRELATION
-- ============================================
-- Analyze bot activity in breakout signals
SELECT 
    CASE 
        WHEN json_extract(metadata, '$.bot_ratio') < 0.1 THEN 'LOW (0-10%)'
        WHEN json_extract(metadata, '$.bot_ratio') < 0.2 THEN 'NORMAL (10-20%)'
        WHEN json_extract(metadata, '$.bot_ratio') < 0.3 THEN 'MODERATE (20-30%)'
        ELSE 'HIGH (30%+)'
    END as bot_activity,
    COUNT(*) as signal_count,
    ROUND(AVG(strength), 3) as avg_strength
FROM token_signals
WHERE signal_type = 'BREAKOUT'
  AND timestamp >= strftime('%s', 'now') - 3600
GROUP BY bot_activity
ORDER BY bot_activity;

-- ============================================
-- 16. WALLET CONCENTRATION ANALYSIS
-- ============================================
-- F-score distribution for focused buyer signals
SELECT 
    mint,
    ROUND(json_extract(metadata, '$.f_score'), 3) as f_score,
    json_extract(metadata, '$.wallets_needed') as whale_count,
    CASE 
        WHEN json_extract(metadata, '$.f_score') <= 0.2 THEN 'EXTREME_CONCENTRATION'
        WHEN json_extract(metadata, '$.f_score') <= 0.3 THEN 'HIGH_CONCENTRATION'
        ELSE 'MODERATE_CONCENTRATION'
    END as concentration_level,
    strength
FROM token_signals
WHERE signal_type = 'FOCUSED_BUYERS'
  AND timestamp >= strftime('%s', 'now') - 3600
ORDER BY f_score ASC;

-- ============================================
-- 17. SIGNAL PERFORMANCE TRACKING
-- ============================================
-- Track how many signals fired per mint (for deduplication)
SELECT 
    mint,
    signal_type,
    COUNT(*) as fire_count,
    ROUND(AVG(strength), 3) as avg_strength,
    datetime(MIN(timestamp), 'unixepoch') as first_signal,
    datetime(MAX(timestamp), 'unixepoch') as last_signal,
    (MAX(timestamp) - MIN(timestamp)) / 60 as duration_minutes
FROM token_signals
WHERE timestamp >= strftime('%s', 'now') - 3600
GROUP BY mint, signal_type
HAVING fire_count > 1
ORDER BY fire_count DESC, mint;

-- ============================================
-- 18. CROSS-SIGNAL CORRELATION
-- ============================================
-- Tokens that trigger BREAKOUT followed by PERSISTENCE
WITH breakout_signals AS (
    SELECT DISTINCT mint, timestamp as breakout_time
    FROM token_signals
    WHERE signal_type = 'BREAKOUT'
      AND timestamp >= strftime('%s', 'now') - 3600
),
persistence_signals AS (
    SELECT DISTINCT mint, timestamp as persistence_time
    FROM token_signals
    WHERE signal_type = 'PERSISTENCE'
      AND timestamp >= strftime('%s', 'now') - 3600
)
SELECT 
    b.mint,
    datetime(b.breakout_time, 'unixepoch') as breakout_time,
    datetime(p.persistence_time, 'unixepoch') as persistence_time,
    (p.persistence_time - b.breakout_time) / 60 as minutes_between
FROM breakout_signals b
INNER JOIN persistence_signals p ON b.mint = p.mint
WHERE p.persistence_time > b.breakout_time
ORDER BY minutes_between ASC;

-- ============================================
-- 19. SIGNAL METADATA SUMMARY
-- ============================================
-- Aggregate metadata statistics
SELECT 
    signal_type,
    COUNT(*) as count,
    ROUND(AVG(json_extract(metadata, '$.net_flow_300s')), 2) as avg_flow_300s,
    ROUND(AVG(json_extract(metadata, '$.unique_wallets')), 1) as avg_wallets,
    ROUND(AVG(strength), 3) as avg_strength
FROM token_signals
WHERE timestamp >= strftime('%s', 'now') - 3600
  AND json_extract(metadata, '$.net_flow_300s') IS NOT NULL
GROUP BY signal_type
ORDER BY count DESC;

-- ============================================
-- 20. LIVE DASHBOARD QUERY
-- ============================================
-- Optimized query for real-time dashboard
SELECT 
    ts.mint,
    ts.signal_type,
    ts.strength,
    ts.window,
    datetime(ts.timestamp, 'unixepoch') as signal_time,
    json_extract(ts.metadata, '$.net_flow_300s') as net_flow,
    json_extract(ts.metadata, '$.unique_wallets') as wallets,
    -- Join with token_rolling_metrics for additional context
    trm.unique_wallets_300s as current_wallets,
    trm.net_flow_300s as current_flow
FROM token_signals ts
LEFT JOIN token_rolling_metrics trm ON ts.mint = trm.mint
WHERE ts.timestamp >= strftime('%s', 'now') - 600
  AND ts.strength >= 0.4
ORDER BY ts.timestamp DESC
LIMIT 50;

-- ============================================
-- END OF SIGNAL QUERIES
-- ============================================
-- 
-- Usage:
-- 1. Replace 'YOUR_MINT_ADDRESS_HERE' with actual mint addresses
-- 2. Adjust time windows (3600 = 1h, 1800 = 30m, 300 = 5m)
-- 3. Modify strength thresholds based on your risk tolerance
-- 4. Combine with token_rolling_metrics for deeper analysis
--
-- Performance Tips:
-- - Use indexed columns (mint, signal_type, timestamp)
-- - Limit time ranges for large datasets
-- - Use json_extract() for metadata queries
-- - Consider creating materialized views for frequent queries
