# Phase 7: Frontend Analysis & Architecture Plan

## Executive Summary

**Status:** Ready to Build
**Approach:** Green-field Next.js 16 implementation with modern React patterns
**Decision:** No existing UI to port — building from scratch based on backend API

---

## 🔍 Key Findings

### No Existing Frontend

After thorough search of the codebase:
- ✅ **No .tsx/.jsx files found**
- ✅ **No package.json (no Node.js project)**
- ✅ **No existing UI components to port**
- ✅ **Pure Rust backend with SQLite database**

**Conclusion:** This is a **green-field frontend project** — we're building fresh UI on top of a mature, production-ready backend.

---

## 🎯 Backend Analysis

### What We Have (Phase 1-6 Complete)

#### 1. Database Schema (SQLite + WAL Mode)
```sql
-- 4 production tables with full indexes
token_metadata           -- Token info, launch platform, decimals
token_rolling_metrics    -- Real-time metrics (6 time windows)
token_trades             -- Append-only trade log (bot/DCA flags)
token_signals            -- 5 signal types with strength + metadata
```

#### 2. Rolling Metrics Engine (6 Time Windows)
- **60s** - Ultra short-term momentum
- **300s** (5 min) - Primary trading window
- **900s** (15 min) - Medium-term trend
- **3600s** (1 hour) - Long-term trend
- **7200s** (2 hours) - Extended trend
- **14400s** (4 hours) - Macro trend

**Metrics Computed:**
- Net flow (buys - sells in SOL)
- Buy/sell counts
- Unique wallets
- Bot detection (rapid trading, MEV patterns)
- DCA detection (Jupiter DCA program)
- Bot flow, DCA flow, DCA ratio

#### 3. Signals Engine (5 Production Signals)

| Signal | Trigger Conditions | Use Case |
|--------|-------------------|----------|
| **BREAKOUT** | net_flow_300s accelerating, momentum shift, increasing wallets, low bot ratio | Early momentum entry |
| **REACCUMULATION** | DCA flow increasing, positive momentum shift, unique DCA wallets ≥2 | Accumulation phase entry |
| **FOCUSED_BUYERS** | F-score ≤ 0.35 (whales), positive flow trend | Whale accumulation detection |
| **PERSISTENCE** | Positive flow across 60s/300s/900s, sustained activity, low bot ratio | Confirmed trend continuation |
| **FLOW_REVERSAL** | 60s negative while 300s positive, wallet concentration | Early exit / exhaustion warning |

**Signal Metadata (JSON):**
- Strength score (0.0 - 1.0)
- Time window (60s, 300s, 900s)
- Timestamp (Unix epoch)
- Rich metadata per signal type

#### 4. Trade Extraction Layer
- **5 DEX decoders:** PumpSwap, Pumpfun, Moonshot, BonkSwap, Jupiter DCA
- **Real-time gRPC ingestion** from Yellowstone/Geyser
- **Bot detection:** Rapid trading, MEV patterns
- **DCA detection:** Jupiter DCA program tracking

---

## 🏗️ Frontend Architecture Plan

### Tech Stack (Next.js 16 Latest)

```
Next.js 16 (Oct 2025)
├── React 19.2
├── App Router (app/)
├── Cache Components (use cache)
├── Partial Pre-Rendering (PPR)
├── Turbopack (default)
├── React Compiler (automatic memoization)
├── Server Actions (API mutations)
└── ShadCN UI (2025 release) + Tailwind + Lucide icons
```

### Why Next.js 16?

- **Cache Components:** Server-side data caching for rolling metrics (invalidate on updates)
- **Partial Pre-Rendering:** Static shell + dynamic data streams (perfect for dashboards)
- **React Compiler:** Automatic memoization (no useMemo/useCallback)
- **Turbopack:** Faster dev builds
- **Server Actions:** Type-safe API calls without REST boilerplate

---

## 📁 Proposed File Structure

