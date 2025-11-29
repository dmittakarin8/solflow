/**
 * Signal Badge - Server Component
 * Display signal type with icon and strength
 */

import { SignalType } from '@/lib/types';
import { Badge } from '@/components/ui/badge';
import {
  TrendingUp,
  Repeat,
  Users,
  Activity,
  AlertTriangle,
} from 'lucide-react';
import { formatStrength } from '@/lib/client/format';

interface SignalBadgeProps {
  type: SignalType;
  strength: number;
}

const signalConfig = {
  BREAKOUT: {
    label: 'Breakout',
    icon: TrendingUp,
    className: 'bg-blue-500/10 text-blue-500 border-blue-500/30',
  },
  REACCUMULATION: {
    label: 'Reaccum',
    icon: Repeat,
    className: 'bg-green-500/10 text-green-500 border-green-500/30',
  },
  FOCUSED_BUYERS: {
    label: 'Focused',
    icon: Users,
    className: 'bg-purple-500/10 text-purple-500 border-purple-500/30',
  },
  PERSISTENCE: {
    label: 'Persist',
    icon: Activity,
    className: 'bg-orange-500/10 text-orange-500 border-orange-500/30',
  },
  FLOW_REVERSAL: {
    label: 'Reversal',
    icon: AlertTriangle,
    className: 'bg-red-500/10 text-red-500 border-red-500/30',
  },
};

export function SignalBadge({ type, strength }: SignalBadgeProps) {
  const config = signalConfig[type];
  const Icon = config.icon;

  return (
    <Badge
      variant="outline"
      className={`${config.className} text-xs font-medium px-2 py-0.5`}
    >
      <Icon className="w-3 h-3 mr-1" />
      {config.label} {formatStrength(strength)}%
    </Badge>
  );
}
