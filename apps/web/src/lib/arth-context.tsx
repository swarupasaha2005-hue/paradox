"use client";

import { createContext, useCallback, useContext, useEffect, useRef, useState } from "react";
import { getAddress, getNetworkDetails, isConnected, requestAccess } from "@stellar/freighter-api";
import { STELLAR_CONFIG } from "@/config/stellar";
import { arth, submitWrite, type CapitalRequest, type Community, type Cycle, type CycleHistory, type CycleMember, type Eligibility, type FinancialHistory, type Member, type PoolBalance, type Round, type TxPhase, type WriteCall } from "@/lib/arth-contract";

const SESSION_KEY = "arth:freighter-session";
type LoadState = "loading" | "success" | "error";

type Snapshot = {
  community: Community;
  pool: PoolBalance;
  assetDecimals: number;
  cycleMembers: CycleMember[];
  commitment: Uint8Array | null;
  reveal: bigint | null;
  balance: bigint | null;
  isMember: boolean;
  member: Member | null;
  eligibility: Eligibility | null;
  cycle: Cycle | null;
  cycleMember: CycleMember | null;
  history: FinancialHistory | null;
  cycleHistory: CycleHistory | null;
  request: CapitalRequest | null;
  round: Round | null;
  ledgerTime: bigint;
};

type ArthContext = {
  wallet: string | null;
  walletError: string | null;
  walletAvailable: boolean | null;
  network: string | null;
  loadState: LoadState;
  readError: string | null;
  snapshot: Snapshot | null;
  txPhase: TxPhase;
  txError: string | null;
  lastTxHash: string | null;
  connect: () => Promise<void>;
  disconnect: () => void;
  refresh: () => Promise<void>;
  write: (call: WriteCall) => Promise<boolean>;
};

const Context = createContext<ArthContext | null>(null);

