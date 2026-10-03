import { Address, BASE_FEE, Contract, TransactionBuilder, nativeToScVal, rpc, scValToNative, xdr } from "@stellar/stellar-sdk";
import { signTransaction } from "@stellar/freighter-api";
import { STELLAR_CONFIG } from "@/config/stellar";

// Public Testnet account used only as the source for unsigned read simulations.
const READ_SOURCE = "GCGQGZ52W5IYLPBWSX3CHAXT75D4YJFTSQ4JXNLZ6RT7AFJ4IYJKKNWJ";
const server = new rpc.Server(STELLAR_CONFIG.rpcUrl);

export type Community = { admin: string; asset: string; initialized: boolean; contribution_amount: bigint; minimum_contributions: number; financing_limit: bigint; member_count: number; available_pool: bigint; reserved_pool: bigint; discount_liability: bigint; current_cycle_id: bigint | null; cycle_counter: bigint; round_counter: bigint; request_counter: bigint };
export type PoolBalance = { available_pool: bigint; reserved_pool: bigint; discount_liability: bigint; token_balance: bigint };
export type Member = { active: boolean; contributions_completed: number; total_contributed: bigint; cycles_joined: number; cycles_completed: number; payouts_received: number; total_payouts_received: bigint; post_payout_contributions: number; discounts_earned: bigint; discounts_claimed: bigint };
export type Cycle = { id: bigint; members: string[]; current_round_number: number; active_round_id: bigint | null; payouts_completed: number; complete: boolean };
export type CycleMember = { member: string; cycle_id: bigint; payout_received: boolean; expected_contributions: number; completed_contributions: number; contributions_after_payout: number; claimable_discount: bigint; payout_amount: bigint };
export type Eligibility = { eligible: boolean; is_member: boolean; active: boolean; in_cycle: boolean; contribution_requirement_met: boolean; current_round_contribution_met: boolean; no_unresolved_default: boolean; payout_not_received: boolean; valid_amount: boolean; within_financing_limit: boolean; contributions_completed: number; minimum_contributions: number };
export type FinancialHistory = { contributions_completed: number; total_contributed: bigint; cycles_joined: number; cycles_completed: number; payouts_received: number; total_payouts_received: bigint; post_payout_contributions: number; discounts_earned: bigint; discounts_claimed: bigint; current_cycle_id: bigint | null; current_cycle_payout_received: boolean; current_cycle_expected: number; current_cycle_completed: number; cycle_ids: bigint[] };
export type CycleHistory = { cycle_id: bigint; cycle_complete: boolean; expected_contributions: number; completed_contributions: number; contributions_before_payout: number; contributions_after_payout: number; post_payout_expected_to_date: number; post_payout_required_total: number; payout_received: boolean; payout_amount: bigint; discounts_earned: bigint; discounts_claimed: bigint; claimable_discount: bigint; obligation_complete: boolean };
export type CapitalRequest = { id: bigint; member: string; maximum_amount: bigint; cycle_id: bigint; cycle_round_number: number; status: string };
export type Round = { id: bigint; cycle_id: bigint; cycle_round_number: number; participants: string[]; request_ids: bigint[]; available_capital: bigint; pot: bigint; commit_deadline: bigint; reveal_deadline: bigint; status: string; winner: string | null; winning_bid: bigint | null; auction_discount: bigint; discount_per_member: bigint; discount_remainder: number; valid_reveal_count: number; settled: boolean };
export type TxPhase = "idle" | "awaiting signature" | "submitting" | "confirmed" | "failed";

const address = (value: string) => new Address(value).toScVal();
const u64 = (value: bigint) => nativeToScVal(value, { type: "u64" });
const i128 = (value: bigint) => nativeToScVal(value, { type: "i128" });
const bytes32 = (value: Uint8Array) => { if (value.length !== 32) throw new Error("Expected 32 bytes."); return xdr.ScVal.scvBytes(value); };
const addresses = (values: string[]) => xdr.ScVal.scvVec(values.map(address));
const ids = (values: bigint[]) => xdr.ScVal.scvVec(values.map(u64));

function configuredContract(): Contract {
  if (!STELLAR_CONFIG.contractId || !STELLAR_CONFIG.assetContractId) throw new Error("Testnet contract configuration is missing.");
  return new Contract(STELLAR_CONFIG.contractId);
}

async function read<T>(contract: Contract, method: string, args: xdr.ScVal[] = []): Promise<T> {
  const source = await server.getAccount(READ_SOURCE);
  const tx = new TransactionBuilder(source, { fee: BASE_FEE, networkPassphrase: STELLAR_CONFIG.networkPassphrase })
    .addOperation(contract.call(method, ...args)).setTimeout(60).build();
  const simulation = await server.simulateTransaction(tx);
  if (!rpc.Api.isSimulationSuccess(simulation) || !simulation.result) {
    throw new Error(`Contract read ${method} failed: ${"error" in simulation ? String(simulation.error) : "no result"}`);
  }
  return scValToNative(simulation.result.retval) as T;
}

function enumName(value: unknown): string {
  if (typeof value === "string") return value;
  if (Array.isArray(value) && typeof value[0] === "string") return value[0];
  throw new Error("Unexpected contract enum representation.");
}

