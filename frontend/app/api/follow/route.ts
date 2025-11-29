/**
 * POST /api/follow - Add token to followed list
 * DELETE /api/follow - Remove token from followed list
 * 
 * Manages the followed_tokens table for continuous price polling
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

export async function POST(request: Request) {
  try {
    const { mint } = await request.json();

    if (!mint || typeof mint !== 'string') {
      return NextResponse.json(
        { error: 'Invalid mint parameter' },
        { status: 400 }
      );
    }

    const db = getDb();
    
    try {
      const stmt = db.prepare(`
        INSERT INTO followed_tokens (mint)
        VALUES (?)
        ON CONFLICT(mint) DO NOTHING
      `);
      
      stmt.run(mint);
      
      return NextResponse.json({ success: true, mint });
    } finally {
      db.close();
    }
  } catch (error) {
    console.error('Follow API error:', error);
    return NextResponse.json(
      { error: 'Failed to follow token' },
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
      const stmt = db.prepare('DELETE FROM followed_tokens WHERE mint = ?');
      stmt.run(mint);
      
      return NextResponse.json({ success: true, mint });
    } finally {
      db.close();
    }
  } catch (error) {
    console.error('Unfollow API error:', error);
    return NextResponse.json(
      { error: 'Failed to unfollow token' },
      { status: 500 }
    );
  }
}
