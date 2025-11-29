/**
 * Hook for polling API endpoints at regular intervals
 */

'use client';

import { useEffect, useRef } from 'react';

export function usePolling(fn: () => Promise<void>, interval: number, enabled: boolean = true) {
  const fnRef = useRef(fn);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);

  // Update ref when fn changes
  useEffect(() => {
    fnRef.current = fn;
  }, [fn]);

  useEffect(() => {
    if (!enabled) {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
      return;
    }

    // Execute immediately on mount
    fnRef.current();

    // Setup polling interval
    intervalRef.current = setInterval(() => {
      fnRef.current();
    }, interval);

    // Cleanup on unmount
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [interval, enabled]);
}
