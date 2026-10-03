"use client";
import { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import { computeCommitment } from "@/lib/commitment";
import { arth, calls } from "@/lib/arth-contract";
import { useArth, transactionPending } from "@/lib/arth-context";
import { formatXlm, parseXlm } from "@/lib/amount";
import { clearRevealedSecret, fromHex, locallyMatchesCommitment, restoreBid, saveBid, toHex, type StoredBid } from "@/lib/sealed-bid";
import { STELLAR_CONFIG } from "@/config/stellar";
import { LiveState } from "@/components/live-state";

export default function BidPage() {
  const { wallet, snapshot: s, write, txPhase, refresh } = useArth();
  const [amountText, setAmountText] = useState("");
  const [backupText, setBackupText] = useState("");
  const [stored, setStored] = useState<StoredBid | null>(null);
  const [chainCommitment, setChainCommitment] = useState<string | null>(null);
  const [ownReveal, setOwnReveal] = useState<bigint | null>(null);
  const [publicReveals, setPublicReveals] = useState<{ wallet: string; amount: bigint }[]>([]);
  const [localMatch, setLocalMatch] = useState<boolean | null>(null);
  const [loadedScope, setLoadedScope] = useState("");
  const [actionError, setActionError] = useState("");
  const round = s?.round;
  const scope = wallet && round ? `${STELLAR_CONFIG.contractId}:${wallet}:${round.id}` : "";
  const scopedStored = stored?.contractId === STELLAR_CONFIG.contractId && stored?.wallet === wallet && stored?.roundId === String(round?.id) ? stored : null;
  const scopedCommitment = loadedScope === scope ? chainCommitment : null;
  const scopedReveal = loadedScope === scope ? ownReveal : null;
  const scopedMatch = loadedScope === scope ? localMatch : null;
  const scopedPublicReveals = loadedScope === scope ? publicReveals : [];
  const active = Boolean(round && s?.cycle?.active_round_id === round.id);
  const participant = Boolean(wallet && round?.participants.includes(wallet));
  const busy = transactionPending(txPhase);
  let bidInputProblem = "Enter the XLM amount you would accept.";
  if (s && round && amountText.trim()) {
    try {
      const bid = parseXlm(amountText, s.assetDecimals);
      bidInputProblem = bid <= 0n ? "Enter a positive bid."
        : bid > round.available_capital ? "The bid exceeds this round's available capital."
        : s.request && bid > s.request.maximum_amount ? "The bid exceeds your request maximum."
        : "";
    } catch (error) {
      bidInputProblem = error instanceof Error ? error.message : "Enter a valid XLM amount.";
    }
  }
  const commitDisabledReason = !active ? "This allocation round is not active."
    : !s?.request ? "This wallet has no request included in the round."
    : s.ledgerTime >= round!.commit_deadline ? "The commit deadline has passed."
    : s.commitment !== null ? "This wallet already committed a bid."
    : bidInputProblem;
  const revealDisabledReason = scopedReveal !== null ? "Your bid is already revealed on-chain."
    : !scopedStored ? "Restore the original bid and secret from this browser or a private backup."
    : !scopedCommitment ? "Waiting for the on-chain commitment."
    : scopedMatch === null ? "Verifying the stored bid against its commitment."
    : !scopedMatch ? "The stored bid does not match the on-chain commitment. Reveal blocked."
    : s && round && s.ledgerTime >= round.reveal_deadline ? "The reveal deadline has passed."
    : "";

  const loadBid = useCallback(async () => {
    if (!wallet || !round) { setStored(null); setChainCommitment(null); setOwnReveal(null); setPublicReveals([]); setLoadedScope(""); return; }
    const local = restoreBid(localStorage, STELLAR_CONFIG.contractId, wallet, round.id);
    setStored(local);
    const [commitment, reveal] = await Promise.all([arth.getCommitment(round.id, wallet), arth.getReveal(round.id, wallet)]);
    const hex = commitment ? toHex(commitment) : null;
    setChainCommitment(hex); setOwnReveal(reveal);
    setLocalMatch(local && hex && !local.revealed ? await locallyMatchesCommitment(local, hex) : null);
    if (reveal !== null && local && !local.revealed) setStored(clearRevealedSecret(localStorage, local));
    if (round.status !== "Commit") {
      const amounts = await Promise.all(round.participants.map((address) => arth.getReveal(round.id, address)));
      setPublicReveals(amounts.flatMap((amount, index) => amount === null ? [] : [{ wallet: round.participants[index], amount }]));
    } else setPublicReveals([]);
    setLoadedScope(`${STELLAR_CONFIG.contractId}:${wallet}:${round.id}`);
  }, [wallet, round]);
  useEffect(() => { let active = true; queueMicrotask(() => { if (active) void loadBid().catch(() => { if (active) setActionError("Could not read the round commitment or reveal. Retry the read."); }); }); return () => { active = false; }; }, [loadBid, txPhase]);

  async function commit() {
    if (!wallet || !round || !s?.request || !s.cycle) return;
    try {
      setActionError("");
      if (scopedCommitment || await arth.getCommitment(round.id, wallet)) throw new Error("This wallet already has a commitment for this round.");
      const amount = parseXlm(amountText, s.assetDecimals);
      if (amount <= 0n || amount > round.available_capital || amount > s.request.maximum_amount) throw new Error("Bid must be positive and no greater than your request maximum or the round pot.");
      const secret = crypto.getRandomValues(new Uint8Array(32));
      const commitment = await computeCommitment(round.id, wallet, amount, secret);
      const record: StoredBid = { contractId: STELLAR_CONFIG.contractId, wallet, cycleId: String(s.cycle.id), roundId: String(round.id), bidBaseUnits: String(amount), secretHex: toHex(secret), commitmentHex: toHex(commitment), createdAt: new Date().toISOString(), revealed: false };
      saveBid(localStorage, record); setStored(record);
      const confirmed = await write(calls.commitBid(round.id, wallet, commitment));
      if (confirmed) { await loadBid(); await refresh(); }
    } catch (error) { setActionError(error instanceof Error ? error.message : "Could not commit bid."); }
  }

  async function reveal() {
    if (!wallet || !round || !scopedStored) return;
    try {
      setActionError("");
      if (!scopedCommitment || !(await locallyMatchesCommitment(scopedStored, scopedCommitment))) throw new Error("Local bid and secret do not match the immutable on-chain commitment. Reveal blocked.");
      const confirmed = await write(calls.revealBid(round.id, wallet, BigInt(scopedStored.bidBaseUnits), fromHex(scopedStored.secretHex)));
      if (confirmed) await loadBid();
    } catch (error) { setActionError(error instanceof Error ? error.message : "Could not reveal bid."); }
  }

  async function copyBackup() {
    if (!scopedStored || scopedStored.revealed) return;
    try { await navigator.clipboard.writeText(JSON.stringify(scopedStored)); setActionError("Reveal backup copied. Keep it private until your bid is revealed."); }
    catch { setActionError("Clipboard unavailable. Keep this browser data until reveal."); }
  }

  async function restoreBackup() {
    if (!wallet || !round || !scopedCommitment) return;
    try {
      const candidate = JSON.parse(backupText) as StoredBid;
      if (candidate.contractId !== STELLAR_CONFIG.contractId || candidate.wallet !== wallet || candidate.roundId !== String(round.id) || !(await locallyMatchesCommitment(candidate, scopedCommitment))) throw new Error("Backup does not match this wallet, round, and on-chain commitment.");
      saveBid(localStorage, candidate); setBackupText(""); await loadBid();
    } catch (error) { setActionError(error instanceof Error ? error.message : "Invalid reveal backup."); }
  }

  const phase = round?.status ?? "Empty";
  const contributed = s?.cycleMembers.filter((item) => s.cycle && item.completed_contributions >= s.cycle.current_round_number).length ?? 0;
  const required = s?.cycle?.members.length ?? 0;
  return <div className="app-screen"><div className="app-page-heading"><div><span className="app-kicker"><i /> 03 / SEALED BIDDING</span><h1>Your bid.<br /><em>Your timing.</em></h1><p>The lowest valid revealed bid receives this round’s discounted payout.</p></div><span className="app-page-index">{round ? `ROUND ${String(round.id).padStart(2,"0")} / ${phase.toUpperCase()}` : "NO ROUND YET"}</span></div>
    <LiveState>{s && !round && <div className="app-live-state"><strong>BIDDING NOT OPEN YET</strong><p>{!s.cycle ? "A cycle must be created from joined members first." : `Contribution pool: ${formatXlm(s.community.contribution_amount * BigInt(contributed), s.assetDecimals)} / ${formatXlm(s.community.contribution_amount * BigInt(required), s.assetDecimals)} ${contributed === required ? "READY" : "COLLECTING"}. Payout requests this round: ${s.pendingRequestCount}. ${contributed < required ? "Every cycle member must contribute before allocation." : s.pendingRequestCount === 0 ? "Next: eligible members submit payout requests." : "Next: the admin creates an allocation round from pending requests."}`}</p><Link className="text-link" href="/app/cycle">VIEW CURRENT CYCLE ↗</Link></div>}{s && round && <><div className="bid-preview-label">LIVE / STELLAR TESTNET <span>{phase === "Commit" ? "Commitments contain hashes only. Other bids are hidden." : "Only valid revealed bids are public."}</span></div><div className="bid-layout"><section className="bid-primary"><div className="bid-panel-top"><span>ARTH / ROUND {round.id.toString()}</span><span>{phase.toUpperCase()} PHASE</span></div>
      {phase === "Commit" && <><span className="bid-panel-kicker">CURRENT ROUND POT</span><div className="bid-big-value">{formatXlm(round.pot, s.assetDecimals)}</div>{participant ? scopedCommitment ? <><h2 className="bid-phase-title">BID SEALED.</h2><div className="bid-detail"><span>ON-CHAIN COMMITMENT</span><strong>{scopedCommitment.slice(0, 12)}…{scopedCommitment.slice(-8)}</strong></div><div className="bid-detail"><span>LOCAL BID</span><strong>{scopedStored ? formatXlm(scopedStored.bidBaseUnits, s.assetDecimals) : "SECRET MISSING"}</strong></div><p className="bid-field-note">{scopedMatch ? "Your locally stored bid still matches the immutable commitment." : "Keep your local bid and secret. Reveal requires the matching secret."}</p></> : <><label className="bid-input-label" htmlFor="bid-amount">AMOUNT YOU’LL ACCEPT / XLM</label><div className="bid-amount-input"><input id="bid-amount" type="text" inputMode="decimal" value={amountText} onChange={(event) => setAmountText(event.target.value)} placeholder="Amount in XLM" /></div><p className="bid-field-note">This is the discounted payout you would accept from the current pot. It is not loan principal. Maximum: {formatXlm(s.request?.maximum_amount ?? round.pot, s.assetDecimals)}.</p><button type="button" className="button button--dark bid-cta" disabled={busy || Boolean(commitDisabledReason)} onClick={() => void commit()}>LOCK MY BID ↗</button><small className="bid-action-note">{busy ? "Waiting for the current transaction." : commitDisabledReason || "Only the commitment hash is sent during Commit. Your amount and secret remain in this browser."}</small></> : <p className="bid-field-note">This wallet is not a participant in this round.</p>}{s.ledgerTime >= round.commit_deadline && active && <button className="preview-outline-button" type="button" disabled={busy} onClick={() => void write(calls.startReveal(round.id))}>START REVEAL PHASE ↗</button>}</>}
      {phase === "Reveal" && <><h2 className="bid-phase-title">Reveal the bid.<br /><em>Not before.</em></h2>{participant ? <><div className="bid-detail"><span>YOUR LOCALLY STORED BID</span><strong>{scopedStored ? formatXlm(scopedStored.bidBaseUnits, s.assetDecimals) : "NOT AVAILABLE"}</strong></div><div className="bid-detail"><span>LOCAL VERIFICATION</span><strong>{scopedReveal !== null ? "REVEALED ✓" : localMatch ? "MATCHES ON-CHAIN ✓" : "MISMATCH / SECRET MISSING"}</strong></div><p className="bid-field-note">{!scopedStored ? "This browser has no stored secret for your bid. Restore your private backup before revealing." : "Your locally stored bid is checked against the on-chain commitment before Freighter is asked to sign."}</p><button type="button" className="button button--dark bid-cta" disabled={busy || Boolean(revealDisabledReason)} onClick={() => void reveal()}>REVEAL BID ↗</button><small className="bid-action-note">{busy ? "Waiting for the current transaction." : revealDisabledReason || "The original bid and secret will be signed through Freighter."}</small></> : <p className="bid-field-note">This wallet is not a participant in this round.</p>}{s.ledgerTime >= round.reveal_deadline && <button className="preview-outline-button" type="button" disabled={busy} onClick={() => void write(calls.finalizeRound(round.id))}>FINALIZE ROUND ↗</button>}</>}
      {(phase === "Finalized" || phase === "Settled") && <><span className="bid-panel-kicker">ROUND RESULT / ON-CHAIN</span><h2 className="bid-phase-title">{round.winner ? round.winner === wallet ? "YOU WON THIS ROUND." : "ROUND WINNER SELECTED." : "NO VALID REVEAL."}</h2><div className="bid-detail"><span>WINNER</span><strong>{round.winner ? `${round.winner.slice(0,6)}…${round.winner.slice(-5)}` : "NONE"}</strong></div><div className="bid-detail"><span>WINNING PAYOUT</span><strong>{round.winning_bid === null ? "—" : formatXlm(round.winning_bid, s.assetDecimals)}</strong></div><div className="bid-detail"><span>ROUND POT</span><strong>{formatXlm(round.pot, s.assetDecimals)}</strong></div><div className="bid-detail"><span>COMMUNITY DISCOUNT</span><strong>{phase === "Settled" ? formatXlm(round.auction_discount, s.assetDecimals) : round.winning_bid === null ? "—" : formatXlm(round.pot - round.winning_bid, s.assetDecimals)}</strong></div>{phase === "Finalized" && round.winner === wallet && <button className="button button--dark bid-cta" type="button" disabled={busy} onClick={() => void write(calls.settle(round.id, wallet!))}>SETTLE MY PAYOUT ↗</button>}{phase === "Settled" && <p className="bid-field-note">Payout settled. The winner continues normal contributions in the remaining cycle rounds.</p>}<Link href="/app/history" className="text-link">See financial history and discount ↗</Link></>}
      {scopedStored && !scopedStored.revealed && <div className="app-secret-note"><p>Keep this browser data until your bid is revealed. Your secret is required to verify the bid. Never share it before reveal.</p><button type="button" className="preview-outline-button" onClick={() => void copyBackup()}>COPY PRIVATE REVEAL BACKUP</button></div>}
      {participant && scopedCommitment && !scopedStored && scopedReveal === null && <div className="app-secret-note"><p>Your local reveal secret is missing. Paste a private backup made for this wallet and round; it will be checked against the on-chain commitment.</p><label className="app-field">PRIVATE REVEAL BACKUP<textarea value={backupText} onChange={(event) => setBackupText(event.target.value)} /></label><button type="button" className="preview-outline-button" onClick={() => void restoreBackup()}>RESTORE BACKUP ↗</button></div>}
      {actionError && <p className="bid-field-note" role="alert">{actionError}</p>}
      <button type="button" className="app-refresh" onClick={() => { void refresh(); void loadBid(); }}>REFRESH ROUND STATE ↻</button>
    </section><aside className="bid-side"><span className="app-panel-kicker">HOW THE SEALED ROUND WORKS</span><h2>Private first.<br /><em>Public when it counts.</em></h2><div><span>01 / LOCK</span><p>Eligible participants commit a hash. Bid amounts stay hidden.</p></div><div><span>02 / REVEAL</span><p>The original bid and secret are checked after the commit deadline.</p></div><div><span>03 / ALLOCATE</span><p>The lowest valid bid wins. The discount is credited equally to cycle members.</p></div>{phase !== "Commit" && <div><span>VALID REVEALED BIDS</span>{scopedPublicReveals.length ? scopedPublicReveals.map((item)=><p key={item.wallet}>{item.wallet.slice(0,5)}…{item.wallet.slice(-4)} / {formatXlm(item.amount, s.assetDecimals)}</p>) : <p>None revealed yet.</p>}</div>}<p className="bid-side__note">Commit deadline: {new Date(Number(round.commit_deadline) * 1000).toLocaleString()}<br />Reveal deadline: {new Date(Number(round.reveal_deadline) * 1000).toLocaleString()}</p></aside></div></>}</LiveState>
  </div>;
}
