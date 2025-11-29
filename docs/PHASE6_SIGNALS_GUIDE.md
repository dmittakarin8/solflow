# Phase 6: Signals Engine - Complete Implementation Guide

## Overview

Phase 6 implements a complete signals engine that evaluates 5 actionable trading signals based on rolling metrics and trade patterns. Signals are computed on every metrics update and persisted to the database for Phase 7 dashboard consumption.

## Architecture

```
┌─────────────────────┐
│  Token Trades       │
│  (Phase 5)          │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐       ┌─────────────────────┐
│  Rolling Metrics    │──────▶│  Signals Engine     │
│  (Phase 5)          │       │  (Phase 6)          │
└─────────────────────┘       └──────────┬──────────┘
                                         │
                                         ▼
                              ┌─────────────────────┐
                              │  token_signals      │
                              │  (Database)         │
                              └─────────────────────┘
                                         │
                                         ▼
                              ┌─────────────────────┐
                              │  Dashboard          │
                              │  (Phase 7)          │
                              └─────────────────────┘
```

## Implemented Signals

### Signal A: BREAKOUT

**Triggered when:**
- `net_flow_300s` accelerating (300s > 900s)
- AND `net_flow_60s` > `net_flow_300s` (momentum shift)
- AND `unique_wallets_300s` >= 5 (increasing participants)
- AND bot ratio <= 0.3 (within normal bounds)

**Strength Calculation:**
- Acceleration factor: (300s - 900s) / 900s
- Momentum factor: 60s / 300s
- Wallet factor: unique_wallets / 20
- Bot factor: 1 - bot_ratio

**Use Case:** Identifies tokens with accelerating momentum and increasing participation.

### Signal B: REACCUMULATION

**Triggered when:**
- DCA flow > 0 and DCA unique wallets >= 2
- AND `net_flow_300s` > 0 (positive total flow)
- AND `net_flow_300s` > `net_flow_900s` (momentum shift)

**Strength Calculation:**
- DCA factor: dca_flow / 10
- Wallet factor: dca_wallets / 5
- Flow factor: net_flow_300s / 50
- Momentum factor: (300s - 900s) / |900s|

**Use Case:** Detects accumulation phases with DCA activity signaling conviction.

### Signal C: FOCUSED BUYERS

**Triggered when:**
- F-score <= 0.35 (35% of wallets responsible for 70%+ of inflow)
- AND positive net_flow trend

**Strength Calculation:**
- Concentration factor: 1 - (f_score / 0.35)
- Flow factor: net_flow_300s / 50

**Use Case:** Identifies whale accumulation with low entropy wallet distribution.

### Signal D: PERSISTENCE

**Triggered when:**
- Positive net_flow across 3 consecutive windows (60s, 300s, 900s)
- AND unique_wallets >= 5 (sustained participation)
- AND bot ratio <= 0.4 (no bot surge)

**Strength Calculation:**
- Flow consistency: 1 - |60s - 300s| / 300s
- Flow magnitude: 900s / 100
- Wallet factor: unique_wallets / 20
- Bot factor: 1 - bot_ratio

**Use Case:** Detects sustained momentum without collapse.

### Signal E: FLOW REVERSAL

**Triggered when:**
- `net_flow_60s` < 0 (negative recent flow)
- AND `net_flow_300s` > 0 (still positive medium-term)
- AND wallets_per_trade < 0.5 (participant drop)

**Strength Calculation:**
- Divergence factor: (300s - 60s) / 300s
- Flow magnitude: 300s / 50

**Use Case:** Early warning signal for momentum exhaustion.

## Database Schema

### Table: `token_signals`

```sql
CREATE TABLE IF NOT EXISTS token_signals (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    mint            TEXT NOT NULL,
    signal_type     TEXT NOT NULL,
    strength        REAL DEFAULT 0.0,
    window          TEXT,
    timestamp       INTEGER,
    metadata        TEXT,
    -- Legacy columns (Phase 1-5)
    window_seconds  INTEGER,
    severity        INTEGER,
    score           REAL,
    details_json    TEXT,
    created_at      INTEGER,
    sent_to_discord INTEGER DEFAULT 0,
    seen_in_terminal INTEGER DEFAULT 0
);

CREATE INDEX idx_token_signals_mint ON token_signals(mint);
CREATE INDEX idx_token_signals_type ON token_signals(signal_type);
CREATE INDEX idx_token_signals_timestamp ON token_signals(timestamp DESC);
CREATE INDEX idx_token_signals_strength ON token_signals(strength DESC);
```

## Querying Signals

### Get Recent Signals for a Token

```sql
SELECT 
    signal_type,
    strength,
    window,
    timestamp,
    metadata
FROM token_signals
WHERE mint = 'YOUR_MINT_ADDRESS'
ORDER BY timestamp DESC
LIMIT 10;
```

### Get All Breakout Signals in Last Hour

```sql
SELECT 
    mint,
    strength,
    timestamp,
    json_extract(metadata, '$.net_flow_300s') as net_flow,
    json_extract(metadata, '$.unique_wallets') as wallets
FROM token_signals
WHERE signal_type = 'BREAKOUT'
  AND timestamp >= strftime('%s', 'now') - 3600
ORDER BY strength DESC;
```

### Get Top Strength Signals by Type

