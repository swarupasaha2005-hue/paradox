"use client";

import Link from "next/link";
import { calls } from "@/lib/arth-contract";
import { useArth, transactionPending } from "@/lib/arth-context";
import { formatXlm } from "@/lib/amount";
import { LiveState } from "@/components/live-state";

export default function Overview() {
  const { wallet, connect, snapshot, write, txPhase } = useArth();
  return <div className="app-screen"><div className="app-page-heading"><div><span className="app-kicker"><i /> 01 / OVERVIEW</span><h1>Your community,<br /><em>in motion.</em></h1><p>Live participation on Stellar Testnet.</p></div><span className="app-page-index">ARTH / TESTNET</span></div>
    <LiveState>{snapshot && <OverviewData wallet={wallet} connect={connect} snapshot={snapshot} write={write} pending={transactionPending(txPhase)} />}</LiveState>
  </div>;
}

type Data = NonNullable<ReturnType<typeof useArth>["snapshot"]>;
function OverviewData({ wallet, connect, snapshot: s, write, pending }: { wallet: string | null; connect: () => Promise<void>; snapshot: Data; write: ReturnType<typeof useArth>["write"]; pending: boolean }) {
  const cycle = s.cycle, mine = s.cycleMember;
  const canContribute = Boolean(wallet && cycle && mine && !cycle.complete && cycle.active_round_id === null && mine.completed_contributions < cycle.current_round_number);
  const balanceLow = s.balance !== null && s.balance < s.community.contribution_amount;
  const contributed = s.cycleMembers.filter((item) => cycle && item.completed_contributions >= cycle.current_round_number).length;
  const participant = Boolean(wallet && s.round?.participants.includes(wallet));
  const round = cycle?.active_round_id === s.round?.id ? s.round : null;
  let title = "Connect your wallet.", detail = "Your Freighter address is your Arth identity.";
  let action = "VIEW CYCLE", href = "/app/cycle";
  if (!wallet) { action = "CONNECT WALLET"; }
  else if (!s.isMember) { title = "Join the community."; detail = "Membership is authenticated by your wallet and recorded on Testnet."; action = "JOIN COMMUNITY"; }
  else if (!cycle) { title = "Waiting for a cycle."; detail = "You are a member. The admin must create a cycle from real joined wallets before contributions begin."; }
  else if (!mine) { title = "Waiting for cycle inclusion."; detail = "The admin has not included this wallet in the current cycle."; }
  else if (cycle.complete && mine.claimable_discount > 0n) { title = "Claim your community discount."; detail = "Your cycle is complete and its discount credit remains claimable."; action = "CLAIM DISCOUNT"; href = "/app/history"; }
  else if (cycle.complete) { title = "Cycle complete."; detail = "Your participation history remains available by wallet."; action = "VIEW HISTORY"; href = "/app/history"; }
  else if (canContribute) { title = "Contribute to this round."; detail = `Your contribution is pending. ${contributed} of ${cycle.members.length} members have paid; all must contribute before allocation.`; action = `CONTRIBUTE ${formatXlm(s.community.contribution_amount, s.assetDecimals)}`; }
  else if (round) {
    href = "/app/bid";
    if (!participant) { title = "Allocation round in progress."; detail = "This wallet is not a bidder in this round. The round's phase is on-chain."; action = "VIEW ROUND"; }
    else if (round.status === "Commit" && s.ledgerTime >= round.commit_deadline) { title = "Commit phase ended."; detail = "Anyone can start the reveal phase now."; action = "START REVEAL"; }
    else if (round.status === "Commit" && s.commitment) { title = "Bid sealed. Wait for reveal."; detail = "Your hash is on-chain. Keep the local secret for the reveal phase."; action = "VIEW SEALED BID"; }
    else if (round.status === "Commit") { title = "Submit your sealed bid."; detail = "Enter the discounted payout you would accept. Only its commitment hash is submitted."; action = "COMMIT BID"; }
    else if (round.status === "Reveal" && s.ledgerTime >= round.reveal_deadline) { title = "Reveal phase ended."; detail = "Anyone can finalize the round now."; action = "FINALIZE ROUND"; }
    else if (round.status === "Reveal" && s.reveal !== null) { title = "Bid revealed. Wait for finalization."; detail = "Your valid reveal is recorded on-chain."; action = "VIEW REVEAL"; }
    else if (round.status === "Reveal" && s.commitment) { title = "Reveal your original bid."; detail = "The locally stored bid and secret must match your on-chain commitment."; action = "REVEAL BID"; }
    else if (round.status === "Reveal") { title = "Reveal phase is open."; detail = "This wallet has no on-chain commitment to reveal."; action = "VIEW ROUND"; }
    else if (round.status === "Finalized" && round.winner === wallet) { title = "Settle your payout."; detail = "Your winning amount is finalized. Sign settlement to receive it."; action = "SETTLE PAYOUT"; }
    else if (round.status === "Finalized") { title = "Waiting for settlement."; detail = "The winner must sign payout settlement before the cycle advances."; action = "VIEW RESULT"; }
    else { title = "Round settled."; detail = "The cycle advances after settlement. Your normal contribution obligation continues."; action = "VIEW RESULT"; }
  }
  else if (mine && !s.request && !mine.payout_received) { title = "Request this round's payout."; detail = "Your contribution is recorded. Request a payout amount within the contract limit."; action = "REQUEST PAYOUT"; }
  else if (s.request) { title = "Waiting for allocation round."; detail = `Request #${s.request.id} is ${s.request.status}. ${contributed} of ${cycle.members.length} members have contributed; the admin creates the round after all are paid.`; action = "VIEW CYCLE"; }
  else if (mine && mine.claimable_discount > 0n) { title = "Claim your community discount."; detail = "The settled round credited your share to this wallet."; action = "CLAIM DISCOUNT"; href = "/app/history"; }
  else { title = "Your contribution is recorded."; detail = "You remain obligated to contribute in future rounds, including after receiving a payout."; }
  return <>
    <section className="next-action" aria-labelledby="next-action-title"><div><span className="app-kicker">NEXT ACTION / LIVE CONTRACT STATE</span><h2 id="next-action-title">{title}</h2><p>{detail}</p></div><div className="next-action__details">
      <div><span>MEMBERSHIP</span><strong>{!wallet ? "CONNECT WALLET" : s.isMember ? "MEMBER ✓" : "NOT JOINED"}</strong></div>
      <div><span>REQUIRED CONTRIBUTION</span><strong>{formatXlm(s.community.contribution_amount, s.assetDecimals)}</strong></div>
      <div><span>WALLET BALANCE</span><strong>{s.balance === null ? "CONNECT WALLET" : formatXlm(s.balance, s.assetDecimals)}</strong></div>
      <div><span>CURRENT CYCLE</span><strong>{cycle ? `#${cycle.id}` : "NOT CREATED"}</strong></div>
      {!wallet ? <button className="button button--dark" type="button" onClick={() => void connect()}>CONNECT WALLET ↗</button> : !s.isMember ? <button className="button button--dark" type="button" disabled={pending} onClick={() => void write(calls.join(wallet))}>JOIN COMMUNITY ↗</button> : <Link href={href} className="button button--dark">{action} ↗</Link>}
      {balanceLow && <small className="app-balance-warning">Your Testnet wallet needs at least {formatXlm(s.community.contribution_amount, s.assetDecimals)} plus network fees to contribute. Fund it with Testnet XLM; this is not real XLM.</small>}
    </div></section>
    <div className="app-intro-panel"><div><span className="app-panel-kicker">CURRENT CYCLE / SHARED CAPITAL</span><h2>{cycle ? `Cycle #${cycle.id}` : "No active cycle"}<br /><em>{cycle ? `Round ${cycle.current_round_number} of ${cycle.members.length}` : "Ready for real members."}</em></h2><p>{cycle ? `${cycle.members.length} authenticated members belong to this cycle. The round pot comes from their fixed contributions.` : "Joined wallets will be included by the community admin. No illustrative cycle or pot is shown as live state."}</p><Link href="/app/cycle" className="app-panel-link">View current cycle <span>↗</span></Link></div><span className="app-intro-symbol" aria-hidden="true">{cycle ? String(cycle.current_round_number).padStart(2,"0") : "—"}<span>/{cycle?.members.length ?? "—"}</span></span></div>
    <div className="app-metrics"><div><span>ROUND</span><strong>{cycle ? `${cycle.current_round_number} / ${cycle.members.length}` : "—"}</strong><small>Current financing round</small></div><div><span>AVAILABLE POOL</span><strong>{formatXlm(s.pool.available_pool, s.assetDecimals)}</strong><small>Contract accounting</small></div><div><span>MY CONTRIBUTION</span><strong>{mine ? mine.completed_contributions >= (cycle?.current_round_number ?? 0) ? "Paid ✓" : "Not paid" : "—"}</strong><small>Current round</small></div><div><span>MY PAYOUT STATUS</span><strong>{mine ? mine.payout_received ? "Received" : s.eligibility?.eligible ? "Eligible" : "Not yet eligible" : "—"}</strong><small>One payout per cycle</small></div><div><span>CLAIMABLE DISCOUNT</span><strong>{formatXlm(mine?.claimable_discount ?? 0n, s.assetDecimals)}</strong><small>Cycle credit</small></div></div>
    <div className="app-split"><div><span className="app-kicker">PROTOCOL / LIVE</span><h2>From a shared pot<br />to a history<br /><em>you can prove.</em></h2><p className="journey-note">Every value above comes from the deployed CommunityPool and native Testnet XLM asset.</p></div><ol className="app-journey-list">{[["Join community","/app"],["Contribute","/app/cycle"],["Request payout","/app/cycle"],["Commit and reveal","/app/bid"],["Financial history","/app/history"]].map(([label,href],index)=><li key={label}><span>{String(index+1).padStart(2,"0")}</span><Link href={href}>{label}</Link><small>↗</small></li>)}</ol></div>
  </>;
}
