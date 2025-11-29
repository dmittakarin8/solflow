/**
 * POST /api/blocklist - Add token to blocklist
 * DELETE /api/blocklist - Remove token from blocklist
 * GET /api/blocklist - Get all blocked tokens
 * 
 * Manages the blocklist table. Blocked tokens are:
 * - Hidden from dashboard
 * - Ignored in backend GRPC ingestion (handled by backend)
 */

import { NextResponse } from 'next/server';
import Database from 'better-sqlite3';

const DB_PATH = process.env.SOLFLOW_DB_PATH;

if (!DB_PATH) {
  throw new Error('SOLFLOW_DB_PATH environment variable is not set');
}

function getDb(): Database.Database {
  return new Database(DB_PATH, { fileMustExist: true });
}

export async function GET() {
  try {
    const db = getDb();

    try {
      const stmt = db.prepare('SELECT mint, created_at, reason FROM blocklist ORDER BY created_at DESC');
      const rows = stmt.all() as any[];

      return NextResponse.json({ blocked: rows.map((r) => r.mint) });
    } finally {
      db.close();
    }
  } catch (error) {
    console.error('Blocklist GET error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch blocklist' },
      { status: 500 }
    );
  }
}

export async function POST(request: Request) {
  try {
    const { mint, reason } = await request.json();

    if (!mint || typeof mint !== 'string') {
      return NextResponse.json(
        { error: 'Invalid mint parameter' },
        { status: 400 }
      );
    }

    const db = getDb();

    try {
      const stmt = db.prepare(`
        INSERT INTO blocklist (mint, reason)
        VALUES (?, ?)
        ON CONFLICT(mint) DO UPDATE SET reason = excluded.reason
      `);

      stmt.run(mint, reason || null);

      return NextResponse.json({ success: true, mint });
    } finally {
      db.close();
    }
  } catch (error) {
    console.error('Blocklist POST error:', error);
    return NextResponse.json(
      { error: 'Failed to block token' },
      { status: 500 }
    );
  }
}

export async function DELETE(request: Request) {
  try {
    const { searchParams } = new URL(request.url);
    const mint = searchParams.get('mint');

    if (!mint) {
      return NextResponse.json(
        { error: 'Missing mint parameter' },
        { status: 400 }
      );
    }

    const db = getDb();

    try {
      const stmt = db.prepare('DELETE FROM blocklist WHERE mint = ?');
      stmt.run(mint);

      return NextResponse.json({ success: true, mint });
    } finally {
      db.close();
    }
  } catch (error) {
    console.error('Blocklist DELETE error:', error);
    return NextResponse.json(
      { error: 'Failed to unblock token' },
      { status: 500 }
    );
  }
}
