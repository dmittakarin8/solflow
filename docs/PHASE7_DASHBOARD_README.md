# SolFlow Dashboard - Quick Reference

## 🚀 Start the Dashboard

```bash
# Terminal 1: Start Rust backend (must be running to populate data)
cd /home/dgem8/projects/solflow
cargo run --release

# Terminal 2: Start Next.js frontend
cd /home/dgem8/projects/solflow/frontend
npm run dev
```

Navigate to: **http://localhost:3000** (redirects to `/dashboard`)

---

## 📊 Dashboard Features

### Columns (8 total)

| Column | Description | Sortable | Notes |
|--------|-------------|----------|-------|
| ★ | Follow toggle | No | Stores in LocalStorage |
| Token | Symbol/Mint + Name | Yes | Links to token detail |
| Flow 5m | net_flow_300s | Yes | Primary sort (default) |
| Flow 1m | net_flow_60s | Yes | Short-term momentum |
| Wallets | unique_wallets_300s | Yes | Shows bot count if >0 |
| DCA | dca_unique_wallets_300s | Yes | Green if >0 |
| Signals | Latest signal badge | No | Shows type + strength % |
| Trend | Sparkline (6 windows) | No | Visual flow trend |

### Signal Types

| Badge | Icon | Color | Meaning |
|-------|------|-------|---------|
| Breakout | 📈 TrendingUp | Blue | Momentum acceleration |
| Reaccum | 🔄 Repeat | Green | DCA accumulation |
| Focused | 👥 Users | Purple | Whale concentration |
| Persist | 📊 Activity | Orange | Sustained momentum |
| Reversal | ⚠️ AlertTriangle | Red | Early exhaustion |

### Interactive Features

- **Follow Tokens:** Click ★ to follow/unfollow (persists in LocalStorage)
- **Sort:** Click any column header to sort (asc/desc toggle)
- **Navigate:** Click token to view details (not yet implemented)
- **Auto-Refresh:** Data updates every 10 seconds
- **Followed First:** Followed tokens always appear at top

---

## 🏗️ Architecture

### Component Hierarchy

```
app/dashboard/page.tsx (Server)
    ↓ initialTokens
DashboardClient.tsx (Client)
    ↓ tokens + sortConfig
DashboardTable.tsx
    ↓ map tokens
DashboardRow.tsx (per token)
    ├── FollowButton.tsx
    ├── SignalBadge.tsx
    └── FlowSparkline.tsx
```

### Data Flow

```
SQLite DB
    ↓ getDashboardTokens()
Server Component (initial fetch)
    ↓ initialTokens prop
Client Component (state + polling)
    ↓ fetch('/api/dashboard') every 10s
Table (re-render with new data)
```

---

## 📁 Component Files

```
components/dashboard/
├── DashboardClient.tsx     # Polling, sorting, state (client)
├── DashboardTable.tsx      # Table with sortable headers
├── DashboardRow.tsx        # Single token row
├── DashboardSkeleton.tsx   # Loading state
├── FollowButton.tsx        # Star toggle (LocalStorage)
├── SignalBadge.tsx         # Signal badge with icon
└── FlowSparkline.tsx       # SVG sparkline (6 windows)
```

---

## 🔧 Troubleshooting

### Empty Dashboard

**Problem:** No tokens showing

**Solutions:**
1. Check Rust backend is running: `cargo run --release`
2. Check database exists: `ls -lh solflow.db`
3. Check database has data:
   ```bash
   sqlite3 solflow.db "SELECT COUNT(*) FROM token_rolling_metrics;"
   ```
4. Check database path in `.env.local`:
   ```bash
   cat frontend/.env.local
   # Should show: SOLFLOW_DB_PATH=/home/dgem8/projects/solflow/solflow.db
   ```

### Follow Button Not Working

**Problem:** Star doesn't toggle

**Solution:** Check browser console for errors. LocalStorage might be disabled.

### No Auto-Refresh

**Problem:** Data doesn't update after 10s

**Solutions:**
1. Check browser console for fetch errors
2. Check API route works: `curl http://localhost:3000/api/dashboard`
3. Ensure polling is enabled (check usePolling hook)

### Build Errors

**Problem:** `npm run build` fails

**Solution:**
```bash
cd frontend
rm -rf .next node_modules package-lock.json
npm install
npm run build
```

---

## 🎨 Customization

### Change Polling Interval

Edit `components/dashboard/DashboardClient.tsx`:

```tsx
usePolling(fetchTokens, 10000, true); // 10s
                    ↑
                 Change to 5000 for 5s
```

### Change Default Sort

Edit `components/dashboard/DashboardClient.tsx`:

