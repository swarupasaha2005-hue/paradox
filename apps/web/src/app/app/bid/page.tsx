"use client";
import { useState } from "react";
import Link from "next/link";
import { demo, formatAmount } from "@/lib/demo-state";

type Phase = "commit" | "sealed" | "reveal" | "result";
const phases: {key: Phase; label: string}[] = [
  {key:"commit",label:"01 / COMMIT"}, {key:"sealed",label:"02 / SEALED"},
  {key:"reveal",label:"03 / REVEAL"}, {key:"result",label:"04 / RESULT"},
];

export default function BidPage() {
  const [phase,setPhase] = useState<Phase>("commit");
  return <div className="app-screen"><div className="app-page-heading"><div><span className="app-kicker"><i /> 03 / SEALED BIDDING</span><h1>Your bid.<br /><em>Your timing.</em></h1><p>See how an eligible member moves from commitment to verified result.</p></div><span className="app-page-index">ROUND 01 / ILLUSTRATIVE</span></div>
    <div className="bid-preview-label">LOCAL INTERFACE PREVIEW <span>These controls do not sign or submit transactions.</span></div>
    <div className="bid-phase-nav" role="tablist" aria-label="Bidding phase preview">{phases.map(({key,label})=><button type="button" role="tab" aria-selected={phase===key} key={key} onClick={()=>setPhase(key)}>{label}</button>)}</div>
    <div className="bid-layout"><section className="bid-primary">
      <div className="bid-panel-top"><span>ARTH / ROUND 01</span><span>{phase.toUpperCase()} PHASE</span></div>
      {phase==="commit" && <><span className="bid-panel-kicker">CURRENT POT</span><div className="bid-big-value">{formatAmount(demo.pot)}</div><label className="bid-input-label" htmlFor="bid-amount">AMOUNT YOU’LL ACCEPT</label><div className="bid-amount-input"><span>₹</span><input id="bid-amount" type="text" inputMode="numeric" value="43,000" readOnly aria-describedby="bid-example-note" /></div><p className="bid-field-note">This is the discounted payout you would accept from the current community pot for receiving your turn in this round. It is not loan principal.</p><button type="button" className="button button--dark bid-cta" onClick={()=>setPhase("sealed")}>LOCK MY BID <span>↗</span></button><small className="bid-action-note" id="bid-example-note">Fixed ₹43,000 example. Preview transition only; no commitment or secret is created.</small></>}
      {phase==="sealed" && <><span className="bid-panel-kicker">BID SEALED / EXAMPLE STATE</span><div className="bid-big-value">LOCKED<span className="bid-lock">●</span></div><p className="bid-field-note">The commitment stores a hash. Other participants cannot see the amount before reveal.</p><div className="bid-detail"><span>EXAMPLE COMMITMENT</span><strong>{demo.commitmentPreview}</strong></div><div className="bid-detail"><span>ROUND PHASE</span><strong>WAITING FOR REVEAL</strong></div><button type="button" className="button button--dark bid-cta" onClick={()=>setPhase("reveal")}>VIEW REVEAL PHASE <span>↗</span></button></>}
      {phase==="reveal" && <><span className="bid-panel-kicker">REVEAL PHASE / EXAMPLE STATE</span><h2 className="bid-phase-title">Your sealed bid<br />can now be revealed.</h2><div className="bid-detail"><span>YOUR SEALED BID</span><strong>{formatAmount(demo.winningPayout)}</strong></div><div className="bid-detail"><span>VERIFICATION</span><strong>AMOUNT + SECRET MATCH COMMITMENT</strong></div><p className="bid-field-note">In the live flow, the contract verifies the original amount and secret against your commitment.</p><button type="button" className="button button--dark bid-cta" onClick={()=>setPhase("result")}>REVEAL BID <span>↗</span></button><small className="bid-action-note">Preview transition only. No secret is submitted.</small></>}
      {phase==="result" && <><span className="bid-panel-kicker">ROUND RESULT / EXAMPLE STATE</span><h2 className="bid-phase-title">YOU WON<br /><em>THIS ROUND.</em></h2><div className="bid-detail"><span>PAYOUT</span><strong>{formatAmount(demo.winningPayout)}</strong></div><div className="bid-detail"><span>COMMUNITY DISCOUNT</span><strong>{formatAmount(demo.discount)}</strong></div><div className="bid-detail"><span>YOUR DISCOUNT CREDIT</span><strong>{formatAmount(demo.discountPerMember)}</strong></div><p className="bid-field-note">The winner continues normal contributions in the remaining cycle rounds.</p><Link href="/app/history" className="text-link">See discount claim preview <span>↗</span></Link></>}
    </section><aside className="bid-side"><span className="app-panel-kicker">HOW THE SEALED ROUND WORKS</span><h2>Private first.<br /><em>Public when it counts.</em></h2><div><span>01 / LOCK</span><p>Eligible participants commit a hash. Bid amounts stay hidden.</p></div><div><span>02 / REVEAL</span><p>The original bid and secret are checked after the commit deadline.</p></div><div><span>03 / ALLOCATE</span><p>The lowest valid bid wins. The discount is credited equally to cycle members.</p></div><p className="bid-side__note">Illustrative UI for the judge demo. Live wallet and Soroban actions arrive in Step 5B.</p></aside></div>
  </div>;
}
