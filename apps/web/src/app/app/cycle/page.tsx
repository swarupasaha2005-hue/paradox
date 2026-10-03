import { demo, formatAmount } from "@/lib/demo-state";

export default function CyclePage() {
  return <div className="app-screen"><div className="app-page-heading"><div><span className="app-kicker"><i /> 02 / CURRENT CYCLE</span><h1>Progress is<br /><em>shared.</em></h1><p>Each member’s fixed contribution forms this round’s pot.</p></div><span className="app-page-index">CYCLE 001 / ROUND 02</span></div>
    <div className="cycle-hero"><div><span className="app-panel-kicker">CURRENT ROUND / CONTRIBUTIONS</span><strong>10 <small>/ 10</small></strong><span>CONTRIBUTED</span></div><div><span className="app-panel-kicker">CAPITAL AVAILABLE FOR THIS ROUND</span><strong>{formatAmount(demo.pot)}</strong><span>POT READY <b>✓</b></span></div></div>
    <div className="app-section-head"><h2>Cycle timeline</h2><span>01 — 10 / ONE PAYOUT PER MEMBER</span></div><div className="cycle-timeline">{Array.from({length:demo.totalRounds},(_,index)=>{const round=index+1;return <div className={`cycle-row${round===demo.currentRound?" cycle-row--active":""}`} key={round}><span>{String(round).padStart(2,"0")}</span><strong>ROUND {String(round).padStart(2,"0")}</strong><span>{round===1?"SETTLED ✓":round===2?"ACTIVE":"UPCOMING —"}</span><b>{round===1?"✓":round===2?"↗":"—"}</b></div>})}</div>
    <div className="cycle-note"><span className="app-kicker">AFTER A PAYOUT</span><p>Winning a round does not end membership. The member continues the normal fixed contribution through the remaining cycle rounds.</p></div>
  </div>;
}
