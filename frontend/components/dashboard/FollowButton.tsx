/**
 * Follow Button - Client Component
 * Toggle favorite status for a token
 */

'use client';

import { Star } from 'lucide-react';
import { useFollowedTokens } from '@/hooks/useFollowedTokens';

interface FollowButtonProps {
  mint: string;
}

export function FollowButton({ mint }: FollowButtonProps) {
  const { isFollowed, toggleToken } = useFollowedTokens();
  const followed = isFollowed(mint);

  return (
    <button
      onClick={() => toggleToken(mint)}
      className="text-muted-foreground hover:text-yellow-500 transition-colors"
      title={followed ? 'Unfollow' : 'Follow'}
    >
      {followed ? (
        <Star className="w-5 h-5 fill-yellow-500 text-yellow-500" />
      ) : (
        <Star className="w-5 h-5" />
      )}
    </button>
  );
}
