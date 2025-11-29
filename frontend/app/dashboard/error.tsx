/**
 * Error boundary for dashboard
 */

'use client';

import { useEffect } from 'react';

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    console.error('Dashboard error:', error);
  }, [error]);

  const isDatabaseError = error.message.includes('Database connection failed') ||
    error.message.includes('unable to open database');

  return (
    <div className="container mx-auto py-6 px-4">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">SolFlow Dashboard</h1>
          <p className="text-muted-foreground text-red-500">Error loading dashboard</p>
        </div>
      </div>

      <div className="border rounded-lg p-8 space-y-6">
        <div className="space-y-2">
          <h2 className="text-2xl font-semibold text-red-500">⚠️ {isDatabaseError ? 'Database Not Available' : 'Error Occurred'}</h2>
          <p className="text-muted-foreground">{error.message}</p>
        </div>

        {isDatabaseError && (
          <div className="space-y-4 border-l-4 border-yellow-500 pl-4">
            <h3 className="text-lg font-semibold">📋 Setup Instructions</h3>
            
            <div className="space-y-3 text-sm">
              <div>
                <p className="font-medium mb-1">Step 1: Start the Rust Backend</p>
                <pre className="bg-muted p-2 rounded overflow-x-auto">
                  <code>{`cd /home/dgem8/projects/solflow
cargo run --release`}</code>
                </pre>
                <p className="text-muted-foreground mt-1">
                  The Rust backend will create the database and start populating it with trade data.
                </p>
              </div>

              <div>
                <p className="font-medium mb-1">Step 2: Wait for Initial Data</p>
                <p className="text-muted-foreground">
                  Allow 1-2 minutes for the backend to connect to the Geyser stream and start processing trades.
                  The database will be created automatically at:{' '}
                  <code className="bg-muted px-1 rounded">{process.env.SOLFLOW_DB_PATH || '/home/dgem8/projects/solflow/solflow.db'}</code>
                </p>
              </div>

              <div>
                <p className="font-medium mb-1">Step 3: Refresh This Page</p>
                <p className="text-muted-foreground">
                  Once the backend is running and has created the database, click the button below to reload.
                </p>
              </div>
            </div>

            <div className="pt-4">
              <button
                onClick={reset}
                className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors"
              >
                🔄 Retry Connection
              </button>
            </div>
          </div>
        )}

        {!isDatabaseError && (
          <div className="space-y-2">
            <h3 className="text-lg font-semibold">Debug Information</h3>
            <pre className="bg-muted p-4 rounded overflow-x-auto text-xs">
              <code>{error.stack}</code>
            </pre>
            <button
              onClick={reset}
              className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors"
            >
              Try Again
            </button>
          </div>
        )}

        <div className="pt-6 border-t space-y-2 text-sm text-muted-foreground">
          <p className="font-medium">Quick Checks:</p>
          <ul className="list-disc list-inside space-y-1">
            <li>Rust backend is running: <code className="bg-muted px-1 rounded">cargo run --release</code></li>
            <li>Database path is correct in <code className="bg-muted px-1 rounded">.env</code></li>
            <li>Rust backend has completed initialization (check logs for "✅ Initial schema applied")</li>
          </ul>
        </div>
      </div>
    </div>
  );
}
