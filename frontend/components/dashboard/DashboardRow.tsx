/**
 * Dashboard Row - Client Component
 * Single row in dashboard table
 */

'use client';

import { DashboardToken } from '@/lib/types';
import { ActionBar } from './ActionBar';
import { SignalBadge } from './SignalBadge';
import { FlowSparkline } from './FlowSparkline';
import { formatSOL, formatAddress } from '@/lib/client/format';

interface DashboardRowProps {
  token: DashboardToken;
  isFollowed: boolean;
}

export function DashboardRow({ token, isFollowed }: DashboardRowProps) {
  const netFlows = [
    token.net_flow_900s,
    token.net_flow_3600s,
    token.net_flow_7200s,
    token.net_flow_14400s,
  ];

  const flow15mColorClass =
    token.net_flow_900s > 0
      ? 'text-green-500'
      : token.net_flow_900s < 0
      ? 'text-red-500'
      : 'text-muted-foreground';

  const flow1hColorClass =
    token.net_flow_3600s > 0
      ? 'text-green-500'
      : token.net_flow_3600s < 0
      ? 'text-red-500'
      : 'text-muted-foreground';

  const flow4hColorClass =
    token.net_flow_14400s > 0
      ? 'text-green-500'
      : token.net_flow_14400s < 0
      ? 'text-red-500'
      : 'text-muted-foreground';

  return (
    <tr
      className={`hover:bg-muted/30 transition-colors ${
        isFollowed ? 'bg-primary/5' : ''
      }`}
    >
      {/* Action Bar */}
      <td className="px-3 py-2">
        <ActionBar mint={token.mint} />
      </td>

      {/* Token Info */}
      <td className="px-3 py-2">
        <div className="font-mono text-sm">
          {token.symbol || formatAddress(token.mint)}
        </div>
        {token.name && (
          <div className="text-xs text-muted-foreground truncate max-w-[150px]">
            {token.name}
          </div>
        )}
      </td>

      {/* Net Flow 15m */}
      <td className={`px-3 py-2 font-mono text-sm font-semibold ${flow15mColorClass}`}>
        {formatSOL(token.net_flow_900s)}
      </td>

      {/* Net Flow 1h */}
      <td className={`px-3 py-2 font-mono text-sm ${flow1hColorClass}`}>
        {formatSOL(token.net_flow_3600s)}
      </td>

      {/* Net Flow 4h */}
      <td className={`px-3 py-2 font-mono text-sm ${flow4hColorClass}`}>
        {formatSOL(token.net_flow_14400s)}
      </td>

      {/* Wallets */}
      <td className="px-3 py-2 text-sm">
        <div className="flex items-center gap-2">
          <span>{token.unique_wallets_300s}</span>
          {token.bot_wallets_300s > 0 && (
            <span className="text-xs text-orange-500" title="Bot wallets detected">
              🤖{token.bot_wallets_300s}
            </span>
          )}
        </div>
      </td>

      {/* DCA Count */}
      <td className="px-3 py-2 text-sm">
        {token.dca_unique_wallets_300s > 0 ? (
          <span className="text-green-600">
            {token.dca_unique_wallets_300s}
          </span>
        ) : (
          <span className="text-muted-foreground">—</span>
        )}
      </td>

      {/* Signals */}
      <td className="px-3 py-2">
        <div className="flex gap-1 flex-wrap">
          {token.latest_signal_type && token.latest_signal_strength !== null && (
            <SignalBadge
              type={token.latest_signal_type}
              strength={token.latest_signal_strength}
            />
          )}
        </div>
      </td>

      {/* Sparkline */}
      <td className="px-3 py-2">
        <FlowSparkline flows={netFlows} />
      </td>
    </tr>
  );
}
