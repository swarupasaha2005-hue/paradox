import Link from "next/link";
import { demo, formatAmount } from "@/lib/demo-state";

const pending = demo.contributionPreview.pending;

const journey = [
  { label: "Connect Wallet", detail: "AVAILABLE IN STEP 5B" },
  { label: "Join Community / Cycle", detail: "INCLUDED IN EXAMPLE" },
  { label: "Contribute", detail: "NEXT ACTION", href: "/app/cycle" },
  { label: "Request Payout", detail: "AFTER POT IS READY", href: "/app/cycle" },
  { label: "Commit Bid", detail: "SEALED PHASE", href: "/app/bid" },
  { label: "Reveal Bid", detail: "AFTER DEADLINE", href: "/app/bid" },
  { label: "Round Result", detail: "LOWEST VALID BID", href: "/app/bid" },
  { label: "Discount / Claim", detail: "COMMUNITY CREDIT", href: "/app/history" },
  { label: "Financial History", detail: "PARTICIPATION FACTS", href: "/app/history" },
];

export default function Overview() {
  return <div className="app-screen">
    <div className="app-page-heading"><div><span className="app-kicker"><i /> 01 / OVERVIEW</span><h1>Your community,<br /><em>in motion.</em></h1><p>A clear view of your cycle and the next step.</p></div><span className="app-page-index">ARTH / CYCLE 001</span></div>

    <section className="next-action" aria-labelledby="next-action-title">
      <div><span className="app-kicker">NEXT ACTION / ROUND 01 CONTRIBUTION</span><h2 id="next-action-title">Contribute<br /><em>to this round.</em></h2><p>Every member makes the same fixed contribution before the round pot is allocated.</p></div>
      <div className="next-action__details"><div><span>REQUIRED CONTRIBUTION</span><strong>{formatAmount(demo.contribution)}</strong></div><div><span>STATUS</span><strong>{pending.myStatus}</strong></div><div><span>COMMUNITY POT</span><strong>{formatAmount(pending.collected)} / {formatAmount(demo.pot)}</strong></div><Link href="/app/cycle" className="button button--dark">CONTRIBUTE {formatAmount(demo.contribution)} <span>↗</span></Link><small>Opens the cycle contribution preview. No transaction is submitted.</small></div>
    </section>

    <div className="app-intro-panel"><div><span className="app-panel-kicker">CURRENT CYCLE / SHARED CAPITAL</span><h2>Cycle #{demo.cycle}<br /><em>Round {demo.currentRound} is open.</em></h2><p>{pending.membersPaid} of {demo.members} members have contributed. Your contribution completes the illustrative {formatAmount(demo.pot)} pot.</p><Link href="/app/cycle" className="app-panel-link">View current cycle <span>↗</span></Link></div><span className="app-intro-symbol" aria-hidden="true">01<span>/10</span></span></div>

    <div className="app-metrics"><div><span>ROUND</span><strong>1 / 10</strong><small>Current financing round</small></div><div><span>COMMUNITY POT</span><strong>{formatAmount(pending.collected)}</strong><small>Of {formatAmount(demo.pot)} required</small></div><div><span>MY CONTRIBUTION</span><strong>Not paid</strong><small>Round 01 contribution</small></div><div><span>MY PAYOUT STATUS</span><strong>Waiting</strong><small>Contribute before requesting</small></div><div><span>CLAIMABLE DISCOUNT</span><strong>₹0</strong><small>No round settled yet</small></div></div>

    <div className="app-split"><div><span className="app-kicker">THE ARTH JOURNEY / INTERFACE PREVIEW</span><h2>From a shared pot<br />to a history<br /><em>you can prove.</em></h2><p className="journey-note">The first two steps are represented here as onboarding previews. This example member is already included in Cycle #1.</p></div><ol className="app-journey-list">{journey.map((step,index)=><li key={step.label}><span>{String(index+1).padStart(2,"0")}</span>{step.href?<Link href={step.href}>{step.label}</Link>:<strong>{step.label}</strong>}<small>{step.detail}</small></li>)}</ol></div>
  </div>;
}
