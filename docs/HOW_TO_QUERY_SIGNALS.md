# How to Query Signals - Quick Start Guide

## Quick Reference

### Basic Signal Query
```sql
SELECT signal_type, strength, window, timestamp, metadata
FROM token_signals
WHERE mint = 'YOUR_MINT_ADDRESS'
ORDER BY timestamp DESC
LIMIT 10;
```

### Recent Strong Signals (High Priority)
```sql
SELECT mint, signal_type, strength, datetime(timestamp, 'unixepoch') as time
FROM token_signals
WHERE timestamp >= strftime('%s', 'now') - 1800  -- Last 30 minutes
  AND strength >= 0.6
ORDER BY strength DESC
LIMIT 20;
```

### Signals by Type
```sql
-- BREAKOUT: Momentum acceleration
SELECT * FROM token_signals WHERE signal_type = 'BREAKOUT' ORDER BY timestamp DESC LIMIT 10;

-- REACCUMULATION: DCA activity
SELECT * FROM token_signals WHERE signal_type = 'REACCUMULATION' ORDER BY timestamp DESC LIMIT 10;

-- FOCUSED_BUYERS: Whale accumulation
SELECT * FROM token_signals WHERE signal_type = 'FOCUSED_BUYERS' ORDER BY timestamp DESC LIMIT 10;

-- PERSISTENCE: Sustained momentum
SELECT * FROM token_signals WHERE signal_type = 'PERSISTENCE' ORDER BY timestamp DESC LIMIT 10;

-- FLOW_REVERSAL: Early warning
SELECT * FROM token_signals WHERE signal_type = 'FLOW_REVERSAL' ORDER BY timestamp DESC LIMIT 10;
```

## Interpreting Strength Scores

| Strength | Meaning | Action |
|----------|---------|--------|
| 0.0-0.2 | Weak | Monitor only |
| 0.2-0.5 | Moderate | Consider position |
| 0.5-0.8 | Strong | High confidence |
| 0.8-1.0 | Very Strong | Immediate attention |

## Accessing Metadata

Signal metadata is stored as JSON. Use `json_extract()` to query:

```sql
SELECT 
    mint,
    signal_type,
    strength,
    json_extract(metadata, '$.net_flow_300s') as net_flow,
    json_extract(metadata, '$.unique_wallets') as wallets,
    json_extract(metadata, '$.bot_ratio') as bot_ratio
FROM token_signals
WHERE signal_type = 'BREAKOUT'
ORDER BY timestamp DESC
LIMIT 10;
```

## Common Queries

### 1. Multi-Signal Tokens (High Conviction)
```sql
SELECT 
    mint,
    COUNT(DISTINCT signal_type) as signal_count,
    GROUP_CONCAT(DISTINCT signal_type) as signals,
    AVG(strength) as avg_strength
FROM token_signals
WHERE timestamp >= strftime('%s', 'now') - 300
GROUP BY mint
HAVING signal_count >= 2
ORDER BY signal_count DESC;
```

### 2. Whale Activity (Focused Buyers)
```sql
SELECT 
    mint,
    strength,
    json_extract(metadata, '$.f_score') as f_score,
    json_extract(metadata, '$.wallets_needed') as whale_count
FROM token_signals
WHERE signal_type = 'FOCUSED_BUYERS'
  AND timestamp >= strftime('%s', 'now') - 1800
ORDER BY strength DESC;
```

### 3. DCA Accumulation
```sql
SELECT 
    mint,
    strength,
    json_extract(metadata, '$.dca_flow') as dca_flow,
    json_extract(metadata, '$.dca_wallets') as dca_wallets,
    json_extract(metadata, '$.dca_ratio') as dca_ratio
FROM token_signals
WHERE signal_type = 'REACCUMULATION'
ORDER BY timestamp DESC;
```

### 4. Early Exit Signals (Flow Reversal)
```sql
SELECT 
    mint,
    strength,
    json_extract(metadata, '$.net_flow_60s') as flow_60s,
    json_extract(metadata, '$.net_flow_300s') as flow_300s
FROM token_signals
WHERE signal_type = 'FLOW_REVERSAL'
  AND timestamp >= strftime('%s', 'now') - 1800
ORDER BY timestamp DESC;
```

## Database Connection

### From Command Line
```bash
# Set database path
export SOLFLOW_DB_PATH="./solflow.db"

# Query signals
sqlite3 $SOLFLOW_DB_PATH "SELECT * FROM token_signals ORDER BY timestamp DESC LIMIT 10;"
```

