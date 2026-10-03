"use client";
import { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { Brand } from "@/components/brand";
import { ArthProvider, useArth } from "@/lib/arth-context";
import { formatXlm } from "@/lib/amount";

const items = [
  { href: "/app", label: "Overview", number: "01" },
  { href: "/app/cycle", label: "Current Cycle", number: "02" },
  { href: "/app/bid", label: "Bid", number: "03" },
  { href: "/app/history", label: "History", number: "04" },
];

function ShellContent({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const [open, setOpen] = useState(false);
  const { wallet, walletAvailable, walletError, network, snapshot, loadState, txPhase, txError, lastTxHash, connect, disconnect, refresh } = useArth();
  const current = items.find((item) => item.href === pathname)?.label ?? "Overview";
  return <div className="app-shell">
    <aside className={`app-sidebar${open ? " is-open" : ""}`} id="app-sidebar">
      <Brand /><p className="app-sidebar__caption">COMMUNITY CAPITAL / ON STELLAR</p>
      <nav aria-label="App navigation"><span className="app-nav-label">WORKSPACE / 01</span>{items.map((item) => <Link key={item.href} href={item.href} aria-current={pathname === item.href ? "page" : undefined} onClick={() => setOpen(false)}><span>{item.number}</span>{item.label}<b>↗</b></Link>)}</nav>
      <div className="app-sidebar__foot"><span className="sidebar-symbol">↗</span><div>One community.<br />Rules everyone can verify.</div></div>
    </aside>
    {open && <button className="app-backdrop" aria-label="Close navigation" onClick={() => setOpen(false)} />}
    <div className="app-work-area"><header className="app-topbar"><button className="app-menu" aria-label="Toggle app navigation" aria-expanded={open} aria-controls="app-sidebar" onClick={() => setOpen(!open)}><span /><span /></button><div className="app-topbar__trail">ARTH <span>/</span> APP <span>/</span> <strong>{current.toUpperCase()}</strong></div><div className="app-topbar__right"><span className="app-network"><i /> {network ? network.toUpperCase() : "STELLAR TESTNET"}</span>{wallet && <span className="app-wallet">{snapshot?.balance !== null && snapshot?.balance !== undefined ? formatXlm(snapshot.balance) : "BALANCE LOADING"}</span>}<button className="app-wallet-button" type="button" onClick={wallet ? disconnect : () => void connect()}>{wallet ? `${wallet.slice(0, 5)}…${wallet.slice(-4)} / DISCONNECT` : "CONNECT WALLET ↗"}</button></div></header><main className="app-workspace"><div className="app-preview-notice"><strong>STELLAR TESTNET</strong><span>Live contract state. Testnet XLM has no real-world value.{walletAvailable === false ? " Freighter is unavailable; install or unlock it to connect." : ""}{loadState === "loading" && snapshot ? " Refreshing…" : ""}</span><button type="button" className="app-refresh" onClick={() => void refresh()}>REFRESH ↻</button></div>{walletError && <p className="app-feedback app-feedback--error" role="alert">{walletError}</p>}{txPhase !== "idle" && <p className={`app-feedback${txPhase === "failed" ? " app-feedback--error" : ""}`} role="status">{txPhase === "awaiting signature" ? "AWAITING WALLET SIGNATURE" : txPhase === "submitting" ? "SUBMITTING TO TESTNET" : txPhase === "confirmed" ? "CONFIRMED ON TESTNET" : "FAILED"}{txError ? ` — ${txError}` : ""}{lastTxHash && <span> / TX {lastTxHash.slice(0, 12)}…</span>}</p>}{children}</main></div>
  </div>;
}

export function AppShell({ children }: { children: React.ReactNode }) {
  return <ArthProvider><ShellContent>{children}</ShellContent></ArthProvider>;
}
