/**
 * Loading state for dashboard
 */

import { DashboardSkeleton } from '@/components/dashboard/DashboardSkeleton';

export default function Loading() {
  return (
    <div className="container mx-auto py-6 px-4">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">SolFlow Dashboard</h1>
          <p className="text-muted-foreground">Loading...</p>
        </div>
      </div>

      <DashboardSkeleton />
    </div>
  );
}