```
frontend/
├── app/
│   ├── layout.tsx                      # Root layout (ShadCN setup)
│   ├── page.tsx                        # Landing page → redirect /dashboard
│   │
│   ├── dashboard/
│   │   ├── page.tsx                    # Main sortable token table
│   │   └── loading.tsx                 # Skeleton UI
│   │
│   ├── token/
│   │   └── [mint]/
│   │       ├── page.tsx                # Token detail view
│   │       └── loading.tsx             # Skeleton UI
│   │
│   └── api/                            # REST API routes (Rust backend proxy)
│       ├── dashboard/
│       │   └── route.ts                # GET /api/dashboard → SQLite query
│       ├── signals/
│       │   └── route.ts                # GET /api/signals?mint=X
│       ├── token/
│       │   └── [mint]/
│       │       └── route.ts            # GET /api/token/[mint]
│       ├── followed/
│       │   ├── route.ts                # GET /api/followed
│       │   ├── add/
│       │   │   └── route.ts            # POST /api/followed/add
│       │   └── remove/
│       │       └── route.ts            # POST /api/followed/remove
│       ├── metadata/
│       │   └── route.ts                # GET /api/metadata?mints=X,Y,Z
│       └── marketcap/
│           └── route.ts                # GET /api/marketcap?mint=X
│
├── components/
│   ├── dashboard/
│   │   ├── TokenTable.tsx              # Main sortable table
│   │   ├── TokenRow.tsx                # Table row with metrics
│   │   ├── SignalBadge.tsx             # Signal type + strength badge
│   │   ├── FlowSparkline.tsx           # Net flow sparkline (6 windows)
│   │   ├── DcaIndicator.tsx            # DCA ratio badge
│   │   └── BotIndicator.tsx            # Bot activity indicator
│   │
│   ├── token/
│   │   ├── TokenHeader.tsx             # Token metadata + follow button
│   │   ├── MetricsPanel.tsx            # Rolling metrics cards
│   │   ├── SignalsTimeline.tsx         # Signal history timeline
│   │   ├── TradesTable.tsx             # Recent trades table
│   │   └── FlowChart.tsx               # Time-series net flow chart
│   │
│   ├── modals/
│   │   └── FollowedTokensModal.tsx     # Manage followed tokens
│   │
│   └── ui/                             # ShadCN components
│       ├── button.tsx
│       ├── badge.tsx
│       ├── table.tsx
│       ├── card.tsx
│       ├── dialog.tsx
│       ├── skeleton.tsx
│       └── ... (ShadCN auto-generated)
│
├── lib/
│   ├── db.ts                           # SQLite connection (server-side)
│   ├── queries.ts                      # SQL query builders
│   ├── cache.ts                        # Cache utilities (use cache wrappers)
│   ├── followed.ts                     # LocalStorage followed tokens
│   ├── format.ts                       # Number formatters (SOL, USD, %)
│   ├── signals.ts                      # Signal type definitions + parsers
│   └── types.ts                        # TypeScript types (backend schema)
│
├── hooks/
│   ├── useFollowedTokens.ts            # LocalStorage state
│   ├── usePolling.ts                   # Polling for live updates
│   └── useTokenMetadata.ts             # Fetch metadata on-demand
│
├── styles/
│   └── globals.css                     # Tailwind + ShadCN overrides
│
├── public/
│   └── (static assets)
│
├── package.json
├── next.config.ts                      # Enable PPR, React Compiler
├── tailwind.config.ts                  # ShadCN integration
├── tsconfig.json                       # Strict mode
├── components.json                     # ShadCN config
└── .env.local                          # SOLFLOW_DB_PATH, API keys
```

---

## 🎨 Component Patterns to Implement

### A. Dashboard Page (Main View)

**File:** `app/dashboard/page.tsx`

**Features:**
- ✅ Sortable table (net_flow_300s, strength, wallets, DCA ratio)
- ✅ Live polling (10s interval via `usePolling()`)
- ✅ Followed tokens highlight (LocalStorage)
- ✅ Signal badges (color-coded by type + strength)
- ✅ Net flow sparklines (6 windows: 60s, 300s, 900s, 3600s, 7200s, 14400s)
- ✅ Bot/DCA indicators
- ✅ Click row → navigate to `/token/[mint]`
- ✅ Skeleton loading states

**SQL Query:**
```sql
-- Fetch top 100 tokens by recent activity
SELECT 
    trm.mint,
    trm.net_flow_60s,
    trm.net_flow_300s,
    trm.net_flow_900s,
    trm.net_flow_3600s,
    trm.net_flow_7200s,
    trm.net_flow_14400s,
    trm.unique_wallets_300s,
    trm.bot_wallets_300s,
    trm.bot_trades_300s,
    trm.dca_ratio_300s,
    trm.dca_unique_wallets_300s,
    trm.updated_at,
    -- Latest signal
    (SELECT signal_type 
     FROM token_signals 
     WHERE mint = trm.mint 
     ORDER BY timestamp DESC LIMIT 1) as latest_signal_type,
    (SELECT strength 
     FROM token_signals 
     WHERE mint = trm.mint 
     ORDER BY timestamp DESC LIMIT 1) as latest_signal_strength
FROM token_rolling_metrics trm
WHERE trm.updated_at >= (strftime('%s', 'now') - 300)  -- Last 5 min
ORDER BY trm.net_flow_300s DESC
LIMIT 100;
```

