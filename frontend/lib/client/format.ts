/**
 * Client-side formatting utilities for numbers, dates, and addresses
 */

export function formatSOL(value: number, decimals: number = 2): string {
  if (Math.abs(value) < 0.01 && value !== 0) {
    return value > 0 ? '+<0.01' : '-<0.01';
  }
  
  const formatted = value.toFixed(decimals);
  return value > 0 ? `+${formatted}` : formatted;
}

export function formatUSD(value: number, decimals: number = 2): string {
  if (Math.abs(value) < 0.01 && value !== 0) {
    return value > 0 ? '+$<0.01' : '-$<0.01';
  }
  
  const formatted = value.toFixed(decimals);
  return value > 0 ? `+$${formatted}` : `$${formatted}`;
}

export function formatPercent(value: number, decimals: number = 1): string {
  const formatted = (value * 100).toFixed(decimals);
  return value > 0 ? `+${formatted}%` : `${formatted}%`;
}

export function formatNumber(value: number, decimals: number = 0): string {
  if (value >= 1_000_000) {
    return `${(value / 1_000_000).toFixed(1)}M`;
  }
  if (value >= 1_000) {
    return `${(value / 1_000).toFixed(1)}K`;
  }
  return value.toFixed(decimals);
}

export function formatAddress(address: string, chars: number = 4): string {
  if (address.length <= chars * 2 + 3) return address;
  return `${address.slice(0, chars)}...${address.slice(-chars)}`;
}

export function formatTimestamp(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHr = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHr / 24);

  if (diffSec < 60) return `${diffSec}s ago`;
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHr < 24) return `${diffHr}h ago`;
  if (diffDay < 7) return `${diffDay}d ago`;

  return date.toLocaleDateString();
}

export function formatTimeAgo(timestamp: number): string {
  return formatTimestamp(timestamp);
}

export function formatDateTime(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  return date.toLocaleString();
}

export function formatStrength(strength: number): string {
  return (strength * 100).toFixed(0);
}

export function getStrengthLabel(strength: number): string {
  if (strength >= 0.8) return 'Very Strong';
  if (strength >= 0.6) return 'Strong';
  if (strength >= 0.4) return 'Moderate';
  if (strength >= 0.2) return 'Weak';
  return 'Very Weak';
}

export function getStrengthColor(strength: number): string {
  if (strength >= 0.8) return 'text-red-500';
  if (strength >= 0.6) return 'text-orange-500';
  if (strength >= 0.4) return 'text-yellow-500';
  if (strength >= 0.2) return 'text-gray-500';
  return 'text-gray-400';
}

export function getFlowColor(flow: number): string {
  if (flow > 0) return 'text-green-500';
  if (flow < 0) return 'text-red-500';
  return 'text-gray-500';
}

export function getFlowBgColor(flow: number): string {
  if (flow > 0) return 'bg-green-500/10';
  if (flow < 0) return 'bg-red-500/10';
  return 'bg-gray-500/10';
}
