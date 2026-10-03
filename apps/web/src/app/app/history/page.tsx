import { demo, formatAmount } from "@/lib/demo-state";

const history = demo.history;
export default function HistoryPage() {
  return <div className="app-screen"><div className="app-page-heading"><div><span className="app-kicker"><i /> 04 / FINANCIAL HISTORY</span><h1>Participation,<br /><em>on record.</em></h1><p>Objective facts from contributions, payouts, discounts, and cycles.</p></div><span className="app-page-index">WALLET-BASED HISTORY / EXAMPLE</span></div>
    <div className="app-history-record"><div className="app-history-top"><div><span className="app-panel-kicker">ARTH / PARTICIPATION HISTORY</span><h2>{demo.memberName}<span> / CYCLE 001</span></h2></div><span className="history-tag">COMPLETED CYCLE</span></div><div className="app-history-grid">{[
      ["CONTRIBUTIONS COMPLETED",`${history.contributionsCompleted} / ${history.contributionsExpected}`],
      ["TOTAL CONTRIBUTED",formatAmount(history.totalContributed)],
      ["PAYOUTS RECEIVED",String(history.payoutsReceived)],
      ["TOTAL PAYOUT RECEIVED",formatAmount(history.totalPayoutReceived)],
      ["POST-PAYOUT CONTRIBUTIONS",`${history.postPayoutContributionsCompleted} / ${history.postPayoutContributionsExpected}`],
      ["DISCOUNTS EARNED",formatAmount(history.discountsEarned)],
      ["DISCOUNTS CLAIMED",formatAmount(history.discountsClaimed)],
      ["CYCLES JOINED",String(history.cyclesJoined)],
      ["CYCLES COMPLETED",String(history.cyclesCompleted)],
    ].map(([label,value])=><div key={label}><span>{label}</span><strong>{value}</strong></div>)}</div></div>
    <div className="history-explainer"><span className="app-kicker">VERIFIABLE PARTICIPATION</span><h2>Facts, not a score.</h2><p>Arth’s history shows actual protocol activity. It is not a traditional credit score, a risk rating, or a promise of future performance. Missed contributions are not shown because the current protocol does not define calendar-based missed contribution enforcement.</p></div>
  </div>;
}
