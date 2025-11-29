/**
 * Hook for managing followed tokens in LocalStorage
 */

'use client';

import { useState, useEffect } from 'react';

const STORAGE_KEY = 'solflow_followed_tokens';

export function useFollowedTokens() {
  const [followed, setFollowed] = useState<string[]>([]);
  const [isLoaded, setIsLoaded] = useState(false);

  useEffect(() => {
    // Load from LocalStorage on mount
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      try {
        const parsed = JSON.parse(stored);
        setFollowed(Array.isArray(parsed) ? parsed : []);
      } catch (e) {
        console.error('Failed to parse followed tokens:', e);
        setFollowed([]);
      }
    }
    setIsLoaded(true);
  }, []);

  const addToken = (mint: string) => {
    if (followed.includes(mint)) return;
    
    const updated = [...followed, mint];
    setFollowed(updated);
    localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
  };

  const removeToken = (mint: string) => {
    const updated = followed.filter((m) => m !== mint);
    setFollowed(updated);
    localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
  };

  const toggleToken = (mint: string) => {
    if (followed.includes(mint)) {
      removeToken(mint);
    } else {
      addToken(mint);
    }
  };

  const isFollowed = (mint: string): boolean => {
    return followed.includes(mint);
  };

  return {
    followed,
    isLoaded,
    addToken,
    removeToken,
    toggleToken,
    isFollowed,
  };
}
