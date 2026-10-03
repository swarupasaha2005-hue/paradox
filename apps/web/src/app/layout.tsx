import type { Metadata } from "next";
import Link from "next/link";
import { PRODUCT_CONFIG } from "@/config/product";
import "./globals.css";

export const metadata: Metadata = {
  title: { default: PRODUCT_CONFIG.name, template: `%s | ${PRODUCT_CONFIG.name}` },
  description: PRODUCT_CONFIG.description,
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body className="bg-white text-slate-900">
        <header className="border-b p-6">
          <Link href="/" className="font-semibold">{PRODUCT_CONFIG.name}</Link>
          <nav aria-label="Main navigation" className="mt-4 flex flex-wrap gap-4 text-sm">
            <Link href="/dashboard">Dashboard</Link>
            <Link href="/community">Community</Link>
            <Link href="/request">Request capital</Link>
            <Link href="/round/1">Demo round</Link>
            <Link href="/history">History</Link>
          </nav>
        </header>
        <main className="mx-auto max-w-4xl p-6">{children}</main>
      </body>
    </html>
  );
}