```tsx
const [sortConfig, setSortConfig] = useState<SortConfig>({
  key: 'net_flow_300s',  // Change to any column key
  direction: 'desc',      // 'asc' or 'desc'
});
```

### Add More Columns

1. Add column in `DashboardTable.tsx` header
2. Add cell in `DashboardRow.tsx`
3. Update `DashboardToken` type if needed

---

## 📊 Database Queries

### Top Tokens by Net Flow

```sql
SELECT 
  mint,
  net_flow_300s,
  unique_wallets_300s,
  updated_at
FROM token_rolling_metrics
WHERE updated_at >= (strftime('%s', 'now') - 300)
ORDER BY net_flow_300s DESC
LIMIT 20;
```

### Tokens with Signals

```sql
SELECT 
  trm.mint,
  trm.net_flow_300s,
  ts.signal_type,
  ts.strength
FROM token_rolling_metrics trm
JOIN token_signals ts ON trm.mint = ts.mint
WHERE trm.updated_at >= (strftime('%s', 'now') - 300)
  AND ts.timestamp >= (strftime('%s', 'now') - 1800)
ORDER BY ts.strength DESC;
```

### Tokens with DCA Activity

```sql
SELECT 
  mint,
  net_flow_300s,
  dca_unique_wallets_300s,
  dca_ratio_300s
FROM token_rolling_metrics
WHERE dca_unique_wallets_300s > 0
ORDER BY dca_unique_wallets_300s DESC;
```

---

## 🧪 API Endpoints

### GET /api/dashboard

Returns top 100 tokens by net_flow_300s with latest signals.

**Query Params:**
- `limit` (default: 100) - Max tokens to return
- `minAge` (default: 300) - Min seconds since last update

**Example:**
```bash
curl "http://localhost:3000/api/dashboard?limit=20&minAge=300"
```

**Response:**
```json
{
  "tokens": [
    {
      "mint": "ABC123...",
      "net_flow_60s": 10.5,
      "net_flow_300s": 45.2,
      "unique_wallets_300s": 12,
      "latest_signal_type": "BREAKOUT",
      "latest_signal_strength": 0.76,
      ...
    }
  ],
  "timestamp": 1701234567
}
```

---

## ✅ Current Status

**Phase 7 - Session 2:** ✅ **COMPLETE**

### Working Features
- ✅ Dashboard table with 8 columns
- ✅ Sortable columns
- ✅ Follow/unfollow (LocalStorage)
- ✅ Signal badges (5 types)
- ✅ Flow sparklines (6 windows)
- ✅ Auto-refresh (10s polling)
- ✅ Bot detection indicators
- ✅ DCA count display
- ✅ Skeleton loading states
- ✅ Dark mode UI

### Pending Features (Session 3)
- [ ] Token detail page (`/token/[mint]`)
- [ ] Full signal history timeline
- [ ] Recent trades table
- [ ] Market cap fetching
- [ ] External links (Dexscreener, etc.)

---

## 📚 Related Documentation

- **Phase 7 Analysis:** `PHASE7_ANALYSIS.md`
- **Session 1 Summary:** `PHASE7_SESSION1_SUMMARY.md`
- **Session 2 Summary:** `PHASE7_SESSION2_SUMMARY.md`
- **Quick Start:** `PHASE7_QUICKSTART.md`

---

## 🎯 Key Files to Know

| File | Purpose | Type |
|------|---------|------|
| `app/dashboard/page.tsx` | Main dashboard page | Server |
| `components/dashboard/DashboardClient.tsx` | State management + polling | Client |
| `lib/server/db.ts` | SQLite queries | Server |
| `lib/types.ts` | TypeScript types | Shared |
| `hooks/useFollowedTokens.ts` | Follow state hook | Client |
| `hooks/usePolling.ts` | Polling hook | Client |

---

## 💡 Tips

1. **Performance:** Dashboard is optimized for 100 tokens. For larger datasets, implement pagination.

2. **Real-time:** Polling interval of 10s is a good balance. Shorter intervals increase load.

3. **Follow State:** Stored in LocalStorage, so it's per-browser. Not synced across devices.

4. **Sparklines:** Pure SVG, no chart library needed. Lightweight and fast.

5. **Sorting:** Followed tokens always appear first, then sorted by selected column.

6. **Empty States:** If no tokens show, check that Rust backend is generating trades.

---

## 🚀 Next: Token Detail Page

Session 3 will implement the token detail page at `/token/[mint]` with:
- Token metadata header
- Metrics panel (6 windows)
- Signal history timeline
- Recent trades table
- Follow button
- External links

Stay tuned! 🎉
