# SolFlow Setup Instructions

## Prerequisites

- Rust toolchain installed
- Node.js 20+ installed
- Valid Geyser/Yellowstone RPC endpoint
- Authentication token for Geyser

---

## Step-by-Step Setup

### 1. Configure Environment Variables

Create `.env` file in the project root:

```bash
cd /home/dgem8/projects/solflow

cat > .env << 'EOF'
# Geyser RPC endpoint
GEYSER_URL=your_geyser_endpoint_here

# Authentication token
X_TOKEN=your_auth_token_here

# Database path (will be created automatically)
SOLFLOW_DB_PATH=./solflow.db

# Logging level
RUST_LOG=info
EOF
```

### 2. Start the Rust Backend

The Rust backend will:
- Connect to Geyser RPC
- Create the SQLite database
- Initialize all tables and indexes
- Start processing trades in real-time

```bash
cd /home/dgem8/projects/solflow

# Build and run
cargo run --release
```

**Expected Output:**
```
🗄️  Initializing database
✅ Executed 11 migrations successfully
📝 Spawning database write loop
🚀 Initializing SolFlow Pipeline
📡 Connecting to Geyser: ...
📝 Database write loop started
🔧 Building Pipeline with 5 DEX Decoders

📊 TRADE | Mint: ABC... | Dir: Buy | SOL: 5.2 | Bot: false | DCA: false
```

**Wait for:**
- "✅ Initial schema applied" - Database created
- "📊 TRADE" logs - Trades being processed

This may take 1-2 minutes on first run.

### 3. Verify Database Created

```bash
ls -lh /home/dgem8/projects/solflow/solflow.db
```

Should show a file (initially small, ~100KB).

### 4. Check Database Has Data

```bash
sqlite3 /home/dgem8/projects/solflow/solflow.db << 'EOF'
SELECT COUNT(*) as total_tokens FROM token_rolling_metrics;
SELECT COUNT(*) as total_trades FROM token_trades;
SELECT COUNT(*) as total_signals FROM token_signals;
EOF
```

**Expected:**
- Tokens: Growing over time (as trades occur)
- Trades: Should be increasing
- Signals: Generated when conditions are met

### 5. Configure Frontend Environment

```bash
cd /home/dgem8/projects/solflow/frontend

# .env.local should already exist with:
cat .env.local
```

Should contain:
```bash
SOLFLOW_DB_PATH=/home/dgem8/projects/solflow/solflow.db
```

### 6. Install Frontend Dependencies (if not done)

```bash
cd /home/dgem8/projects/solflow/frontend
npm install
```

### 7. Start the Frontend

```bash
cd /home/dgem8/projects/solflow/frontend
npm run dev
```

**Expected Output:**
```
  ▲ Next.js 16.0.5 (Turbopack)
  - Local:        http://localhost:3000
  - Environments: .env.local

 ✓ Starting...
 ✓ Ready in 1.2s
```

### 8. Access Dashboard

Open browser: **http://localhost:3000**

Should redirect to `/dashboard` and show:
- Token table with metrics
- Signal badges
- Flow sparklines
- Auto-refresh every 10s

---

## Troubleshooting

### Problem: "unable to open database file"

**Cause:** Rust backend hasn't created the database yet.

**Solution:**
1. Start Rust backend first: `cargo run --release`
2. Wait for "✅ Initial schema applied" log message
3. Verify database file exists: `ls -lh solflow.db`
4. Refresh frontend

### Problem: Empty Dashboard

**Cause:** No recent trades in database.

**Solutions:**
1. Check Rust backend logs for trade activity
2. Verify Geyser connection is working
3. Check database has data:
   ```bash
   sqlite3 solflow.db "SELECT COUNT(*) FROM token_rolling_metrics;"
   ```
4. If zero, wait for trades to be processed (may take a few minutes depending on market activity)

### Problem: Frontend Build Error

**Cause:** Dependencies not installed or TypeScript errors.

**Solution:**
```bash
cd frontend
rm -rf .next node_modules package-lock.json
npm install
npm run build
```

### Problem: Rust Backend Won't Start

**Cause:** Missing environment variables or invalid Geyser endpoint.

