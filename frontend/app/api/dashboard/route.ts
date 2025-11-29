/**
 * GET /api/dashboard
 * Returns top tokens by net_flow_300s with latest signals
 */

import { NextResponse } from 'next/server';
import { getDashboardTokens } from '@/lib/server/db';
import type { DashboardResponse } from '@/lib/types';

export const dynamic = 'force-dynamic';

export async function GET(request: Request) {
  try {
    const { searchParams } = new URL(request.url);
    const limit = parseInt(searchParams.get('limit') || '100');
    const minAge = parseInt(searchParams.get('minAge') || '300');

    const tokens = getDashboardTokens(limit, minAge);

    const response: DashboardResponse = {
      tokens,
      timestamp: Math.floor(Date.now() / 1000),
    };

    return NextResponse.json(response);
  } catch (error) {
    console.error('Dashboard API error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch dashboard data' },
      { status: 500 }
    );
  }
}