### From Rust Code
```rust
use rusqlite::{Connection, params};

let conn = Connection::open("./solflow.db")?;

let mut stmt = conn.prepare(
    "SELECT signal_type, strength, timestamp 
     FROM token_signals 
     WHERE mint = ?1 
     ORDER BY timestamp DESC"
)?;

let signals = stmt.query_map(params!["YOUR_MINT"], |row| {
    Ok((
        row.get::<_, String>(0)?,
        row.get::<_, f64>(1)?,
        row.get::<_, i64>(2)?,
    ))
})?;

for signal in signals {
    let (signal_type, strength, timestamp) = signal?;
    println!("{} - {:.2}", signal_type, strength);
}
```

## Signal Metadata Fields

### BREAKOUT
```json
{
  "net_flow_60s": 60.0,
  "net_flow_300s": 50.0,
  "net_flow_900s": 40.0,
  "unique_wallets": 15,
  "bot_ratio": 0.14
}
```

### REACCUMULATION
```json
{
  "dca_flow": 15.0,
  "dca_wallets": 3,
  "net_flow_300s": 50.0,
  "net_flow_900s": 40.0,
  "dca_ratio": 0.30
}
```

### FOCUSED_BUYERS
```json
{
  "f_score": 0.25,
  "wallets_needed": 3,
  "total_wallets": 12,
  "net_flow_300s": 50.0,
  "total_inflow": 75.0
}
```

### PERSISTENCE
```json
{
  "net_flow_60s": 10.0,
  "net_flow_300s": 50.0,
  "net_flow_900s": 100.0,
  "unique_wallets": 10,
  "bot_ratio": 0.14
}
```

### FLOW_REVERSAL
```json
{
  "net_flow_60s": -5.0,
  "net_flow_300s": 50.0,
  "unique_wallets": 5,
  "total_trades_60s": 15,
  "wallets_per_trade": 0.33
}
```

## Time Windows

Understanding signal time windows:

- **60s** - Very short-term (1 minute)
- **300s** - Short-term (5 minutes)
- **900s** - Medium-term (15 minutes)
- **3600s** - Long-term (1 hour)

Most signals use 300s as primary window with 60s/900s for context.

## Filtering Best Practices

1. **Time-based**: Always filter by recent timestamp to avoid stale signals
   ```sql
   WHERE timestamp >= strftime('%s', 'now') - 1800  -- Last 30 min
   ```

2. **Strength-based**: Filter by minimum strength for actionable signals
   ```sql
   WHERE strength >= 0.5  -- Only strong signals
   ```

3. **Type-based**: Focus on specific signal types for your strategy
   ```sql
   WHERE signal_type IN ('BREAKOUT', 'REACCUMULATION')
   ```

## Complete Example: Dashboard Query

```sql
-- Real-time dashboard query with all key metrics
SELECT 
    ts.mint,
    ts.signal_type,
    ts.strength,
    ts.window,
    datetime(ts.timestamp, 'unixepoch') as signal_time,
    -- Metadata
    json_extract(ts.metadata, '$.net_flow_300s') as net_flow,
    json_extract(ts.metadata, '$.unique_wallets') as wallets,
    json_extract(ts.metadata, '$.bot_ratio') as bot_ratio,
    -- Current state from metrics
    trm.net_flow_300s as current_flow,
    trm.unique_wallets_300s as current_wallets,
    trm.dca_ratio_300s as current_dca_ratio
FROM token_signals ts
LEFT JOIN token_rolling_metrics trm ON ts.mint = trm.mint
WHERE ts.timestamp >= strftime('%s', 'now') - 600  -- Last 10 minutes
  AND ts.strength >= 0.4
ORDER BY ts.strength DESC, ts.timestamp DESC
LIMIT 50;
```

## Resources

- **Full Query Examples**: See `example_signal_queries.sql` for 20+ queries
- **Signal Details**: See `PHASE6_SIGNALS_GUIDE.md` for signal logic
- **Implementation**: See `PHASE6_SUMMARY.md` for technical details

## Quick Tips

1. **Start with high strength**: Filter `strength >= 0.6` for most reliable signals
2. **Recent only**: Use 30-60 minute windows for active trading
3. **Multi-signal confirmation**: Look for tokens with 2+ simultaneous signals
4. **Check metadata**: Always examine JSON metadata for context
5. **Join with metrics**: Combine with `token_rolling_metrics` for full picture

## Support

For more examples and advanced queries, see:
- `example_signal_queries.sql` - 20 production queries
- `PHASE6_SIGNALS_GUIDE.md` - Complete guide
- `PHASE6_SUMMARY.md` - Technical reference
