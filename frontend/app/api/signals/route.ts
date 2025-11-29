/**
 * GET /api/signals?mint=X
 * Returns signals for a specific token or recent signals across all tokens
 */

import { NextResponse } from 'next/server';
import { getTokenSignals, getRecentSignals } from '@/lib/server/db';
import type { SignalsResponse } from '@/lib/types';

export const dynamic = 'force-dynamic';

export async function GET(request: Request) {
  try {
    const { searchParams } = new URL(request.url);
    const mint = searchParams.get('mint');
    const minStrength = parseFloat(searchParams.get('minStrength') || '0.0');
    const limit = parseInt(searchParams.get('limit') || '50');
    const minAge = parseInt(searchParams.get('minAge') || '1800');

    let signals;

    if (mint) {
      // Get signals for specific token
      signals = getTokenSignals(mint, limit);
    } else {
      // Get recent signals across all tokens
      signals = getRecentSignals(minStrength, limit, minAge);
    }

    const response: SignalsResponse = {
      signals,
    };

    return NextResponse.json(response);
  } catch (error) {
    console.error('Signals API error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch signals' },
      { status: 500 }
    );
  }
}
