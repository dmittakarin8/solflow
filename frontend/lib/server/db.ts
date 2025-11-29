/**
 * Server-side SQLite database connection
 * Only import this in Server Components or API routes
 */

import Database from 'better-sqlite3';
import type {
  TokenMetadata,
  TokenRollingMetrics,
  TokenSignal,
  TokenTrade,
  DashboardToken,
} from '../types';

if (!process.env.SOLFLOW_DB_PATH) {
  throw new Error('SOLFLOW_DB_PATH environment variable is not set');
}

// Create singleton database connection with lazy initialization
let db: Database.Database | null = null;

function getDb(): Database.Database {
  if (!db) {
    try {
      db = new Database(process.env.SOLFLOW_DB_PATH!, {
        readonly: true, // Dashboard only reads
        fileMustExist: true,
      });
      
      // Enable WAL mode for better concurrency
      db.pragma('journal_mode = WAL');
    } catch (error) {
      console.error('Failed to connect to database:', error);
      throw new Error(
        `Database connection failed. Please ensure:\n` +
        `1. Rust backend is running (cargo run --release)\n` +
        `2. Database path is correct: ${process.env.SOLFLOW_DB_PATH}\n` +
        `3. Database file exists and is accessible\n\n` +
        `Original error: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }
  return db;
}

// ═══════════════════════════════════════════════════════════════════════
// Dashboard Queries
// ═══════════════════════════════════════════════════════════════════════

export function getDashboardTokens(
  limit: number = 100,
  minAge: number = 300
): DashboardToken[] {
  try {
    const db = getDb();
    const cutoffTime = Math.floor(Date.now() / 1000) - minAge;

    const query = `
      SELECT 
        trm.mint,
        trm.updated_at,
        trm.net_flow_60s,
        trm.net_flow_300s,
        trm.net_flow_900s,
        trm.net_flow_3600s,
        trm.net_flow_7200s,
        trm.net_flow_14400s,
        trm.unique_wallets_300s,
        trm.bot_wallets_300s,
        trm.bot_trades_300s,
        trm.bot_flow_300s,
        trm.dca_flow_300s,
        trm.dca_unique_wallets_300s,
        trm.dca_ratio_300s,
        tm.symbol,
        tm.name,
        tm.price_usd,
        tm.market_cap,
        tm.token_age,
        (SELECT signal_type 
         FROM token_signals 
         WHERE mint = trm.mint 
         ORDER BY timestamp DESC LIMIT 1) as latest_signal_type,
        (SELECT strength 
         FROM token_signals 
         WHERE mint = trm.mint 
         ORDER BY timestamp DESC LIMIT 1) as latest_signal_strength
      FROM token_rolling_metrics trm
      LEFT JOIN token_metadata tm ON trm.mint = tm.mint
      WHERE trm.updated_at >= ?
        AND NOT EXISTS (SELECT 1 FROM blocklist WHERE mint = trm.mint)
      ORDER BY trm.net_flow_300s DESC
      LIMIT ?
    `;

    const stmt = db.prepare(query);
    const rows = stmt.all(cutoffTime, limit) as any[];

    return rows.map((row) => ({
    mint: row.mint,
    updated_at: row.updated_at,
    net_flow_60s: row.net_flow_60s,
    net_flow_300s: row.net_flow_300s,
    net_flow_900s: row.net_flow_900s,
    net_flow_3600s: row.net_flow_3600s,
    net_flow_7200s: row.net_flow_7200s,
    net_flow_14400s: row.net_flow_14400s,
    unique_wallets_300s: row.unique_wallets_300s,
    bot_wallets_300s: row.bot_wallets_300s,
    bot_trades_300s: row.bot_trades_300s,
    bot_flow_300s: row.bot_flow_300s,
    dca_flow_300s: row.dca_flow_300s,
    dca_unique_wallets_300s: row.dca_unique_wallets_300s,
    dca_ratio_300s: row.dca_ratio_300s,
    symbol: row.symbol || undefined,
    name: row.name || undefined,
    price_usd: row.price_usd !== null ? row.price_usd : null,
    market_cap: row.market_cap !== null ? row.market_cap : null,
    token_age: row.token_age !== null ? row.token_age : null,
    latest_signal_type: row.latest_signal_type || null,
    latest_signal_strength: row.latest_signal_strength || null,
  }));
  } catch (error) {
    console.error('getDashboardTokens error:', error);
    return []; // Return empty array instead of crashing
  }
}

// ═══════════════════════════════════════════════════════════════════════
// Token Detail Queries
// ═══════════════════════════════════════════════════════════════════════

export function getTokenMetadata(mint: string): TokenMetadata | null {
  try {
    const db = getDb();
    const query = `
      SELECT * FROM token_metadata WHERE mint = ?
    `;

    const stmt = db.prepare(query);
    const row = stmt.get(mint) as any;

    if (!row) return null;

    return {
      mint: row.mint,
      symbol: row.symbol,
      name: row.name,
      decimals: row.decimals,
      launch_platform: row.launch_platform,
      pair_created_at: row.pair_created_at,
      price_usd: row.price_usd !== null ? row.price_usd : null,
      market_cap: row.market_cap !== null ? row.market_cap : null,
      token_age: row.token_age !== null ? row.token_age : null,
      created_at: row.created_at,
      updated_at: row.updated_at,
    };
  } catch (error) {
    console.error('getTokenMetadata error:', error);
    return null;
  }
}

export function getTokenMetrics(mint: string): TokenRollingMetrics | null {
  try {
    const db = getDb();
    const query = `
      SELECT * FROM token_rolling_metrics WHERE mint = ?
    `;

    const stmt = db.prepare(query);
    const row = stmt.get(mint) as any;

    if (!row) return null;

    return {
      mint: row.mint,
      updated_at: row.updated_at,
      net_flow_60s: row.net_flow_60s,
      net_flow_300s: row.net_flow_300s,
      net_flow_900s: row.net_flow_900s,
      net_flow_3600s: row.net_flow_3600s,
      net_flow_7200s: row.net_flow_7200s,
      net_flow_14400s: row.net_flow_14400s,
      unique_wallets_300s: row.unique_wallets_300s,
      bot_wallets_300s: row.bot_wallets_300s,
      bot_trades_300s: row.bot_trades_300s,
      bot_flow_300s: row.bot_flow_300s,
      dca_flow_300s: row.dca_flow_300s,
      dca_unique_wallets_300s: row.dca_unique_wallets_300s,
      dca_ratio_300s: row.dca_ratio_300s,
    };
  } catch (error) {
    console.error('getTokenMetrics error:', error);
    return null;
  }
}

export function getTokenSignals(mint: string, limit: number = 20): TokenSignal[] {
  try {
    const db = getDb();
    const query = `
      SELECT * FROM token_signals 
      WHERE mint = ? 
      ORDER BY timestamp DESC 
      LIMIT ?
    `;

    const stmt = db.prepare(query);
    const rows = stmt.all(mint, limit) as any[];

    return rows.map((row) => ({
      id: row.id,
      mint: row.mint,
      signal_type: row.signal_type,
      strength: row.strength,
      window: row.window,
      timestamp: row.timestamp,
      metadata: JSON.parse(row.metadata || '{}'),
      created_at: row.created_at,
    }));
  } catch (error) {
    console.error('getTokenSignals error:', error);
    return [];
  }
}

export function getTokenTrades(mint: string, limit: number = 50): TokenTrade[] {
  try {
    const db = getDb();
    const query = `
      SELECT * FROM token_trades 
      WHERE mint = ? 
      ORDER BY timestamp DESC 
      LIMIT ?
    `;

    const stmt = db.prepare(query);
    const rows = stmt.all(mint, limit) as any[];

    return rows.map((row) => ({
      id: row.id,
      mint: row.mint,
      timestamp: row.timestamp,
      wallet: row.wallet,
      side: row.side as 'buy' | 'sell' | 'unknown',
      sol_amount: row.sol_amount,
      is_bot: row.is_bot === 1,
      is_dca: row.is_dca === 1,
    }));
  } catch (error) {
    console.error('getTokenTrades error:', error);
    return [];
  }
}

// ═══════════════════════════════════════════════════════════════════════
// Signal Queries
// ═══════════════════════════════════════════════════════════════════════

export function getRecentSignals(
  minStrength: number = 0.0,
  limit: number = 50,
  minAge: number = 1800
): TokenSignal[] {
  try {
    const db = getDb();
    const cutoffTime = Math.floor(Date.now() / 1000) - minAge;

    const query = `
      SELECT * FROM token_signals 
      WHERE timestamp >= ?
        AND strength >= ?
      ORDER BY strength DESC, timestamp DESC
      LIMIT ?
    `;

    const stmt = db.prepare(query);
    const rows = stmt.all(cutoffTime, minStrength, limit) as any[];

    return rows.map((row) => ({
      id: row.id,
      mint: row.mint,
      signal_type: row.signal_type,
      strength: row.strength,
      window: row.window,
      timestamp: row.timestamp,
      metadata: JSON.parse(row.metadata || '{}'),
      created_at: row.created_at,
    }));
  } catch (error) {
    console.error('getRecentSignals error:', error);
    return [];
  }
}

// ═══════════════════════════════════════════════════════════════════════
// Batch Queries
// ═══════════════════════════════════════════════════════════════════════

export function getMultipleTokenMetadata(mints: string[]): Record<string, TokenMetadata> {
  if (mints.length === 0) return {};

  try {
    const db = getDb();
    const placeholders = mints.map(() => '?').join(',');
    const query = `
      SELECT * FROM token_metadata WHERE mint IN (${placeholders})
    `;

    const stmt = db.prepare(query);
    const rows = stmt.all(...mints) as any[];

    const result: Record<string, TokenMetadata> = {};

    for (const row of rows) {
      result[row.mint] = {
        mint: row.mint,
        symbol: row.symbol,
        name: row.name,
        decimals: row.decimals,
        launch_platform: row.launch_platform,
        pair_created_at: row.pair_created_at,
        price_usd: row.price_usd !== null ? row.price_usd : null,
        market_cap: row.market_cap !== null ? row.market_cap : null,
        token_age: row.token_age !== null ? row.token_age : null,
        created_at: row.created_at,
        updated_at: row.updated_at,
      };
    }

    return result;
  } catch (error) {
    console.error('getMultipleTokenMetadata error:', error);
    return {};
  }
}

// ═══════════════════════════════════════════════════════════════════════
// Export database getter for custom queries
// ═══════════════════════════════════════════════════════════════════════

export { getDb };
