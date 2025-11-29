/**
 * Dashboard Client Component
 * Handles polling, sorting, and client-side state
 */

'use client';

import { useState, useCallback, useMemo } from 'react';
import { DashboardToken, SortConfig } from '@/lib/types';
import { DashboardTable } from './DashboardTable';
import { usePolling } from '@/hooks/usePolling';
import { useFollowedTokens } from '@/hooks/useFollowedTokens';

interface DashboardClientProps {
  initialTokens: DashboardToken[];
}

export function DashboardClient({ initialTokens }: DashboardClientProps) {
  const [tokens, setTokens] = useState<DashboardToken[]>(initialTokens);
  const [sortConfig, setSortConfig] = useState<SortConfig>({
    key: 'net_flow_300s',
    direction: 'desc',
  });
  const { followed, isLoaded } = useFollowedTokens();

  // Polling for live updates
  const fetchTokens = useCallback(async () => {
    try {
      const response = await fetch('/api/dashboard');
      if (response.ok) {
        const data = await response.json();
        setTokens(data.tokens);
      }
    } catch (error) {
      console.error('Failed to fetch tokens:', error);
    }
  }, []);

  usePolling(fetchTokens, 10000, true); // Poll every 10s

  // Sorting logic
  const sortedTokens = useMemo(() => {
    const sorted = [...tokens];

    sorted.sort((a, b) => {
      // Followed tokens always at top
      const aFollowed = followed.includes(a.mint);
      const bFollowed = followed.includes(b.mint);

      if (aFollowed && !bFollowed) return -1;
      if (!aFollowed && bFollowed) return 1;

      // Then sort by selected column
      const aValue = a[sortConfig.key] ?? 0;
      const bValue = b[sortConfig.key] ?? 0;

      if (typeof aValue === 'number' && typeof bValue === 'number') {
        return sortConfig.direction === 'asc'
          ? aValue - bValue
          : bValue - aValue;
      }

      // String comparison
      const aStr = String(aValue);
      const bStr = String(bValue);

      return sortConfig.direction === 'asc'
        ? aStr.localeCompare(bStr)
        : bStr.localeCompare(aStr);
    });

    return sorted;
  }, [tokens, sortConfig, followed]);

  const handleSort = useCallback((key: keyof DashboardToken) => {
    setSortConfig((prev) => ({
      key,
      direction: prev.key === key && prev.direction === 'desc' ? 'asc' : 'desc',
    }));
  }, []);

  if (!isLoaded) {
    return <div>Loading...</div>;
  }

  return (
    <DashboardTable
      tokens={sortedTokens}
      sortConfig={sortConfig}
      onSort={handleSort}
      followedTokens={followed}
    />
  );
}
