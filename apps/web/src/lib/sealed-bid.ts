import { computeCommitment } from "./commitment.ts";

export type StoredBid = {
  contractId: string;
  wallet: string;
  cycleId: string;
  roundId: string;
  bidBaseUnits: string;
  secretHex: string;
  commitmentHex: string;
  createdAt: string;
  revealed: boolean;
};

export function toHex(bytes: Uint8Array): string {
  return Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join("");
}

export function fromHex(hex: string): Uint8Array {
  if (!/^(?:[0-9a-f]{2})+$/i.test(hex)) throw new Error("Invalid hexadecimal data.");
  return Uint8Array.from(hex.match(/.{2}/g)!, (pair) => Number.parseInt(pair, 16));
}

export function sealedBidKey(contractId: string, wallet: string, roundId: bigint | string): string {
  return `arth:sealed-bid:${contractId}:${wallet}:${roundId}`;
}

export function saveBid(storage: Pick<Storage, "setItem">, bid: StoredBid): void {
  storage.setItem(sealedBidKey(bid.contractId, bid.wallet, bid.roundId), JSON.stringify(bid));
}

export function restoreBid(storage: Pick<Storage, "getItem">, contractId: string, wallet: string, roundId: bigint | string): StoredBid | null {
  const raw = storage.getItem(sealedBidKey(contractId, wallet, roundId));
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw) as StoredBid;
    if (parsed.contractId !== contractId || parsed.wallet !== wallet || parsed.roundId !== String(roundId)) return null;
    if (!/^\d+$/.test(parsed.bidBaseUnits) || !(parsed.revealed && parsed.secretHex === "") && !/^[0-9a-f]{64}$/i.test(parsed.secretHex) || !/^[0-9a-f]{64}$/i.test(parsed.commitmentHex)) return null;
    return parsed;
  } catch { return null; }
}

export function clearRevealedSecret(storage: Pick<Storage, "setItem">, bid: StoredBid): StoredBid {
  const cleared = { ...bid, secretHex: "", revealed: true };
  saveBid(storage, cleared);
  return cleared;
}

export async function locallyMatchesCommitment(bid: StoredBid, onChainHex: string): Promise<boolean> {
  if (bid.revealed || !/^[0-9a-f]{64}$/i.test(bid.secretHex)) return false;
  const recomputed = await computeCommitment(BigInt(bid.roundId), bid.wallet, BigInt(bid.bidBaseUnits), fromHex(bid.secretHex));
  const hex = toHex(recomputed);
  return hex === bid.commitmentHex.toLowerCase() && hex === onChainHex.toLowerCase();
}