**Solution:**
1. Check `.env` file exists in project root
2. Verify `GEYSER_URL` is valid
3. Verify `X_TOKEN` is correct
4. Check Rust logs: `RUST_LOG=debug cargo run --release`

### Problem: Port Already in Use

**Frontend (3000):**
```bash
# Kill process on port 3000
lsof -ti:3000 | xargs kill -9
```

**Rust Backend (if using HTTP):**
Check `Cargo.toml` for configured port and kill accordingly.

---

## Architecture Overview

```
┌─────────────────────────────────────────┐
│  Geyser RPC (Solana Real-time Stream)  │
└────────────┬────────────────────────────┘
             │
             v
┌─────────────────────────────────────────┐
│  Rust Backend (SolFlow)                 │
│  - Trade extraction (5 DEX decoders)    │
│  - Rolling metrics engine               │
│  - Signal detection                     │
│  - SQLite persistence                   │
└────────────┬────────────────────────────┘
             │
             v
┌─────────────────────────────────────────┐
│  SQLite Database (solflow.db)           │
│  - token_rolling_metrics                │
│  - token_trades                         │
│  - token_signals                        │
│  - token_metadata                       │
└────────────┬────────────────────────────┘
             │
             v
┌─────────────────────────────────────────┐
│  Next.js 16 Frontend                    │
│  - Direct SQLite read access            │
│  - Real-time dashboard                  │
│  - Signal visualization                 │
│  - Auto-refresh (10s polling)           │
└─────────────────────────────────────────┘
```

---

## Quick Start (After Initial Setup)

### Start Everything

```bash
# Terminal 1: Rust backend
cd /home/dgem8/projects/solflow
cargo run --release

# Terminal 2: Next.js frontend
cd /home/dgem8/projects/solflow/frontend
npm run dev
```

### Stop Everything

- `Ctrl+C` in both terminals

---

## Database Inspection

### View Top Tokens

```bash
sqlite3 solflow.db << 'EOF'
SELECT 
  mint,
  net_flow_300s,
  unique_wallets_300s,
  datetime(updated_at, 'unixepoch') as last_update
FROM token_rolling_metrics
WHERE updated_at >= strftime('%s', 'now') - 300
ORDER BY net_flow_300s DESC
LIMIT 10;
EOF
```

### View Recent Signals

```bash
sqlite3 solflow.db << 'EOF'
SELECT 
  signal_type,
  strength,
  window,
  datetime(timestamp, 'unixepoch') as signal_time
FROM token_signals
ORDER BY timestamp DESC
LIMIT 10;
EOF
```

### View Recent Trades

```bash
sqlite3 solflow.db << 'EOF'
SELECT 
  side,
  sol_amount,
  is_bot,
  is_dca,
  datetime(timestamp, 'unixepoch') as trade_time
FROM token_trades
ORDER BY timestamp DESC
LIMIT 10;
EOF
```

---

## Production Deployment

### Backend

```bash
# Build optimized release
cargo build --release

# Run with process manager (PM2, systemd, etc.)
# Example with PM2:
pm2 start target/release/solflow --name solflow-backend
```

### Frontend

```bash
cd frontend

# Build production bundle
npm run build

# Start production server
npm start

# Or deploy to Vercel/Netlify/etc.
```

---

## Performance Notes

### Database Growth

- **Rolling Metrics:** ~500 bytes per active token (~5MB for 10k tokens)
- **Trades:** ~200 bytes per trade (~200MB for 1M trades)
- **Signals:** ~150 bytes per signal

**Estimated Growth:** 10-50 MB/hour (varies by market activity)

### Cleanup (Optional)

To prune old trades (>7 days):

```bash
sqlite3 solflow.db << 'EOF'
DELETE FROM token_trades 
WHERE timestamp < (strftime('%s', 'now') - 604800);

VACUUM;
EOF
```

---

## Support

For issues:
1. Check Rust backend logs: `RUST_LOG=debug cargo run --release`
2. Check frontend console: Open DevTools in browser
3. Verify database integrity: `sqlite3 solflow.db "PRAGMA integrity_check;"`

---

## Next Steps

Once dashboard is running:
1. Monitor token flow in real-time
2. Follow interesting tokens (click ★)
3. Explore signal detection patterns
4. Click tokens to view details (Session 3 feature)

Enjoy! 🚀