function friendlyError(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  const contractCode = message.match(/Error\(Contract, #(\d+)\)/)?.[1];
  const contractErrors: Record<string, string> = {
    "4": "This wallet has already joined the community.",
    "8": "The contract says this wallet is not eligible.",
    "13": "The community pool has insufficient available funds.",
    "20": "The commit deadline has passed.",
    "21": "The reveal phase has not started.",
    "22": "The reveal deadline has passed.",
    "23": "This wallet has already committed a bid in this round.",
    "25": "The bid and secret do not match the on-chain commitment.",
    "26": "This bid has already been revealed.",
    "31": "No active cycle exists yet. Wait for the admin to create one.",
    "34": "This wallet is not included in the current cycle.",
    "35": "This round's contribution was already made.",
    "36": "Required contributions are incomplete.",
    "37": "A round is already active.",
    "38": "This wallet has already received a payout in this cycle.",
    "39": "There is no discount credit to claim.",
  };
  if (contractCode) return contractErrors[contractCode] ?? `The contract rejected this action (code ${contractCode}).`;
  if (/denied|reject|cancel/i.test(message)) return "Wallet request rejected. You can try again when ready.";
  if (/insufficient|balance|op_underfunded/i.test(message)) return "Insufficient Testnet XLM for this action and network fees.";
  if (/not.?eligible/i.test(message)) return "The contract says this wallet is not eligible for this action.";
  if (/already.?made/i.test(message)) return "This round's contribution was already made.";
  if (/already.?exists/i.test(message)) return "This action has already been recorded on-chain.";
  if (/CommitClosed/i.test(message)) return "The commit deadline has passed.";
  if (/RevealNotOpen/i.test(message)) return "The reveal phase has not started.";
  if (/RevealClosed/i.test(message)) return "The reveal deadline has passed.";
  if (/CommitmentMismatch/i.test(message)) return "The bid and secret do not match the on-chain commitment.";
  if (/NoDiscountToClaim/i.test(message)) return "There is no discount credit to claim.";
  return message.length > 220 ? "Transaction or contract read failed. Check the Testnet connection and try again." : message;
}

async function findCurrentRequest(community: Community, cycle: Cycle, wallet: string): Promise<CapitalRequest | null> {
  for (let last = community.request_counter; last > 0n;) {
    const first = last > 9n ? last - 9n : 1n;
    const ids = Array.from({ length: Number(last - first + 1n) }, (_, index) => last - BigInt(index));
    const requests = await Promise.all(ids.map((id) => arth.getRequest(id)));
    const found = requests.find((request) => request.member === wallet && request.cycle_id === cycle.id && request.cycle_round_number === cycle.current_round_number && (request.status === "Pending" || request.status === "IncludedInRound"));
    if (found) return found;
    if (requests.some((request) => request.cycle_id < cycle.id || request.cycle_id === cycle.id && request.cycle_round_number < cycle.current_round_number)) return null;
    last = first - 1n;
  }
  return null;
}

export function ArthProvider({ children }: { children: React.ReactNode }) {
  const [wallet, setWallet] = useState<string | null>(null);
  const [walletAvailable, setWalletAvailable] = useState<boolean | null>(null);
  const [walletError, setWalletError] = useState<string | null>(null);
  const [network, setNetwork] = useState<string | null>(null);
  const [loadState, setLoadState] = useState<LoadState>("loading");
  const [readError, setReadError] = useState<string | null>(null);
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [txPhase, setTxPhase] = useState<TxPhase>("idle");
  const [txError, setTxError] = useState<string | null>(null);
  const [lastTxHash, setLastTxHash] = useState<string | null>(null);
  const busy = useRef(false);
  const generation = useRef(0);

  const load = useCallback(async (address: string | null) => {
    const ticket = ++generation.current;
    setLoadState("loading");
    setReadError(null);
    try {
      const [community, pool, latest] = await Promise.all([arth.getCommunity(), arth.getPoolBalance(), arth.getLatestLedger()]);
      if (community.asset !== STELLAR_CONFIG.assetContractId) throw new Error("Configured asset differs from the community asset on-chain.");
      const assetDecimals = await arth.getAssetDecimals(community.asset);
      let balance: bigint | null = null, isMember = false, member: Member | null = null, eligibility: Eligibility | null = null, history: FinancialHistory | null = null;
      if (address) {
        [balance, isMember, eligibility] = await Promise.all([arth.getXlmBalance(address, community.asset), arth.isMember(address), arth.getEligibility(address, community.financing_limit)]);
        if (isMember) [member, history] = await Promise.all([arth.getMember(address), arth.getFinancialHistory(address)]);
      }
      const cycle = community.current_cycle_id === null ? null : await arth.getCycle(community.current_cycle_id);
      const included = Boolean(address && cycle?.members.includes(address));
      const [cycleMember, cycleHistory, round, request] = await Promise.all([
        included ? arth.getCycleMember(cycle!.id, address!) : Promise.resolve(null),
        included ? arth.getCycleHistory(cycle!.id, address!) : Promise.resolve(null),
        cycle?.active_round_id != null ? arth.getRound(cycle.active_round_id) : cycle && community.round_counter > 0n ? arth.getRound(community.round_counter) : Promise.resolve(null),
        included ? findCurrentRequest(community, cycle!, address!) : Promise.resolve(null),
      ]);
      const currentRound = round?.cycle_id === cycle?.id ? round : null;
      const roundRequest = address && currentRound?.participants.includes(address)
        ? (await Promise.all(currentRound.request_ids.map((id) => arth.getRequest(id)))).find((item) => item.member === address) ?? null
        : null;
      const [cycleMembers, commitment, reveal] = await Promise.all([
        cycle ? Promise.all(cycle.members.map((participant) => arth.getCycleMember(cycle.id, participant))) : Promise.resolve([]),
        address && currentRound?.participants.includes(address) ? arth.getCommitment(currentRound.id, address) : Promise.resolve(null),
        address && currentRound?.participants.includes(address) && currentRound.status !== "Commit" ? arth.getReveal(currentRound.id, address) : Promise.resolve(null),
      ]);
      if (ticket === generation.current) {
        setSnapshot({ community, pool, assetDecimals, cycleMembers, commitment, reveal, balance, isMember, member, eligibility, cycle, cycleMember, history, cycleHistory, round: currentRound, request: roundRequest ?? request, ledgerTime: BigInt(latest.closeTime) });
        setLoadState("success");
      }
    } catch (error) {
      if (ticket === generation.current) { setSnapshot(null); setReadError(friendlyError(error)); setLoadState("error"); }
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    async function restore() {
      const detected = await isConnected();
      if (cancelled) return;
      setWalletAvailable(Boolean(detected.isConnected));
      if (detected.isConnected && localStorage.getItem(SESSION_KEY) === "connected") {
        const [savedAddress, details] = await Promise.all([getAddress(), getNetworkDetails()]);
        if (!cancelled && !savedAddress.error && savedAddress.address && !details.error && details.networkPassphrase === STELLAR_CONFIG.networkPassphrase) {
          setWallet(savedAddress.address); setNetwork(details.network); void load(savedAddress.address); return;
        }
      }
      if (!cancelled) void load(null);
    }
    void restore();
    return () => { cancelled = true; };
  }, [load]);

  const connect = useCallback(async () => {
    setWalletError(null);
    try {
      const detected = await isConnected();
      setWalletAvailable(Boolean(detected.isConnected));
      if (!detected.isConnected) throw new Error("Freighter is unavailable. Install or unlock Freighter to connect.");
      const details = await getNetworkDetails();
      if (details.error || details.networkPassphrase !== STELLAR_CONFIG.networkPassphrase) throw new Error("Select Stellar Testnet in Freighter before connecting.");
      const access = await requestAccess();
      if (access.error || !access.address) throw new Error("Freighter account access was rejected.");
      localStorage.setItem(SESSION_KEY, "connected");
      setWallet(access.address); setNetwork(details.network); setTxPhase("idle"); setTxError(null);
      await load(access.address);
    } catch (error) { setWalletError(friendlyError(error)); }
  }, [load]);

  const disconnect = useCallback(() => {
    localStorage.removeItem(SESSION_KEY);
    generation.current++;
    setWallet(null); setNetwork(null); setWalletError(null); setSnapshot(null); setTxPhase("idle"); setTxError(null); setLastTxHash(null);
    void load(null);
  }, [load]);

  const refresh = useCallback(() => load(wallet), [load, wallet]);
  const write = useCallback(async (call: WriteCall) => {
    if (busy.current) return false;
    if (!wallet) { setTxError("Connect your Testnet wallet first."); setTxPhase("failed"); return false; }
    busy.current = true; setTxPhase("idle"); setTxError(null); setLastTxHash(null);
    try {
      const details = await getNetworkDetails();
      if (details.error || details.networkPassphrase !== STELLAR_CONFIG.networkPassphrase) throw new Error("Select Stellar Testnet in Freighter before signing.");
      const selected = await getAddress();
      if (selected.error || selected.address !== wallet) throw new Error("Freighter account changed. Disconnect and reconnect the selected wallet.");
      const hash = await submitWrite(wallet, call, setTxPhase);
      setLastTxHash(hash);
      await load(wallet);
      return true;
    } catch (error) {
      setTxError(friendlyError(error)); setTxPhase("failed"); return false;
    } finally { busy.current = false; }
  }, [wallet, load]);

  return <Context.Provider value={{ wallet, walletError, walletAvailable, network, loadState, readError, snapshot, txPhase, txError, lastTxHash, connect, disconnect, refresh, write }}>{children}</Context.Provider>;
}

export function useArth(): ArthContext {
  const value = useContext(Context);
  if (!value) throw new Error("ArthProvider is missing.");
  return value;
}

export function transactionPending(phase: TxPhase): boolean {
  return phase === "awaiting signature" || phase === "submitting";
}
