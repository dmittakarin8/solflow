/**
 * POST /api/token/fetch
 * 
 * Fetch or refresh token metadata from DexScreener
 * - If token has NO metadata: fetch all fields (symbol, name, price, marketcap, age)
 * - If token HAS metadata: only update price and marketcap (preserve name/symbol/age)
 */

import { NextResponse } from 'next/server';
import Database from 'better-sqlite3';
import { fetchDexScreenerData } from '@/lib/client/dexscreener';

const DB_PATH = process.env.SOLFLOW_DB_PATH;

if (!DB_PATH) {
  throw new Error('SOLFLOW_DB_PATH environment variable is not set');
}

function getDb(): Database.Database {
  return new Database(DB_PATH, { fileMustExist: true });
}

export async function POST(request: Request) {
  try {
    const { mint } = await request.json();

    if (!mint || typeof mint !== 'string') {
      return NextResponse.json(
        { error: 'Invalid mint parameter' },
        { status: 400 }
      );
    }

    // Fetch data from DexScreener
    const data = await fetchDexScreenerData(mint);

    if (!data) {
      return NextResponse.json(
        { error: 'No data found for token' },
        { status: 404 }
      );
    }

    const db = getDb();

    try {
      // Check if token already has metadata
      const existing = db
        .prepare('SELECT * FROM token_metadata WHERE mint = ?')
        .get(mint) as any;

      const now = Math.floor(Date.now() / 1000);

      if (!existing) {
        // NEW TOKEN: Insert all fields
        const insertStmt = db.prepare(`
          INSERT INTO token_metadata (
            mint, symbol, name, decimals, price_usd, market_cap, token_age, created_at, updated_at
          ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        `);

        insertStmt.run(
          mint,
          data.symbol,
          data.name,
          9, // Default decimals for SPL tokens
          data.priceUsd,
          data.marketCap,
          data.tokenAge,
          now,
          now
        );

        return NextResponse.json({
          success: true,
          mode: 'created',
          data: {
            mint,
            symbol: data.symbol,
            name: data.name,
            price_usd: data.priceUsd,
            market_cap: data.marketCap,
            token_age: data.tokenAge,
          },
        });
      } else {
        // EXISTING TOKEN: Only update price and market cap
        const updateStmt = db.prepare(`
          UPDATE token_metadata
          SET price_usd = ?, market_cap = ?, updated_at = ?
          WHERE mint = ?
        `);

        updateStmt.run(data.priceUsd, data.marketCap, now, mint);

        return NextResponse.json({
          success: true,
          mode: 'refreshed',
          data: {
            mint,
            symbol: existing.symbol,
            name: existing.name,
            price_usd: data.priceUsd,
            market_cap: data.marketCap,
            token_age: existing.token_age,
          },
        });
      }
    } finally {
      db.close();
    }
  } catch (error) {
    console.error('Token fetch API error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch token data' },
      { status: 500 }
    );
  }
}
