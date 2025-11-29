/**
 * Dashboard Skeleton - Loading state
 */

import { Skeleton } from '@/components/ui/skeleton';

export function DashboardSkeleton() {
  return (
    <div className="border rounded-lg overflow-hidden">
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead className="bg-muted/50">
            <tr>
              <th className="px-3 py-3 w-12"></th>
              <th className="px-3 py-3 text-left">
                <Skeleton className="h-4 w-16" />
              </th>
              <th className="px-3 py-3 text-left">
                <Skeleton className="h-4 w-20" />
              </th>
              <th className="px-3 py-3 text-left">
                <Skeleton className="h-4 w-20" />
              </th>
              <th className="px-3 py-3 text-left">
                <Skeleton className="h-4 w-16" />
              </th>
              <th className="px-3 py-3 text-left">
                <Skeleton className="h-4 w-12" />
              </th>
              <th className="px-3 py-3 text-left">
                <Skeleton className="h-4 w-16" />
              </th>
              <th className="px-3 py-3 text-left">
                <Skeleton className="h-4 w-16" />
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-border">
            {Array.from({ length: 10 }).map((_, i) => (
              <tr key={i} className="hover:bg-muted/30">
                <td className="px-3 py-3">
                  <Skeleton className="h-5 w-5" />
                </td>
                <td className="px-3 py-3">
                  <Skeleton className="h-4 w-32" />
                </td>
                <td className="px-3 py-3">
                  <Skeleton className="h-4 w-20" />
                </td>
                <td className="px-3 py-3">
                  <Skeleton className="h-4 w-20" />
                </td>
                <td className="px-3 py-3">
                  <Skeleton className="h-4 w-12" />
                </td>
                <td className="px-3 py-3">
                  <Skeleton className="h-4 w-12" />
                </td>
                <td className="px-3 py-3">
                  <Skeleton className="h-6 w-24" />
                </td>
                <td className="px-3 py-3">
                  <Skeleton className="h-8 w-20" />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
