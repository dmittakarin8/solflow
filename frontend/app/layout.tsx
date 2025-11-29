import type { Metadata } from "next";
import Link from "next/link";
import "./globals.css";

export const metadata: Metadata = {
  title: "SolFlow - Solana Token Flow Dashboard",
  description: "Real-time Solana token flow analysis with signal detection",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body className="min-h-screen bg-background font-sans antialiased">
        <header className="border-b">
          <div className="container mx-auto px-4 py-3 flex items-center justify-between">
            <Link href="/dashboard" className="flex items-center gap-2">
              <div className="text-2xl font-bold">⚡️ SolFlow</div>
              <div className="text-xs text-muted-foreground hidden sm:block">
                Real-time token flow analysis
              </div>
            </Link>
            <nav className="flex items-center gap-4">
              <Link
                href="/dashboard"
                className="text-sm hover:text-foreground transition-colors"
              >
                Dashboard
              </Link>
            </nav>
          </div>
        </header>
        <main>{children}</main>
      </body>
    </html>
  );
}