**UI Pattern:**
```tsx
<TokenTable>
  {tokens.map(token => (
    <TokenRow
      key={token.mint}
      mint={token.mint}
      netFlows={[token.net_flow_60s, ..., token.net_flow_14400s]}
      wallets={token.unique_wallets_300s}
      botRatio={token.bot_trades_300s / (token.unique_wallets_300s || 1)}
      dcaRatio={token.dca_ratio_300s}
      signal={token.latest_signal_type}
      signalStrength={token.latest_signal_strength}
      isFollowed={followedTokens.includes(token.mint)}
      onClick={() => router.push(`/token/${token.mint}`)}
    />
  ))}
</TokenTable>
```

---

### B. Token Detail Page

**File:** `app/token/[mint]/page.tsx`

**Features:**
- ✅ Token metadata (symbol, name, decimals, launch platform)
- ✅ Current metrics (6 windows in cards)
- ✅ Signal history timeline (last 20 signals)
- ✅ Recent trades table (last 50 trades with bot/DCA flags)
- ✅ Net flow chart (time-series)
- ✅ Follow/unfollow button
- ✅ External links (Dexscreener, Birdeye, Solscan)

**SQL Queries:**
```sql
-- Token metadata
SELECT * FROM token_metadata WHERE mint = ?;

-- Current metrics
SELECT * FROM token_rolling_metrics WHERE mint = ?;

-- Signal history
SELECT * FROM token_signals 
WHERE mint = ? 
ORDER BY timestamp DESC 
LIMIT 20;

-- Recent trades
SELECT * FROM token_trades 
WHERE mint = ? 
ORDER BY timestamp DESC 
LIMIT 50;
```

---

### C. Signal Rendering Logic

**Component:** `SignalBadge.tsx`

**Signal → Badge Mapping:**

| Signal Type | Color | Icon | Strength Display |
|-------------|-------|------|------------------|
| BREAKOUT | Blue | TrendingUp | `0.76 ⚡` |
| REACCUMULATION | Green | Repeat | `0.65 🔄` |
| FOCUSED_BUYERS | Purple | Users | `0.82 🐋` |
| PERSISTENCE | Orange | Activity | `0.70 📈` |
| FLOW_REVERSAL | Red | AlertTriangle | `0.55 ⚠️` |

**Strength Styling:**
```tsx
const getStrengthColor = (strength: number) => {
  if (strength >= 0.8) return "bg-red-500";      // Very Strong
  if (strength >= 0.6) return "bg-orange-500";   // Strong
  if (strength >= 0.4) return "bg-yellow-500";   // Moderate
  return "bg-gray-500";                          // Weak
};
```

---

### D. Followed Tokens State Machine

**Hook:** `hooks/useFollowedTokens.ts`

**LocalStorage Pattern:**
```tsx
const useFollowedTokens = () => {
  const [followed, setFollowed] = useState<string[]>([]);

  useEffect(() => {
    // Load from LocalStorage
    const stored = localStorage.getItem("solflow_followed");
    if (stored) setFollowed(JSON.parse(stored));
  }, []);

  const addToken = (mint: string) => {
    const updated = [...followed, mint];
    setFollowed(updated);
    localStorage.setItem("solflow_followed", JSON.stringify(updated));
  };

  const removeToken = (mint: string) => {
    const updated = followed.filter(m => m !== mint);
    setFollowed(updated);
    localStorage.setItem("solflow_followed", JSON.stringify(updated));
  };

  return { followed, addToken, removeToken };
};
```

---

### E. Metadata Fetching (On-Demand)

**Pattern:** Batch fetch from Dexscreener/Birdeye

**Hook:** `hooks/useTokenMetadata.ts`

```tsx
const useTokenMetadata = (mints: string[]) => {
  const [metadata, setMetadata] = useState<Record<string, Metadata>>({});

  useEffect(() => {
    // Batch fetch for mints not in DB
    fetch(`/api/metadata?mints=${mints.join(",")}`)
      .then(res => res.json())
      .then(data => setMetadata(data));
  }, [mints]);

  return metadata;
};
```

