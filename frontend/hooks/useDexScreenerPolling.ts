/**
 * Hook for continuous DexScreener polling of followed tokens
 * Fetches price and market cap data every 30 seconds
 */

'use client';

import { useEffect, useRef } from 'react';
import { useFollowedTokens } from './useFollowedTokens';

const POLLING_INTERVAL = 30000; // 30 seconds

// Global cache to prevent overlapping requests
const fetchingCache = new Set<string>();

/**
 * Fetch and update token data from DexScreener
 */
async function fetchTokenData(mint: string): Promise<void> {
  // Prevent duplicate requests
  if (fetchingCache.has(mint)) {
    return;
  }

  fetchingCache.add(mint);

  try {
    const response = await fetch('/api/token/fetch', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ mint }),
    });

    if (!response.ok) {
      console.error(`Failed to fetch token ${mint}:`, response.statusText);
    }
  } catch (error) {
    console.error(`Error fetching token ${mint}:`, error);
  } finally {
    fetchingCache.delete(mint);
  }
}

/**
 * Hook to manage continuous polling for followed tokens
 */
export function useDexScreenerPolling() {
  const { followed } = useFollowedTokens();
  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  const followedRef = useRef<string[]>([]);

  // Update ref when followed list changes
  useEffect(() => {
    followedRef.current = followed;
  }, [followed]);

  useEffect(() => {
    // Clear existing interval
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }

    // Don't start polling if no tokens are followed
    if (followed.length === 0) {
      return;
    }

    // Fetch immediately on mount
    followed.forEach((mint) => {
      fetchTokenData(mint);
    });

    // Setup polling interval
    intervalRef.current = setInterval(() => {
      const currentFollowed = followedRef.current;
      
      if (currentFollowed.length === 0) {
        return;
      }

      // Fetch data for all followed tokens
      currentFollowed.forEach((mint) => {
        fetchTokenData(mint);
      });
    }, POLLING_INTERVAL);

    // Cleanup on unmount
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [followed.length]); // Re-run when followed count changes

  return {
    isPolling: followed.length > 0,
    pollingCount: followed.length,
  };
}
