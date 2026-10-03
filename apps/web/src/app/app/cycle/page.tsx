"use client";
import { useState } from "react";
import Link from "next/link";
import { arth, calls } from "@/lib/arth-contract";
import { useArth, transactionPending } from "@/lib/arth-context";
import { formatXlm, parseXlm } from "@/lib/amount";
import { LiveState } from "@/components/live-state";

export default function CyclePage() {
  const { wallet, snapshot: s, write, txPhase } = useArth();
  const [requestAmount, setRequestAmount] = useState("");
  const [purpose, setPurpose] = useState("");
  const [eligibilityReason, setEligibilityReason] = useState("");
  const [membersInput, setMembersInput] = useState("");
  const [requestIds, setRequestIds] = useState("");
  const [commitMinutes, setCommitMinutes] = useState("10");
  const [revealMinutes, setRevealMinutes] = useState("10");
  const [actionError, setActionError] = useState("");
  const cycle = s?.cycle;
  const rows = s?.cycleMembers ?? [];
  let requestAmountBase: bigint | null = null;
  let requestAmountProblem = "Enter the payout amount in XLM.";
  if (s && requestAmount.trim()) {
    try {
      requestAmountBase = parseXlm(requestAmount, s.assetDecimals);
      requestAmountProblem = requestAmountBase <= 0n
        ? "Enter a positive payout amount."
        : requestAmountBase > s.community.financing_limit
          ? `Maximum request: ${formatXlm(s.community.financing_limit, s.assetDecimals)}.`
          : "";
    } catch (error) {
      requestAmountProblem = error instanceof Error ? error.message : "Enter a valid XLM amount.";
    }
  }
  const requestDisabledReason = !wallet ? "Connect your Testnet wallet first."
    : !s?.isMember ? "Join the community first."
    : !cycle ? "Wait for the admin to create a cycle."
    : !s.cycleMember ? "This wallet is not in the current cycle."
    : cycle.complete ? "This cycle is complete."
    : cycle.active_round_id !== null ? "Payout requests close while an allocation round is active."
    : !s.eligibility?.current_round_contribution_met ? "Complete this round's contribution first."
    : !s.eligibility?.contribution_requirement_met ? `Needs ${s.community.minimum_contributions} contributions.`
    : !s.eligibility?.payout_not_received ? "This wallet already received its cycle payout."
    : !s.eligibility.eligible ? "The contract reports this wallet is not eligible."
    : requestAmountProblem;
  const requestPending = transactionPending(txPhase);

  async function requestPayout() {
    if (!wallet || !s) return;
    try {
      setActionError(""); setEligibilityReason("");
      const amount = parseXlm(requestAmount, s.assetDecimals);
      const check = await arth.getEligibility(wallet, amount);
      if (!check.eligible) {
        const reason = !check.in_cycle ? "Wallet is not in this cycle." : !check.current_round_contribution_met ? "Complete this round's contribution first." : !check.contribution_requirement_met ? `Needs ${check.minimum_contributions} contributions.` : !check.payout_not_received ? "Already received a payout in this cycle." : !check.within_financing_limit ? "Amount exceeds the financing limit." : !check.valid_amount ? "Enter a positive amount." : "Not eligible under the current contract rules.";
        setEligibilityReason(reason); return;
      }
      const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(purpose.trim() || "Arth payout request")));
      await write(calls.requestCapital(wallet, amount, digest));
    } catch (error) { setActionError(error instanceof Error ? error.message : "Unable to prepare request."); }
  }

  async function createCycle() {
    if (!wallet) return;
    try {
      setActionError("");
      const members = membersInput.split(/[\s,]+/).filter(Boolean);
      if (members.length < 1 || members.length > 10 || new Set(members).size !== members.length) throw new Error("Enter 1–10 distinct joined wallet addresses.");
      const joined = await Promise.all(members.map((address) => arth.isMember(address)));
      if (joined.some((value) => !value)) throw new Error("At least one address has not joined the community.");
      await write(calls.createCycle(wallet, members));
    } catch (error) { setActionError(error instanceof Error ? error.message : "Could not create cycle."); }
  }

  async function createRound() {
    if (!wallet || !s) return;
    try {
      setActionError("");
      const ids = requestIds.split(/[\s,]+/).filter(Boolean).map((value) => BigInt(value));
      if (!ids.length || ids.length > 10 || ids.some((id) => id <= 0n)) throw new Error("Enter 1–10 valid request IDs.");
      const now = BigInt((await arth.getLatestLedger()).closeTime);
      const commit = now + BigInt(commitMinutes) * 60n;
      const reveal = commit + BigInt(revealMinutes) * 60n;
      if (commit <= now || reveal <= commit) throw new Error("Deadlines must be in the future.");
      await write(calls.createRound(wallet, ids, commit, reveal));
    } catch (error) { setActionError(error instanceof Error ? error.message : "Could not create round."); }
  }

  return <div className="app-screen"><div className="app-page-heading"><div><span className="app-kicker"><i /> 02 / CURRENT CYCLE</span><h1>Progress is<br /><em>shared.</em></h1><p>Each member’s fixed contribution forms this round’s pot.</p></div><span className="app-page-index">{cycle ? `CYCLE ${String(cycle.id).padStart(3,"0")} / ROUND ${String(cycle.current_round_number).padStart(2,"0")}` : "NO CYCLE YET"}</span></div>
    <LiveState>{s && <>
      {!cycle ? <section className="cycle-contribution"><div className="cycle-contribution__main"><span className="app-panel-kicker">COMMUNITY / LIVE TESTNET</span><h2>Waiting for cycle creation.</h2><p>{s.community.member_count} wallet{s.community.member_count === 1 ? " has" : "s have"} joined. A cycle is created by the community admin from real joined addresses.</p><div className="cycle-contribution__facts"><div><span>FIXED CONTRIBUTION</span><strong>{formatXlm(s.community.contribution_amount, s.assetDecimals)}</strong></div><div><span>ROUND POT</span><strong>NOT CREATED</strong></div></div></div><div className="cycle-contribution__aside"><span className="app-panel-kicker">YOUR MEMBERSHIP</span><strong>{!wallet ? "CONNECT" : s.isMember ? "MEMBER ✓" : "NOT JOINED"}</strong><p>{s.isMember ? "Your wallet is ready for admin cycle inclusion." : "Connect and join the community to become eligible for a cycle."}</p>{wallet && !s.isMember && <button className="preview-outline-button" type="button" disabled={transactionPending(txPhase)} onClick={() => void write(calls.join(wallet))}>JOIN COMMUNITY ↗</button>}{wallet && <><small>Wallet address for the admin: <code>{wallet}</code></small><button className="preview-outline-button" type="button" onClick={() => void navigator.clipboard.writeText(wallet)}>COPY WALLET ADDRESS</button></>}</div></section> : <>
        <section className="cycle-contribution"><div className="cycle-contribution__main"><span className="app-panel-kicker">CURRENT ROUND / {String(cycle.current_round_number).padStart(2,"0")} OF {cycle.members.length}</span><h2>{s.cycleMember?.completed_contributions === cycle.current_round_number ? "Your part is in." : "Your contribution is next."}</h2><p>Every cycle member contributes the fixed amount for the current round. Once all required contributions are collected, the pot is ready for allocation.</p><div className="cycle-contribution__facts"><div><span>YOUR CONTRIBUTION</span><strong>{formatXlm(s.community.contribution_amount, s.assetDecimals)}</strong></div><div><span>STATUS</span><strong>{!s.cycleMember ? "NOT IN CYCLE" : s.cycleMember.completed_contributions >= cycle.current_round_number ? "PAID ✓" : "NOT PAID"}</strong></div></div><button type="button" className="button button--dark cycle-contribution__button" disabled={!wallet || !s.cycleMember || cycle.complete || cycle.active_round_id !== null || s.cycleMember.completed_contributions >= cycle.current_round_number || s.balance === null || s.balance < s.community.contribution_amount || transactionPending(txPhase)} onClick={() => wallet && void write(calls.contribute(wallet))}>{s.cycleMember && s.cycleMember.completed_contributions >= cycle.current_round_number ? "CONTRIBUTION COMPLETE ✓" : `CONTRIBUTE ${formatXlm(s.community.contribution_amount, s.assetDecimals)} ↗`}</button><small>{!wallet ? "Connect your Testnet wallet first." : !s.cycleMember ? "This wallet is not included in this cycle." : s.cycleMember.completed_contributions >= cycle.current_round_number ? "Contribution confirmed on Testnet." : cycle.complete ? "This cycle is complete." : cycle.active_round_id !== null ? "The current allocation round is active; contributions are closed." : s.balance === null ? "Reading this wallet’s XLM balance." : s.balance < s.community.contribution_amount ? "Insufficient Testnet XLM. Fund this wallet for the contribution and network fee." : transactionPending(txPhase) ? "Waiting for the current transaction." : "Freighter signs the real Testnet token transfer."}</small></div><div className="cycle-contribution__aside"><span className="app-panel-kicker">COMMUNITY / CYCLE {cycle.id.toString().padStart(3,"0")}</span><strong>{s.cycleMember ? "INCLUDED" : "NOT INCLUDED"}</strong><p>{cycle.members.length} authenticated member{cycle.members.length === 1 ? "" : "s"} in this cycle. A payout does not end later contribution obligations.</p>{wallet && !s.isMember && <><button className="preview-outline-button" type="button" disabled={transactionPending(txPhase)} onClick={() => void write(calls.join(wallet))}>JOIN COMMUNITY ↗</button><small>Membership is recorded now; cycle inclusion is set by the admin.</small></>}</div></section>
        <section className="pot-progress"><div className="pot-progress__header"><span className="app-kicker">THE POT / BUILT TOGETHER</span><span>{rows.every((row) => row.completed_contributions >= cycle.current_round_number) ? "POT READY" : "COLLECTING CONTRIBUTIONS"}</span></div><div className="pot-equation"><div><span>MEMBERS</span><strong>{cycle.members.length}</strong></div><b>×</b><div><span>EACH CONTRIBUTES</span><strong>{formatXlm(s.community.contribution_amount, s.assetDecimals)}</strong></div><b>=</b><div><span>ROUND POT</span><strong>{formatXlm(s.community.contribution_amount * BigInt(cycle.members.length), s.assetDecimals)}</strong></div></div><div className="pot-progress__collected"><div><span>COLLECTED THIS ROUND</span><strong>{formatXlm(s.community.contribution_amount * BigInt(rows.filter((row) => row.completed_contributions >= cycle.current_round_number).length), s.assetDecimals)} <small>/ {formatXlm(s.community.contribution_amount * BigInt(cycle.members.length), s.assetDecimals)}</small></strong></div><div className="pot-progress__track"><span style={{ width: `${rows.length ? rows.filter((row) => row.completed_contributions >= cycle.current_round_number).length * 100 / cycle.members.length : 0}%` }} /></div><p>{rows.filter((row) => row.completed_contributions >= cycle.current_round_number).length} / {cycle.members.length} MEMBERS CONTRIBUTED</p></div></section>
        <div className="cycle-columns"><section className="cycle-member-list"><div className="app-section-head"><h2>Member contributions</h2><span>ROUND {cycle.current_round_number} / ON-CHAIN WALLETS</span></div><div className="cycle-member-list__rows">{rows.map((row,index)=><div key={row.member}><span>{String(index+1).padStart(2,"0")}</span><strong>{row.member === wallet ? "YOU" : `${row.member.slice(0,5)}…${row.member.slice(-4)}`}</strong><small className={row.completed_contributions >= cycle.current_round_number ? "" : "is-pending"}>{row.completed_contributions >= cycle.current_round_number ? "PAID ✓" : "PENDING"}</small></div>)}</div></section><section className="cycle-request"><span className="app-kicker">NEXT / PAYOUT REQUEST</span><h2>Request this round’s<br /><em>payout.</em></h2><p>Eligible members who have not received a payout in this cycle can enter the allocation round.</p><div className="cycle-request__status"><span>ELIGIBILITY</span><strong>{s.eligibility?.eligible ? "ELIGIBLE" : "NOT YET ELIGIBLE"}</strong></div>{s.request ? <p>Your request #{s.request.id} is {s.request.status}. Waiting for the allocation round.</p> : <><label className="app-field">MAXIMUM PAYOUT / XLM<input value={requestAmount} inputMode="decimal" onChange={(event) => setRequestAmount(event.target.value)} /></label><label className="app-field">PURPOSE REFERENCE / HASHED<input value={purpose} maxLength={100} onChange={(event) => setPurpose(event.target.value)} placeholder="Optional short purpose" /></label><button type="button" className="button button--dark" disabled={Boolean(requestDisabledReason) || requestPending || requestAmountBase === null} onClick={() => void requestPayout()}>REQUEST PAYOUT ↗</button><small role="status">{requestPending ? "Waiting for the current transaction." : requestDisabledReason || "Your amount is valid. Freighter will sign a real Testnet request."}</small></>}{eligibilityReason && <small role="alert">{eligibilityReason}</small>}{!s.eligibility?.contribution_requirement_met && s.cycleMember && <small>Needs {s.community.minimum_contributions} contributions.</small>}<Link href="/app/bid" className="text-link">See sealed bidding <span>↗</span></Link></section></div>
        <div className="app-section-head cycle-timeline-heading"><h2>Cycle timeline</h2><span>ONE PAYOUT PER MEMBER</span></div><div className="cycle-timeline">{Array.from({length:cycle.members.length},(_,index)=>{const round=index+1;return <div className={`cycle-row${round===cycle.current_round_number?" cycle-row--active":""}`} key={round}><span>{String(round).padStart(2,"0")}</span><strong>ROUND {String(round).padStart(2,"0")}</strong><span>{round<cycle.current_round_number?"SETTLED ✓":round===cycle.current_round_number?cycle.active_round_id!==null?"ALLOCATION ACTIVE":"CONTRIBUTING":"UPCOMING —"}</span><b>{round===cycle.current_round_number?"↗":"—"}</b></div>})}</div><div className="cycle-note"><span className="app-kicker">AFTER A PAYOUT</span><p>Winning a round does not end membership. The member continues the normal fixed contribution through the remaining cycle rounds.</p></div>
      </>}
      {wallet === s.community.admin && <section className="app-admin"><span className="app-kicker">ADMIN / TESTNET SETUP</span><h2>Protocol controls</h2><p>Only the configured admin wallet can sign these transactions. If this CLI admin identity is unavailable in Freighter, run the admin calls through Stellar CLI.</p>{!cycle ? <><label className="app-field">JOINED WALLET ADDRESSES / ONE PER LINE<textarea value={membersInput} onChange={(event) => setMembersInput(event.target.value)} placeholder="Paste authenticated G… addresses" /></label><button className="preview-outline-button" type="button" disabled={transactionPending(txPhase)} onClick={() => void createCycle()}>CREATE CYCLE ↗</button></> : !cycle.complete && cycle.active_round_id === null ? <><label className="app-field">PENDING REQUEST IDS / COMMA SEPARATED<input value={requestIds} onChange={(event) => setRequestIds(event.target.value)} placeholder="1, 2, 3" /></label><label className="app-field">COMMIT WINDOW / MINUTES<input value={commitMinutes} onChange={(event) => setCommitMinutes(event.target.value)} /></label><label className="app-field">REVEAL WINDOW / MINUTES<input value={revealMinutes} onChange={(event) => setRevealMinutes(event.target.value)} /></label><button className="preview-outline-button" type="button" disabled={transactionPending(txPhase)} onClick={() => void createRound()}>CREATE ROUND ↗</button></> : <p>Round creation is unavailable while another round is active or the cycle is complete.</p>}</section>}
      {actionError && <p className="app-feedback app-feedback--error" role="alert">{actionError}</p>}
    </>}</LiveState>
  </div>;
}
