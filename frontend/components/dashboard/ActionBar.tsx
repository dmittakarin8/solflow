/**
 * Action Bar - Client Component
 * Provides token-specific actions: Follow, Fetch, Copy, Block
 * Phase 8 Preparation: UI placeholders only, no logic yet
 */

'use client';

import { Star, RefreshCw, Copy, Ban } from 'lucide-react';
import { useFollowedTokens } from '@/hooks/useFollowedTokens';

interface ActionBarProps {
  mint: string;
}

export function ActionBar({ mint }: ActionBarProps) {
  const { isFollowed, toggleToken } = useFollowedTokens();
  const followed = isFollowed(mint);

  const handleFollow = () => {
    toggleToken(mint);
  };

  const handleFetch = () => {
    // TODO Phase 8: Implement one-time fetch/refresh
    console.log('Fetch/Refresh:', mint);
  };

  const handleCopy = () => {
    // TODO Phase 8: Copy address to clipboard
    console.log('Copy address:', mint);
  };

  const handleBlock = () => {
    // TODO Phase 8: Block token (add to blacklist)
    console.log('Block token:', mint);
  };

  const iconClass = 'w-4 h-4';
  const buttonClass = 'p-1 rounded hover:bg-muted transition-colors';

  return (
    <div className="flex items-center gap-1">
      {/* Follow/Unfollow - Continuous price+marketcap fetch */}
      <button
        onClick={handleFollow}
        className={buttonClass}
        title={followed ? 'Unfollow (stop continuous fetch)' : 'Follow (continuous price+marketcap fetch)'}
      >
        {followed ? (
          <Star className={`${iconClass} fill-yellow-500 text-yellow-500`} />
        ) : (
          <Star className={`${iconClass} text-muted-foreground hover:text-yellow-500`} />
        )}
      </button>

      {/* Fetch Once / Refresh */}
      <button
        onClick={handleFetch}
        className={buttonClass}
        title="Fetch once / Refresh"
      >
        <RefreshCw className={`${iconClass} text-muted-foreground hover:text-blue-500`} />
      </button>

      {/* Copy Address */}
      <button
        onClick={handleCopy}
        className={buttonClass}
        title="Copy address"
      >
        <Copy className={`${iconClass} text-muted-foreground hover:text-green-500`} />
      </button>

      {/* Block Token */}
      <button
        onClick={handleBlock}
        className={buttonClass}
        title="Block token"
      >
        <Ban className={`${iconClass} text-muted-foreground hover:text-red-500`} />
      </button>
    </div>
  );
}
