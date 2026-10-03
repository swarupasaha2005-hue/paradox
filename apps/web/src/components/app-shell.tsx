"use client";
import { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { Brand } from "@/components/brand";

const items = [
  { href: "/app", label: "Overview", number: "01" },
  { href: "/app/cycle", label: "Current Cycle", number: "02" },
  { href: "/app/bid", label: "Bid", number: "03" },
  { href: "/app/history", label: "History", number: "04" },
];

export function AppShell({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const [open, setOpen] = useState(false);
  const current = items.find((item) => item.href === pathname)?.label ?? "Overview";
  return <div className="app-shell">
    <aside className={`app-sidebar${open ? " is-open" : ""}`} id="app-sidebar">
      <Brand /><p className="app-sidebar__caption">COMMUNITY CAPITAL / ON STELLAR</p>
      <nav aria-label="App navigation"><span className="app-nav-label">WORKSPACE / 01</span>{items.map((item) => <Link key={item.href} href={item.href} aria-current={pathname === item.href ? "page" : undefined} onClick={() => setOpen(false)}><span>{item.number}</span>{item.label}<b>↗</b></Link>)}</nav>
      <div className="app-sidebar__foot"><span className="sidebar-symbol">↗</span><div>One community.<br />Rules everyone can verify.</div></div>
    </aside>
    {open && <button className="app-backdrop" aria-label="Close navigation" onClick={() => setOpen(false)} />}
    <div className="app-work-area"><header className="app-topbar"><button className="app-menu" aria-label="Toggle app navigation" aria-expanded={open} aria-controls="app-sidebar" onClick={() => setOpen(!open)}><span /><span /></button><div className="app-topbar__trail">ARTH <span>/</span> APP <span>/</span> <strong>{current.toUpperCase()}</strong></div><div className="app-topbar__right"><span className="app-network"><i /> STELLAR TESTNET</span><button className="app-wallet-button" type="button" disabled title="Wallet connection is a Step 5B preview">CONNECT WALLET ↗</button></div></header><main className="app-workspace"><div className="app-preview-notice"><strong>INTERFACE PREVIEW</strong><span>Illustrative values only. Wallet and contract actions arrive in Step 5B.</span></div>{children}</main></div>
  </div>;
}
