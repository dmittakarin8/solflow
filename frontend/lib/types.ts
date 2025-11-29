/**
 * TypeScript types matching the SolFlow SQLite database schema
 * Generated from: /sql/00_initial.sql
 */

// ═══════════════════════════════════════════════════════════════════════
// Core Database Types
// ═══════════════════════════════════════════════════════════════════════

export interface TokenMetadata {
  mint: string;
  symbol: string | null;
  name: string | null;
  decimals: number;
  launch_platform: string | null;
  pair_created_at: number | null;
  price_usd: number | null;
  market_cap: number | null;
  token_age: number | null;
  created_at: number;
  updated_at: number;
}

export interface TokenRollingMetrics {
  mint: string;
  updated_at: number;
  
  // Net flow metrics (6 windows: 60s, 300s, 900s, 3600s, 7200s, 14400s)
  net_flow_60s: number;
  net_flow_300s: number;
  net_flow_900s: number;
  net_flow_3600s: number;
  net_flow_7200s: number;
  net_flow_14400s: number;
  
  // Advanced metrics (300s window)
  unique_wallets_300s: number;
  bot_wallets_300s: number;
  bot_trades_300s: number;
  bot_flow_300s: number;
  
  // DCA metrics (300s window)
  dca_flow_300s: number;
  dca_unique_wallets_300s: number;
  dca_ratio_300s: number;
}

export interface TokenTrade {
  id: number;
  mint: string;
  timestamp: number;
  wallet: string;
  side: 'buy' | 'sell' | 'unknown';
  sol_amount: number;
  is_bot: boolean;
  is_dca: boolean;
}

export type SignalType = 
  | 'BREAKOUT'
  | 'REACCUMULATION'
  | 'FOCUSED_BUYERS'
  | 'PERSISTENCE'
  | 'FLOW_REVERSAL';

export interface TokenSignal {
  id: number;
  mint: string;
  signal_type: SignalType;
  strength: number; // 0.0 - 1.0
  window: string; // '60s', '300s', '900s', etc.
  timestamp: number;
  metadata: SignalMetadata;
  created_at: number;
}

// ═══════════════════════════════════════════════════════════════════════
// Signal Metadata Types
// ═══════════════════════════════════════════════════════════════════════

export interface BreakoutMetadata {
  net_flow_60s: number;
  net_flow_300s: number;
  net_flow_900s: number;
  unique_wallets: number;
  bot_ratio: number;
}

export interface ReaccumulationMetadata {
  dca_flow: number;
  dca_wallets: number;
  net_flow_300s: number;
  net_flow_900s: number;
  dca_ratio: number;
}

export interface FocusedBuyersMetadata {
  f_score: number;
  wallets_needed: number;
  total_wallets: number;
  net_flow_300s: number;
  total_inflow: number;
}

export interface PersistenceMetadata {
  net_flow_60s: number;
  net_flow_300s: number;
  net_flow_900s: number;
  unique_wallets: number;
  bot_ratio: number;
}

export interface FlowReversalMetadata {
  net_flow_60s: number;
  net_flow_300s: number;
  unique_wallets: number;
  total_trades_60s: number;
  wallets_per_trade: number;
}

export type SignalMetadata = 
  | BreakoutMetadata
  | ReaccumulationMetadata
  | FocusedBuyersMetadata
  | PersistenceMetadata
  | FlowReversalMetadata;

// ═══════════════════════════════════════════════════════════════════════
// API Response Types
// ═══════════════════════════════════════════════════════════════════════

export interface DashboardToken extends TokenRollingMetrics {
  symbol?: string;
  name?: string;
  price_usd?: number | null;
  market_cap?: number | null;
  token_age?: number | null;
  latest_signal_type: SignalType | null;
  latest_signal_strength: number | null;
}

export interface DashboardResponse {
  tokens: DashboardToken[];
  timestamp: number;
}

