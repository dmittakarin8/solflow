/**
 * Action Bar - Client Component
 * Provides token-specific actions: Follow, Fetch, Copy, Block
 * Phase 9: Full implementation with DexScreener integration
 */

'use client';

import { Star, RefreshCw, Copy, Ban } from 'lucide-react';
import { useState } from 'react';
import { useFollowedTokens } from '@/hooks/useFollowedTokens';
import { showToast } from '@/lib/client/toast';

interface ActionBarProps {
  mint: string;
  onTokenBlocked?: (mint: string) => void;
}

export function ActionBar({ mint, onTokenBlocked }: ActionBarProps) {
  const { isFollowed, toggleToken } = useFollowedTokens();
  const followed = isFollowed(mint);
  const [isFetching, setIsFetching] = useState(false);
  const [isBlocking, setIsBlocking] = useState(false);

  const handleFollow = async () => {
    const wasFollowed = isFollowed(mint);
    
    try {
      if (wasFollowed) {
        // Unfollow
        const response = await fetch(`/api/follow?mint=${mint}`, {
          method: 'DELETE',
        });

        if (response.ok) {
          toggleToken(mint);
          showToast({ message: 'Stopped following', type: 'info' });
        } else {
          showToast({ message: 'Failed to unfollow', type: 'error' });
        }
      } else {
        // Follow
        const response = await fetch('/api/follow', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ mint }),
        });

        if (response.ok) {
          toggleToken(mint);
          showToast({ message: 'Following token', type: 'success' });
        } else {
          showToast({ message: 'Failed to follow', type: 'error' });
        }
      }
    } catch (error) {
      console.error('Follow error:', error);
      showToast({ message: 'Network error', type: 'error' });
    }
  };

  const handleFetch = async () => {
    if (isFetching) return;

    setIsFetching(true);

    try {
      const response = await fetch('/api/token/fetch', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ mint }),
      });

      if (response.ok) {
        const data = await response.json();
        const mode = data.mode === 'created' ? 'Fetched' : 'Refreshed';
        showToast({ message: `${mode} token data`, type: 'success' });
        
        // Trigger dashboard refresh by dispatching custom event
        window.dispatchEvent(new CustomEvent('token-data-updated', { detail: { mint } }));
      } else {
        const errorData = await response.json();
        showToast({ 
          message: errorData.error || 'Failed to fetch', 
          type: 'error' 
        });
      }
    } catch (error) {
      console.error('Fetch error:', error);
      showToast({ message: 'Network error', type: 'error' });
    } finally {
      setIsFetching(false);
    }
  };

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(mint);
      showToast({ message: 'Address copied!', type: 'success' });
    } catch (error) {
      console.error('Copy error:', error);
      showToast({ message: 'Failed to copy', type: 'error' });
    }
  };

  const handleBlock = async () => {
    if (isBlocking) return;

    const confirmed = confirm(
      'Block this token? It will be hidden from the dashboard and ignored by the backend.'
    );

    if (!confirmed) return;

    setIsBlocking(true);

    try {
      const response = await fetch('/api/blocklist', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ mint }),
      });

      if (response.ok) {
        showToast({ message: 'Token blocked', type: 'success' });
        
        // Notify parent component to remove from display
        if (onTokenBlocked) {
          onTokenBlocked(mint);
        }
        
        // Also trigger dashboard refresh
        window.dispatchEvent(new CustomEvent('token-blocked', { detail: { mint } }));
      } else {
        showToast({ message: 'Failed to block token', type: 'error' });
      }
    } catch (error) {
      console.error('Block error:', error);
      showToast({ message: 'Network error', type: 'error' });
    } finally {
      setIsBlocking(false);
    }
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
        disabled={isFetching}
      >
        <RefreshCw className={`${iconClass} text-muted-foreground hover:text-blue-500 ${isFetching ? 'animate-spin' : ''}`} />
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
        disabled={isBlocking}
      >
        <Ban className={`${iconClass} text-muted-foreground hover:text-red-500 ${isBlocking ? 'opacity-50' : ''}`} />
      </button>
    </div>
  );
}
