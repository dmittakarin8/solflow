/**
 * Dashboard Page - Server Component
 * Main view showing top tokens by net_flow_300s with signals
 */

import { getDashboardTokens } from '@/lib/server/db';
import { DashboardClient } from '@/components/dashboard/DashboardClient';
import { cookies } from 'next/headers';

export const metadata = {
  title: 'SolFlow Dashboard',
  description: 'Real-time Solana token flow dashboard',
};

export default async function DashboardPage() {
  // Access cookies to mark route as dynamic (required for Date.now())
  await cookies();
  
  // Fetch initial data server-side
  const initialTokens = getDashboardTokens(100, 300);

  return (
    <div className="container mx-auto py-6 px-4">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">SolFlow Dashboard</h1>
          <p className="text-muted-foreground">
            Real-time token flow analysis • Updates every 10s
          </p>
        </div>
      </div>

      <DashboardClient initialTokens={initialTokens} />
    </div>
  );
}
