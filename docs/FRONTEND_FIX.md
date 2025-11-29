# Frontend Database Path Fix

## Issue

The frontend was unable to connect to the database because `.env.local` had an incorrect path.

**Error:**
```
SqliteError: unable to open database file
```

## Root Cause

Mismatch between paths:
- **Main project** `.env`: `SOLFLOW_DB_PATH=/var/lib/solflow/solflow.db`
- **Frontend** `.env.local`: `SOLFLOW_DB_PATH=/home/dgem8/projects/solflow/solflow.db` ❌

## Solution

Updated `frontend/.env.local` to match the main project:

```bash
SOLFLOW_DB_PATH=/var/lib/solflow/solflow.db
```

## Database Status

✅ **Database exists and has data:**
- Location: `/var/lib/solflow/solflow.db`
- Size: 21 MB
- Tokens: 977
- Trades: 25,126
- Signals: 31,453

## How to Verify

1. **Check database exists:**
   ```bash
   ls -lh /var/lib/solflow/solflow.db
   ```

2. **Check database has data:**
   ```bash
   sqlite3 /var/lib/solflow/solflow.db << 'EOF'
   SELECT COUNT(*) as tokens FROM token_rolling_metrics;
   SELECT COUNT(*) as trades FROM token_trades;
   SELECT COUNT(*) as signals FROM token_signals;
   EOF
   ```

3. **Start frontend:**
   ```bash
   cd frontend
   npm run dev
   ```

4. **Open browser:**
   http://localhost:3000

## Important Notes

### Environment Variable Sync

**The frontend `.env.local` MUST match the main project `.env`:**

```bash
# Main project: /home/dgem8/projects/solflow/.env
SOLFLOW_DB_PATH=/var/lib/solflow/solflow.db

# Frontend: /home/dgem8/projects/solflow/frontend/.env.local
SOLFLOW_DB_PATH=/var/lib/solflow/solflow.db  # Must match!
```

### Different Database Locations

If your Rust backend uses a different database location:

1. Check main `.env`:
   ```bash
   grep SOLFLOW_DB_PATH /home/dgem8/projects/solflow/.env
   ```

2. Update frontend `.env.local` to match

3. Restart frontend dev server

### Template File

Created `.env.local.example` with both common paths:
- Production: `/var/lib/solflow/solflow.db`
- Development: `/home/dgem8/projects/solflow/solflow.db`

Copy and uncomment the one you need.

## Frontend Should Now Work

With the correct database path, the dashboard should load successfully showing:
- 977 tokens in the table
- Signal badges
- Flow sparklines
- Auto-refresh every 10s

Enjoy! 🚀
