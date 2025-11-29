/**
 * DexScreener API Integration
 * Fetches token metadata, price, market cap, and calculates token age
 * Based on old SolFlow UI implementation
 */

export interface DexPair {
  chainId: string;
  dexId: string;
  url: string;
  pairAddress: string;
  baseToken: {
    address: string;
    name: string;
    symbol: string;
  };
  quoteToken: {
    address: string;
    name: string;
    symbol: string;
  };
  priceNative: string;
  priceUsd: string;
  txns: {
    h24: { buys: number; sells: number };
  };
  volume: {
    h24: number;
  };
  priceChange: {
    h24: number;
  };
  liquidity?: {
    usd: number;
  };
  fdv?: number;
  marketCap?: number;
  pairCreatedAt: number;
}

export interface DexScreenerResponse {
  schemaVersion: string;
  pairs: DexPair[] | null;
}

export interface TokenMetadataResult {
  symbol: string;
  name: string;
  priceUsd: number;
  marketCap: number;
  tokenAge: number;
}

const DEXSCREENER_API_BASE = 'https://api.dexscreener.com/latest/dex';

/**
 * Fetch token metadata from DexScreener
 * Returns null if no pairs found or API error
 */
export async function fetchDexScreenerData(
  mint: string
): Promise<TokenMetadataResult | null> {
  try {
    const response = await fetch(`${DEXSCREENER_API_BASE}/tokens/${mint}`);
    
    if (!response.ok) {
      console.error(`DexScreener API error: ${response.status} ${response.statusText}`);
      return null;
    }

    const data: DexScreenerResponse = await response.json();
    
    if (!data.pairs || data.pairs.length === 0) {
      console.log(`No pairs found for token: ${mint}`);
      return null;
    }

    // Choose best pair: prefer Solana pairs, fallback to first pair
    const pair = chooseBestPair(data.pairs);
    
    if (!pair) {
      return null;
    }

    return parseDexScreenerPair(pair);
  } catch (error) {
    console.error('Failed to fetch DexScreener data:', error);
    return null;
  }
}

/**
 * Choose best pair from array
 * Prefer Solana pairs, fallback to first pair
 */
function chooseBestPair(pairs: DexPair[]): DexPair | null {
  if (pairs.length === 0) return null;

  // Prefer Solana pairs
  const solanaPair = pairs.find(
    (p) => p.chainId.toLowerCase() === 'solana'
  );

  return solanaPair || pairs[0];
}

/**
 * Parse pair data into structured result
 */
function parseDexScreenerPair(pair: DexPair): TokenMetadataResult {
  const symbol = pair.baseToken.symbol;
  const name = pair.baseToken.name;
  const priceUsd = parseFloat(pair.priceUsd) || 0;
  
  // Market cap: prefer FDV, fallback to marketCap
  const marketCap = pair.fdv || pair.marketCap || 0;
  
  // Calculate token age in seconds
  const tokenAge = getTokenAge(pair.pairCreatedAt);

  return {
    symbol,
    name,
    priceUsd,
    marketCap,
    tokenAge,
  };
}

/**
 * Calculate token age from pair creation timestamp
 * Returns age in seconds
 */
function getTokenAge(pairCreatedAt: number): number {
  const nowMs = Date.now();
  const createdMs = pairCreatedAt;
  const ageMs = nowMs - createdMs;
  return Math.floor(ageMs / 1000);
}

/**
 * Format token age into human-readable string
 * Examples: "2h", "1d", "3w"
 */
export function formatTokenAge(ageSeconds: number | null): string {
  if (ageSeconds === null) return '—';

  const minutes = Math.floor(ageSeconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);
  const weeks = Math.floor(days / 7);

  if (weeks > 0) return `${weeks}w`;
  if (days > 0) return `${days}d`;
  if (hours > 0) return `${hours}h`;
  if (minutes > 0) return `${minutes}m`;
  return '<1m';
}

/**
 * Format price with appropriate precision
 */
export function formatPrice(price: number | null): string {
  if (price === null) return '—';
  
  if (price >= 1) {
    return `$${price.toFixed(2)}`;
  }
  
  if (price >= 0.01) {
    return `$${price.toFixed(4)}`;
  }
  
  if (price >= 0.0001) {
    return `$${price.toFixed(6)}`;
  }
  
  // Very small values: use scientific notation
  return `$${price.toExponential(2)}`;
}

/**
 * Format market cap with K/M/B suffixes
 */
export function formatMarketCap(marketCap: number | null): string {
  if (marketCap === null) return '—';
  
  if (marketCap >= 1_000_000_000) {
    return `$${(marketCap / 1_000_000_000).toFixed(2)}B`;
  }
  
  if (marketCap >= 1_000_000) {
    return `$${(marketCap / 1_000_000).toFixed(2)}M`;
  }
  
  if (marketCap >= 1_000) {
    return `$${(marketCap / 1_000).toFixed(2)}K`;
  }
  
  return `$${marketCap.toFixed(0)}`;
}