```sql
SELECT 
    signal_type,
    mint,
    MAX(strength) as max_strength,
    COUNT(*) as signal_count
FROM token_signals
WHERE timestamp >= strftime('%s', 'now') - 3600
GROUP BY signal_type, mint
ORDER BY max_strength DESC
LIMIT 20;
```

### Get Focused Buyers with F-Score

```sql
SELECT 
    mint,
    strength,
    json_extract(metadata, '$.f_score') as f_score,
    json_extract(metadata, '$.wallets_needed') as wallets_needed,
    json_extract(metadata, '$.total_wallets') as total_wallets,
    timestamp
FROM token_signals
WHERE signal_type = 'FOCUSED_BUYERS'
ORDER BY timestamp DESC
LIMIT 10;
```

### Get Tokens with Multiple Simultaneous Signals

```sql
SELECT 
    mint,
    COUNT(DISTINCT signal_type) as signal_count,
    GROUP_CONCAT(signal_type) as signals,
    AVG(strength) as avg_strength
FROM token_signals
WHERE timestamp >= strftime('%s', 'now') - 300
GROUP BY mint
HAVING signal_count >= 2
ORDER BY signal_count DESC, avg_strength DESC;
```

### Get DCA Reaccumulation Details

```sql
SELECT 
    mint,
    strength,
    json_extract(metadata, '$.dca_flow') as dca_flow,
    json_extract(metadata, '$.dca_wallets') as dca_wallets,
    json_extract(metadata, '$.dca_ratio') as dca_ratio,
    timestamp
FROM token_signals
WHERE signal_type = 'REACCUMULATION'
ORDER BY timestamp DESC;
```

### Get Flow Reversal Early Warnings

```sql
SELECT 
    mint,
    strength,
    json_extract(metadata, '$.net_flow_60s') as flow_60s,
    json_extract(metadata, '$.net_flow_300s') as flow_300s,
    json_extract(metadata, '$.wallets_per_trade') as wallets_per_trade,
    timestamp
FROM token_signals
WHERE signal_type = 'FLOW_REVERSAL'
  AND timestamp >= strftime('%s', 'now') - 1800
ORDER BY strength DESC;
```

## Testing

Run the comprehensive test suite:

```bash
cargo test signals::
```

Test coverage:
- ✅ Breakout signal triggering
- ✅ Reaccumulation signal triggering
- ✅ Focused buyers F-score calculation
- ✅ Persistence signal across windows
- ✅ Flow reversal divergence detection
- ✅ Strength bounds validation (0.0 - 1.0)
- ✅ Metadata completeness
- ✅ Edge cases (zero trades, negative flows, etc.)

## Performance Considerations

1. **In-Memory Trade Data**: Signals use in-memory `trades_300s` from `TokenRollingState` for performance, avoiding DB queries on every update.

2. **Batched Writes**: Signals are sent through the async write channel and batched with metrics/trades for efficient DB writes.

3. **Selective Evaluation**: Signals only evaluate when conditions are met, avoiding unnecessary computation.

## Runtime Behavior

When signals are triggered, you'll see logs like:

```
🔔 SIGNAL | Mint: ABC123... | Type: Breakout | Strength: 0.76 | Window: 300s | Metadata: {"net_flow_60s":60.0,"net_flow_300s":50.0,...}
```

Signals are:
1. Evaluated on every trade event (after metrics update)
2. Logged to console with emoji prefix 🔔
3. Sent to database writer (non-blocking)
4. Persisted to `token_signals` table

## Integration with Phase 7 Dashboard

The dashboard can consume signals via:

1. **REST API** (to be implemented):
   ```
   GET /api/signals?mint={mint}&limit=10
   GET /api/signals/recent?type={signal_type}
   GET /api/signals/top?window=1h
   ```

2. **WebSocket Updates** (to be implemented):
   ```
   ws://localhost:8080/signals
   {"type": "BREAKOUT", "mint": "...", "strength": 0.76}
   ```

3. **Direct DB Queries** (shown above)

## Files Modified/Created

### New Files
- `sql/10_phase6_signals_engine.sql` - Database migration

### Modified Files
- `src/signals.rs` - Complete signals engine with 5 signal types
- `src/db.rs` - Added `write_signal()` and `get_recent_trades()`
- `src/processor.rs` - Integrated signal evaluation
- `src/main.rs` - Added signals module

### Test Coverage
- 17 new tests in `src/signals.rs`
- All tests passing ✅

## Next Steps (Phase 7)

1. Build REST API for signal queries
2. Implement WebSocket for real-time signal updates
3. Create dashboard UI to visualize signals
4. Add signal filtering and alerting
5. Implement signal performance tracking (hit rate, profitability)

## Example Usage

```rust
use solflow::signals;

// Evaluate signals for a token
let signals = signals::evaluate_signals(
    "YOUR_MINT_ADDRESS",
    &rolling_metrics,
    &recent_trades
);

for signal in signals {
    println!("Signal: {:?}", signal.signal_type);
    println!("Strength: {:.2}", signal.strength);
    println!("Window: {}", signal.window);
    println!("Metadata: {}", signal.metadata);
}
```

## Notes

- Signals are **not** deduplicated - the same signal can trigger multiple times if conditions persist
- Use `timestamp` to filter recent signals and avoid duplicates in downstream consumers
- Strength scores are normalized to [0.0, 1.0] for consistent comparison
- Metadata is stored as JSON for flexible querying and future expansion
