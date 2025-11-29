# Phase 7: Quick Start Guide

## 🚀 Start the Frontend

```bash
cd /home/dgem8/projects/solflow/frontend

# Development mode (with Turbopack)
npm run dev

# Production build
npm run build
npm start
```

Navigate to: **http://localhost:3000**

---

## 📋 Pre-Deployment Checklist

### ✅ Session 1 Complete
- [x] Next.js 16 project initialized
- [x] React 19.2 + TypeScript setup
- [x] Cache Components (PPR) enabled
- [x] ShadCN UI + Tailwind 4.0 configured
- [x] Database connection (`lib/server/db.ts`)
- [x] TypeScript types (`lib/types.ts`)
- [x] API routes (4 endpoints)
- [x] Client utilities (hooks, formatters)
- [x] Build successful

### 🔲 Session 2: Dashboard UI (Pending)
- [ ] Create `/dashboard` page
- [ ] Build `TokenTable` component
- [ ] Build `TokenRow` component
- [ ] Implement sorting logic
- [ ] Add live polling (10s)
- [ ] Build `SignalBadge` component
- [ ] Build `FlowSparkline` component
- [ ] Build `BotIndicator` component
- [ ] Build `DcaIndicator` component
- [ ] Add skeleton loading states

### 🔲 Session 3: Token Detail Page (Pending)
- [ ] Create `/token/[mint]` page
- [ ] Build `TokenHeader` component
- [ ] Build `MetricsPanel` component
- [ ] Build `SignalsTimeline` component
- [ ] Build `TradesTable` component
- [ ] Add follow/unfollow button
- [ ] External links (Dexscreener, Birdeye, Solscan)

### 🔲 Session 4: Polish & Deploy (Pending)
- [ ] Update root layout with navigation
- [ ] Add landing page with redirect
- [ ] Add error boundaries
- [ ] Mobile responsive design
- [ ] Performance optimization
- [ ] Production deployment

---

## 🔧 Available API Endpoints

All endpoints are ready and tested:

### 1. Dashboard Data
```bash
GET http://localhost:3000/api/dashboard?limit=100&minAge=300
```

Returns top tokens by net_flow_300s with latest signals.

### 2. Token Details
```bash
GET http://localhost:3000/api/token/YOUR_MINT_ADDRESS
```

Returns full token details (metadata, metrics, signals, trades).

### 3. Signals Query
```bash
# By mint
GET http://localhost:3000/api/signals?mint=YOUR_MINT_ADDRESS

# Recent across all tokens
GET http://localhost:3000/api/signals?minStrength=0.5&limit=20&minAge=1800
```

Returns signal data with metadata.

### 4. Batch Metadata
```bash
GET http://localhost:3000/api/metadata?mints=MINT1,MINT2,MINT3
```

Returns metadata for multiple tokens.

---

## 📁 Project Structure

```
frontend/
├── app/                           # Next.js App Router
│   ├── api/                       # API routes (4 endpoints)
│   │   ├── dashboard/route.ts     # Dashboard data
│   │   ├── token/[mint]/route.ts  # Token details
│   │   ├── signals/route.ts       # Signal queries
│   │   └── metadata/route.ts      # Batch metadata
│   ├── dashboard/                 # Dashboard page (TODO Session 2)
│   ├── token/[mint]/              # Token detail page (TODO Session 3)
│   ├── layout.tsx                 # Root layout
│   └── page.tsx                   # Landing page
│
├── components/                    # React components
│   ├── dashboard/                 # Dashboard-specific (TODO Session 2)
│   ├── token/                     # Token detail-specific (TODO Session 3)
│   ├── modals/                    # Modal dialogs (TODO)
│   └── ui/                        # ShadCN components (6 ready)
│
├── hooks/                         # Custom React hooks
│   ├── useFollowedTokens.ts       # LocalStorage state ✅
│   └── usePolling.ts              # Polling utility ✅
│
├── lib/                           # Utilities and helpers
│   ├── server/                    # Server-side only
│   │   └── db.ts                  # SQLite queries ✅
│   ├── client/                    # Client-side utilities
│   │   └── format.ts              # Formatters ✅
│   ├── types.ts                   # TypeScript types ✅
│   └── utils.ts                   # ShadCN utilities ✅
│
├── .env.local                     # Environment variables ✅
├── next.config.ts                 # Next.js config ✅
├── tsconfig.json                  # TypeScript config ✅
└── package.json                   # Dependencies ✅
```

---

## 🔑 Environment Variables

Required in `.env.local`:

```bash
# Path to SQLite database (absolute path)
SOLFLOW_DB_PATH=/home/dgem8/projects/solflow/solflow.db
```

Optional:

```bash
# External API keys for metadata enrichment
DEXSCREENER_API_KEY=your_key_here
BIRDEYE_API_KEY=your_key_here
```

---

## 🎨 Key Features (Ready to Build)

### Signal Badge Component
```tsx
<SignalBadge 
  type="BREAKOUT" 
  strength={0.76}
  window="300s"
/>
```

Color-coded badges for 5 signal types.

### Flow Sparkline Component
```tsx
<FlowSparkline 
  flows={[60s, 300s, 900s, 3600s, 7200s, 14400s]}
/>
```

Visualize net flow across 6 time windows.

### Token Table
```tsx
<TokenTable 
  tokens={dashboardTokens}
  onSort={handleSort}
  followedTokens={followed}
/>
```

Sortable table with live updates.

---

## 🧪 Testing API Locally

### Test Dashboard Endpoint
```bash
curl http://localhost:3000/api/dashboard | jq
```

Expected: Array of tokens with rolling metrics.

### Test Token Detail
```bash
# Replace YOUR_MINT with actual mint address
curl http://localhost:3000/api/token/YOUR_MINT | jq
```

Expected: Full token details with metadata, metrics, signals, trades.

### Test Signals
```bash
curl "http://localhost:3000/api/signals?minStrength=0.6&limit=10" | jq
```

Expected: Recent strong signals across all tokens.

---

## 📚 Documentation

- **Analysis Report:** `PHASE7_ANALYSIS.md`
- **Session 1 Summary:** `PHASE7_SESSION1_SUMMARY.md`
- **Quick Start:** This file

---

## 🐛 Troubleshooting

### Build Errors
```bash
cd frontend
rm -rf .next node_modules package-lock.json
npm install
npm run build
```

### Database Connection Error
Ensure `SOLFLOW_DB_PATH` in `.env.local` points to a valid SQLite database.

```bash
# Verify database exists
ls -lh /home/dgem8/projects/solflow/solflow.db

# Test database
sqlite3 /home/dgem8/projects/solflow/solflow.db "SELECT COUNT(*) FROM token_rolling_metrics;"
```

### TypeScript Errors
```bash
cd frontend
npx tsc --noEmit
```

---

## 🚀 Next Steps

1. **Start Development Server:**
   ```bash
   cd frontend && npm run dev
   ```

2. **Build Dashboard UI (Session 2):**
   - Create `/dashboard` page
   - Implement `TokenTable` component
   - Add sorting and live polling

3. **Build Token Detail Page (Session 3):**
   - Create `/token/[mint]` page
   - Implement metrics visualization
   - Add signal timeline

4. **Polish & Deploy (Session 4):**
   - Mobile responsiveness
   - Error handling
   - Production build

---

## ✅ Current Status

**Phase 7 - Session 1:** ✅ **COMPLETE**

- Infrastructure: ✅ Ready
- API Routes: ✅ Working
- Database: ✅ Connected
- Types: ✅ Complete
- Utilities: ✅ Ready
- Build: ✅ Successful

**Next:** Dashboard UI implementation (Session 2)