**Backend API:** `/api/metadata/route.ts`
```ts
// Check DB first, fallback to Dexscreener API
const metadata = await fetchFromDB(mints);
const missing = mints.filter(m => !metadata[m]);
if (missing.length) {
  const external = await fetchDexscreener(missing);
  Object.assign(metadata, external);
}
return Response.json(metadata);
```

---

### F. Live Polling (Dashboard Updates)

**Hook:** `hooks/usePolling.ts`

```tsx
const usePolling = (fn: () => Promise<void>, interval: number) => {
  useEffect(() => {
    fn(); // Initial fetch
    const timer = setInterval(fn, interval);
    return () => clearInterval(timer);
  }, [fn, interval]);
};

// Usage in dashboard
usePolling(async () => {
  const data = await fetch("/api/dashboard").then(r => r.json());
  setTokens(data);
}, 10000); // 10s
```

---

## 🔧 Backend Integration (No Rust Changes Needed)

### SQLite Direct Access (Server-Side Only)

**File:** `lib/db.ts`

```ts
import Database from "better-sqlite3";

const db = new Database(process.env.SOLFLOW_DB_PATH!);

export const getDashboardTokens = () => {
  return db.prepare(`
    SELECT 
      trm.mint,
      trm.net_flow_300s,
      trm.unique_wallets_300s,
      ...
    FROM token_rolling_metrics trm
    WHERE trm.updated_at >= ?
    ORDER BY trm.net_flow_300s DESC
    LIMIT 100
  `).all(Date.now() / 1000 - 300);
};
```

**Why Direct SQLite Access?**
- ✅ Rust backend has no HTTP API (pure data pipeline)
- ✅ SQLite WAL mode supports concurrent reads
- ✅ Next.js server components can read directly
- ✅ No need to modify Rust codebase

**Environment Variable:**
```bash
# .env.local
SOLFLOW_DB_PATH=/home/dgem8/projects/solflow/solflow.db
```

---

