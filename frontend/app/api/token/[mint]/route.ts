/**
 * GET /api/token/[mint]
 * Returns full token details (metadata, metrics, signals, trades)
 */

import { NextResponse } from 'next/server';
import {
  getTokenMetadata,
  getTokenMetrics,
  getTokenSignals,
  getTokenTrades,
} from '@/lib/server/db';
import type { TokenDetailResponse } from '@/lib/types';

export const dynamic = 'force-dynamic';

export async function GET(
  request: Request,
  { params }: { params: Promise<{ mint: string }> }
) {
  try {
    const { mint } = await params;

    const [metadata, metrics, signals, trades] = await Promise.all([
      getTokenMetadata(mint),
      getTokenMetrics(mint),
      getTokenSignals(mint, 20),
      getTokenTrades(mint, 50),
    ]);

    const response: TokenDetailResponse = {
      metadata,
      metrics,
      signals,
      trades,
    };

    return NextResponse.json(response);
  } catch (error) {
    console.error('Token API error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch token data' },
      { status: 500 }
    );
  }
}
