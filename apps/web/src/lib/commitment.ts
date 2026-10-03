import { Address, xdr } from "@stellar/stellar-sdk";

export const COMMITMENT_DOMAIN = "COMMUNITY_FINANCE_BID_V1";
const MAX_U64 = (1n << 64n) - 1n;
const MAX_I128 = (1n << 127n) - 1n;

function unsignedBytes(value: bigint, length: number): Uint8Array {
  const bytes = new Uint8Array(length);
  for (let index = length - 1; index >= 0; index--) {
    bytes[index] = Number(value & 255n);
    value >>= 8n;
  }
  return bytes;
}

export function encodeCommitment(
  roundId: bigint,
  participant: string,
  bidAmount: bigint,
  secret: Uint8Array,
): Uint8Array {
  if (roundId < 0n || roundId > MAX_U64) throw new RangeError("roundId must be u64");
  if (bidAmount <= 0n || bidAmount > MAX_I128) throw new RangeError("bidAmount must be positive i128");
  if (secret.length !== 32) throw new RangeError("secret must be 32 bytes");
  if (!participant.startsWith("G")) throw new Error("participant must be a Stellar wallet address");

  const addressXdr = xdr.ScVal.scvAddress(Address.fromString(participant).toScAddress()).toXDR();
  const domain = new TextEncoder().encode(COMMITMENT_DOMAIN);
  const result = new Uint8Array(domain.length + 8 + 4 + addressXdr.length + 16 + 32);
  let offset = 0;
  for (const part of [
    domain,
    unsignedBytes(roundId, 8),
    unsignedBytes(BigInt(addressXdr.length), 4),
    addressXdr,
    unsignedBytes(bidAmount, 16),
    secret,
  ]) {
    result.set(part, offset);
    offset += part.length;
  }
  return result;
}

export async function computeCommitment(
  roundId: bigint,
  participant: string,
  bidAmount: bigint,
  secret: Uint8Array,
): Promise<Uint8Array> {
  const preimage = encodeCommitment(roundId, participant, bidAmount, secret);
  const digestInput = new ArrayBuffer(preimage.length);
  new Uint8Array(digestInput).set(preimage);
  return new Uint8Array(await crypto.subtle.digest("SHA-256", digestInput));
}