export const arth = {
  getCommunity: () => read<Community>(configuredContract(), "get_community"),
  getPoolBalance: () => read<PoolBalance>(configuredContract(), "get_pool_balance"),
  isMember: (wallet: string) => read<boolean>(configuredContract(), "is_member", [address(wallet)]),
  getMember: (wallet: string) => read<Member>(configuredContract(), "get_member", [address(wallet)]),
  getEligibility: (wallet: string, amount: bigint) => read<Eligibility>(configuredContract(), "get_eligibility", [address(wallet), i128(amount)]),
  getCycle: (cycleId: bigint) => read<Cycle>(configuredContract(), "get_cycle", [u64(cycleId)]),
  getCycleMember: (cycleId: bigint, wallet: string) => read<CycleMember>(configuredContract(), "get_cycle_member", [u64(cycleId), address(wallet)]),
  getFinancialHistory: (wallet: string) => read<FinancialHistory>(configuredContract(), "get_financial_history", [address(wallet)]),
  getCycleHistory: (cycleId: bigint, wallet: string) => read<CycleHistory>(configuredContract(), "get_cycle_history", [u64(cycleId), address(wallet)]),
  getRequest: async (requestId: bigint) => { const request = await read<CapitalRequest>(configuredContract(), "get_request", [u64(requestId)]); return { ...request, status: enumName(request.status) }; },
  getRound: async (roundId: bigint) => { const round = await read<Round>(configuredContract(), "get_round", [u64(roundId)]); return { ...round, status: enumName(round.status) }; },
  getCommitment: async (roundId: bigint, wallet: string) => {
    const result = await read<Uint8Array | null>(configuredContract(), "get_commitment", [u64(roundId), address(wallet)]);
    return result === null ? null : new Uint8Array(result);
  },
  getReveal: (roundId: bigint, wallet: string) => read<bigint | null>(configuredContract(), "get_reveal", [u64(roundId), address(wallet)]),
  getXlmBalance: (wallet: string) => read<bigint>(new Contract(STELLAR_CONFIG.assetContractId), "balance", [address(wallet)]),
  getLatestLedger: () => server.getLatestLedger(),
};

export type WriteCall = { method: string; args: xdr.ScVal[] };
export const calls = {
  join: (wallet: string): WriteCall => ({ method: "join", args: [address(wallet)] }),
  createCycle: (admin: string, members: string[]): WriteCall => ({ method: "create_cycle", args: [address(admin), addresses(members)] }),
  contribute: (wallet: string): WriteCall => ({ method: "contribute", args: [address(wallet)] }),
  requestCapital: (wallet: string, amount: bigint, purposeHash: Uint8Array): WriteCall => ({ method: "request_capital", args: [address(wallet), i128(amount), bytes32(purposeHash)] }),
  createRound: (admin: string, requestIds: bigint[], commitDeadline: bigint, revealDeadline: bigint): WriteCall => ({ method: "create_round", args: [address(admin), ids(requestIds), u64(commitDeadline), u64(revealDeadline)] }),
  commitBid: (roundId: bigint, wallet: string, commitment: Uint8Array): WriteCall => ({ method: "commit_bid", args: [u64(roundId), address(wallet), bytes32(commitment)] }),
  startReveal: (roundId: bigint): WriteCall => ({ method: "start_reveal", args: [u64(roundId)] }),
  revealBid: (roundId: bigint, wallet: string, amount: bigint, secret: Uint8Array): WriteCall => ({ method: "reveal_bid", args: [u64(roundId), address(wallet), i128(amount), bytes32(secret)] }),
  finalizeRound: (roundId: bigint): WriteCall => ({ method: "finalize_round", args: [u64(roundId)] }),
  settle: (roundId: bigint, winner: string): WriteCall => ({ method: "settle", args: [u64(roundId), address(winner)] }),
  claimDiscount: (cycleId: bigint, wallet: string): WriteCall => ({ method: "claim_discount", args: [u64(cycleId), address(wallet)] }),
};

export async function submitWrite(wallet: string, call: WriteCall, setPhase: (phase: TxPhase) => void): Promise<string> {
  const source = await server.getAccount(wallet);
  const tx = new TransactionBuilder(source, { fee: BASE_FEE, networkPassphrase: STELLAR_CONFIG.networkPassphrase })
    .addOperation(configuredContract().call(call.method, ...call.args)).setTimeout(120).build();
  const prepared = await server.prepareTransaction(tx);
  setPhase("awaiting signature");
  const signed = await signTransaction(prepared.toXDR(), { networkPassphrase: STELLAR_CONFIG.networkPassphrase, address: wallet });
  if (signed.error || !signed.signedTxXdr || signed.signerAddress !== wallet) throw new Error("Freighter signature was rejected or came from another account.");
  setPhase("submitting");
  const envelope = TransactionBuilder.fromXDR(signed.signedTxXdr, STELLAR_CONFIG.networkPassphrase);
  const sent = await server.sendTransaction(envelope);
  if (sent.status === "ERROR") throw new Error("Testnet RPC rejected the transaction.");
  for (let attempt = 0; attempt < 30; attempt++) {
    const confirmed = await server.getTransaction(sent.hash);
    if (confirmed.status === "SUCCESS") { setPhase("confirmed"); return sent.hash; }
    if (confirmed.status === "FAILED") throw new Error(`Testnet transaction failed. Hash: ${sent.hash}`);
    await new Promise((resolve) => setTimeout(resolve, 1500));
  }
  throw new Error(`Confirmation timed out. Check transaction ${sent.hash} before retrying.`);
}
