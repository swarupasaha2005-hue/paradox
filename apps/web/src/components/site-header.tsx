"use client";
import { useState } from "react";
import Link from "next/link";
import { Brand } from "@/components/brand";

const links = [
  ["/#how-it-works", "How it works"], ["/#sealed-bidding", "Sealed bidding"],
  ["/#history", "History"], ["/#protocol", "Protocol"],
] as const;

export function SiteHeader() {
  const [open, setOpen] = useState(false);
  return <header className="site-header"><div className="page-shell site-header__inner">
    <Brand />
    <nav className="desktop-nav" aria-label="Main navigation">{links.map(([href,label]) => <Link href={href} key={href}>{label}</Link>)}</nav>
    <Link className="header-cta" href="/app">Launch App <span>↗</span></Link>
    <button className="menu-button" type="button" aria-label="Toggle menu" aria-expanded={open} aria-controls="mobile-nav" onClick={() => setOpen(!open)}><span /><span /></button>
  </div>{open && <nav className="mobile-nav" id="mobile-nav" aria-label="Mobile navigation">{links.map(([href,label]) => <Link href={href} key={href} onClick={() => setOpen(false)}>{label}</Link>)}<Link href="/app" onClick={() => setOpen(false)}>Launch App ↗</Link></nav>}</header>;
}