export interface TokenDetailResponse {
  metadata: TokenMetadata | null;
  metrics: TokenRollingMetrics | null;
  signals: TokenSignal[];
  trades: TokenTrade[];
}

export interface SignalsResponse {
  signals: TokenSignal[];
}

export interface MetadataResponse {
  [mint: string]: TokenMetadata;
}

// ═══════════════════════════════════════════════════════════════════════
// Frontend-Only Types
// ═══════════════════════════════════════════════════════════════════════

export interface SortConfig {
  key: keyof DashboardToken;
  direction: 'asc' | 'desc';
}

export interface FilterConfig {
  signalTypes: SignalType[];
  minStrength: number;
  minWallets: number;
  showFollowedOnly: boolean;
}

// ═══════════════════════════════════════════════════════════════════════
// Utility Types
// ═══════════════════════════════════════════════════════════════════════

export type TimeWindow = '60s' | '300s' | '900s' | '3600s' | '7200s' | '14400s';

export interface NetFlowData {
  window: TimeWindow;
  value: number;
  label: string;
}

export interface SignalBadgeConfig {
  type: SignalType;
  color: string;
  icon: string;
  bgColor: string;
  textColor: string;
}

// ═══════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════

export const TIME_WINDOWS: TimeWindow[] = ['60s', '300s', '900s', '3600s', '7200s', '14400s'];

export const TIME_WINDOW_LABELS: Record<TimeWindow, string> = {
  '60s': '1m',
  '300s': '5m',
  '900s': '15m',
  '3600s': '1h',
  '7200s': '2h',
  '14400s': '4h',
};

export const SIGNAL_CONFIGS: Record<SignalType, SignalBadgeConfig> = {
  BREAKOUT: {
    type: 'BREAKOUT',
    color: 'blue',
    icon: 'TrendingUp',
    bgColor: 'bg-blue-500/10',
    textColor: 'text-blue-500',
  },
  REACCUMULATION: {
    type: 'REACCUMULATION',
    color: 'green',
    icon: 'Repeat',
    bgColor: 'bg-green-500/10',
    textColor: 'text-green-500',
  },
  FOCUSED_BUYERS: {
    type: 'FOCUSED_BUYERS',
    color: 'purple',
    icon: 'Users',
    bgColor: 'bg-purple-500/10',
    textColor: 'text-purple-500',
  },
  PERSISTENCE: {
    type: 'PERSISTENCE',
    color: 'orange',
    icon: 'Activity',
    bgColor: 'bg-orange-500/10',
    textColor: 'text-orange-500',
  },
  FLOW_REVERSAL: {
    type: 'FLOW_REVERSAL',
    color: 'red',
    icon: 'AlertTriangle',
    bgColor: 'bg-red-500/10',
    textColor: 'text-red-500',
  },
};

export const STRENGTH_THRESHOLDS = {
  VERY_WEAK: 0.0,
  WEAK: 0.2,
  MODERATE: 0.4,
  STRONG: 0.6,
  VERY_STRONG: 0.8,
} as const;

export const STRENGTH_LABELS = {
  [STRENGTH_THRESHOLDS.VERY_WEAK]: 'Very Weak',
  [STRENGTH_THRESHOLDS.WEAK]: 'Weak',
  [STRENGTH_THRESHOLDS.MODERATE]: 'Moderate',
  [STRENGTH_THRESHOLDS.STRONG]: 'Strong',
  [STRENGTH_THRESHOLDS.VERY_STRONG]: 'Very Strong',
} as const;

export const STRENGTH_COLORS = {
  [STRENGTH_THRESHOLDS.VERY_WEAK]: 'text-gray-500',
  [STRENGTH_THRESHOLDS.WEAK]: 'text-yellow-500',
  [STRENGTH_THRESHOLDS.MODERATE]: 'text-orange-500',
  [STRENGTH_THRESHOLDS.STRONG]: 'text-red-500',
  [STRENGTH_THRESHOLDS.VERY_STRONG]: 'text-rose-600',
} as const;
