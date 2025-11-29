/**
 * GET /api/metadata?mints=X,Y,Z
 * Returns metadata for multiple tokens
 */

import { NextResponse } from 'next/server';
import { getMultipleTokenMetadata } from '@/lib/server/db';
import type { MetadataResponse } from '@/lib/types';

export const dynamic = 'force-dynamic';

export async function GET(request: Request) {
  try {
    const { searchParams } = new URL(request.url);
    const mintsParam = searchParams.get('mints');

    if (!mintsParam) {
      return NextResponse.json(
        { error: 'Missing mints parameter' },
        { status: 400 }
      );
    }

    const mints = mintsParam.split(',').map((m) => m.trim());
    const metadata = getMultipleTokenMetadata(mints);

    const response: MetadataResponse = metadata;

    return NextResponse.json(response);
  } catch (error) {
    console.error('Metadata API error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch metadata' },
      { status: 500 }
    );
  }
}
