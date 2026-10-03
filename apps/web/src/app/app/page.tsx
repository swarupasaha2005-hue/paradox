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
  let title = "Connect your wallet.", detail = "Your Freighter address is your Arth identity.";
  if (!wallet) { title = "Connect your wallet."; }
  else if (!s.isMember) { title = "Join the community."; detail = "Membership is authenticated by your wallet and recorded on Testnet."; }
  else if (wallet && !cycle) { title = "Waiting for a cycle."; detail = "You are a member. The admin must create a cycle from real joined wallets before contributions begin."; }
  else if (wallet && !mine) { title = "Waiting for cycle inclusion."; detail = "The admin has not included this wallet in the current cycle."; }
  else if (cycle?.complete && mine && mine.claimable_discount > 0n) { title = "Claim your community discount."; detail = "Your cycle is complete and its discount credit remains claimable."; }
  else if (cycle?.complete) { title = "Cycle complete."; detail = "Your participation history remains available by wallet."; }
  else if (canContribute) { title = "Contribute to this round."; detail = "A fixed contribution is required before you can request the current round's payout."; }
  else if (cycle?.active_round_id !== null && cycle?.active_round_id !== undefined) { title = "Enter the allocation round."; detail = "Follow the on-chain phase to commit, reveal, or view the result."; }
  else if (mine && !s.request && !mine.payout_received) { title = "Request this round's payout."; detail = "Your contribution is recorded. Request a payout amount within the contract limit."; }
  else if (s.request) { title = "Waiting for round creation."; detail = `Request #${s.request.id} is ${s.request.status}. The admin creates the allocation round.`; }
  else if (mine && mine.claimable_discount > 0n) { title = "Claim your community discount."; detail = "The settled round credited your share to this wallet."; }
  else { title = "Your contribution is recorded."; detail = "You remain obligated to contribute in future rounds, including after receiving a payout."; }
  return <>
    <section className="next-action" aria-labelledby="next-action-title"><div><span className="app-kicker">NEXT ACTION / LIVE CONTRACT STATE</span><h2 id="next-action-title">{title}</h2><p>{detail}</p></div><div className="next-action__details">
      <div><span>MEMBERSHIP</span><strong>{!wallet ? "CONNECT WALLET" : s.isMember ? "MEMBER ✓" : "NOT JOINED"}</strong></div>
      <div><span>REQUIRED CONTRIBUTION</span><strong>{formatXlm(s.community.contribution_amount)}</strong></div>
      <div><span>WALLET BALANCE</span><strong>{s.balance === null ? "CONNECT WALLET" : formatXlm(s.balance)}</strong></div>
      <div><span>CURRENT CYCLE</span><strong>{cycle ? `#${cycle.id}` : "NOT CREATED"}</strong></div>
      {!wallet ? <button className="button button--dark" type="button" onClick={() => void connect()}>CONNECT WALLET ↗</button> : !s.isMember ? <button className="button button--dark" type="button" disabled={pending} onClick={() => void write(calls.join(wallet))}>JOIN COMMUNITY ↗</button> : canContribute ? <Link href="/app/cycle" className="button button--dark">CONTRIBUTE {formatXlm(s.community.contribution_amount)} ↗</Link> : <Link href={title.startsWith("Claim") ? "/app/history" : cycle?.active_round_id != null ? "/app/bid" : "/app/cycle"} className="button button--dark">{title.startsWith("Claim") ? "CLAIM DISCOUNT" : cycle?.active_round_id != null ? "VIEW ROUND" : "VIEW CYCLE"} ↗</Link>}
      {balanceLow && <small className="app-balance-warning">Your Testnet wallet needs at least {formatXlm(s.community.contribution_amount)} plus network fees to contribute. Fund it with Testnet XLM; this is not real XLM.</small>}
    </div></section>
    <div className="app-intro-panel"><div><span className="app-panel-kicker">CURRENT CYCLE / SHARED CAPITAL</span><h2>{cycle ? `Cycle #${cycle.id}` : "No active cycle"}<br /><em>{cycle ? `Round ${cycle.current_round_number} of ${cycle.members.length}` : "Ready for real members."}</em></h2><p>{cycle ? `${cycle.members.length} authenticated members belong to this cycle. The round pot comes from their fixed contributions.` : "Joined wallets will be included by the community admin. No illustrative cycle or pot is shown as live state."}</p><Link href="/app/cycle" className="app-panel-link">View current cycle <span>↗</span></Link></div><span className="app-intro-symbol" aria-hidden="true">{cycle ? String(cycle.current_round_number).padStart(2,"0") : "—"}<span>/{cycle?.members.length ?? "—"}</span></span></div>
    <div className="app-metrics"><div><span>ROUND</span><strong>{cycle ? `${cycle.current_round_number} / ${cycle.members.length}` : "—"}</strong><small>Current financing round</small></div><div><span>AVAILABLE POOL</span><strong>{formatXlm(s.pool.available_pool)}</strong><small>Contract accounting</small></div><div><span>MY CONTRIBUTION</span><strong>{mine ? mine.completed_contributions >= (cycle?.current_round_number ?? 0) ? "Paid ✓" : "Not paid" : "—"}</strong><small>Current round</small></div><div><span>MY PAYOUT STATUS</span><strong>{mine ? mine.payout_received ? "Received" : "Available" : "—"}</strong><small>One payout per cycle</small></div><div><span>CLAIMABLE DISCOUNT</span><strong>{formatXlm(mine?.claimable_discount ?? 0n)}</strong><small>Cycle credit</small></div></div>
    <div className="app-split"><div><span className="app-kicker">PROTOCOL / LIVE</span><h2>From a shared pot<br />to a history<br /><em>you can prove.</em></h2><p className="journey-note">Every value above comes from the deployed CommunityPool and native Testnet XLM asset.</p></div><ol className="app-journey-list">{[["Join community","/app"],["Contribute","/app/cycle"],["Request payout","/app/cycle"],["Commit and reveal","/app/bid"],["Financial history","/app/history"]].map(([label,href],index)=><li key={label}><span>{String(index+1).padStart(2,"0")}</span><Link href={href}>{label}</Link><small>↗</small></li>)}</ol></div>
  </>;
}
