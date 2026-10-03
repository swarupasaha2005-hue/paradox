"use client";
import { useEffect, useState } from "react";
import { arth, calls, type CycleHistory } from "@/lib/arth-contract";
import { useArth, transactionPending } from "@/lib/arth-context";
import { formatXlm } from "@/lib/amount";
import { LiveState } from "@/components/live-state";

export default function HistoryPage() {
  const { wallet, snapshot: s, write, txPhase, refresh } = useArth();
  const history = s?.history, cycle = s?.cycleHistory;
  const [pastCycles, setPastCycles] = useState<CycleHistory[]>([]);
  const [pastWallet, setPastWallet] = useState<string | null>(null);
  const [pastError, setPastError] = useState(false);
  useEffect(() => {
    const ids = history?.cycle_ids.filter((id) => id !== cycle?.cycle_id) ?? [];
    let active = true;
    if (!wallet || !ids.length) return;
    Promise.all(ids.map((id) => arth.getCycleHistory(id, wallet))).then((items) => { if (active) { setPastCycles(items); setPastWallet(wallet); setPastError(false); } }).catch(() => { if (active) { setPastCycles([]); setPastWallet(wallet); setPastError(true); } });
    return () => { active = false; };
  }, [wallet, history?.cycle_ids, cycle?.cycle_id]);
  const claimable = s?.cycleMember?.claimable_discount ?? 0n;
  return <div className="app-screen"><div className="app-page-heading"><div><span className="app-kicker"><i /> 04 / FINANCIAL HISTORY</span><h1>Participation,<br /><em>on record.</em></h1><p>Objective facts from on-chain contributions, payouts, discounts, and cycles.</p></div><span className="app-page-index">WALLET-BASED / TESTNET</span></div>
    <LiveState>{s && (!wallet ? <div className="app-live-state"><strong>CONNECT A WALLET</strong><p>Your connected wallet address identifies your Arth financial history.</p></div> : !s.isMember || !history ? <div className="app-live-state"><strong>NO MEMBERSHIP HISTORY YET</strong><p>Join the community to begin recording verifiable participation.</p></div> : <>
      <div className="app-history-record"><div className="app-history-top"><div><span className="app-panel-kicker">ARTH / PARTICIPATION HISTORY</span><h2>{wallet.slice(0,6)}…{wallet.slice(-5)}<span> / ON-CHAIN WALLET</span></h2></div><span className="history-tag">VERIFIABLE FACTS</span></div><div className="app-history-grid">{[
        ["CONTRIBUTIONS COMPLETED",String(history.contributions_completed)],
        ["TOTAL CONTRIBUTED",formatXlm(history.total_contributed)],
        ["PAYOUTS RECEIVED",String(history.payouts_received)],
        ["TOTAL PAYOUT RECEIVED",formatXlm(history.total_payouts_received)],
        ["POST-PAYOUT CONTRIBUTIONS",String(history.post_payout_contributions)],
        ["DISCOUNTS EARNED",formatXlm(history.discounts_earned)],
        ["DISCOUNTS CLAIMED",formatXlm(history.discounts_claimed)],
        ["CYCLES JOINED",String(history.cycles_joined)],
        ["CYCLES COMPLETED",String(history.cycles_completed)],
      ].map(([label,value])=><div key={label}><span>{label}</span><strong>{value}</strong></div>)}</div></div>
      {cycle && <section className="discount-claim-preview"><div><span className="app-kicker">CURRENT CYCLE / #{cycle.cycle_id.toString()}</span><h2>Your participation<br /><em>this cycle.</em></h2><p>Contributions {cycle.completed_contributions} / {cycle.expected_contributions}. Payout received: {cycle.payout_received ? "Yes" : "No"}. Post-payout contributions: {cycle.contributions_after_payout} / {cycle.post_payout_required_total}. {cycle.cycle_complete ? "Cycle complete." : "Cycle in progress."}</p><p>Winning a payout does not end the normal contribution obligation.</p></div><div><div className="bid-detail"><span>PAYOUT AMOUNT</span><strong>{formatXlm(cycle.payout_amount)}</strong></div><div className="bid-detail"><span>DISCOUNT EARNED</span><strong>{formatXlm(cycle.discounts_earned)}</strong></div><div className="bid-detail"><span>DISCOUNT CLAIMED</span><strong>{formatXlm(cycle.discounts_claimed)}</strong></div><div className="bid-detail"><span>CURRENTLY CLAIMABLE</span><strong>{formatXlm(claimable)}</strong></div><button className="button button--dark" type="button" disabled={claimable <= 0n || transactionPending(txPhase)} onClick={() => void write(calls.claimDiscount(cycle.cycle_id, wallet))}>CLAIM DISCOUNT ↗</button><small>{claimable > 0n ? "Freighter signs a real Testnet claim. Credits can be claimed only once." : "Nothing claimable at present."}</small></div></section>}
      {pastWallet === wallet && pastCycles.filter((item) => history.cycle_ids.includes(item.cycle_id)).map((item) => <section className="app-admin" key={item.cycle_id.toString()}><span className="app-kicker">COMPLETED CYCLE / #{item.cycle_id.toString()}</span><h2>Participation record</h2><p>Contributions {item.completed_contributions} / {item.expected_contributions} · Post-payout {item.contributions_after_payout} / {item.post_payout_required_total} · Payout {formatXlm(item.payout_amount)} · Discount earned {formatXlm(item.discounts_earned)} · Claimable {formatXlm(item.claimable_discount)}</p><button type="button" className="preview-outline-button" disabled={item.claimable_discount <= 0n || transactionPending(txPhase)} onClick={() => void write(calls.claimDiscount(item.cycle_id, wallet))}>CLAIM CYCLE DISCOUNT ↗</button></section>)}
      {pastError && pastWallet === wallet && <div className="app-live-state" role="alert">Could not load completed-cycle history. <button type="button" className="preview-outline-button" onClick={() => void refresh()}>RETRY ↗</button></div>}
      <div className="history-explainer"><span className="app-kicker">VERIFIABLE PARTICIPATION</span><h2>Facts, not a score.</h2><p>Arth’s history shows actual protocol activity. It is not a traditional credit score, a risk rating, or a promise of future performance. Missed contributions are not shown because the current protocol does not define calendar-based missed contribution enforcement.</p></div>
    </>)}</LiveState>
  </div>;
}
