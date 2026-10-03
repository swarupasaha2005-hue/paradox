import Link from "next/link";
import { demo, formatAmount } from "@/lib/demo-state";

export default function Overview() {
  return <div className="app-screen"><div className="app-page-heading"><div><span className="app-kicker"><i /> 01 / OVERVIEW</span><h1>Your community,<br /><em>in motion.</em></h1><p>A clear view of your cycle and the next step.</p></div><span className="app-page-index">ARTH / CYCLE 001</span></div>
    <div className="app-intro-panel"><div><span className="app-panel-kicker">CURRENT CYCLE / SHARED CAPITAL</span><h2>Cycle #{demo.cycle}<br /><em>Round {demo.currentRound} is open.</em></h2><p>Ten members contribute together. Eligible members can enter the sealed bidding round.</p><Link href="/app/bid" className="app-panel-link">Enter Round <span>↗</span></Link></div><span className="app-intro-symbol" aria-hidden="true">02<span>/10</span></span></div>
    <div className="app-metrics"><div><span>ROUND</span><strong>2 / 10</strong><small>Current financing round</small></div><div><span>COMMUNITY POT</span><strong>{formatAmount(demo.pot)}</strong><small>Illustrative current round</small></div><div><span>MY CONTRIBUTION</span><strong>Paid <b>✓</b></strong><small>Round 02 contribution</small></div><div><span>MY PAYOUT STATUS</span><strong>Eligible</strong><small>Example member state</small></div><div><span>CLAIMABLE DISCOUNT</span><strong>{formatAmount(demo.discountPerMember)}</strong><small>Example round credit</small></div></div>
    <div className="app-split"><div><span className="app-kicker">NEXT / IN THIS CYCLE</span><h2>One round.<br />One payout.<br /><em>Shared benefit.</em></h2></div><div className="app-action-list"><Link href="/app/cycle"><span>01</span><strong>See cycle progress</strong><b>↗</b></Link><Link href="/app/bid"><span>02</span><strong>Explore sealed bidding</strong><b>↗</b></Link><Link href="/app/history"><span>03</span><strong>View financial history</strong><b>↗</b></Link></div></div>
  </div>;
}