## 📊 Data Flow Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Rust Backend (Existing)                  │
├─────────────────────────────────────────────────────────────┤
│  gRPC Stream → Trade Extractor → Rolling Metrics            │
│       ↓                ↓                  ↓                  │
│  SQLite (WAL)   token_trades   token_rolling_metrics        │
│                        ↓                  ↓                  │
│                 Signals Engine    token_signals             │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          │ (Direct SQLite Read)
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                  Next.js 16 Frontend (New)                  │
├─────────────────────────────────────────────────────────────┤
│  Server Components (RSC)                                    │
│    ↓                                                         │
│  lib/db.ts → SQLite queries                                 │
│    ↓                                                         │
│  Cache Components (use cache)                               │
│    ↓                                                         │
│  /api/* routes → JSON responses                             │
│    ↓                                                         │
│  Client Components (polling, interactions)                  │
│    ↓                                                         │
│  Dashboard / Token Detail Pages                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 🎯 Implementation Phases

### Phase 1: Project Setup (Day 1)
- ✅ Initialize Next.js 16 with App Router
- ✅ Configure React Compiler + PPR
- ✅ Setup ShadCN UI + Tailwind
- ✅ Install better-sqlite3 for DB access
- ✅ Create TypeScript types from DB schema
- ✅ Setup environment variables

### Phase 2: Backend Integration (Day 1-2)
- ✅ Create `lib/db.ts` with SQLite connection
- ✅ Implement SQL query builders
- ✅ Create `/api/dashboard` route
- ✅ Create `/api/signals` route
- ✅ Create `/api/token/[mint]` route
- ✅ Test with real database

### Phase 3: Dashboard UI (Day 2-3)
- ✅ Build TokenTable component
- ✅ Build TokenRow with metrics + signals
- ✅ Implement sorting logic
- ✅ Add FlowSparkline component (6 windows)
- ✅ Add SignalBadge component
- ✅ Implement live polling
- ✅ Add skeleton loading states

### Phase 4: Token Detail Page (Day 3-4)
- ✅ Build TokenHeader with metadata
- ✅ Build MetricsPanel (6 windows in cards)
- ✅ Build SignalsTimeline component
- ✅ Build TradesTable component
- ✅ Add follow/unfollow button
- ✅ External links (Dexscreener, etc.)

### Phase 5: Polish & Optimization (Day 4-5)
- ✅ Implement Cache Components
- ✅ Enable Partial Pre-Rendering
- ✅ Add error boundaries
- ✅ Optimize bundle size
- ✅ Mobile responsive design
- ✅ Add loading states everywhere

---

## 🚀 Key Design Decisions

### 1. Direct SQLite Access vs REST API
**Decision:** Direct SQLite access from Next.js server components

**Reasoning:**
- ✅ Rust backend is a data pipeline (no HTTP server)
- ✅ SQLite WAL mode supports concurrent reads
- ✅ Eliminates network round-trip
- ✅ Type-safe queries with TypeScript
- ✅ No need to modify Rust codebase

### 2. Followed Tokens → LocalStorage
**Decision:** Store followed tokens in browser LocalStorage

**Reasoning:**
- ✅ No database schema changes needed
- ✅ Instant updates (no API calls)
- ✅ Per-user preferences (no auth needed)
- ✅ Easy to implement

### 3. Live Updates → Polling (Not WebSocket)
**Decision:** 10s polling for dashboard updates

**Reasoning:**
- ✅ Simpler implementation (no WebSocket server)
- ✅ Sufficient for 5-minute rolling windows
- ✅ Lower backend complexity
- ✅ Can upgrade to WebSocket later if needed

### 4. Metadata Fetching → Lazy + Cached
**Decision:** Fetch metadata on-demand, cache in DB

**Reasoning:**
- ✅ Not all tokens have metadata (new launches)
- ✅ External APIs (Dexscreener) have rate limits
- ✅ Cache in `token_metadata` table
- ✅ Fallback to mint address if unavailable

### 5. Signal Metadata → JSON Parse in Frontend
**Decision:** Parse JSON metadata in frontend, not backend

**Reasoning:**
- ✅ SQLite stores metadata as TEXT (JSON string)
- ✅ Different signal types have different metadata schemas
- ✅ Frontend can display rich signal details
- ✅ No need for complex backend JSON parsing

---

## 📚 API Endpoint Specifications

### GET /api/dashboard

**Response:**
```json
{
  "tokens": [
    {
      "mint": "ABC123...",
      "net_flow_60s": 10.5,
      "net_flow_300s": 45.2,
      "net_flow_900s": 120.8,
      "net_flow_3600s": 250.0,
      "net_flow_7200s": 400.0,
      "net_flow_14400s": 650.0,
      "unique_wallets_300s": 12,
      "bot_wallets_300s": 2,
      "bot_trades_300s": 6,
      "dca_ratio_300s": 0.22,
      "dca_unique_wallets_300s": 3,
      "updated_at": 1701234567,
      "latest_signal_type": "BREAKOUT",
      "latest_signal_strength": 0.76
    }
  ],
  "timestamp": 1701234567
}
```

### GET /api/signals?mint=X

**Response:**
```json
{
  "signals": [
    {
      "id": 123,
      "mint": "ABC123...",
      "signal_type": "BREAKOUT",
      "strength": 0.76,
      "window": "300s",
      "timestamp": 1701234567,
      "metadata": {
        "net_flow_60s": 60.0,
        "net_flow_300s": 50.0,
        "unique_wallets": 15,
        "bot_ratio": 0.14
      }
    }
  ]
}
```

### GET /api/token/[mint]

**Response:**
```json
{
  "metadata": {
    "mint": "ABC123...",
    "symbol": "TEST",
    "name": "Test Token",
    "decimals": 6,
    "launch_platform": "pumpswap"
  },
  "metrics": {
    "net_flow_60s": 10.5,
    "net_flow_300s": 45.2,
    ...
  },
  "signals": [ ... ],
  "trades": [ ... ]
}
```

### POST /api/followed/add

**Request:**
```json
{ "mint": "ABC123..." }
```

**Response:**
```json
{ "success": true }
```

*(LocalStorage-based, no server storage)*

---

## ✅ Success Criteria

### Functional Requirements
- ✅ Dashboard displays top 100 active tokens
- ✅ Sortable by net_flow_300s, strength, wallets, DCA ratio
- ✅ Signal badges color-coded by type + strength
- ✅ Net flow sparklines (6 windows)
- ✅ Token detail page with full metrics + signal history
- ✅ Follow/unfollow tokens (LocalStorage)
- ✅ Live updates (10s polling)
- ✅ Mobile responsive

### Performance Requirements
- ✅ Dashboard loads in <1s (cache components)
- ✅ Polling doesn't block UI (background fetching)
- ✅ Skeleton loading states (no blank screens)
- ✅ SQLite queries <50ms (indexed)
- ✅ Bundle size <500KB (treeshaking)

### Code Quality
- ✅ TypeScript strict mode
- ✅ ESLint + Prettier
- ✅ Component composition (DRY)
- ✅ Consistent naming conventions
- ✅ No console warnings

---

## 🎨 UX Patterns (Minimal, Clean, Modern)

### Design Principles
- **Minimal:** No unnecessary UI chrome, focus on data
- **Clean:** Lots of whitespace, clear hierarchy
- **Modern:** Glassmorphism, subtle shadows, smooth transitions
- **Fast:** Skeleton loading, optimistic UI updates
- **Accessible:** ARIA labels, keyboard navigation

### Color Palette (ShadCN Default + Custom)
- **Background:** `hsl(222.2 84% 4.9%)` (dark slate)
- **Foreground:** `hsl(210 40% 98%)` (off-white)
- **Primary:** `hsl(217.2 91.2% 59.8%)` (blue)
- **Signal Colors:**
  - BREAKOUT: Blue (`hsl(217 91% 60%)`)
  - REACCUMULATION: Green (`hsl(142 76% 36%)`)
  - FOCUSED_BUYERS: Purple (`hsl(262 83% 58%)`)
  - PERSISTENCE: Orange (`hsl(25 95% 53%)`)
  - FLOW_REVERSAL: Red (`hsl(0 84% 60%)`)

### Typography
- **Headings:** Inter (ShadCN default)
- **Body:** Inter
- **Monospace:** JetBrains Mono (numbers, mint addresses)

---

## 📦 Dependencies to Install

```json
{
  "dependencies": {
    "next": "^16.0.0",
    "react": "^19.2.0",
    "react-dom": "^19.2.0",
    "better-sqlite3": "^11.0.0",
    "lucide-react": "^0.460.0",
    "recharts": "^2.13.0",
    "date-fns": "^4.1.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.5.0"
  },
  "devDependencies": {
    "@types/node": "^22.0.0",
    "@types/react": "^19.0.0",
    "@types/better-sqlite3": "^7.6.0",
    "typescript": "^5.6.0",
    "tailwindcss": "^4.0.0",
    "postcss": "^8.4.0",
    "autoprefixer": "^10.4.0",
    "eslint": "^9.0.0",
    "eslint-config-next": "^16.0.0",
    "prettier": "^3.0.0",
    "@tailwindcss/typography": "^0.5.0"
  }
}
```

---

## 🚦 Next Steps

1. **Initialize Next.js 16 project** (this session)
2. **Setup ShadCN UI + Tailwind** (this session)
3. **Create TypeScript types from DB schema** (this session)
4. **Build backend API routes** (next session)
5. **Implement dashboard UI** (next session)
6. **Implement token detail page** (next session)
7. **Polish & deploy** (next session)

---

## 📝 Notes & Considerations

### Cache Invalidation Strategy
- **Dashboard:** Revalidate every 10s (polling)
- **Token Detail:** Revalidate on navigation (PPR)
- **Metadata:** Cache in DB, fallback to API
- **Signals:** Revalidate on token change

### Error Handling
- **DB Connection Errors:** Show fallback UI
- **API Errors:** Toast notifications
- **Empty States:** Helpful messages ("No signals yet")
- **Loading States:** Skeleton UI everywhere

### Mobile Responsiveness
- **Dashboard:** Scroll horizontally on mobile
- **Token Detail:** Stack cards vertically
- **Modals:** Full-screen on mobile
- **Typography:** Scale down on mobile

### Accessibility
- **Keyboard Navigation:** Tab through rows, press Enter to view
- **Screen Readers:** ARIA labels on badges, indicators
- **Color Contrast:** WCAG AA compliant
- **Focus States:** Visible focus rings

---

## ✨ Summary

**Status:** Ready to build! 🚀

**Key Decisions:**
- ✅ Green-field Next.js 16 project (no existing UI to port)
- ✅ Direct SQLite access from Next.js server components
- ✅ Cache Components + PPR for optimal performance
- ✅ LocalStorage for followed tokens (no auth needed)
- ✅ 10s polling for live updates (no WebSocket complexity)
- ✅ ShadCN UI + Tailwind + Lucide icons
- ✅ Minimal, clean, modern design

**Next Action:**
Initialize Next.js 16 project with all required dependencies and configurations.
