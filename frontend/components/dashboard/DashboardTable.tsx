/**
 * Dashboard Table - Server Component
 * Main table showing tokens with metrics and signals
 */

'use client';

import { DashboardToken, SortConfig } from '@/lib/types';
import { DashboardRow } from './DashboardRow';
import { ArrowUpDown, ArrowUp, ArrowDown } from 'lucide-react';

interface DashboardTableProps {
  tokens: DashboardToken[];
  sortConfig: SortConfig;
  onSort: (key: keyof DashboardToken) => void;
  followedTokens: string[];
}

export function DashboardTable({
  tokens,
  sortConfig,
  onSort,
  followedTokens,
}: DashboardTableProps) {
  const getSortIcon = (key: keyof DashboardToken) => {
    if (sortConfig.key !== key) {
      return <ArrowUpDown className="w-4 h-4 opacity-30" />;
    }
    return sortConfig.direction === 'asc' ? (
      <ArrowUp className="w-4 h-4" />
    ) : (
      <ArrowDown className="w-4 h-4" />
    );
  };

  const headerClass =
    'px-3 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider cursor-pointer hover:text-foreground transition-colors';

  return (
    <div className="border rounded-lg overflow-hidden">
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead className="bg-muted/50 sticky top-0 z-10">
            <tr>
              <th className="px-3 py-3 text-left w-12">
                <span className="text-xs font-medium text-muted-foreground uppercase">★</span>
              </th>
              <th
                className={headerClass}
                onClick={() => onSort('mint')}
              >
                <div className="flex items-center gap-1">
                  Token {getSortIcon('mint')}
                </div>
              </th>
              <th
                className={headerClass}
                onClick={() => onSort('net_flow_300s')}
              >
                <div className="flex items-center gap-1">
                  Flow 5m {getSortIcon('net_flow_300s')}
                </div>
              </th>
              <th
                className={headerClass}
                onClick={() => onSort('net_flow_60s')}
              >
                <div className="flex items-center gap-1">
                  Flow 1m {getSortIcon('net_flow_60s')}
                </div>
              </th>
              <th
                className={headerClass}
                onClick={() => onSort('unique_wallets_300s')}
              >
                <div className="flex items-center gap-1">
                  Wallets {getSortIcon('unique_wallets_300s')}
                </div>
              </th>
              <th
                className={headerClass}
                onClick={() => onSort('dca_unique_wallets_300s')}
              >
                <div className="flex items-center gap-1">
                  DCA {getSortIcon('dca_unique_wallets_300s')}
                </div>
              </th>
              <th className="px-3 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                Signals
              </th>
              <th className="px-3 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                Trend
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-border">
            {tokens.map((token) => (
              <DashboardRow
                key={token.mint}
                token={token}
                isFollowed={followedTokens.includes(token.mint)}
              />
            ))}
          </tbody>
        </table>
      </div>

      {tokens.length === 0 && (
        <div className="text-center py-12 text-muted-foreground">
          No active tokens found. Waiting for trading activity...
        </div>
      )}
    </div>
  );
}
